import test from 'node:test';
import assert from 'node:assert/strict';
import { Agent, InMemorySessionRepo, convertToLlm } from '@earendil-works/pi-agent-core';
import { Type, fauxAssistantMessage, fauxProvider, fauxToolCall } from '@earendil-works/pi-ai';

test('Pi 0.80.10 blocks a current-package tool call before execute', async () => {
  const repo = new InMemorySessionRepo();
  const stored = await repo.create({ id: 'c4os-pi-conformance' });
  const reopened = await repo.open(stored.storage.metadata);
  assert.equal(reopened.storage.metadata.id, stored.storage.metadata.id);

  const faux = fauxProvider({
    provider: 'c4os-proof',
    models: [{ id: 'conformance', name: 'Conformance' }],
  });
  faux.setResponses([
    fauxAssistantMessage([
      fauxToolCall('write_file', { path: 'blocked.txt', content: 'no' }, { id: 'tool-1' }),
    ], { stopReason: 'toolUse' }),
    fauxAssistantMessage('The application denied the tool.', { stopReason: 'stop' }),
  ]);

  let executed = 0;
  let intercepted;
  const agent = new Agent({
    sessionId: stored.storage.metadata.id,
    initialState: {
      systemPrompt: 'C4OS conformance proof',
      model: faux.getModel(),
      tools: [{
        name: 'write_file', label: 'Write file', description: 'Proof tool',
        parameters: Type.Object({ path: Type.String(), content: Type.String() }),
        execute: async () => {
          executed += 1;
          return { content: [{ type: 'text', text: 'executed' }], details: {} };
        },
      }],
    },
    convertToLlm,
    streamFn: faux.provider.streamSimple,
    beforeToolCall: async (context) => {
      intercepted = { name: context.toolCall.name, args: context.args };
      return { block: true, reason: 'C4OS policy denied proof write' };
    },
  });

  const eventTypes = [];
  agent.subscribe((event) => eventTypes.push(event.type));
  await agent.prompt('Attempt write_file');
  await agent.waitForIdle();

  assert.deepEqual(intercepted, {
    name: 'write_file', args: { path: 'blocked.txt', content: 'no' },
  });
  assert.equal(executed, 0);
  assert.ok(eventTypes.includes('tool_execution_start'));
  assert.ok(eventTypes.includes('tool_execution_end'));
  assert.ok(agent.state.messages.some((message) => message.role === 'toolResult' && message.isError));
});

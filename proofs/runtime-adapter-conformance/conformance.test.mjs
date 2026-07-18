import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';
import { OCAdapter, PIAdapter, requiredCapabilities } from './adapter.mjs';

const proofDir = fileURLToPath(new URL('.', import.meta.url));

const binding = {
  workspaceId: 'workspace-1',
  trustedRoot: '/repo',
  environment: { kind: 'local', id: 'local-desktop' },
};

const toolRequest = (overrides = {}) => ({
  nativeTool: 'write_file', nativeArguments: { path: '/repo/a.txt', content: 'hello' },
  tool: 'file.write', toolCallId: 'call-1', argumentsHash: 'sha256:args',
  target: 'workspace:/repo/a.txt', resolvedTargets: ['/repo/a.txt'],
  sessionId: 'session-1', surface: 'file', effects: ['modify'], scope: 'workspace',
  sensitivity: 'ordinary', reversibility: 'reversible', confidence: 'known',
  targetResolved: true,
  ...overrides,
});

for (const [name, Adapter] of [['OCAdapter', OCAdapter], ['PIAdapter', PIAdapter]]) {
  test(`${name} declares and fulfills the shared baseline`, () => {
    const adapter = new Adapter();
    assert.equal(adapter.probe().compatible, true);
    adapter.start(binding);
    const capabilities = adapter.describeCapabilities();
    assert.equal(capabilities.schemaVersion, 1);
    for (const capability of requiredCapabilities) assert.equal(capabilities.required[capability], true);
    assert.ok(capabilities.transport);

    const created = adapter.createSession('session-1');
    assert.equal(created.workspaceId, binding.workspaceId);
    assert.equal(adapter.resumeSession('session-1').nativeSessionId, created.nativeSessionId);
    assert.equal(adapter.enumerateSessions().length, 1);
    assert.equal(adapter.send('session-1', { text: 'hello' }).category, 'lifecycle.started');
    assert.equal(adapter.cancel('session-1'), true);
    assert.equal(adapter.cancel('session-1'), false);
    assert.equal(adapter.closeSession('session-1'), true);
    adapter.stop();
    assert.equal(adapter.status, 'stopped');
  });

  test(`${name} emits complete action intent and denies before side effects`, async () => {
    const adapter = new Adapter();
    adapter.start(binding);
    adapter.createSession('session-1');
    let sideEffects = 0;
    const result = await adapter.requestTool(toolRequest(), {
      preset: 'approve-for-me',
      categoryRules: [{ id: 'no-write', decision: 'deny', match: { effects: 'modify' } }],
    }, async () => { sideEffects += 1; });

    assert.equal(result.executed, false);
    assert.equal(sideEffects, 0);
    assert.equal(result.policyResult.decision, 'deny');
    for (const field of [
      'nativeTool', 'nativeArguments', 'resolvedTargets', 'runtime', 'runtimeGeneration',
      'workspaceId', 'sessionId', 'executionEnvironment', 'initiator', 'authenticated',
      'credentialInvolvement', 'effects', 'confidence',
    ]) assert.ok(Object.hasOwn(result.actionIntent, field), field);
  });

  test(`${name} executes once only after an application-owned allow`, async () => {
    const adapter = new Adapter();
    adapter.start(binding);
    adapter.createSession('session-1');
    let sideEffects = 0;
    const result = await adapter.requestTool(toolRequest(), { preset: 'safe' }, async () => {
      sideEffects += 1;
      return 'written';
    });
    assert.equal(result.executed, true);
    assert.equal(result.result, 'written');
    assert.equal(sideEffects, 1);
  });

  test(`${name} rejects cancelled and stale-generation events and recovers mappings`, () => {
    const adapter = new Adapter();
    adapter.start(binding);
    adapter.createSession('session-1');
    const oldGeneration = adapter.generation;
    adapter.cancel('session-1');
    assert.deepEqual(adapter.ingestNativeEvent({
      type: 'native.delta', category: 'content.delta', c4osSessionId: 'session-1',
    }, oldGeneration), { accepted: false, reason: 'cancelled-session' });

    adapter.crash('token=secret-value password=hunter2');
    const newGeneration = adapter.recover();
    assert.ok(newGeneration > oldGeneration);
    assert.equal(adapter.resumeSession('session-1').generation, newGeneration);
    assert.deepEqual(adapter.ingestNativeEvent({ type: 'native.end' }, oldGeneration), {
      accepted: false, reason: 'stale-generation',
    });
    assert.equal(adapter.diagnostics().logs.some((line) => line.includes('secret-value') || line.includes('hunter2')), false);
  });
}

test('pinned current packages and exported surfaces match the expectation manifests', async () => {
  const packageJson = JSON.parse(readFileSync(new URL('./package.json', import.meta.url), 'utf8'));
  const lock = JSON.parse(readFileSync(new URL('./package-lock.json', import.meta.url), 'utf8'));
  assert.equal(packageJson.dependencies['@opencode-ai/sdk'], '1.18.3');
  assert.equal(packageJson.dependencies['opencode-ai'], '1.18.3');
  assert.equal(packageJson.dependencies['@earendil-works/pi-coding-agent'], '0.80.10');
  assert.equal(packageJson.dependencies['@earendil-works/pi-agent-core'], '0.80.10');
  for (const [name, version] of Object.entries(packageJson.dependencies)) {
    assert.equal(lock.packages[`node_modules/${name}`].version, version);
    assert.ok(lock.packages[`node_modules/${name}`].integrity);
  }

  const oc = await import('@opencode-ai/sdk/v2');
  const pi = await import('@earendil-works/pi-coding-agent');
  const piCore = await import('@earendil-works/pi-agent-core');
  for (const name of ['createOpencode', 'createOpencodeClient', 'createOpencodeServer']) {
    assert.equal(typeof oc[name], 'function');
  }
  for (const name of ['AgentSessionRuntime', 'createAgentSession', 'RpcClient']) {
    assert.equal(typeof pi[name], 'function');
  }
  assert.equal(typeof piCore.Agent, 'function');
  assert.equal(execFileSync(`${proofDir}node_modules/.bin/opencode`, ['--version'], { encoding: 'utf8' }).trim(), '1.18.3');
  assert.equal(execFileSync(`${proofDir}node_modules/.bin/pi`, ['--version'], { encoding: 'utf8' }).trim(), '0.80.10');
});

test('PIAdapter selects the SDK sidecar without ranking Pi against OpenCode', () => {
  const capabilities = new PIAdapter().describeCapabilities();
  assert.equal(capabilities.transport, 'c4os-node-sdk-sidecar');
  assert.equal(capabilities.alternatives.rpc.available, true);
  assert.equal(capabilities.alternatives.rpc.selected, false);
  assert.match(capabilities.alternatives.rpc.reason, /direct tool interception/);
});

import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createTerminalManager,
  createRuntimeToolGateway
} from './proof.mjs';

test('one user PTY per chat uses project or home cwd and stays separate from runtime terminal tools', () => {
  const terminals = createTerminalManager({ homeDir: '/Users/chris' });
  const gateway = createRuntimeToolGateway();

  const userPty = terminals.openForChat({
    chatId: 'chat-1',
    projectPath: '/repo/api',
    env: { C4OS_CHAT_ID: 'chat-1' }
  });
  const sameUserPty = terminals.openForChat({
    chatId: 'chat-1',
    projectPath: '/repo/other'
  });
  const unassigned = terminals.openForChat({ chatId: 'chat-2' });

  assert.equal(userPty.id, sameUserPty.id);
  assert.equal(userPty.cwd, '/repo/api');
  assert.equal(unassigned.cwd, '/Users/chris');

  const runtimeResult = gateway.runTerminalTool({
    chatId: 'chat-1',
    command: 'pwd',
    cwd: '/repo/api'
  });

  assert.equal(runtimeResult.visibleInTerminalPanel, false);
  assert.deepEqual(runtimeResult.eventSources, ['thread-context', 'chat-debug']);
  assert.deepEqual(terminals.panelEvents.map((event) => event.kind), ['pty:create', 'pty:create']);

  const removal = terminals.removeChat('chat-1');
  assert.equal(removal.terminated, true);
  assert.equal(removal.deletedState, true);
  assert.equal(terminals.get('chat-1'), undefined);
});

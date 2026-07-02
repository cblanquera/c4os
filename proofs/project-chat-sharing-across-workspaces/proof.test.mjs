import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createRegistry,
  loadWorkspaceFile,
  listWorkspaceChats
} from './proof.mjs';

test('same canonical project folder shares chats across multiple workspace files', () => {
  const registry = createRegistry({
    canonicalize: (path) => path.replace(/\/+$/, '')
  });

  const openSource = loadWorkspaceFile(registry, {
    name: 'Open Source',
    projects: [{ folder: '/work/shared-lib', displayName: 'Shared Lib' }]
  });
  const client = loadWorkspaceFile(registry, {
    name: 'Client',
    projects: [
      { folder: '/work/client-app', displayName: 'Client App' },
      { folder: '/work/shared-lib/', displayName: 'Shared Lib' }
    ]
  });

  registry.addChat('/work/shared-lib', { id: 'chat-1', title: 'Fix package' });
  registry.addChat('/work/client-app', { id: 'chat-2', title: 'Client rollout' });
  registry.addChat('/work/shared-lib/', { id: 'chat-3', title: 'Publish patch' });

  assert.deepEqual(listWorkspaceChats(openSource), {
    '/work/shared-lib': ['chat-1', 'chat-3']
  });
  assert.deepEqual(listWorkspaceChats(client), {
    '/work/client-app': ['chat-2'],
    '/work/shared-lib': ['chat-1', 'chat-3']
  });
});

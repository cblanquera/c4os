import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createRegistry,
  loadWorkspaceFile,
  removeChat,
  removeProjectFromWorkspace,
  listWorkspaceChats
} from './proof.mjs';

test('remove chat deletes C4OS history while remove project only removes workspace membership', () => {
  const registry = createRegistry();
  const workspace = loadWorkspaceFile(registry, {
    name: 'Daily',
    projects: [
      { folder: '/repo/api', displayName: 'API' },
      { folder: '/repo/web', displayName: 'Web' }
    ]
  });

  registry.addChat('/repo/api', { id: 'api-1', title: 'API bug' });
  registry.addChat('/repo/api', { id: 'api-2', title: 'API follow-up' });
  registry.addChat('/repo/web', { id: 'web-1', title: 'Web polish' });

  const chatRemoval = removeChat(registry, '/repo/api', 'api-1');
  assert.equal(chatRemoval.deleted, true);
  assert.deepEqual(registry.getChats('/repo/api').map((chat) => chat.id), ['api-2']);

  const projectRemoval = removeProjectFromWorkspace(workspace, '/repo/api');
  assert.equal(projectRemoval.removed, true);
  assert.deepEqual(workspace.projects.map((project) => project.canonicalPath), ['/repo/web']);
  assert.deepEqual(registry.getChats('/repo/api').map((chat) => chat.id), ['api-2']);
  assert.deepEqual(listWorkspaceChats(workspace), {
    '/repo/web': ['web-1']
  });
});

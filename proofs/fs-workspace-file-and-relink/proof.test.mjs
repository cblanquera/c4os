import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createRegistry,
  loadWorkspaceFile,
  saveWorkspaceFile,
  markProjectMissing,
  relocateProject
} from './proof.mjs';

test('workspace files store project folders while chats remain user-level and missing projects become read-only until relinked', () => {
  const registry = createRegistry({
    canonicalize: (path) => path.replace(/\/+$/, '')
  });

  const workspace = loadWorkspaceFile(registry, {
    name: 'Ops Workspace',
    projects: [
      { folder: '/Users/chris/work/api', displayName: 'API' },
      { folder: '/Users/chris/work/docs/', displayName: 'Docs' }
    ]
  });

  registry.addChat('/Users/chris/work/api', {
    id: 'chat-api-1',
    title: 'Investigate release issue'
  });
  markProjectMissing(registry, '/Users/chris/work/api');

  const missing = registry.getProject('/Users/chris/work/api');
  assert.equal(missing.status, 'missing');
  assert.equal(missing.readOnly, true);
  assert.equal(missing.muted, true);
  assert.equal(missing.lastKnownPath, '/Users/chris/work/api');
  assert.deepEqual(missing.chats.map((chat) => chat.id), ['chat-api-1']);

  const relocated = relocateProject(
    registry,
    '/Users/chris/work/api',
    '/Volumes/projects/api',
    { inode: 'node-123' }
  );

  assert.equal(relocated.canonicalPath, '/Volumes/projects/api');
  assert.equal(relocated.status, 'active');
  assert.equal(relocated.readOnly, false);
  assert.deepEqual(relocated.relinkHistory, [
    { from: '/Users/chris/work/api', to: '/Volumes/projects/api' }
  ]);
  assert.deepEqual(
    registry.getChats('/Volumes/projects/api').map((chat) => chat.id),
    ['chat-api-1']
  );

  assert.deepEqual(saveWorkspaceFile(workspace), {
    name: 'Ops Workspace',
    projects: [
      { folder: '/Users/chris/work/api', displayName: 'API' },
      { folder: '/Users/chris/work/docs', displayName: 'Docs' }
    ]
  });
});

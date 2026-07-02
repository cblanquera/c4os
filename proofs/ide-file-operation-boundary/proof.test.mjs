import test from 'node:test';
import assert from 'node:assert/strict';
import {
  createEditorSession,
  createBackendFileService,
  insertPromptReference
} from './proof.mjs';

test('editor operations route through backend service with conflict, trash, and prompt-reference events', () => {
  const backend = createBackendFileService({
    files: {
      '/repo/notes.md': { content: 'draft', version: 1 }
    }
  });
  const editor = createEditorSession({ backend, activeProject: '/repo' });

  editor.open('/repo/notes.md');
  editor.edit('updated');
  backend.externalChange('/repo/notes.md', 'outside update');

  const conflict = editor.save('/repo/notes.md');
  assert.equal(conflict.status, 'conflict');
  assert.equal(backend.operations.at(-1).type, 'writeRejected');

  editor.resolveConflict('/repo/notes.md', 'overwrite');
  assert.equal(backend.snapshot('/repo/notes.md').content, 'updated');

  editor.create('/repo/tasks.txt', 'task');
  editor.rename('/repo/tasks.txt', '/repo/tasks-renamed.txt');
  const deletion = editor.deleteToTrash('/repo/tasks-renamed.txt', { confirmed: true });
  assert.equal(deletion.status, 'trashed');

  const prompt = insertPromptReference({ prompt: 'Review ', filePath: '/repo/notes.md' });
  assert.equal(prompt.text, 'Review @notes.md');
  assert.equal(prompt.reference.kind, 'file-tag');
  assert.equal(prompt.reference.path, '/repo/notes.md');

  assert.deepEqual(
    backend.operations.map((operation) => operation.type),
    ['read', 'writeRejected', 'write', 'create', 'rename', 'trash']
  );
  assert.ok(editor.events.every((event) => event.source === 'editor'));
});

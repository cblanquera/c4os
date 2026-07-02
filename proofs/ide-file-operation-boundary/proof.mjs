export function createBackendFileService({ files = {} } = {}) {
  const store = new Map(
    Object.entries(files).map(([path, file]) => [
      path,
      { content: file.content, version: file.version ?? 1 }
    ])
  );
  const operations = [];

  return {
    operations,
    read(path) {
      operations.push({ type: 'read', path });
      const file = store.get(path);
      if (!file) throw new Error(`missing file: ${path}`);
      return { ...file };
    },
    snapshot(path) {
      const file = store.get(path);
      if (!file) throw new Error(`missing file: ${path}`);
      return { ...file };
    },
    write(path, content, expectedVersion, { force = false } = {}) {
      const current = store.get(path);
      if (current && current.version !== expectedVersion && !force) {
        operations.push({ type: 'writeRejected', path, reason: 'external-change' });
        return { status: 'conflict', current: { ...current } };
      }
      store.set(path, {
        content,
        version: (current?.version ?? 0) + 1
      });
      operations.push({ type: 'write', path });
      return { status: 'saved' };
    },
    create(path, content) {
      store.set(path, { content, version: 1 });
      operations.push({ type: 'create', path });
      return { status: 'created' };
    },
    rename(from, to) {
      const file = store.get(from);
      if (!file) throw new Error(`missing file: ${from}`);
      store.delete(from);
      store.set(to, file);
      operations.push({ type: 'rename', from, to });
      return { status: 'renamed' };
    },
    trash(path, { confirmed }) {
      if (!confirmed) return { status: 'confirmation-required' };
      store.delete(path);
      operations.push({ type: 'trash', path });
      return { status: 'trashed' };
    },
    externalChange(path, content) {
      const current = store.get(path);
      store.set(path, {
        content,
        version: (current?.version ?? 0) + 1
      });
    }
  };
}

export function createEditorSession({ backend, activeProject }) {
  const buffers = new Map();
  const events = [];

  return {
    events,
    open(path) {
      ensureTrusted(activeProject, path);
      const file = backend.read(path);
      buffers.set(path, {
        content: file.content,
        savedVersion: file.version,
        dirty: false
      });
      events.push(event('open', path));
    },
    edit(content, path = [...buffers.keys()].at(-1)) {
      const buffer = buffers.get(path);
      buffer.content = content;
      buffer.dirty = true;
      events.push(event('dirty', path));
    },
    save(path) {
      const buffer = buffers.get(path);
      const result = backend.write(path, buffer.content, buffer.savedVersion);
      if (result.status === 'conflict') {
        events.push(event('conflict', path));
        return result;
      }
      buffer.dirty = false;
      buffer.savedVersion += 1;
      events.push(event('save', path));
      return result;
    },
    resolveConflict(path, choice) {
      const buffer = buffers.get(path);
      if (choice !== 'overwrite') return { status: 'kept-external' };
      const result = backend.write(path, buffer.content, buffer.savedVersion, { force: true });
      buffer.dirty = false;
      buffer.savedVersion += 1;
      events.push(event('conflict:overwrite', path));
      return result;
    },
    create(path, content) {
      ensureTrusted(activeProject, path);
      const result = backend.create(path, content);
      events.push(event('create', path));
      return result;
    },
    rename(from, to) {
      ensureTrusted(activeProject, from);
      ensureTrusted(activeProject, to);
      const result = backend.rename(from, to);
      events.push(event('rename', to));
      return result;
    },
    deleteToTrash(path, options) {
      ensureTrusted(activeProject, path);
      const result = backend.trash(path, options);
      events.push(event('delete-to-trash', path));
      return result;
    }
  };
}

export function insertPromptReference({ prompt, filePath }) {
  const label = `@${filePath.split('/').at(-1)}`;
  return {
    text: `${prompt}${label}`,
    reference: {
      kind: 'file-tag',
      path: filePath,
      display: label
    }
  };
}

function ensureTrusted(activeProject, path) {
  if (!path.startsWith(`${activeProject}/`)) {
    throw new Error(`file operation outside active project: ${path}`);
  }
}

function event(type, path) {
  return { source: 'editor', type, path };
}

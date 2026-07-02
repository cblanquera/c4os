export function createRegistry({ canonicalize = defaultCanonicalize } = {}) {
  const projects = new Map();
  const chatsByProject = new Map();

  const registry = {
    canonicalize,
    ensureProject(folder, metadata = {}) {
      const canonicalPath = canonicalize(folder);
      if (!projects.has(canonicalPath)) {
        projects.set(canonicalPath, {
          canonicalPath,
          displayName: metadata.displayName ?? basename(canonicalPath),
          lastKnownPath: canonicalPath,
          status: 'active',
          readOnly: false,
          muted: false,
          metadata: metadata.metadata ?? null,
          relinkHistory: []
        });
      }
      return projects.get(canonicalPath);
    },
    getProject(folder) {
      const project = projects.get(canonicalize(folder));
      return project ? withChats(project, chatsByProject) : undefined;
    },
    addChat(folder, chat) {
      const canonicalPath = canonicalize(folder);
      registry.ensureProject(canonicalPath);
      const chats = chatsByProject.get(canonicalPath) ?? [];
      chats.push({ ...chat });
      chatsByProject.set(canonicalPath, chats);
    },
    getChats(folder) {
      return [...(chatsByProject.get(canonicalize(folder)) ?? [])];
    },
    migrateChats(from, to) {
      const fromPath = canonicalize(from);
      const toPath = canonicalize(to);
      const current = chatsByProject.get(toPath) ?? [];
      chatsByProject.set(toPath, [...(chatsByProject.get(fromPath) ?? []), ...current]);
      chatsByProject.delete(fromPath);
    },
    projects,
    chatsByProject
  };

  return registry;
}

export function loadWorkspaceFile(registry, workspaceFile) {
  const workspace = {
    name: workspaceFile.name,
    registry,
    projects: workspaceFile.projects.map((project) => {
      const record = registry.ensureProject(project.folder, {
        displayName: project.displayName
      });
      return {
        canonicalPath: record.canonicalPath,
        displayName: project.displayName ?? record.displayName,
        originalFolder: project.folder
      };
    })
  };
  return workspace;
}

export function saveWorkspaceFile(workspace) {
  return {
    name: workspace.name,
    projects: workspace.projects.map((project) => ({
      folder: project.canonicalPath,
      displayName: project.displayName
    }))
  };
}

export function markProjectMissing(registry, folder) {
  const project = registry.ensureProject(folder);
  project.status = 'missing';
  project.readOnly = true;
  project.muted = true;
  project.lastKnownPath = project.canonicalPath;
  return withChats(project, registry.chatsByProject);
}

export function relocateProject(registry, from, to, metadata = {}) {
  const fromPath = registry.canonicalize(from);
  const toPath = registry.canonicalize(to);
  const project = registry.ensureProject(fromPath);
  registry.projects.delete(fromPath);
  registry.migrateChats(fromPath, toPath);
  const relocated = {
    ...project,
    canonicalPath: toPath,
    lastKnownPath: toPath,
    status: 'active',
    readOnly: false,
    muted: false,
    metadata,
    relinkHistory: [...project.relinkHistory, { from: fromPath, to: toPath }]
  };
  registry.projects.set(toPath, relocated);
  return withChats(relocated, registry.chatsByProject);
}

function withChats(project, chatsByProject) {
  return {
    ...project,
    chats: [...(chatsByProject.get(project.canonicalPath) ?? [])]
  };
}

function defaultCanonicalize(path) {
  return path.replace(/\/+$/, '');
}

function basename(path) {
  return path.split('/').filter(Boolean).at(-1) ?? path;
}

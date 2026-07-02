export { createRegistry, loadWorkspaceFile } from '../fs-workspace-file-and-relink/proof.mjs';

export function removeChat(registry, folder, chatId) {
  const canonicalPath = registry.canonicalize(folder);
  const before = registry.getChats(canonicalPath);
  const after = before.filter((chat) => chat.id !== chatId);
  registry.chatsByProject.set(canonicalPath, after);
  return { deleted: before.length !== after.length };
}

export function removeProjectFromWorkspace(workspace, folder) {
  const canonicalPath = workspace.registry.canonicalize(folder);
  const before = workspace.projects.length;
  workspace.projects = workspace.projects.filter(
    (project) => project.canonicalPath !== canonicalPath
  );
  return { removed: workspace.projects.length !== before };
}

export function listWorkspaceChats(workspace) {
  return Object.fromEntries(
    workspace.projects.map((project) => [
      project.canonicalPath,
      workspace.registry.getChats(project.canonicalPath).map((chat) => chat.id)
    ])
  );
}

export { createRegistry, loadWorkspaceFile } from '../fs-workspace-file-and-relink/proof.mjs';

export function listWorkspaceChats(workspace) {
  return Object.fromEntries(
    workspace.projects.map((project) => [
      project.canonicalPath,
      workspace.registry.getChats(project.canonicalPath).map((chat) => chat.id)
    ])
  );
}

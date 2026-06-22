export const connectorMethods = [
  "loadWorkspace",
  "sendPrompt",
  "openWorkspace",
  "createSession",
  "readFile",
  "saveFile",
  "runTerminalCommand",
  "openBrowserPreview",
  "listExtensions",
  "listProviderProfiles",
  "saveProviderProfile",
  "deleteProviderProfile",
  "listProviderModels",
  "setModelEnabled",
  "setProviderEnabled",
  "selectSessionModel"
];

export const workspacePayloadFields = [
  "workspace",
  "projects",
  "providers",
  "models",
  "pluginCatalog",
  "pluginMarketplaces",
  "skillCatalog",
  "mcpServers",
  "browser",
  "files",
  "terminal",
  "thread"
];

export function validateWorkspacePayload(payload) {
  if (!payload || typeof payload !== "object") return workspacePayloadFields;
  return workspacePayloadFields.filter((field) => !(field in payload));
}

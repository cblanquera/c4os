export type AppSurfaceKind =
  "launch" | "workspace" | "settings" | "advancedPolicies";

export interface AppRouteDefinition {
  readonly id: AppRouteId;
  readonly path: AppRoutePath;
  readonly title: string;
  readonly kind: AppSurfaceKind;
}

export type AppRouteId =
  | "onboarding"
  | "start"
  | "chat"
  | "chatSearch"
  | "chatCapabilities"
  | "files"
  | "browser"
  | "terminal"
  | "providers"
  | "models"
  | "runtimes"
  | "configuration"
  | "plugins"
  | "skills"
  | "mcp"
  | "advancedPolicies";

export type AppRoutePath =
  | "/onboarding"
  | "/start"
  | "/chat"
  | "/chat-search"
  | "/chat-capabilities"
  | "/files"
  | "/browser"
  | "/terminal"
  | "/settings/providers"
  | "/settings/models"
  | "/settings/runtimes"
  | "/settings/configuration"
  | "/settings/plugins"
  | "/settings/skills"
  | "/settings/mcp"
  | "/settings/advanced-policies";

export const APP_ROUTE_DEFINITIONS = [
  {
    id: "onboarding",
    path: "/onboarding",
    title: "Connect your AI provider",
    kind: "launch",
  },
  { id: "start", path: "/start", title: "Workspace Start", kind: "launch" },
  { id: "chat", path: "/chat", title: "Chat", kind: "workspace" },
  {
    id: "chatSearch",
    path: "/chat-search",
    title: "Search Chat Sessions",
    kind: "workspace",
  },
  {
    id: "chatCapabilities",
    path: "/chat-capabilities",
    title: "Capability-aware Chat",
    kind: "workspace",
  },
  { id: "files", path: "/files", title: "Files", kind: "workspace" },
  { id: "browser", path: "/browser", title: "Browser", kind: "workspace" },
  { id: "terminal", path: "/terminal", title: "Terminal", kind: "workspace" },
  {
    id: "providers",
    path: "/settings/providers",
    title: "Providers",
    kind: "settings",
  },
  {
    id: "models",
    path: "/settings/models",
    title: "Models",
    kind: "settings",
  },
  {
    id: "runtimes",
    path: "/settings/runtimes",
    title: "Runtimes",
    kind: "settings",
  },
  {
    id: "configuration",
    path: "/settings/configuration",
    title: "Configuration",
    kind: "settings",
  },
  {
    id: "plugins",
    path: "/settings/plugins",
    title: "Plugins",
    kind: "settings",
  },
  {
    id: "skills",
    path: "/settings/skills",
    title: "Skills",
    kind: "settings",
  },
  {
    id: "mcp",
    path: "/settings/mcp",
    title: "MCP Servers",
    kind: "settings",
  },
  {
    id: "advancedPolicies",
    path: "/settings/advanced-policies",
    title: "Advanced Policies",
    kind: "advancedPolicies",
  },
] as const satisfies readonly AppRouteDefinition[];

export const SETTINGS_NAVIGATION = APP_ROUTE_DEFINITIONS.filter(
  (route) => route.kind === "settings",
);

export function findAppRoute(path: string): AppRouteDefinition | null {
  return (
    APP_ROUTE_DEFINITIONS.find((candidate) => candidate.path === path) ?? null
  );
}

export function isSettingsRoute(path: string): boolean {
  const route = findAppRoute(path);
  return route?.kind === "settings" || route?.kind === "advancedPolicies";
}

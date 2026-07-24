import type { AppRoutePath } from "../../../app/route-contract";
import {
  APP_ROUTE_DEFINITIONS,
  findAppRoute,
  SETTINGS_NAVIGATION,
} from "../../../app/route-contract";

// Every accepted top-level surface remains directly addressable by the host
// router. Deriving the view list prevents a second route authority here.
export const SHELL_ROUTE_PATHS = APP_ROUTE_DEFINITIONS.map(
  (route) => route.path,
);

// ShellRoutePath prevents a silent, unreviewable fallback for unknown routes.
export type ShellRoutePath = AppRoutePath;

// WorkspaceRoutePath identifies routes that share the project/transcript shell.
export type WorkspaceRoutePath = Extract<
  ShellRoutePath,
  | "/chat"
  | "/chat-search"
  | "/chat-capabilities"
  | "/files"
  | "/browser"
  | "/terminal"
>;

// SettingsRoutePath identifies routes that share the Settings navigation shell.
export type SettingsRoutePath = Extract<ShellRoutePath, `/settings/${string}`>;

// LaunchRoutePath identifies the two standalone launch surfaces.
export type LaunchRoutePath = Extract<ShellRoutePath, "/onboarding" | "/start">;

// Composer modes are drafts only; later tasks decide whether a submitted mode
// can bind to an authoritative runtime session.
export type ShellComposerMode = "chat" | "files" | "browser" | "terminal";

// Settings navigation order is accepted product behavior, not fixture order.
export const SETTINGS_DESTINATIONS = SETTINGS_NAVIGATION.map(
  (destination, index) => ({
    path: destination.path,
    label: destination.title,
    symbol: settingsSymbol(destination.path),
    group: index < 4 ? "runtime" : "extensions",
  }),
) satisfies ReadonlyArray<{
  readonly path: SettingsRoutePath;
  readonly label: string;
  readonly symbol: string;
  readonly group: "runtime" | "extensions";
}>;

const ROUTE_COPY: Readonly<
  Record<ShellRoutePath, { readonly support: string }>
> = {
  "/onboarding": {
    support: "Add a provider to choose the models C4OS can use.",
  },
  "/start": {
    support: "Start something new or continue where you left off.",
  },
  "/chat": {
    support: "Continue your conversation.",
  },
  "/chat-search": {
    support: "Find a chat session across the current workspace.",
  },
  "/chat-capabilities": {
    support: "Review model and attachment compatibility before sending.",
  },
  "/files": {
    support: "Open a file or browse a folder.",
  },
  "/browser": {
    support: "Open and operate a webpage.",
  },
  "/terminal": {
    support: "Run a command in this chat's shell session.",
  },
  "/settings/providers": {
    support: "Manage provider connections.",
  },
  "/settings/models": {
    support: "Choose the models available to C4OS.",
  },
  "/settings/runtimes": {
    support: "Choose the runtime used for new work.",
  },
  "/settings/configuration": {
    support: "Set approval and environment defaults.",
  },
  "/settings/plugins": {
    support: "Discover and manage plugins.",
  },
  "/settings/skills": {
    support: "Manage installed skills and availability.",
  },
  "/settings/mcp": {
    support: "Configure external tool servers.",
  },
  "/settings/advanced-policies": {
    support: "Review category rules and concrete exceptions.",
  },
};

/** Returns accepted title and support copy for a directly addressable route. */
export function getShellRouteCopy(route: ShellRoutePath) {
  const definition = findAppRoute(route);
  if (!definition) throw new Error(`Missing accepted route: ${route}`);
  return { title: definition.title, support: ROUTE_COPY[route].support };
}

/** Identifies a route rendered inside the Settings shell. */
export function isSettingsRoute(
  route: ShellRoutePath,
): route is SettingsRoutePath {
  return route.startsWith("/settings/");
}

/** Identifies a route rendered inside a standalone launch layout. */
export function isLaunchRoute(route: ShellRoutePath): route is LaunchRoutePath {
  return route === "/onboarding" || route === "/start";
}

/** Provides compact visual labels without duplicating navigation order. */
function settingsSymbol(route: SettingsRoutePath): string {
  const symbols: Record<SettingsRoutePath, string> = {
    "/settings/providers": "P",
    "/settings/models": "M",
    "/settings/runtimes": "R",
    "/settings/configuration": "C",
    "/settings/plugins": "+",
    "/settings/skills": "S",
    "/settings/mcp": "⇄",
    "/settings/advanced-policies": "A",
  };
  return symbols[route];
}

import { findAppRoute, type AppRoutePath } from "./route-contract";
import type { RootState } from "./store";
import type {
  SettingsSection,
  SettingsReturnState,
} from "../features/shell/state";
import type { ShellFocusTarget } from "../features/shell/ui";

const SHELL_FOCUS_TARGETS = new Set<ShellFocusTarget>([
  "workspace-settings",
  "project-panel-toggle",
  "composer-draft",
]);

/** Keeps native and review Settings entry on an accepted return route. */
export function resolveSettingsReturnRoute(path: string): AppRoutePath {
  const route = findAppRoute(path);
  return route && route.kind !== "settings" && route.kind !== "advancedPolicies"
    ? route.path
    : "/start";
}

/** Captures only stable, shell-owned focus identifiers from the live document. */
export function readActiveShellFocusTarget(): ShellFocusTarget | null {
  const value = document.activeElement?.getAttribute("data-shell-focus-target");
  return value && SHELL_FOCUS_TARGETS.has(value as ShellFocusTarget)
    ? (value as ShellFocusTarget)
    : null;
}

/** Builds the complete Settings return record from authoritative projections. */
export function createSettingsVisit(
  state: RootState,
  path: string,
  focusTarget: ShellFocusTarget | null,
): SettingsReturnState & { readonly section: SettingsSection } {
  return {
    section: "providers",
    route: resolveSettingsReturnRoute(path),
    workspaceId: state.shellAuthority.workspace.value.activeWorkspaceId,
    sessionId: state.shellAuthority.sessions.value.activeSessionId,
    focusTarget,
  };
}

/** Derives the draft section from the accepted Settings route contract. */
export function settingsSectionForRoute(
  route: Extract<AppRoutePath, `/settings/${string}`>,
): SettingsSection {
  return route.slice("/settings/".length) as SettingsSection;
}

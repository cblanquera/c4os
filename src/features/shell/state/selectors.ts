import type {
  AuthorityDomain,
  AuthorityProjectionMap,
  AuthoritativeProjection,
  SettingsReturnState,
  ShellStateOwner,
} from "./types";

export function selectAuthoritativeDomain<Domain extends AuthorityDomain>(
  state: ShellStateOwner,
  domain: Domain,
): AuthoritativeProjection<AuthorityProjectionMap[Domain]> {
  return state.shellAuthority[domain];
}

export const selectPlatform = (state: ShellStateOwner) =>
  state.shellAuthority.platform;
export const selectLaunch = (state: ShellStateOwner) =>
  state.shellAuthority.launch;
export const selectWorkspace = (state: ShellStateOwner) =>
  state.shellAuthority.workspace;
export const selectSessions = (state: ShellStateOwner) =>
  state.shellAuthority.sessions;
export const selectConversation = (state: ShellStateOwner) =>
  state.shellAuthority.conversation;
export const selectComposerProjection = (state: ShellStateOwner) =>
  state.shellAuthority.composer;
export const selectArtifacts = (state: ShellStateOwner) =>
  state.shellAuthority.artifacts;
export const selectSettingsProjection = (state: ShellStateOwner) =>
  state.shellAuthority.settings;
export const selectApprovals = (state: ShellStateOwner) =>
  state.shellAuthority.approvals;
export const selectRuntime = (state: ShellStateOwner) =>
  state.shellAuthority.runtime;
export const selectExtensions = (state: ShellStateOwner) =>
  state.shellAuthority.extensions;
export const selectNotices = (state: ShellStateOwner) =>
  state.shellAuthority.notices;

export const selectWorkspaceUiDraft = (state: ShellStateOwner) =>
  state.shellDrafts.workspace;
export const selectComposerDraft = (state: ShellStateOwner) =>
  state.shellDrafts.composer;
export const selectSettingsDraft = (state: ShellStateOwner) =>
  state.shellDrafts.settings;
export const selectFocusedArtifactId = (state: ShellStateOwner) =>
  state.shellDrafts.workspace.focusedArtifactId;
export const selectQaState = (state: ShellStateOwner) => state.shellQa;

export function selectSettingsReturnState(
  state: ShellStateOwner,
): SettingsReturnState | null {
  const visit = state.shellDrafts.settings.visit;
  return visit === null ? null : { ...visit };
}

export function selectVisibleNotices(state: ShellStateOwner) {
  const dismissed = new Set(state.shellDrafts.dismissedNoticeIds);
  return state.shellAuthority.notices.value.notices.filter(
    ({ id }) => !dismissed.has(id),
  );
}

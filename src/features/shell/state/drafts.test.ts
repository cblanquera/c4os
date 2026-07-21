import { configureStore } from "@reduxjs/toolkit";

import type {
  ArtifactId,
  SessionId,
  WorkspaceId,
} from "../../../platform/protocol";
import { shellAuthorityReducer } from "./authority";
import {
  shellDraftActions,
  shellDraftReducer,
  shellPanelBounds,
} from "./drafts";
import { qaReducer } from "./qa-fixtures";
import {
  selectComposerDraft,
  selectSettingsReturnState,
  selectWorkspaceUiDraft,
} from "./selectors";

function createStateStore() {
  return configureStore({
    reducer: {
      shellAuthority: shellAuthorityReducer,
      shellDrafts: shellDraftReducer,
      shellQa: qaReducer,
    },
  });
}

describe("renderer-local shell drafts", () => {
  it("reconciles Reply only when the command still owns the local target", () => {
    const store = createStateStore();
    store.dispatch(shellDraftActions.composerReplyChanged("turn:one"));
    store.dispatch(
      shellDraftActions.composerReplyReconciled({
        expectedReplyTargetId: "turn:one",
        authoritativeReplyTargetId: "turn:one",
      }),
    );
    expect(selectComposerDraft(store.getState()).replyTargetId).toBe(
      "turn:one",
    );

    store.dispatch(shellDraftActions.composerReplyChanged("turn:newer"));
    store.dispatch(
      shellDraftActions.composerReplyReconciled({
        expectedReplyTargetId: "turn:one",
        authoritativeReplyTargetId: null,
      }),
    );
    expect(selectComposerDraft(store.getState()).replyTargetId).toBe(
      "turn:newer",
    );
  });

  it("preserves workspace, composer, panel, and focus state across Settings", () => {
    const store = createStateStore();
    const workspaceId = "workspace:settings" as WorkspaceId;
    const sessionId = "session:settings" as SessionId;
    const artifactId = "artifact:settings" as ArtifactId;

    store.dispatch(shellDraftActions.composerModeChanged("terminal"));
    store.dispatch(shellDraftActions.composerTextChanged("npm test"));
    store.dispatch(
      shellDraftActions.leftPanelResized({ width: 344, viewportWidth: 1200 }),
    );
    store.dispatch(shellDraftActions.leftPanelCollapsed(true));
    store.dispatch(
      shellDraftActions.artifactFocused({
        artifactId,
        restoreTarget: "expand-file",
      }),
    );

    const beforeSettings = store.getState().shellDrafts;
    store.dispatch(
      shellDraftActions.settingsVisited({
        section: "providers",
        route: "/files",
        workspaceId,
        sessionId,
        focusTarget: "expand-file",
      }),
    );
    store.dispatch(shellDraftActions.settingsSectionChanged("models"));
    store.dispatch(
      shellDraftActions.settingsVisited({
        section: "runtimes",
        route: "/settings/models",
        workspaceId: null,
        sessionId: null,
        focusTarget: "settings-models",
      }),
    );

    expect(selectSettingsReturnState(store.getState())).toEqual({
      route: "/files",
      workspaceId,
      sessionId,
      focusTarget: "expand-file",
    });
    expect(store.getState().shellDrafts.settings.activeSection).toBe(
      "runtimes",
    );
    expect(selectWorkspaceUiDraft(store.getState())).toEqual(
      beforeSettings.workspace,
    );
    expect(selectComposerDraft(store.getState())).toEqual(
      beforeSettings.composer,
    );

    store.dispatch(shellDraftActions.settingsVisitEnded());
    expect(selectSettingsReturnState(store.getState())).toBeNull();
    expect(selectWorkspaceUiDraft(store.getState())).toEqual(
      beforeSettings.workspace,
    );
    expect(selectComposerDraft(store.getState())).toEqual(
      beforeSettings.composer,
    );
  });

  it("locks focused artifacts to Chat without reopening the panel and restores mode", () => {
    const store = createStateStore();
    const artifactId = "artifact:browser" as ArtifactId;
    store.dispatch(shellDraftActions.composerModeChanged("browser"));
    store.dispatch(shellDraftActions.leftPanelCollapsed(true));

    store.dispatch(
      shellDraftActions.artifactFocused({
        artifactId,
        restoreTarget: "expand-browser",
      }),
    );

    expect(selectComposerDraft(store.getState())).toMatchObject({
      mode: "chat",
      modeBeforeFocus: "browser",
    });
    expect(selectWorkspaceUiDraft(store.getState())).toMatchObject({
      panel: { collapsed: true, overlayOpen: false },
      focusedArtifactId: artifactId,
    });

    store.dispatch(shellDraftActions.composerModeChanged("terminal"));
    expect(selectComposerDraft(store.getState()).mode).toBe("chat");

    store.dispatch(shellDraftActions.chatRestored());
    expect(selectComposerDraft(store.getState())).toMatchObject({
      mode: "browser",
      modeBeforeFocus: null,
    });
  });

  it("clamps repeated panel resizes to the shell geometry contract", () => {
    const store = createStateStore();
    store.dispatch(
      shellDraftActions.leftPanelResized({ width: 80, viewportWidth: 1280 }),
    );
    expect(selectWorkspaceUiDraft(store.getState()).panel.width).toBe(180);

    store.dispatch(
      shellDraftActions.leftPanelResized({ width: 900, viewportWidth: 1000 }),
    );
    expect(selectWorkspaceUiDraft(store.getState()).panel.width).toBe(550);

    store.dispatch(
      shellDraftActions.leftPanelResized({ width: 310, viewportWidth: 1280 }),
    );
    expect(selectWorkspaceUiDraft(store.getState()).panel.width).toBe(310);
  });

  it("reopens a docked-collapsed panel when it becomes an overlay", () => {
    const store = createStateStore();
    store.dispatch(shellDraftActions.leftPanelCollapsed(true));
    store.dispatch(shellDraftActions.leftPanelOverlayChanged(true));

    expect(selectWorkspaceUiDraft(store.getState()).panel).toMatchObject({
      collapsed: false,
      overlayOpen: true,
    });
  });

  it("publishes the same panel maximum used by reducer clamping", () => {
    expect(shellPanelBounds(1280)).toEqual({ minimum: 180, maximum: 704 });

    const store = createStateStore();
    store.dispatch(
      shellDraftActions.leftPanelResized({ width: 860, viewportWidth: 1280 }),
    );
    expect(selectWorkspaceUiDraft(store.getState()).panel.width).toBe(704);
  });
});

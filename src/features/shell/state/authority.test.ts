import { configureStore } from "@reduxjs/toolkit";

import type {
  ArtifactId,
  StateGeneration,
  WorkspaceId,
} from "../../../platform/protocol";
import { shellAuthorityActions, shellAuthorityReducer } from "./authority";
import { shellDraftActions, shellDraftReducer } from "./drafts";
import { qaReducer } from "./qa-fixtures";
import {
  selectComposerDraft,
  selectFocusedArtifactId,
  selectPlatform,
  selectWorkspace,
} from "./selectors";

function generation(value: number) {
  return value as StateGeneration;
}

function createStateStore() {
  return configureStore({
    reducer: {
      shellAuthority: shellAuthorityReducer,
      shellDrafts: shellDraftReducer,
      shellQa: qaReducer,
    },
  });
}

describe("authoritative shell projections", () => {
  it("accepts only strictly newer publications within each domain", () => {
    const store = createStateStore();

    store.dispatch(
      shellAuthorityActions.publicationReceived({
        source: "snapshot",
        domain: "platform",
        generation: generation(7),
        value: {
          appearance: "dark",
          appearanceSource: "native-snapshot",
          reducedMotion: false,
        },
      }),
    );
    store.dispatch(
      shellAuthorityActions.publicationReceived({
        source: "event",
        domain: "platform",
        generation: generation(7),
        value: {
          appearance: "light",
          appearanceSource: "webview-live-preference",
          reducedMotion: false,
        },
      }),
    );
    store.dispatch(
      shellAuthorityActions.publicationReceived({
        source: "snapshot",
        domain: "platform",
        generation: generation(6),
        value: {
          appearance: "light",
          appearanceSource: "native-snapshot",
          reducedMotion: true,
        },
      }),
    );

    expect(selectPlatform(store.getState())).toEqual({
      generation: 7,
      value: {
        appearance: "dark",
        appearanceSource: "native-snapshot",
        reducedMotion: false,
      },
    });
  });

  it("tracks monotonic cursors independently for publications from one snapshot", () => {
    const store = createStateStore();
    const workspaceId = "workspace:one" as WorkspaceId;

    store.dispatch(
      shellAuthorityActions.publicationReceived({
        source: "snapshot",
        domain: "platform",
        generation: generation(11),
        value: {
          appearance: "dark",
          appearanceSource: "native-snapshot",
          reducedMotion: true,
        },
      }),
    );
    store.dispatch(
      shellAuthorityActions.publicationReceived({
        source: "snapshot",
        domain: "workspace",
        generation: generation(11),
        value: {
          activeWorkspaceId: workspaceId,
          displayName: "One",
          projects: [],
          activeProjectId: null,
        },
      }),
    );

    expect(selectPlatform(store.getState()).generation).toBe(11);
    expect(selectWorkspace(store.getState()).value.activeWorkspaceId).toBe(
      workspaceId,
    );
  });

  it("never replaces local drafts when Rust publishes a newer projection", () => {
    const store = createStateStore();
    const artifactId = "artifact:one" as ArtifactId;
    store.dispatch(shellDraftActions.composerTextChanged("unsent local text"));
    store.dispatch(
      shellDraftActions.artifactFocused({
        artifactId,
        restoreTarget: "artifact-expand",
      }),
    );

    store.dispatch(
      shellAuthorityActions.publicationReceived({
        source: "event",
        domain: "composer",
        generation: generation(12),
        value: {
          activeModelId: "model:new",
          allowedModes: ["chat", "files"],
          reasoningEfforts: ["off", "low"],
          activeBranch: "main",
        },
      }),
    );

    expect(selectComposerDraft(store.getState()).text).toBe(
      "unsent local text",
    );
    expect(selectFocusedArtifactId(store.getState())).toBe(artifactId);
  });
});

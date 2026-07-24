import { useEffect, useRef, useState } from "react";

import { Button, Notice, StatusRegion } from "../../../components/accessible";
import type {
  RuntimeOptionView,
  RuntimeSettingsActions,
  RuntimeSettingsSnapshot,
} from "./types";

import "./runtime-settings.css";

export interface RuntimeSettingsProps {
  readonly actions: RuntimeSettingsActions;
  readonly snapshot: RuntimeSettingsSnapshot;
}

/** Keeps the runtime selection as a local draft until the service accepts Save. */
export function RuntimeSettings({ actions, snapshot }: RuntimeSettingsProps) {
  if (snapshot.status === "loading") {
    return (
      <StatusRegion
        aria-busy="true"
        className="runtime-settings-state runtime-settings-state--loading"
      >
        <span aria-hidden="true" className="runtime-settings-spinner" />
        <span>
          <strong>Loading runtimes</strong>
          <span>{snapshot.message}</span>
        </span>
      </StatusRegion>
    );
  }

  if (snapshot.status === "error") {
    return (
      <Notice title="Runtimes are unavailable" tone="danger">
        <p>{snapshot.message}</p>
        {snapshot.retryable ? (
          <Button onPress={() => void actions.onRetry()}>Try again</Button>
        ) : null}
      </Notice>
    );
  }

  return <RuntimeChoices actions={actions} snapshot={snapshot} />;
}

function RuntimeChoices({
  actions,
  snapshot,
}: {
  readonly actions: RuntimeSettingsActions;
  readonly snapshot: Extract<RuntimeSettingsSnapshot, { status: "ready" }>;
}) {
  const [draftRuntimeId, setDraftRuntimeId] = useState<string | null>(
    snapshot.savedRuntimeId,
  );
  const priorSavedRuntimeId = useRef(snapshot.savedRuntimeId);

  useEffect(() => {
    const previousSaved = priorSavedRuntimeId.current;
    if (previousSaved === snapshot.savedRuntimeId) return;
    setDraftRuntimeId((current) =>
      current === previousSaved ? snapshot.savedRuntimeId : current,
    );
    priorSavedRuntimeId.current = snapshot.savedRuntimeId;
  }, [snapshot.savedRuntimeId]);

  const selectedRuntime = snapshot.runtimes.find(
    (runtime) => runtime.id === draftRuntimeId,
  );
  const isDirty = draftRuntimeId !== snapshot.savedRuntimeId;
  const isSaving = snapshot.save.status === "pending";
  const canSave =
    isDirty &&
    selectedRuntime !== undefined &&
    selectedRuntime.health !== "unavailable";

  if (snapshot.runtimes.length === 0) {
    return (
      <section
        className="runtime-settings-empty"
        aria-label="No runtimes available"
      >
        <span aria-hidden="true">R</span>
        <h2>No runtimes available</h2>
        <p>C4OS did not receive a verified OpenCode or Pi installation.</p>
      </section>
    );
  }

  return (
    <section
      aria-labelledby="runtime-settings-title"
      className="runtime-settings runtime-choice-settings"
      data-generation={snapshot.generation}
    >
      <header className="runtime-settings-heading">
        <div>
          <h2 id="runtime-settings-title">Default runtime</h2>
          <p>
            Choose the runtime captured by new Chats when their first valid
            submission is bound.
          </p>
        </div>
        <Button
          isDisabled={!canSave || isSaving}
          onPress={() => {
            if (draftRuntimeId !== null) {
              void actions.onSaveRuntime(draftRuntimeId, snapshot.generation);
            }
          }}
          variant="primary"
        >
          {isSaving ? "Saving Runtime…" : "Save Runtime"}
        </Button>
      </header>

      <Notice title="Existing Chat bindings do not change" tone="neutral">
        Existing Chats retain their current runtime bindings. The saved choice
        applies only to new Chats.
      </Notice>

      <SaveStatus save={snapshot.save} />

      <fieldset className="runtime-choice-grid">
        <legend>Runtime for new Chats</legend>
        {snapshot.runtimes.map((runtime) => (
          <RuntimeChoice
            isSelected={draftRuntimeId === runtime.id}
            key={runtime.id}
            onSelect={() => setDraftRuntimeId(runtime.id)}
            runtime={runtime}
            saved={snapshot.savedRuntimeId === runtime.id}
          />
        ))}
      </fieldset>

      <StatusRegion className="runtime-draft-status" data-dirty={isDirty}>
        {isDirty && selectedRuntime
          ? `${selectedRuntime.label} is selected as an unsaved draft.`
          : snapshot.savedRuntimeId === null
            ? "No runtime default is saved. Choose an available runtime to activate Save Runtime."
            : `${savedRuntimeLabel(snapshot)} is the saved default for new Chats.`}
      </StatusRegion>
    </section>
  );
}

function RuntimeChoice({
  isSelected,
  onSelect,
  runtime,
  saved,
}: {
  readonly isSelected: boolean;
  readonly onSelect: () => void;
  readonly runtime: RuntimeOptionView;
  readonly saved: boolean;
}) {
  return (
    <label
      className="runtime-choice-card"
      data-health={runtime.health}
      data-selected={isSelected}
    >
      <input
        checked={isSelected}
        disabled={runtime.health === "unavailable"}
        name="runtime-default"
        onChange={onSelect}
        type="radio"
        value={runtime.id}
      />
      <span className="runtime-choice-card__copy">
        <span className="runtime-choice-card__heading">
          <strong>
            {runtime.label} {runtime.version}
          </strong>
          <span data-state={runtime.health}>{healthLabel(runtime.health)}</span>
        </span>
        <span>{runtime.detail}</span>
        <small>
          {saved ? "Saved default" : "Available as a new-Chat default"}
        </small>
      </span>
    </label>
  );
}

function SaveStatus({
  save,
}: {
  readonly save: Extract<RuntimeSettingsSnapshot, { status: "ready" }>["save"];
}) {
  if (save.status === "error") {
    return (
      <Notice title="Runtime default was not saved" tone="danger">
        <p>{save.message}</p>
      </Notice>
    );
  }
  if (save.status === "success") {
    return (
      <Notice title="Runtime default saved" tone="success">
        <p>{save.message}</p>
      </Notice>
    );
  }
  return (
    <StatusRegion
      aria-busy={save.status === "pending"}
      className="runtime-operation-status"
      data-state={save.status}
    >
      {save.message}
    </StatusRegion>
  );
}

function savedRuntimeLabel(
  snapshot: Extract<RuntimeSettingsSnapshot, { status: "ready" }>,
) {
  return (
    snapshot.runtimes.find((runtime) => runtime.id === snapshot.savedRuntimeId)
      ?.label ?? "No runtime"
  );
}

function healthLabel(health: RuntimeOptionView["health"]) {
  return health.charAt(0).toLocaleUpperCase() + health.slice(1);
}

import { useEffect, useRef, useState } from "react";

import {
  Button,
  Notice,
  StatusRegion,
  Switch,
} from "../../../components/accessible";
import type {
  BrowserEnvironmentScope,
  ConfigurationApprovalPreset,
  ConfigurationOperationView,
  ConfigurationSettingsActions,
  ConfigurationSettingsSnapshot,
  ConfigurationValues,
} from "./types";

import "./configuration-settings.css";

export interface ConfigurationSettingsProps {
  readonly actions: ConfigurationSettingsActions;
  readonly snapshot: ConfigurationSettingsSnapshot;
}

const APPROVAL_PRESETS: readonly {
  readonly label: string;
  readonly value: ConfigurationApprovalPreset;
}[] = [
  { label: "Ask for approval", value: "askForApproval" },
  { label: "Approve safe actions", value: "approveSafeActions" },
  { label: "Approve for me", value: "approveForMe" },
  { label: "Custom", value: "custom" },
];

const BROWSER_ENVIRONMENTS: readonly {
  readonly detail: string;
  readonly label: string;
  readonly value: BrowserEnvironmentScope;
}[] = [
  {
    detail: "Share one persistent Browser Environment across C4OS.",
    label: "All browsers",
    value: "appWide",
  },
  {
    detail: "Keep one persistent environment for each Workspace and Project.",
    label: "Per project",
    value: "workspaceProject",
  },
  {
    detail: "Keep one persistent environment for each Chat session.",
    label: "Per chat session",
    value: "chat",
  },
  {
    detail: "Use an ephemeral environment destroyed when its Browser closes.",
    label: "None",
    value: "none",
  },
];

/** Renders Configuration from a service-owned snapshot and submits typed drafts. */
export function ConfigurationSettings({
  actions,
  snapshot,
}: ConfigurationSettingsProps) {
  if (snapshot.status === "loading") {
    return (
      <StatusRegion aria-busy="true" className="configuration-settings-state">
        <span aria-hidden="true" className="configuration-settings-spinner" />
        <span>
          <strong>Loading configuration</strong>
          <span>{snapshot.message}</span>
        </span>
      </StatusRegion>
    );
  }

  if (snapshot.status === "error") {
    return (
      <Notice title="Configuration is unavailable" tone="danger">
        <p>{snapshot.message}</p>
        {snapshot.retryable ? (
          <Button onPress={() => void actions.onRetry()}>Try again</Button>
        ) : null}
      </Notice>
    );
  }

  return <ConfigurationForm actions={actions} snapshot={snapshot} />;
}

function ConfigurationForm({
  actions,
  snapshot,
}: {
  readonly actions: ConfigurationSettingsActions;
  readonly snapshot: Extract<
    ConfigurationSettingsSnapshot,
    { status: "ready" }
  >;
}) {
  const [draft, setDraft] = useState<ConfigurationValues>(() => ({
    ...snapshot.saved,
  }));
  const priorSaved = useRef(snapshot.saved);

  useEffect(() => {
    const previous = priorSaved.current;
    setDraft((current) =>
      sameConfiguration(current, previous) ? { ...snapshot.saved } : current,
    );
    priorSaved.current = snapshot.saved;
  }, [snapshot.saved]);

  const isDirty = !sameConfiguration(draft, snapshot.saved);
  const isSaving = snapshot.save.status === "pending";
  const canSave = isDirty && !isSaving;

  function update<Key extends keyof ConfigurationValues>(
    key: Key,
    value: ConfigurationValues[Key],
  ) {
    setDraft((current) => ({ ...current, [key]: value }));
  }

  return (
    <section
      aria-labelledby="configuration-settings-title"
      className="configuration-settings"
      data-generation={snapshot.generation}
    >
      <header className="configuration-settings__heading">
        <div>
          <h2 id="configuration-settings-title">Application defaults</h2>
          <p>
            Review the policy and environment defaults C4OS applies when new
            work begins.
          </p>
        </div>
        <div className="configuration-settings__actions">
          <Button
            isDisabled={!isDirty || isSaving}
            onPress={() => setDraft({ ...snapshot.saved })}
            variant="quiet"
          >
            Revert
          </Button>
          <Button
            isDisabled={!canSave}
            onPress={() =>
              void actions.onSaveConfiguration(
                { ...draft },
                snapshot.generation,
              )
            }
            variant="primary"
          >
            {isSaving ? "Saving Configuration…" : "Save Configuration"}
          </Button>
        </div>
      </header>

      <StatusRegion
        className="configuration-live-state"
        data-source={snapshot.live.source}
      >
        <span className="configuration-live-state__badge">Live</span>
        <span>
          <strong>{snapshot.live.displayPath}</strong>
          <span>{snapshot.live.detail}</span>
        </span>
      </StatusRegion>

      <OperationStatus operation={snapshot.save} subject="Configuration" />

      <section
        aria-labelledby="configuration-runtime-title"
        className="configuration-card"
      >
        <header>
          <div>
            <h3 id="configuration-runtime-title">Runtime</h3>
            <p>Choose the policy and restore behavior for new work.</p>
          </div>
        </header>

        <div className="configuration-field configuration-field--policy">
          <label htmlFor="configuration-approval-preset">
            Default Approval Policy
          </label>
          <div>
            <select
              disabled={isSaving}
              id="configuration-approval-preset"
              onChange={(event) =>
                update(
                  "approvalPreset",
                  event.currentTarget.value as ConfigurationApprovalPreset,
                )
              }
              value={draft.approvalPreset}
            >
              {APPROVAL_PRESETS.map((preset) => (
                <option
                  disabled={preset.value === "custom"}
                  key={preset.value}
                  value={preset.value}
                >
                  {preset.label}
                </option>
              ))}
            </select>
            <Button
              isDisabled={isSaving}
              onPress={() => void actions.onNavigateAdvanced()}
              variant="quiet"
            >
              Advanced
            </Button>
          </div>
        </div>

        <Notice title="Approval guardrails remain active" tone="neutral">
          Approve for me remains bounded by the active sandbox, trusted roots,
          maximum authority, and managed policy. Custom reflects category rules
          or concrete exceptions that differ from a preset.
        </Notice>

        <Switch
          description="Reopen the last active Workspace when C4OS launches."
          isDisabled={isSaving}
          isSelected={draft.restoreLastWorkspace}
          label="Restore last workspace"
          onChange={(selected) => update("restoreLastWorkspace", selected)}
        />
      </section>

      <section
        aria-labelledby="configuration-environment-title"
        className="configuration-card"
      >
        <header>
          <div>
            <h3 id="configuration-environment-title">Environment</h3>
            <p>Control inherited shell names and browser storage boundaries.</p>
          </div>
        </header>

        <Switch
          description="Allow approved shell environment names to be inherited; raw values remain native-owned."
          isDisabled={isSaving}
          isSelected={draft.inheritShellEnvironment}
          label="Shell environment"
          onChange={(selected) => update("inheritShellEnvironment", selected)}
        />

        <fieldset className="configuration-browser-environments">
          <legend>Browser Environment</legend>
          <p>
            This boundary covers applicable cookies, session and local storage,
            and IndexedDB while preserving normal web-origin rules.
          </p>
          <div>
            {BROWSER_ENVIRONMENTS.map((environment) => (
              <label
                data-selected={draft.browserEnvironment === environment.value}
                key={environment.value}
              >
                <input
                  checked={draft.browserEnvironment === environment.value}
                  disabled={isSaving}
                  name="browser-environment"
                  onChange={() =>
                    update("browserEnvironment", environment.value)
                  }
                  type="radio"
                  value={environment.value}
                />
                <span>
                  <strong>{environment.label}</strong>
                  <small>{environment.detail}</small>
                </span>
              </label>
            ))}
          </div>
        </fieldset>

        <div className="configuration-file-action">
          <span>
            <strong>C4OS configuration file</strong>
            <span>Open {snapshot.live.displayPath} externally.</span>
          </span>
          <Button
            isDisabled={snapshot.openConfiguration.status === "pending"}
            onPress={() => void actions.onOpenConfigurationFile()}
          >
            {snapshot.openConfiguration.status === "pending"
              ? "Opening…"
              : "Open C4OS config"}
          </Button>
        </div>
        <OperationStatus
          operation={snapshot.openConfiguration}
          subject="C4OS config"
        />
      </section>

      <StatusRegion className="configuration-draft-state" data-dirty={isDirty}>
        {isDirty
          ? "Configuration has unsaved changes."
          : "Configuration matches the active saved values."}
      </StatusRegion>
    </section>
  );
}

function OperationStatus({
  operation,
  subject,
}: {
  readonly operation: ConfigurationOperationView;
  readonly subject: string;
}) {
  if (operation.status === "error") {
    return (
      <Notice title={`${subject} action failed`} tone="danger">
        {operation.message}
      </Notice>
    );
  }
  if (operation.status === "success") {
    return (
      <Notice title={`${subject} action completed`} tone="success">
        {operation.message}
      </Notice>
    );
  }
  return (
    <StatusRegion
      aria-busy={operation.status === "pending"}
      className="configuration-operation-state"
      data-state={operation.status}
    >
      {operation.message}
    </StatusRegion>
  );
}

function sameConfiguration(
  left: ConfigurationValues,
  right: ConfigurationValues,
) {
  return (
    left.approvalPreset === right.approvalPreset &&
    left.browserEnvironment === right.browserEnvironment &&
    left.inheritShellEnvironment === right.inheritShellEnvironment &&
    left.restoreLastWorkspace === right.restoreLastWorkspace
  );
}

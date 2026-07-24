import { useEffect, useRef, useState } from "react";

import {
  Button,
  ModalDialog,
  Notice,
  StatusRegion,
  Switch,
} from "../../../components/accessible";
import {
  deleteProviderProfile,
  saveProviderProfile,
  selectProviderModel,
  testProviderConnection,
  type ProviderRecord,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";
import { ProviderProfileForm } from "./ProviderProfileForm";
import {
  ProviderApprovalNotice,
  type ProviderApprovalDecision,
} from "./ProviderApprovalNotice";
import { ProviderCredentialFallbackNotice } from "./ProviderCredentialFallbackNotice";
import {
  draftFromProvider,
  isConfigurableProvider,
  providerKindLabel,
  providerProfileForSave,
} from "./provider-profile";
import { useProviderProfile } from "./use-provider-profile";
import { useProviderSnapshot } from "./use-provider-snapshot";

/** Composes Provider Settings against native snapshots and mutations. */
export function ProviderSettings() {
  const providerSnapshot = useProviderSnapshot();

  if (providerSnapshot.state.status === "loading") {
    return (
      <StatusRegion className="provider-surface-state">
        <strong>Loading providers</strong>
        <span>C4OS is reading configured provider profiles.</span>
      </StatusRegion>
    );
  }
  if (providerSnapshot.state.status === "error") {
    return (
      <Notice
        className="provider-surface-state"
        title="Providers are unavailable"
        tone="danger"
      >
        <p>{providerSnapshot.state.message}</p>
        <Button onPress={providerSnapshot.retry}>Try again</Button>
      </Notice>
    );
  }

  return (
    <ProviderSettingsReady
      onSnapshot={providerSnapshot.publish}
      snapshot={providerSnapshot.state.snapshot}
    />
  );
}

/** Renders the provider list while the native snapshot remains authoritative. */
function ProviderSettingsReady({
  onSnapshot,
  snapshot,
}: {
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly snapshot: ProviderSettingsSnapshot;
}) {
  const [activeMutationOwner, setActiveMutationOwner] = useState<string | null>(
    null,
  );
  const activeMutationOwnerRef = useRef<string | null>(null);
  const approvalReturnFocusRef = useRef<HTMLHeadingElement>(null);
  const previousApprovalPromptRef = useRef<string | null>(
    snapshot.pendingApproval?.promptId ?? null,
  );
  const [operationError, setOperationError] = useState<string | null>(null);
  const [testAfterSaveApprovalProviderId, setTestAfterSaveApprovalProviderId] =
    useState<string | null>(null);

  useEffect(() => {
    const currentPromptId = snapshot.pendingApproval?.promptId ?? null;
    if (
      previousApprovalPromptRef.current !== null &&
      currentPromptId === null
    ) {
      queueMicrotask(() => approvalReturnFocusRef.current?.focus());
    }
    previousApprovalPromptRef.current = currentPromptId;
  }, [snapshot.pendingApproval]);

  /** Serializes every provider mutation, including mutations started in dialogs. */
  async function withMutationLock<Result>(
    owner: string,
    operation: () => Promise<Result>,
  ): Promise<Result> {
    if (activeMutationOwnerRef.current !== null) {
      throw new Error("Another provider change is already in progress.");
    }
    activeMutationOwnerRef.current = owner;
    setActiveMutationOwner(owner);
    try {
      return await operation();
    } finally {
      activeMutationOwnerRef.current = null;
      setActiveMutationOwner(null);
    }
  }

  /** Applies one provider mutation and publishes only its returned snapshot. */
  async function mutate(
    owner: string,
    operation: () => Promise<ProviderSettingsSnapshot>,
  ): Promise<boolean> {
    setOperationError(null);
    try {
      const next = await withMutationLock(owner, operation);
      onSnapshot(next);
      return next.pendingApproval === null;
    } catch (error) {
      setOperationError(providerMessage(error));
      return false;
    }
  }

  /** Preserves the opaque credential while changing availability. */
  function setEnabled(provider: ProviderRecord, enabled: boolean) {
    if (!isConfigurableProvider(provider)) return;
    const draft = { ...draftFromProvider(provider), enabled };
    void mutate(`toggle:${provider.providerId}`, () =>
      saveProviderProfile(providerProfileForSave(draft, snapshot.providers)),
    );
  }

  const mutationPending =
    activeMutationOwner !== null || snapshot.pendingApproval !== null;

  async function continueTestAfterSaveApproval(
    decision: ProviderApprovalDecision,
    next: ProviderSettingsSnapshot,
  ) {
    const providerId = testAfterSaveApprovalProviderId;
    setTestAfterSaveApprovalProviderId(null);
    if (decision !== "allow" || providerId === null) return;
    if (next.pendingApproval !== null) {
      setOperationError(
        "The Provider save approval did not settle before its connection test.",
      );
      return;
    }
    setOperationError(null);
    try {
      const tested = await withMutationLock(
        `test-after-save:${providerId}`,
        () => testProviderConnection(providerId),
      );
      onSnapshot(tested);
    } catch (error) {
      setOperationError(providerMessage(error));
    }
  }

  return (
    <div
      aria-busy={mutationPending}
      className="provider-settings"
      data-generation={snapshot.generation}
    >
      <div className="provider-settings__heading">
        <div>
          <h2 ref={approvalReturnFocusRef} tabIndex={-1}>
            Provider profiles
          </h2>
          <p>
            Credentials stay in secure native storage. Settings receives only
            opaque credential presence and bounded connection evidence.
          </p>
        </div>
        <ProviderDialog
          isLocked={
            mutationPending && activeMutationOwner !== "dialog:add-provider"
          }
          mutationOwner="dialog:add-provider"
          onTestAfterSaveApproval={setTestAfterSaveApprovalProviderId}
          onSnapshot={onSnapshot}
          snapshot={snapshot}
          withMutationLock={withMutationLock}
        />
      </div>

      <ProviderCredentialFallbackNotice
        onSnapshot={onSnapshot}
        snapshot={snapshot}
      />

      <ProviderApprovalNotice
        {...(testAfterSaveApprovalProviderId === null
          ? {}
          : { onAnswered: continueTestAfterSaveApproval })}
        onSnapshot={onSnapshot}
        snapshot={snapshot}
      />

      {operationError ? (
        <Notice title="Provider change was not applied" tone="danger">
          {operationError}
        </Notice>
      ) : null}

      {snapshot.providers.length === 0 ? (
        <section
          className="provider-empty"
          aria-labelledby="provider-empty-title"
        >
          <span aria-hidden="true">AI</span>
          <h2 id="provider-empty-title">No providers configured</h2>
          <p>Add and test a provider before choosing models for C4OS.</p>
        </section>
      ) : (
        <div
          className="provider-list"
          role="list"
          aria-label="Provider profiles"
        >
          {snapshot.providers.map((provider) => (
            <ProviderRow
              activeMutationOwner={activeMutationOwner}
              isBusy={mutationPending}
              key={provider.providerId}
              mutate={mutate}
              onSnapshot={onSnapshot}
              provider={provider}
              setEnabled={setEnabled}
              snapshot={snapshot}
              onTestAfterSaveApproval={setTestAfterSaveApprovalProviderId}
              withMutationLock={withMutationLock}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/** Renders one provider profile with CRUD, test, and model actions. */
function ProviderRow({
  activeMutationOwner,
  isBusy,
  mutate,
  onSnapshot,
  onTestAfterSaveApproval,
  provider,
  setEnabled,
  snapshot,
  withMutationLock,
}: {
  readonly activeMutationOwner: string | null;
  readonly isBusy: boolean;
  readonly mutate: (
    providerId: string,
    operation: () => Promise<ProviderSettingsSnapshot>,
  ) => Promise<boolean>;
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly onTestAfterSaveApproval: (providerId: string) => void;
  readonly provider: ProviderRecord;
  readonly setEnabled: (provider: ProviderRecord, enabled: boolean) => void;
  readonly snapshot: ProviderSettingsSnapshot;
  readonly withMutationLock: ProviderMutationLock;
}) {
  const configurable = isConfigurableProvider(provider);
  const editMutationOwner = `dialog:edit:${provider.providerId}`;
  const deleteMutationOwner = `dialog:delete:${provider.providerId}`;
  return (
    <article
      aria-busy={isBusy}
      className="provider-row"
      data-enabled={provider.enabled}
      role="listitem"
    >
      <div className="provider-row__identity">
        <span aria-hidden="true" className="provider-row__mark">
          {providerKindLabel(provider.kind).slice(0, 1)}
        </span>
        <div>
          <h3>{provider.displayName}</h3>
          <p>{providerKindLabel(provider.kind)}</p>
        </div>
      </div>

      <div className="provider-row__state">
        <span data-state={provider.testStatus.state}>
          {providerTestLabel(provider)}
        </span>
        <span>
          {
            provider.models.filter(({ productionReady }) => productionReady)
              .length
          }{" "}
          production-ready models
        </span>
      </div>

      <div className="provider-row__actions">
        <Switch
          aria-label={`${provider.displayName} availability`}
          description="Updates provider-derived model availability."
          isDisabled={isBusy || !configurable}
          isSelected={provider.enabled}
          label={`${provider.displayName} availability`}
          onChange={(enabled) => setEnabled(provider, enabled)}
        />
        {configurable ? (
          <ProviderDialog
            initialProvider={provider}
            isLocked={isBusy && activeMutationOwner !== editMutationOwner}
            mutationOwner={editMutationOwner}
            onTestAfterSaveApproval={onTestAfterSaveApproval}
            onSnapshot={onSnapshot}
            snapshot={snapshot}
            withMutationLock={withMutationLock}
          />
        ) : null}
        <Button
          aria-label={`Test ${provider.displayName}`}
          isDisabled={
            isBusy ||
            (!provider.hasCredential &&
              provider.authentication.type !== "none") ||
            (snapshot.credentialFallbackRequired &&
              provider.authentication.type !== "none")
          }
          onPress={() =>
            void mutate(`test:${provider.providerId}`, () =>
              testProviderConnection(provider.providerId),
            )
          }
        >
          Test
        </Button>
        <DeleteProviderDialog
          isBusy={isBusy}
          isLocked={isBusy && activeMutationOwner !== deleteMutationOwner}
          mutate={mutate}
          mutationOwner={deleteMutationOwner}
          provider={provider}
        />
      </div>

      {provider.models.length > 0 ? (
        <label className="provider-row__model">
          <span>Selected model</span>
          <select
            aria-label={`${provider.displayName} selected model`}
            disabled={isBusy}
            onChange={(event) => {
              const modelId = event.currentTarget.value;
              if (!modelId) return;
              void mutate(`model:${provider.providerId}`, () =>
                selectProviderModel(provider.providerId, modelId),
              );
            }}
            value={provider.selectedModelId ?? ""}
          >
            <option value="">Choose a model</option>
            {provider.models.map((model) => (
              <option
                disabled={!model.productionReady}
                key={model.modelId}
                value={model.modelId}
              >
                {model.displayName}
                {model.productionReady ? "" : " — unavailable"}
              </option>
            ))}
          </select>
        </label>
      ) : null}
    </article>
  );
}

/** Shares one Add/Edit dialog and its connection-test rules with onboarding. */
function ProviderDialog({
  initialProvider,
  isLocked,
  mutationOwner,
  onTestAfterSaveApproval,
  onSnapshot,
  snapshot,
  withMutationLock,
}: {
  readonly initialProvider?: ProviderRecord;
  readonly isLocked: boolean;
  readonly mutationOwner: string;
  readonly onTestAfterSaveApproval: (providerId: string) => void;
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly snapshot: ProviderSettingsSnapshot;
  readonly withMutationLock: ProviderMutationLock;
}) {
  const controller = useProviderProfile({
    ...(initialProvider ? { initialProvider } : {}),
    onTestAfterSaveApproval,
    onSnapshot,
    snapshot,
  });
  const editing = initialProvider !== undefined;
  const triggerLabel = editing
    ? `Edit ${initialProvider.displayName}`
    : "Add Provider";
  if (isLocked) {
    return (
      <Button isDisabled variant={editing ? "secondary" : "primary"}>
        {triggerLabel}
      </Button>
    );
  }
  const runDialogMutation = async <Result,>(
    operation: () => Promise<Result>,
  ): Promise<Result | undefined> => {
    try {
      return await withMutationLock(mutationOwner, operation);
    } catch {
      // The surface lock makes this reachable only for a same-frame duplicate.
      return undefined;
    }
  };
  const guardedController = {
    ...controller,
    selectModel: async (modelId: string) => {
      await runDialogMutation(() => controller.selectModel(modelId));
    },
  };
  const credentialBlocked =
    snapshot.credentialFallbackRequired &&
    controller.draft.authenticationType !== "none";
  const dismissLabel = controller.isPersisted ? "Close" : "Cancel";
  return (
    <ModalDialog
      closeLabel={dismissLabel}
      isDismissable={!controller.isBusy}
      renderActions={(close) => (
        <>
          <Button
            isDisabled={controller.isBusy}
            onPress={() => {
              controller.reset();
              close();
            }}
          >
            {dismissLabel}
          </Button>
          <Button
            isDisabled={controller.isBusy || credentialBlocked}
            onPress={() => void runDialogMutation(controller.testConnection)}
          >
            {controller.operation === "testing"
              ? "Testing…"
              : "Save & Test Connection"}
          </Button>
          <Button
            isDisabled={controller.isBusy || credentialBlocked}
            onPress={() => {
              void runDialogMutation(controller.save).then((saved) => {
                if (saved) {
                  controller.reset();
                  close();
                }
              });
            }}
            variant="primary"
          >
            {controller.operation === "saving" ? "Saving…" : "Save Profile"}
          </Button>
        </>
      )}
      title={
        initialProvider ? `Edit ${initialProvider.displayName}` : "Add Provider"
      }
      triggerLabel={triggerLabel}
      triggerVariant={editing ? "secondary" : "primary"}
    >
      <ProviderProfileForm
        controller={guardedController}
        mode="settings"
        onDismiss={controller.reset}
      />
    </ModalDialog>
  );
}

/** Confirms deletion before asking the native service to remove a profile. */
function DeleteProviderDialog({
  isBusy,
  isLocked,
  mutate,
  mutationOwner,
  provider,
}: {
  readonly isBusy: boolean;
  readonly isLocked: boolean;
  readonly mutate: (
    providerId: string,
    operation: () => Promise<ProviderSettingsSnapshot>,
  ) => Promise<boolean>;
  readonly mutationOwner: string;
  readonly provider: ProviderRecord;
}) {
  const triggerLabel = `Delete ${provider.displayName}`;
  if (isLocked) {
    return (
      <Button isDisabled variant="danger">
        {triggerLabel}
      </Button>
    );
  }
  return (
    <ModalDialog
      renderActions={(close) => (
        <>
          <Button onPress={close}>Cancel</Button>
          <Button
            isDisabled={isBusy}
            onPress={() => {
              void mutate(mutationOwner, () =>
                deleteProviderProfile(provider.providerId),
              ).then((deleted) => {
                if (deleted) close();
              });
            }}
            variant="danger"
          >
            Delete Provider
          </Button>
        </>
      )}
      title={`Delete ${provider.displayName}?`}
      triggerLabel={triggerLabel}
      triggerVariant="danger"
    >
      <p>
        This removes the provider profile and its model availability. The native
        credential service applies the corresponding secure-storage policy.
      </p>
    </ModalDialog>
  );
}

type ProviderMutationLock = <Result>(
  owner: string,
  operation: () => Promise<Result>,
) => Promise<Result>;

/** Converts frozen test states into concise provider-row status labels. */
function providerTestLabel(provider: ProviderRecord): string {
  switch (provider.testStatus.state) {
    case "untested":
      return "Not tested";
    case "succeeded":
      return "Connection passed";
    case "succeededNoUsableModels":
      return "Connected · no usable models";
    case "failed":
      return "Connection failed";
  }
}

/** Keeps provider mutations bounded to service-authored renderer messages. */
function providerMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Provider service could not complete the request.";
}

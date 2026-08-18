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
  type ProviderRecord,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";
import { ProviderProfileForm } from "./ProviderProfileForm";
import { ProviderApprovalNotice } from "./ProviderApprovalNotice";
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
  const approvalReturnFocusRef = useRef<HTMLDivElement>(null);
  const previousApprovalPromptRef = useRef<string | null>(
    snapshot.pendingApproval?.promptId ?? null,
  );
  const [operationError, setOperationError] = useState<string | null>(null);

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

  return (
    <div
      aria-label="Provider settings content"
      aria-busy={mutationPending}
      className="provider-settings"
      data-generation={snapshot.generation}
      ref={approvalReturnFocusRef}
      role="region"
      tabIndex={-1}
    >
      <div className="provider-settings__actions">
        <ProviderDialog
          isLocked={
            mutationPending && activeMutationOwner !== "dialog:add-provider"
          }
          mutationOwner="dialog:add-provider"
          mutate={mutate}
          onSnapshot={onSnapshot}
          snapshot={snapshot}
          withMutationLock={withMutationLock}
        />
      </div>

      <ProviderCredentialFallbackNotice
        onSnapshot={onSnapshot}
        snapshot={snapshot}
      />

      <ProviderApprovalNotice onSnapshot={onSnapshot} snapshot={snapshot} />

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
              withMutationLock={withMutationLock}
            />
          ))}
        </div>
      )}
    </div>
  );
}

/** Renders the compact provider identity, availability, and Edit boundary. */
function ProviderRow({
  activeMutationOwner,
  isBusy,
  mutate,
  onSnapshot,
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
  readonly provider: ProviderRecord;
  readonly setEnabled: (provider: ProviderRecord, enabled: boolean) => void;
  readonly snapshot: ProviderSettingsSnapshot;
  readonly withMutationLock: ProviderMutationLock;
}) {
  const configurable = isConfigurableProvider(provider);
  const editMutationOwner = `dialog:edit:${provider.providerId}`;
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
            mutate={mutate}
            mutationOwner={editMutationOwner}
            onSnapshot={onSnapshot}
            snapshot={snapshot}
            withMutationLock={withMutationLock}
          />
        ) : null}
      </div>
    </article>
  );
}

/** Shares one Add/Edit dialog and its connection-test rules with onboarding. */
function ProviderDialog({
  initialProvider,
  isLocked,
  mutate,
  mutationOwner,
  onSnapshot,
  snapshot,
  withMutationLock,
}: {
  readonly initialProvider?: ProviderRecord;
  readonly isLocked: boolean;
  readonly mutate: (
    providerId: string,
    operation: () => Promise<ProviderSettingsSnapshot>,
  ) => Promise<boolean>;
  readonly mutationOwner: string;
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly snapshot: ProviderSettingsSnapshot;
  readonly withMutationLock: ProviderMutationLock;
}) {
  const controller = useProviderProfile({
    ...(initialProvider ? { initialProvider } : {}),
    onSnapshot,
    snapshot,
  });
  const [confirmingDelete, setConfirmingDelete] = useState(false);
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
      renderActions={(close) =>
        confirmingDelete && initialProvider ? (
          <>
            <Button onPress={() => setConfirmingDelete(false)}>
              Keep Provider
            </Button>
            <Button
              onPress={() => {
                void mutate(`${mutationOwner}:delete`, () =>
                  deleteProviderProfile(initialProvider.providerId),
                ).then(() => {
                  setConfirmingDelete(false);
                  close();
                });
              }}
              variant="danger"
            >
              Delete Provider
            </Button>
          </>
        ) : (
          <>
            {initialProvider ? (
              <Button
                isDisabled={controller.isBusy}
                onPress={() => setConfirmingDelete(true)}
                variant="danger"
              >
                Delete…
              </Button>
            ) : null}
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
                : "Test Connection"}
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
        )
      }
      title={
        confirmingDelete && initialProvider
          ? `Delete ${initialProvider.displayName}?`
          : initialProvider
            ? `Edit ${initialProvider.displayName}`
            : "Add Provider"
      }
      triggerLabel={triggerLabel}
      triggerVariant={editing ? "secondary" : "primary"}
    >
      {confirmingDelete && initialProvider ? (
        <p>
          This removes the provider and its model availability. This action
          requires destructive confirmation.
        </p>
      ) : (
        <ProviderProfileForm
          controller={guardedController}
          mode="settings"
          onDismiss={controller.reset}
        />
      )}
    </ModalDialog>
  );
}

type ProviderMutationLock = <Result>(
  owner: string,
  operation: () => Promise<Result>,
) => Promise<Result>;

/** Keeps provider mutations bounded to service-authored renderer messages. */
function providerMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Provider service could not complete the request.";
}

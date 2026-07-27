import { useRef, useState } from "react";

import { Button, Notice, StatusRegion } from "../../../components/accessible";
import {
  testProviderConnection,
  type ProviderRecord,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";
import { ProviderCredentialFallbackNotice } from "./ProviderCredentialFallbackNotice";
import {
  ProviderApprovalNotice,
  type ProviderApprovalDecision,
} from "./ProviderApprovalNotice";
import { ProviderProfileForm } from "./ProviderProfileForm";
import { isConfigurableProvider } from "./provider-profile";
import { useProviderProfile } from "./use-provider-profile";
import { useProviderSnapshot } from "./use-provider-snapshot";

export type ProviderOnboardingProps = {
  readonly onComplete: () => void;
};

/** Composes first-provider setup directly against the native Provider service. */
export function ProviderOnboarding({ onComplete }: ProviderOnboardingProps) {
  const providerSnapshot = useProviderSnapshot();

  if (providerSnapshot.state.status === "loading") {
    return (
      <StatusRegion className="provider-surface-state">
        <strong>Loading provider setup</strong>
        <span>C4OS is reading the authoritative Provider service.</span>
      </StatusRegion>
    );
  }
  if (providerSnapshot.state.status === "error") {
    return (
      <Notice
        className="provider-surface-state"
        title="Provider setup is unavailable"
        tone="danger"
      >
        <p>{providerSnapshot.state.message}</p>
        <Button onPress={providerSnapshot.retry}>Try again</Button>
      </Notice>
    );
  }

  return (
    <ProviderOnboardingReady
      onComplete={onComplete}
      onSnapshot={providerSnapshot.publish}
      snapshot={providerSnapshot.state.snapshot}
    />
  );
}

/** Renders the current onboarding profile without replacing the app document. */
function ProviderOnboardingReady({
  onComplete,
  onSnapshot,
  snapshot,
}: {
  readonly onComplete: () => void;
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly snapshot: ProviderSettingsSnapshot;
}) {
  const [approvalContinuationError, setApprovalContinuationError] = useState<
    string | null
  >(null);
  const [isContinuingApprovedTest, setIsContinuingApprovedTest] =
    useState(false);
  const testAfterSaveApprovalProviderIdRef = useRef<string | null>(null);
  const initialProvider = firstConfigurableProvider(snapshot.providers);
  const controller = useProviderProfile({
    ...(initialProvider ? { initialProvider } : {}),
    onComplete,
    onSnapshot,
    onTestAfterSaveApproval(providerId) {
      setApprovalContinuationError(null);
      testAfterSaveApprovalProviderIdRef.current = providerId;
    },
    snapshot,
  });

  /** Resumes the connection test whose credential save just received approval. */
  async function continueTestAfterSaveApproval(
    decision: ProviderApprovalDecision,
    next: ProviderSettingsSnapshot,
  ) {
    const providerId = testAfterSaveApprovalProviderIdRef.current;
    testAfterSaveApprovalProviderIdRef.current = null;
    if (decision !== "allow" || providerId === null) return;
    if (next.pendingApproval !== null) {
      setApprovalContinuationError(
        "The Provider save approval did not settle before its connection test.",
      );
      return;
    }
    setApprovalContinuationError(null);
    setIsContinuingApprovedTest(true);
    try {
      onSnapshot(await testProviderConnection(providerId));
    } catch (error) {
      setApprovalContinuationError(providerMessage(error));
    } finally {
      setIsContinuingApprovedTest(false);
    }
  }

  return (
    <section
      aria-busy={controller.isBusy || isContinuingApprovedTest}
      className="provider-onboarding"
      aria-label="Connect provider"
    >
      <header className="provider-onboarding__heading">
        <span>C4OS</span>
        <h1>Connect your AI provider</h1>
        <p>Add a provider to choose the models C4OS can use.</p>
      </header>

      {snapshot.onboardingCompleted ? (
        <Notice title="Provider setup already completed" tone="neutral">
          You can review the current profile here or continue to Workspace
          Start.
        </Notice>
      ) : null}

      <ProviderCredentialFallbackNotice
        onSnapshot={onSnapshot}
        snapshot={snapshot}
      />

      <ProviderApprovalNotice
        onAnswered={continueTestAfterSaveApproval}
        onSnapshot={onSnapshot}
        snapshot={snapshot}
      />

      {approvalContinuationError ? (
        <Notice title="Provider setup was not completed" tone="danger">
          {approvalContinuationError}
        </Notice>
      ) : null}

      <ProviderProfileForm controller={controller} mode="onboarding" />

      <div className="provider-onboarding__actions">
        <Button
          isDisabled={
            controller.isBusy ||
            isContinuingApprovedTest ||
            snapshot.pendingApproval !== null ||
            (snapshot.credentialFallbackRequired &&
              controller.draft.authenticationType !== "none")
          }
          onPress={() => void controller.testConnection()}
        >
          {controller.operation === "testing"
            ? "Testing Connection…"
            : "Test Connection"}
        </Button>
        <Button
          isDisabled={
            !controller.canContinue ||
            isContinuingApprovedTest ||
            snapshot.pendingApproval !== null
          }
          onPress={() => void controller.continueOnboarding()}
          variant="primary"
        >
          {controller.operation === "completing" ? "Continuing…" : "Continue"}
        </Button>
      </div>
    </section>
  );
}

/** Finds the resumable configurable provider without assuming fixture order. */
function firstConfigurableProvider(
  providers: readonly ProviderRecord[],
): ProviderRecord | undefined {
  return providers.find(isConfigurableProvider);
}

/** Keeps provider failures bounded to service-authored renderer messages. */
function providerMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Provider service could not complete the request.";
}

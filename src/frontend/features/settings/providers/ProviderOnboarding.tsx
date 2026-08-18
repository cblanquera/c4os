import {
  BrandMark,
  Button,
  Notice,
  StatusRegion,
} from "../../../components/accessible";
import {
  type ProviderRecord,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";
import { ProviderCredentialFallbackNotice } from "./ProviderCredentialFallbackNotice";
import { ProviderApprovalNotice } from "./ProviderApprovalNotice";
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
  const initialProvider = firstConfigurableProvider(snapshot.providers);
  const controller = useProviderProfile({
    ...(initialProvider ? { initialProvider } : {}),
    onComplete,
    onSnapshot,
    snapshot,
  });

  return (
    <section
      aria-busy={controller.isBusy}
      className="provider-onboarding"
      aria-label="Connect provider"
    >
      <div className="provider-onboarding__brand" aria-label="C4OS">
        <BrandMark size={28} />
        <strong>C4OS</strong>
      </div>
      <header className="provider-onboarding__heading">
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

      <ProviderApprovalNotice onSnapshot={onSnapshot} snapshot={snapshot} />

      <ProviderProfileForm
        actions={
          <>
            <Button
              isDisabled={
                controller.isBusy ||
                snapshot.pendingApproval !== null ||
                (snapshot.credentialFallbackRequired &&
                  controller.draft.authenticationType !== "none")
              }
              onPress={() => void controller.testConnection()}
              type="button"
            >
              {controller.operation === "testing"
                ? "Testing Connection…"
                : "Test Connection"}
            </Button>
            <Button
              isDisabled={
                !controller.canContinue || snapshot.pendingApproval !== null
              }
              type="submit"
              variant="primary"
            >
              {controller.operation === "completing"
                ? "Continuing…"
                : "Continue"}
            </Button>
          </>
        }
        controller={controller}
        mode="onboarding"
        onSubmit={() => void controller.continueOnboarding()}
      />
    </section>
  );
}

/** Finds the resumable configurable provider without assuming fixture order. */
function firstConfigurableProvider(
  providers: readonly ProviderRecord[],
): ProviderRecord | undefined {
  return providers.find(isConfigurableProvider);
}

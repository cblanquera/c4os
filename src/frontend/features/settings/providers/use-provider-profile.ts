import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import {
  completeProviderOnboarding,
  saveProviderProfile,
  selectProviderModel,
  testProviderConnection,
  type ConfigurableProviderKind,
  type ProviderAuthentication,
  type ProviderModel,
  type ProviderRecord,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";
import {
  createProviderDraft,
  draftForProviderKind,
  draftFromProvider,
  providerProfileForSave,
  providerTestFingerprint,
  recommendedProviderModel,
  validateProviderDraft,
  type ProviderFormDraft,
  type ProviderFormErrors,
} from "./provider-profile";

export type ProviderProfileOperation =
  "idle" | "saving" | "selectingModel" | "testing" | "completing";

export type ProviderConnectionView = {
  readonly detail: string;
  readonly state:
    | "failure"
    | "stale"
    | "submitting"
    | "success"
    | "testing"
    | "untested"
    | "zero";
  readonly title: string;
};

export type ProviderProfileController = {
  readonly canContinue: boolean;
  readonly connection: ProviderConnectionView;
  readonly draft: ProviderFormDraft;
  readonly errors: ProviderFormErrors;
  readonly isBusy: boolean;
  readonly isPersisted: boolean;
  readonly models: readonly ProviderModel[];
  readonly operation: ProviderProfileOperation;
  readonly operationError: string | null;
  readonly selectedModelId: string;
  readonly showValidation: boolean;
  readonly changeAuthentication: (
    authentication: ProviderAuthentication["type"],
  ) => void;
  readonly changeKind: (kind: ConfigurableProviderKind) => void;
  readonly changeSecret: (secret: string) => void;
  readonly changeValue: <Key extends keyof ProviderFormDraft>(
    key: Key,
    value: ProviderFormDraft[Key],
  ) => void;
  readonly continueOnboarding: () => Promise<boolean>;
  readonly reset: () => void;
  readonly save: () => Promise<boolean>;
  readonly selectModel: (modelId: string) => Promise<void>;
  readonly testConnection: () => Promise<void>;
};

type UseProviderProfileOptions = {
  readonly initialProvider?: ProviderRecord;
  readonly onComplete?: () => void;
  readonly onTestAfterSaveApproval?: (providerId: string) => void;
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly snapshot: ProviderSettingsSnapshot;
};

/** Owns one provider draft while native snapshots remain authoritative. */
export function useProviderProfile({
  initialProvider,
  onComplete,
  onSnapshot,
  snapshot,
}: UseProviderProfileOptions): ProviderProfileController {
  const initialDraft = initialProvider
    ? draftFromProvider(initialProvider)
    : createProviderDraft();
  const [draft, setDraft] = useState<ProviderFormDraft>(initialDraft);
  const [selectedModelId, setSelectedModelId] = useState(
    initialProvider?.selectedModelId ??
      recommendedProviderModel(initialProvider?.models ?? [])?.modelId ??
      "",
  );
  const [lastTestFingerprint, setLastTestFingerprint] = useState<string | null>(
    initialProvider && initialProvider.testStatus.state !== "untested"
      ? providerTestFingerprint(initialDraft)
      : null,
  );
  const [operation, setOperation] = useState<ProviderProfileOperation>("idle");
  const [operationError, setOperationError] = useState<string | null>(null);
  const [showValidation, setShowValidation] = useState(false);
  const [syncedRecordGeneration, setSyncedRecordGeneration] = useState<
    number | null
  >(initialProvider?.generation ?? null);
  const [syncedTransientToken, setSyncedTransientToken] = useState<
    string | null
  >(null);
  const operationInFlight = useRef(false);

  const errors = useMemo(
    () => validateProviderDraft(draft, snapshot.providers),
    [draft, snapshot.providers],
  );
  const currentProvider = snapshot.providers.find(
    ({ providerId }) => providerId === draft.providerId,
  );
  const transientTest =
    snapshot.transientTest !== null &&
    (draft.providerId.length === 0 ||
      snapshot.transientTest.provider.providerId === draft.providerId)
      ? snapshot.transientTest
      : null;
  const activeProvider =
    transientTest?.provider ?? currentProvider ?? initialProvider;
  const models = activeProvider?.models ?? [];
  const selectedModel = models.find(
    ({ modelId }) => modelId === selectedModelId,
  );
  const fingerprint = providerTestFingerprint(draft);
  const isFresh =
    lastTestFingerprint !== null && lastTestFingerprint === fingerprint;
  const connection = providerConnectionView(
    activeProvider,
    isFresh,
    operation,
    operationError,
  );
  const hasFreshSuccessfulTest =
    transientTest !== null &&
    isFresh &&
    activeProvider?.testStatus.state === "succeeded";
  const canContinue =
    operation === "idle" &&
    hasFreshSuccessfulTest &&
    selectedModel?.productionReady === true &&
    Object.keys(errors).length === 0 &&
    (draft.authenticationType === "none" ||
      activeProvider?.hasCredential === true);

  useEffect(() => {
    const nextTransient = snapshot.transientTest;
    if (
      nextTransient === null ||
      operation !== "idle" ||
      nextTransient.testToken === syncedTransientToken
    ) {
      return;
    }
    let active = true;
    window.queueMicrotask(() => {
      if (!active) return;
      const nextDraft = draftFromProvider(nextTransient.provider);
      const recommended = recommendedProviderModel(
        nextTransient.provider.models,
      );
      const selected = nextTransient.provider.models.some(
        (model) =>
          model.modelId === nextTransient.provider.selectedModelId &&
          model.productionReady,
      )
        ? nextTransient.provider.selectedModelId
        : (recommended?.modelId ?? "");
      setDraft(nextDraft);
      setSelectedModelId(selected ?? "");
      setLastTestFingerprint(providerTestFingerprint(nextDraft));
      setSyncedTransientToken(nextTransient.testToken);
    });
    return () => {
      active = false;
    };
  }, [operation, snapshot.transientTest, syncedTransientToken]);

  useEffect(() => {
    if (
      !initialProvider ||
      operation !== "idle" ||
      initialProvider.generation === syncedRecordGeneration
    ) {
      return;
    }
    let active = true;
    window.queueMicrotask(() => {
      if (!active) return;
      const nextDraft = draftFromProvider(initialProvider);
      setDraft(nextDraft);
      setSelectedModelId(
        initialProvider.selectedModelId ??
          recommendedProviderModel(initialProvider.models)?.modelId ??
          "",
      );
      setLastTestFingerprint(
        initialProvider.testStatus.state === "untested"
          ? null
          : providerTestFingerprint(nextDraft),
      );
      setSyncedRecordGeneration(initialProvider.generation);
    });
    return () => {
      active = false;
    };
  }, [initialProvider, operation, syncedRecordGeneration]);

  /** Updates one ordinary field and clears only transient operation feedback. */
  function changeValue<Key extends keyof ProviderFormDraft>(
    key: Key,
    value: ProviderFormDraft[Key],
  ) {
    setDraft((current) => ({ ...current, [key]: value }));
    setOperationError(null);
  }

  /** Changes provider family and drops secret input from the prior family. */
  function changeKind(kind: ConfigurableProviderKind) {
    setDraft((current) => draftForProviderKind(current, kind));
    setSelectedModelId("");
    setOperationError(null);
  }

  /** Changes conditional authentication without retaining an irrelevant key. */
  function changeAuthentication(
    authenticationType: ProviderAuthentication["type"],
  ) {
    setDraft((current) => ({
      ...current,
      authenticationType,
      credentialRevision: current.credentialRevision + 1,
      secret: authenticationType === "none" ? "" : current.secret,
    }));
    setOperationError(null);
  }

  /** Tracks secret edits by revision while keeping raw text out of fingerprints. */
  function changeSecret(secret: string) {
    setDraft((current) => ({
      ...current,
      credentialRevision: current.credentialRevision + 1,
      secret,
    }));
    setOperationError(null);
  }

  /** Restores the authoritative edit record or a new blank Add draft. */
  const reset = useCallback(() => {
    const nextDraft = initialProvider
      ? draftFromProvider(initialProvider)
      : createProviderDraft();
    setDraft(nextDraft);
    setSelectedModelId(
      initialProvider?.selectedModelId ??
        recommendedProviderModel(initialProvider?.models ?? [])?.modelId ??
        "",
    );
    setLastTestFingerprint(
      initialProvider && initialProvider.testStatus.state !== "untested"
        ? providerTestFingerprint(nextDraft)
        : null,
    );
    setOperation("idle");
    setOperationError(null);
    setShowValidation(false);
    setSyncedRecordGeneration(initialProvider?.generation ?? null);
    setSyncedTransientToken(null);
  }, [initialProvider]);

  /** Saves the visible draft and clears raw secret state after native success. */
  async function save(): Promise<boolean> {
    setShowValidation(true);
    if (Object.keys(errors).length > 0) return false;
    if (operationInFlight.current) return false;
    operationInFlight.current = true;
    setOperation("saving");
    setOperationError(null);
    try {
      const saved = await saveProviderProfile(
        providerProfileForSave(draft, snapshot.providers),
      );
      if (saved.pendingApproval !== null) {
        setDraft((current) => ({ ...current, secret: "" }));
        setShowValidation(false);
        onSnapshot(saved);
        return false;
      }
      const provider = findSavedProvider(saved, draft);
      const savedDraft = draftFromProvider(provider);
      setDraft(savedDraft);
      setSelectedModelId(provider.selectedModelId ?? "");
      setLastTestFingerprint(null);
      onSnapshot(saved);
      return true;
    } catch (error) {
      setOperationError(providerMessage(error));
      return false;
    } finally {
      operationInFlight.current = false;
      setOperation("idle");
    }
  }

  /** Tests the exact renderer draft without creating durable Provider state. */
  async function testConnection(): Promise<void> {
    setShowValidation(true);
    if (Object.keys(errors).length > 0) return;
    if (operationInFlight.current) return;
    operationInFlight.current = true;
    setOperation("testing");
    setOperationError(null);
    try {
      const tested = await testProviderConnection(
        providerProfileForSave(draft, snapshot.providers),
      );
      if (tested.pendingApproval !== null) {
        setDraft((current) => ({ ...current, secret: "" }));
        setShowValidation(false);
        setLastTestFingerprint(null);
        onSnapshot(tested);
        return;
      }
      if (tested.transientTest === null) {
        setDraft((current) => ({ ...current, secret: "" }));
        throw new Error(
          "The transient Provider test was absent from its snapshot.",
        );
      }
      const testedDraft = draftFromProvider(tested.transientTest.provider);
      const recommended = recommendedProviderModel(
        tested.transientTest.provider.models,
      );
      const selected = tested.transientTest.provider.models.some(
        (model) =>
          model.modelId === tested.transientTest?.provider.selectedModelId &&
          model.productionReady,
      )
        ? tested.transientTest.provider.selectedModelId
        : (recommended?.modelId ?? "");
      setDraft(testedDraft);
      setSelectedModelId(selected ?? "");
      setLastTestFingerprint(providerTestFingerprint(testedDraft));
      setSyncedTransientToken(tested.transientTest.testToken);
      onSnapshot(tested);
    } catch (error) {
      setLastTestFingerprint(null);
      setOperationError(providerMessage(error));
    } finally {
      operationInFlight.current = false;
      setOperation("idle");
    }
  }

  /** Keeps transient test selection local; durable edits retain explicit Save. */
  async function selectModel(modelId: string): Promise<void> {
    const provider = activeProvider;
    const model = provider?.models.find(
      (candidate) => candidate.modelId === modelId,
    );
    if (!provider || !model?.productionReady || operationInFlight.current) {
      return;
    }
    if (transientTest !== null) {
      setSelectedModelId(modelId);
      setOperationError(null);
      return;
    }
    operationInFlight.current = true;
    const previousModelId = selectedModelId;
    setSelectedModelId(modelId);
    setOperation("selectingModel");
    setOperationError(null);
    try {
      onSnapshot(await selectProviderModel(provider.providerId, modelId));
    } catch (error) {
      setSelectedModelId(previousModelId);
      setOperationError(providerMessage(error));
    } finally {
      operationInFlight.current = false;
      setOperation("idle");
    }
  }

  /** Persists the automatically recommended model and fixed launch defaults. */
  async function continueOnboarding(): Promise<boolean> {
    if (!canContinue || !transientTest || operationInFlight.current) {
      return false;
    }
    operationInFlight.current = true;
    setOperation("completing");
    setOperationError(null);
    try {
      const completed = await completeProviderOnboarding(
        transientTest.testToken,
        selectedModelId,
      );
      onSnapshot(completed);
      onComplete?.();
      return true;
    } catch (error) {
      setOperationError(providerMessage(error));
      return false;
    } finally {
      operationInFlight.current = false;
      setOperation("idle");
    }
  }

  return {
    canContinue,
    connection,
    draft,
    errors,
    isBusy: operation !== "idle",
    isPersisted: currentProvider !== undefined || initialProvider !== undefined,
    models,
    operation,
    operationError,
    selectedModelId,
    showValidation,
    changeAuthentication,
    changeKind,
    changeSecret,
    changeValue,
    continueOnboarding,
    reset,
    save,
    selectModel,
    testConnection,
  };
}

/** Maps native test evidence and local invalidation into one live-region state. */
function providerConnectionView(
  provider: ProviderRecord | undefined,
  isFresh: boolean,
  operation: ProviderProfileOperation,
  operationError: string | null,
): ProviderConnectionView {
  if (operation === "testing") {
    return {
      state: "testing",
      title: "Testing connection",
      detail:
        "C4OS is checking the exact endpoint and authentication settings.",
    };
  }
  if (operation === "completing") {
    return {
      state: "submitting",
      title: "Saving provider setup",
      detail: "C4OS is saving the provider and its strongest supported model.",
    };
  }
  if (operationError !== null) {
    return {
      state: "failure",
      title: "Provider action failed",
      detail: operationError,
    };
  }
  if (!provider || provider.testStatus.state === "untested") {
    return {
      state: "untested",
      title: "Connection not tested",
      detail: "Test this provider before choosing it for C4OS.",
    };
  }
  if (!isFresh) {
    return {
      state: "stale",
      title: "Connection test no longer current",
      detail: "A tested endpoint or authentication field changed. Test again.",
    };
  }
  if (provider.testStatus.state === "failed") {
    return {
      state: "failure",
      title: "Connection failed",
      detail: failureDetail(provider.testStatus.code),
    };
  }
  if (
    provider.testStatus.state === "succeededNoUsableModels" ||
    recommendedProviderModel(provider.models) === null
  ) {
    return {
      state: "zero",
      title: "No usable models",
      detail:
        "The connection passed, but it returned no production-ready model.",
    };
  }
  const productionReadyCount = provider.models.filter(
    ({ productionReady }) => productionReady,
  ).length;
  return {
    state: "success",
    title: "Connection passed",
    detail: `${productionReadyCount} production-ready model${
      productionReadyCount === 1 ? "" : "s"
    } discovered.`,
  };
}

/** Finds the record created or updated by a successful profile save. */
function findSavedProvider(
  snapshot: ProviderSettingsSnapshot,
  draft: ProviderFormDraft,
): ProviderRecord {
  const provider = draft.providerId
    ? snapshot.providers.find(
        ({ providerId }) => providerId === draft.providerId,
      )
    : snapshot.providers.find(
        ({ displayName }) =>
          displayName.toLocaleLowerCase() ===
          draft.displayName.trim().toLocaleLowerCase(),
      );
  if (!provider) {
    throw new Error("The saved provider was absent from its snapshot.");
  }
  return provider;
}

/** Keeps provider failures bounded to the service-authored renderer message. */
function providerMessage(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Provider service could not complete the request.";
}

/** Explains the bounded native connection failure without exposing secrets. */
function failureDetail(
  code: Extract<ProviderRecord["testStatus"], { state: "failed" }>["code"],
): string {
  const details = {
    authentication: "Authentication was rejected. Check the stored credential.",
    network: "The provider endpoint could not be reached.",
    "rate-limited": "The provider rate-limited this connection test.",
    incompatible: "The endpoint did not provide a compatible model response.",
    cancelled:
      "The connection test was cancelled without changing the profile.",
    internal: "The provider test failed inside the supervised native service.",
  } satisfies Record<typeof code, string>;
  return details[code];
}

import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import type { AppRoutePath } from "../../app/route-contract";
import {
  openConfigurationExternal,
  readConfigurationSettings,
  saveConfigurationSettings,
  type ConfigurationSettingsDraft,
  type ConfigurationSettingsSnapshot as NativeConfigurationSnapshot,
} from "../../platform/configuration-service";
import {
  readPolicySettings,
  revokePolicyException,
  savePolicySettings,
  type PolicySettingsSnapshot as NativePolicySnapshot,
} from "../../platform/policy-service";
import {
  readProviderSnapshot,
  setProviderModelsEnabled,
  testProviderConnection,
  type ProviderModel,
  type ProviderRecord,
  type ProviderSettingsSnapshot,
} from "../../platform/provider-service";
import {
  readRuntimeCoreSnapshot,
  type RuntimeCoreSnapshot,
  type RuntimeModelRouteSummary,
  type RuntimeProcessSummary,
} from "../../platform/runtime-core";
import { ProtocolBoundaryError } from "../../platform/tauri-adapter";
import {
  ConfigurationSettings,
  type ConfigurationSettingsSnapshot,
  type ConfigurationValues,
} from "./configuration";
import { DiagnosticSettingsController } from "./diagnostics";
import {
  AdvancedPolicySettings,
  type PolicySettingsSnapshot,
  type ReadyPolicySettingsSnapshot,
  type RevokePolicyExceptionInput,
  type SavePolicySettingsInput,
} from "./policies";
import {
  ProviderApprovalNotice,
  ProviderSettings,
  type ProviderApprovalDecision,
} from "./providers";
import {
  ModelSettings,
  RuntimeSettings,
  type ModelCapabilityKey,
  type ModelCapabilityState,
  type ModelRouteView,
  type ModelSettingsSnapshot,
  type RuntimeOptionView,
  type RuntimeSettingsSnapshot,
} from "./runtime";
import { UpdateSettingsController } from "./updates";

type CoreSettingsRoute = Extract<
  AppRoutePath,
  | "/settings/providers"
  | "/settings/models"
  | "/settings/runtimes"
  | "/settings/configuration"
  | "/settings/advanced-policies"
>;

export function CoreSettingsController({
  onNavigate,
  route,
}: {
  readonly onNavigate: (route: AppRoutePath) => void;
  readonly route: CoreSettingsRoute;
}) {
  if (route === "/settings/providers") return <ProviderSettings />;
  if (route === "/settings/models") return <ModelSettingsController />;
  if (route === "/settings/runtimes") return <RuntimeSettingsController />;
  if (route === "/settings/configuration") {
    return <ConfigurationSettingsController onNavigate={onNavigate} />;
  }
  return <PolicySettingsController />;
}

function ModelSettingsController() {
  const [native, setNative] = useState<ProviderSettingsSnapshot | null>(null);
  const [runtime, setRuntime] = useState<RuntimeCoreSnapshot | null>(null);
  const [state, setState] = useState<ModelSettingsSnapshot>({
    status: "loading",
    message: "Reading authoritative provider and model evidence.",
  });
  const [pendingRoutes, setPendingRoutes] = useState<ReadonlySet<string>>(
    () => new Set(),
  );
  const [operationError, setOperationError] = useState<string>();
  const [refresh, setRefresh] = useState<
    Extract<ModelSettingsSnapshot, { status: "ready" }>["refresh"]
  >({ status: "idle", message: "Model evidence is ready." });
  const [refreshQueue, setRefreshQueue] = useState<readonly string[]>([]);
  const [refreshAwaitingApproval, setRefreshAwaitingApproval] = useState(false);
  const operationLock = useRef<"refresh" | "write" | null>(null);

  const publish = useCallback((snapshot: ProviderSettingsSnapshot) => {
    setNative(snapshot);
  }, []);

  const load = useCallback(async () => {
    setState({
      status: "loading",
      message: "Reading authoritative provider and model evidence.",
    });
    try {
      const [providerSnapshot, runtimeSnapshot] = await Promise.all([
        readProviderSnapshot(),
        readRuntimeCoreSnapshot(),
      ]);
      publish(providerSnapshot);
      setRuntime(runtimeSnapshot);
      setState({
        status: "ready",
        generation: providerSnapshot.generation,
        models: projectModels(
          providerSnapshot,
          runtimeSnapshot.modelRoutes,
          new Set(),
        ),
        refresh: { status: "idle", message: "Model evidence is ready." },
        writesDisabled: false,
      });
      setOperationError(undefined);
    } catch (error) {
      setState({
        status: "error",
        message: messageFor(error),
        retryable: true,
      });
    }
  }, [publish]);

  useEffect(() => {
    let active = true;
    window.queueMicrotask(() => {
      if (active) void load();
    });
    return () => {
      active = false;
    };
  }, [load]);

  const projected = useMemo<ModelSettingsSnapshot>(() => {
    if (state.status !== "ready" || native === null || runtime === null)
      return state;
    return {
      status: "ready",
      generation: native.generation,
      models: projectModels(native, runtime.modelRoutes, pendingRoutes),
      refresh:
        native.pendingApproval !== null && !refreshAwaitingApproval
          ? {
              status: "pending",
              message:
                "Answer the current Provider approval before refreshing models.",
            }
          : refresh,
      ...(operationError === undefined ? {} : { operationError }),
      writesDisabled:
        refresh.status === "pending" ||
        pendingRoutes.size > 0 ||
        native.pendingApproval !== null,
    };
  }, [
    native,
    operationError,
    pendingRoutes,
    refresh,
    refreshAwaitingApproval,
    runtime,
    state,
  ]);

  async function setEnabled(routeIds: readonly string[], enabled: boolean) {
    if (
      native === null ||
      runtime === null ||
      routeIds.length === 0 ||
      operationLock.current !== null
    )
      return;
    operationLock.current = "write";
    const routes = new Map(
      projectModels(native, runtime.modelRoutes, new Set()).map((model) => [
        model.id,
        model,
      ]),
    );
    const requested = routeIds
      .map((routeId) => routes.get(routeId))
      .filter((route): route is ModelRouteView => route !== undefined);
    if (requested.length !== routeIds.length) {
      setOperationError("One or more model routes changed before the update.");
      operationLock.current = null;
      return;
    }
    setPendingRoutes(new Set(routeIds));
    setOperationError(undefined);
    try {
      let latest = native;
      const providers = new Map<string, string[]>();
      for (const route of requested) {
        providers.set(route.providerId, [
          ...(providers.get(route.providerId) ?? []),
          route.modelId,
        ]);
      }
      for (const [providerId, modelIds] of providers) {
        latest = await setProviderModelsEnabled(providerId, modelIds, enabled);
        setNative(latest);
      }
      setRuntime(await readRuntimeCoreSnapshot());
    } catch (error) {
      setOperationError(messageFor(error));
      try {
        setNative(await readProviderSnapshot());
      } catch {
        setOperationError(
          `${messageFor(error)} The authoritative model state could not be refreshed; retry before another change.`,
        );
      }
    } finally {
      setPendingRoutes(new Set());
      operationLock.current = null;
    }
  }

  async function reconcileRefreshFailure(error: unknown) {
    operationLock.current = null;
    setRefreshAwaitingApproval(false);
    setRefreshQueue([]);
    setRefresh({ status: "error", message: messageFor(error) });
    try {
      setNative(await readProviderSnapshot());
    } catch {
      setOperationError(
        "Provider refresh changed native state, but its authoritative snapshot could not be reconciled. Retry before another change.",
      );
    }
  }

  async function continueRefresh(providerIds: readonly string[]) {
    for (const [index, providerId] of providerIds.entries()) {
      const next = await testProviderConnection(providerId);
      setNative(next);
      if (next.pendingApproval !== null) {
        setRefreshQueue(providerIds.slice(index + 1));
        setRefreshAwaitingApproval(true);
        setRefresh({
          status: "pending",
          message:
            "Approve the Provider connection test to continue refreshing models.",
        });
        return;
      }
    }
    setRefreshAwaitingApproval(false);
    setRefreshQueue([]);
    setRuntime(await readRuntimeCoreSnapshot());
    setRefresh({
      status: "success",
      message: "Provider availability and model evidence were refreshed.",
    });
    operationLock.current = null;
  }

  async function refreshModels() {
    if (
      native === null ||
      native.pendingApproval !== null ||
      operationLock.current !== null
    )
      return;
    operationLock.current = "refresh";
    setRefresh({ status: "pending", message: "Refreshing provider evidence…" });
    setOperationError(undefined);
    try {
      await continueRefresh(
        native.providers
          .filter(({ enabled }) => enabled)
          .map(({ providerId }) => providerId),
      );
    } catch (error) {
      await reconcileRefreshFailure(error);
    }
  }

  async function answerRefreshApproval(decision: ProviderApprovalDecision) {
    if (!refreshAwaitingApproval) return;
    if (decision === "deny") {
      operationLock.current = null;
      setRefreshAwaitingApproval(false);
      setRefreshQueue([]);
      setRefresh({
        status: "error",
        message:
          "Model refresh stopped because the Provider connection test was denied.",
      });
      return;
    }
    try {
      await continueRefresh(refreshQueue);
    } catch (error) {
      await reconcileRefreshFailure(error);
    }
  }

  return (
    <>
      {native === null ? null : (
        <ProviderApprovalNotice
          {...(refreshAwaitingApproval
            ? { onAnswered: answerRefreshApproval }
            : {})}
          onSnapshot={publish}
          snapshot={native}
        />
      )}
      <ModelSettings
        actions={{
          onRefresh: refreshModels,
          onRetry: load,
          onSetEnabled: (routeId, enabled) => setEnabled([routeId], enabled),
          onSetVisibleEnabled: setEnabled,
        }}
        snapshot={projected}
      />
    </>
  );
}

function RuntimeSettingsController() {
  const [configuration, setConfiguration] =
    useState<NativeConfigurationSnapshot | null>(null);
  const [runtime, setRuntime] = useState<RuntimeCoreSnapshot | null>(null);
  const [state, setState] = useState<RuntimeSettingsSnapshot>({
    status: "loading",
    message: "Reading installed runtime health and saved defaults.",
  });
  const [save, setSave] = useState<
    Extract<RuntimeSettingsSnapshot, { status: "ready" }>["save"]
  >({ status: "idle", message: "Runtime default is unchanged." });

  const load = useCallback(async () => {
    setState({
      status: "loading",
      message: "Reading installed runtime health and saved defaults.",
    });
    try {
      const [nextConfiguration, nextRuntime] = await Promise.all([
        readConfigurationSettings(),
        readRuntimeCoreSnapshot(),
      ]);
      setConfiguration(nextConfiguration);
      setRuntime(nextRuntime);
      setSave({ status: "idle", message: "Runtime default is unchanged." });
    } catch (error) {
      setState({
        status: "error",
        message: messageFor(error),
        retryable: true,
      });
    }
  }, []);

  useEffect(() => {
    let active = true;
    window.queueMicrotask(() => {
      if (active) void load();
    });
    return () => {
      active = false;
    };
  }, [load]);

  const projected = useMemo<RuntimeSettingsSnapshot>(() => {
    if (configuration === null || runtime === null) return state;
    const runtimes = projectRuntimeOptions(runtime.runtimes);
    return {
      status: "ready",
      generation: configuration.generation,
      runtimes,
      save,
      savedRuntimeId: configuration.defaultRuntime,
    };
  }, [configuration, runtime, save, state]);

  async function saveRuntime(runtimeId: string, expectedGeneration: number) {
    if (
      configuration === null ||
      expectedGeneration !== configuration.generation
    ) {
      setSave({
        status: "error",
        message: "Configuration changed before the runtime could be saved.",
      });
      return;
    }
    if (runtimeId !== "opencode" && runtimeId !== "pi") return;
    setSave({ status: "pending", message: "Saving the new-Chat runtime…" });
    try {
      const next = await saveConfigurationSettings({
        ...configurationDraft(configuration),
        defaultRuntime: runtimeId,
      });
      setConfiguration(next);
      setSave({
        status: "success",
        message: "New Chats will bind to the saved runtime after first submit.",
      });
    } catch (error) {
      if (
        error instanceof ProtocolBoundaryError &&
        error.code === "staleGeneration"
      ) {
        const changed = error.details.changedKeys;
        const changedKeys =
          changed?.kind === "text" && changed.value.length > 0
            ? changed.value
            : "the active document";
        try {
          const latest = await readConfigurationSettings();
          setConfiguration(latest);
          setSave({
            status: "error",
            message: `Configuration changed in ${changedKeys}. The latest saved values were refreshed and your unsaved draft was retained. Review it before retrying Save.`,
          });
        } catch (refreshError) {
          setSave({
            status: "error",
            message: `${messageFor(error)} The latest saved values could not be refreshed: ${messageFor(refreshError)}`,
          });
        }
      } else {
        setSave({ status: "error", message: messageFor(error) });
      }
    }
  }

  return (
    <RuntimeSettings
      actions={{ onRetry: load, onSaveRuntime: saveRuntime }}
      snapshot={projected}
    />
  );
}

function ConfigurationSettingsController({
  onNavigate,
}: {
  readonly onNavigate: (route: AppRoutePath) => void;
}) {
  const [configuration, setConfiguration] =
    useState<NativeConfigurationSnapshot | null>(null);
  const [state, setState] = useState<ConfigurationSettingsSnapshot>({
    status: "loading",
    message: "Reading the live C4OS configuration.",
  });
  const [save, setSave] = useState<
    Extract<ConfigurationSettingsSnapshot, { status: "ready" }>["save"]
  >({ status: "idle", message: "Configuration is unchanged." });
  const [openConfiguration, setOpenConfiguration] = useState<
    Extract<
      ConfigurationSettingsSnapshot,
      { status: "ready" }
    >["openConfiguration"]
  >({ status: "idle", message: "The configuration file is closed." });

  const load = useCallback(async () => {
    setState({
      status: "loading",
      message: "Reading the live C4OS configuration.",
    });
    try {
      const nextConfiguration = await readConfigurationSettings();
      setConfiguration(nextConfiguration);
      setSave({ status: "idle", message: "Configuration is unchanged." });
    } catch (error) {
      setState({
        status: "error",
        message: messageFor(error),
        retryable: true,
      });
    }
  }, []);

  useEffect(() => {
    let active = true;
    window.queueMicrotask(() => {
      if (active) void load();
    });
    return () => {
      active = false;
    };
  }, [load]);

  const projected = useMemo<ConfigurationSettingsSnapshot>(() => {
    if (configuration === null) return state;
    return {
      status: "ready",
      generation: configuration.generation,
      saved: configurationValues(configuration),
      live: {
        displayPath: "~/.c4os/config.toml",
        source: configuration.hasExternalError
          ? "lastKnownGood"
          : configuration.generation === 0
            ? "default"
            : "lastKnownGood",
        detail: configuration.hasExternalError
          ? "An external edit was rejected; C4OS retained the last known-good document."
          : "C4OS is using the validated app-level configuration document.",
      },
      save,
      openConfiguration,
    };
  }, [configuration, openConfiguration, save, state]);

  async function saveValues(
    values: ConfigurationValues,
    expectedGeneration: number,
  ) {
    if (
      configuration === null ||
      expectedGeneration !== configuration.generation
    ) {
      setSave({
        status: "error",
        message: "Configuration changed before Save.",
      });
      return;
    }
    setSave({ status: "pending", message: "Saving Configuration…" });
    try {
      const next = await saveConfigurationSettings({
        defaultApprovalPreset: fromViewPreset(values.approvalPreset),
        browserEnvironment: fromViewBrowser(values.browserEnvironment),
        defaultEnvironment: configuration.defaultEnvironment,
        defaultRuntime: configuration.defaultRuntime,
        inheritShellEnvironment: values.inheritShellEnvironment,
        restoreLastWorkspace: values.restoreLastWorkspace,
      });
      setConfiguration(next);
      setSave({ status: "success", message: "Configuration saved." });
    } catch (error) {
      if (
        error instanceof ProtocolBoundaryError &&
        error.code === "staleGeneration"
      ) {
        const changed = error.details.changedKeys;
        const changedKeys =
          changed?.kind === "text" && changed.value.length > 0
            ? changed.value
            : "the active document";
        try {
          const latest = await readConfigurationSettings();
          setConfiguration(latest);
          setSave({
            status: "error",
            message: `Configuration changed in ${changedKeys}. The latest saved values were refreshed and your unsaved draft was retained. Review it before retrying Save.`,
          });
        } catch (refreshError) {
          setSave({
            status: "error",
            message: `${messageFor(error)} The latest saved values could not be refreshed: ${messageFor(refreshError)}`,
          });
        }
      } else {
        setSave({ status: "error", message: messageFor(error) });
      }
    }
  }

  async function openExternal() {
    setOpenConfiguration({
      status: "pending",
      message: "Opening ~/.c4os/config.toml…",
    });
    try {
      await openConfigurationExternal();
      setOpenConfiguration({
        status: "success",
        message: "C4OS opened the configuration file externally.",
      });
    } catch (error) {
      setOpenConfiguration({ status: "error", message: messageFor(error) });
    }
  }

  return (
    <>
      <ConfigurationSettings
        actions={{
          onNavigateAdvanced: async () =>
            onNavigate("/settings/advanced-policies"),
          onOpenConfigurationFile: openExternal,
          onRetry: load,
          onSaveConfiguration: saveValues,
        }}
        snapshot={projected}
      />
      <UpdateSettingsController />
      <DiagnosticSettingsController />
    </>
  );
}

function PolicySettingsController() {
  const [native, setNative] = useState<NativePolicySnapshot | null>(null);
  const [state, setState] = useState<PolicySettingsSnapshot>({
    status: "loading",
    message: "Reading effective category and exception policy.",
  });

  const load = useCallback(async () => {
    setState({
      status: "loading",
      message: "Reading effective category and exception policy.",
    });
    try {
      const snapshot = await readPolicySettings();
      setNative(snapshot);
      setState(toPolicySnapshot(snapshot));
    } catch (error) {
      setState({
        status: "error",
        message: messageFor(error),
        retryable: true,
      });
    }
  }, []);

  useEffect(() => {
    let active = true;
    window.queueMicrotask(() => {
      if (active) void load();
    });
    return () => {
      active = false;
    };
  }, [load]);

  async function refreshReadyAuthority(): Promise<void> {
    const refreshed = await readPolicySettings();
    setNative(refreshed);
    setState(toPolicySnapshot(refreshed));
  }

  async function refreshStaleAuthority(error: unknown): Promise<void> {
    if (
      error instanceof ProtocolBoundaryError &&
      (error.code === "staleGeneration" || error.code === "invalidGeneration")
    ) {
      await refreshReadyAuthority();
    }
  }

  async function save(
    input: SavePolicySettingsInput,
  ): Promise<ReadyPolicySettingsSnapshot> {
    if (
      native === null ||
      input.expectedCoordinatorGeneration !== native.coordinatorGeneration ||
      input.expectedPolicyVersion !== native.policyVersion
    ) {
      await refreshReadyAuthority();
      throw new Error("Policy changed before Save.");
    }
    let next: NativePolicySnapshot;
    try {
      next = await savePolicySettings(input.categoryValues);
    } catch (error) {
      await refreshStaleAuthority(error);
      throw error;
    }
    setNative(next);
    const projected = toPolicySnapshot(next);
    setState(projected);
    return projected;
  }

  async function revoke(
    input: RevokePolicyExceptionInput,
  ): Promise<ReadyPolicySettingsSnapshot> {
    if (
      native === null ||
      input.expectedCoordinatorGeneration !== native.coordinatorGeneration ||
      input.expectedPolicyVersion !== native.policyVersion
    ) {
      await refreshReadyAuthority();
      throw new Error("Policy changed before the exception could be revoked.");
    }
    let next: NativePolicySnapshot;
    try {
      next = await revokePolicyException(input.exceptionId);
    } catch (error) {
      await refreshStaleAuthority(error);
      throw error;
    }
    setNative(next);
    const projected = toPolicySnapshot(next);
    setState(projected);
    return projected;
  }

  return (
    <AdvancedPolicySettings
      actions={{ onRetry: load, onRevokeException: revoke, onSave: save }}
      configurationNavigationSelected
      snapshot={state}
    />
  );
}

function projectModels(
  snapshot: ProviderSettingsSnapshot,
  effectiveRoutes: readonly RuntimeModelRouteSummary[],
  pending: ReadonlySet<string>,
): readonly ModelRouteView[] {
  const effective = new Map(
    effectiveRoutes.map((route) => [
      modelRouteId(route.providerId, route.modelId),
      route,
    ]),
  );
  return snapshot.providers.flatMap((provider) =>
    provider.models.map((model) => {
      const id = modelRouteId(provider.providerId, model.modelId);
      const route = effective.get(id);
      const effectiveRoute = route?.lifecycle === "active" ? route : undefined;
      const credentialAvailable =
        provider.authentication.type === "none" || provider.hasCredential;
      const available =
        provider.enabled &&
        credentialAvailable &&
        model.productionReady &&
        effectiveRoute !== undefined;
      const observedAt = effectiveRoute
        ? Math.max(
            ...Object.values(effectiveRoute.capabilities).map(
              (capability) => capability.checkedAtMs,
            ),
          )
        : model.checkedAtMs;
      return {
        id,
        providerId: provider.providerId,
        providerName: provider.displayName,
        modelId: model.modelId,
        modelName: model.displayName,
        revision: `Observed ${formatObservedAt(observedAt)}`,
        runtimeLabel: effectiveRoute
          ? `${effectiveRoute.runtimeKind} ${effectiveRoute.nativeRuntimeVersion} · ${effectiveRoute.adapterKind} effective evidence`
          : route?.lifecycle === "unavailable"
            ? `${route.runtimeKind} reports this route unavailable`
            : "No ready runtime has published effective evidence for this route",
        contextTokens: effectiveRoute?.contextTokens ?? null,
        available,
        availabilityDetail: modelAvailability(provider, model, route),
        enabled: !provider.disabledModelIds.includes(model.modelId),
        operationPending: pending.has(id),
        capabilities: (["vision", "tools", "reasoning", "audio"] as const).map(
          (key) => modelCapability(effectiveRoute, key),
        ),
      };
    }),
  );
}

function modelCapability(
  route: RuntimeModelRouteSummary | undefined,
  key: ModelCapabilityKey,
) {
  const evidence = route?.capabilities[key];
  const state: ModelCapabilityState = evidence?.state ?? "unknown";
  return {
    key,
    state,
    summary: route
      ? `${key} is ${state} in the runtime-bound effective intersection.`
      : `${key} is unknown until a ready runtime publishes effective evidence.`,
    evidence: evidence
      ? [
          {
            source: evidence.source,
            checkedAt: formatObservedAt(evidence.checkedAtMs),
            detail: evidence.detail ?? `${key}: ${state}`,
          },
        ]
      : [],
  };
}

function modelAvailability(
  provider: ProviderRecord,
  model: ProviderModel,
  route: RuntimeModelRouteSummary | undefined,
): string {
  if (!provider.enabled) return "The provider profile is disabled.";
  if (provider.authentication.type !== "none" && !provider.hasCredential) {
    return "The provider credential must be re-entered before this route can be used.";
  }
  if (!model.productionReady) {
    return `The model route is ${model.availability} and not production-ready.`;
  }
  if (provider.disabledModelIds.includes(model.modelId)) {
    return "The model is available but disabled in Settings.";
  }
  if (route === undefined) {
    return "No ready runtime has published an effective route for this provider model.";
  }
  if (route.lifecycle !== "active") {
    return `The runtime reports this model route as ${route.lifecycle}.`;
  }
  return "The latest exact-profile and runtime evidence reports an effective route.";
}

function modelRouteId(providerId: string, modelId: string): string {
  return `${encodeURIComponent(providerId)}::${encodeURIComponent(modelId)}`;
}

function projectRuntimeOptions(
  records: readonly RuntimeProcessSummary[],
): readonly RuntimeOptionView[] {
  return (["open-code", "pi"] as const).map((kind) => {
    const candidates = records.filter((record) => record.runtimeKind === kind);
    const record =
      candidates.find(
        (candidate) =>
          candidate.lifecycle === "ready" && candidate.health === "healthy",
      ) ?? candidates.at(0);
    const id = kind === "open-code" ? "opencode" : "pi";
    const label = kind === "open-code" ? "OpenCode" : "Pi";
    if (record === undefined) {
      return {
        id,
        kind,
        label,
        version: "not installed",
        health: "unavailable",
        detail: `${label} has no verified local installation.`,
      };
    }
    const health: RuntimeOptionView["health"] =
      record.lifecycle === "stopped"
        ? "healthy"
        : record.lifecycle === "ready" && record.health === "healthy"
          ? "healthy"
          : (record.lifecycle === "ready" || record.lifecycle === "degraded") &&
              record.health === "degraded"
            ? "degraded"
            : "unavailable";
    return {
      id,
      kind,
      label,
      version: record.nativeVersion,
      health,
      detail:
        record.lifecycle === "stopped"
          ? `${record.runtimeId} is installed and will start when a Chat binds to it.`
          : `${record.runtimeId} is ${record.lifecycle} with ${record.health} health.`,
    };
  });
}

function configurationDraft(
  snapshot: NativeConfigurationSnapshot,
): ConfigurationSettingsDraft {
  return {
    defaultApprovalPreset: snapshot.defaultApprovalPreset,
    restoreLastWorkspace: snapshot.restoreLastWorkspace,
    inheritShellEnvironment: snapshot.inheritShellEnvironment,
    browserEnvironment: snapshot.browserEnvironment,
    defaultRuntime: snapshot.defaultRuntime,
    defaultEnvironment: snapshot.defaultEnvironment,
  };
}

function configurationValues(
  snapshot: NativeConfigurationSnapshot,
): ConfigurationValues {
  return {
    approvalPreset: toViewPreset(snapshot.defaultApprovalPreset),
    browserEnvironment: toViewBrowser(snapshot.browserEnvironment),
    inheritShellEnvironment: snapshot.inheritShellEnvironment,
    restoreLastWorkspace: snapshot.restoreLastWorkspace,
  };
}

function toViewPreset(
  value: NativeConfigurationSnapshot["defaultApprovalPreset"],
): ConfigurationValues["approvalPreset"] {
  return {
    ask_for_approval: "askForApproval",
    approve_safe_actions: "approveSafeActions",
    approve_for_me: "approveForMe",
    custom: "custom",
  }[value] as ConfigurationValues["approvalPreset"];
}

function fromViewPreset(
  value: ConfigurationValues["approvalPreset"],
): NativeConfigurationSnapshot["defaultApprovalPreset"] {
  return {
    askForApproval: "ask_for_approval",
    approveSafeActions: "approve_safe_actions",
    approveForMe: "approve_for_me",
    custom: "custom",
  }[value] as NativeConfigurationSnapshot["defaultApprovalPreset"];
}

function toViewBrowser(
  value: NativeConfigurationSnapshot["browserEnvironment"],
): ConfigurationValues["browserEnvironment"] {
  return {
    app_wide: "appWide",
    workspace_project: "workspaceProject",
    chat: "chat",
    none: "none",
  }[value] as ConfigurationValues["browserEnvironment"];
}

function fromViewBrowser(
  value: ConfigurationValues["browserEnvironment"],
): NativeConfigurationSnapshot["browserEnvironment"] {
  return {
    appWide: "app_wide",
    workspaceProject: "workspace_project",
    chat: "chat",
    none: "none",
  }[value] as NativeConfigurationSnapshot["browserEnvironment"];
}

function toPolicySnapshot(
  snapshot: NativePolicySnapshot,
): ReadyPolicySettingsSnapshot {
  return {
    status: "ready",
    authority: snapshot.authority,
    basePreset: snapshot.basePreset,
    coordinatorGeneration: snapshot.coordinatorGeneration,
    policyVersion: snapshot.policyVersion,
    revocationEpoch: snapshot.revocationEpoch,
    preset: snapshot.preset,
    categoryValues: snapshot.categoryValues,
    effectiveCategoryValues: snapshot.effectiveCategoryValues,
    exceptions: snapshot.exceptions,
    maximumAuthorityRuleCount: snapshot.maximumAuthorityRuleCount,
    managedRequirementCount: snapshot.managedRequirementCount,
  };
}

function formatObservedAt(value: number): string {
  const date = new Date(value);
  return Number.isNaN(date.valueOf()) ? "unknown time" : date.toISOString();
}

function messageFor(error: unknown): string {
  return error instanceof Error
    ? error.message
    : "The native Settings service is unavailable.";
}

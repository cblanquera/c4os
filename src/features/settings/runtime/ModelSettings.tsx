import { useMemo, useState } from "react";

import {
  Button,
  ModalDialog,
  Notice,
  StatusRegion,
  Switch,
} from "../../../components/accessible";
import type {
  ModelCapabilityKey,
  ModelCapabilityState,
  ModelRouteView,
  ModelSettingsActions,
  ModelSettingsSnapshot,
} from "./types";

import "./runtime-settings.css";

const CAPABILITY_LABELS: Readonly<Record<ModelCapabilityKey, string>> = {
  vision: "Vision",
  tools: "Tools",
  reasoning: "Reasoning",
  audio: "Audio",
};

export interface ModelSettingsProps {
  readonly actions: ModelSettingsActions;
  readonly snapshot: ModelSettingsSnapshot;
}

/** Renders service-owned model routes without making renderer filters authoritative. */
export function ModelSettings({ actions, snapshot }: ModelSettingsProps) {
  if (snapshot.status === "loading") {
    return (
      <StatusRegion
        aria-busy="true"
        className="runtime-settings-state runtime-settings-state--loading"
      >
        <span aria-hidden="true" className="runtime-settings-spinner" />
        <span>
          <strong>Loading models</strong>
          <span>{snapshot.message}</span>
        </span>
      </StatusRegion>
    );
  }

  if (snapshot.status === "error") {
    return (
      <Notice title="Models are unavailable" tone="danger">
        <p>{snapshot.message}</p>
        {snapshot.retryable ? (
          <Button onPress={() => void actions.onRetry()}>Try again</Button>
        ) : null}
      </Notice>
    );
  }

  return <ModelCatalog actions={actions} snapshot={snapshot} />;
}

function ModelCatalog({
  actions,
  snapshot,
}: {
  readonly actions: ModelSettingsActions;
  readonly snapshot: Extract<ModelSettingsSnapshot, { status: "ready" }>;
}) {
  const [query, setQuery] = useState("");
  const [providerId, setProviderId] = useState("all");
  const [capability, setCapability] = useState<"all" | ModelCapabilityKey>(
    "all",
  );
  const providers = useMemo(
    () =>
      [
        ...new Map(
          snapshot.models.map((model) => [model.providerId, model]),
        ).values(),
      ]
        .map((model) => ({ id: model.providerId, name: model.providerName }))
        .sort((left, right) => left.name.localeCompare(right.name)),
    [snapshot.models],
  );
  const normalizedQuery = query.trim().toLocaleLowerCase();
  const visibleModels = useMemo(
    () =>
      snapshot.models.filter((model) => {
        if (providerId !== "all" && model.providerId !== providerId)
          return false;
        if (
          capability !== "all" &&
          !model.capabilities.some(
            (item) =>
              item.key === capability &&
              (item.state === "supported" || item.state === "degraded"),
          )
        ) {
          return false;
        }
        return `${model.providerName} ${model.providerId} ${model.modelName} ${model.modelId} ${model.revision}`
          .toLocaleLowerCase()
          .includes(normalizedQuery);
      }),
    [capability, normalizedQuery, providerId, snapshot.models],
  );
  const actionableVisibleModels = visibleModels.filter(
    (model) => model.available,
  );
  const enableVisible =
    actionableVisibleModels.length > 0 &&
    actionableVisibleModels.every((model) => !model.enabled);
  const bulkLabel = enableVisible
    ? "Enable visible models"
    : "Disable visible models";

  return (
    <section
      aria-labelledby="model-settings-title"
      className="runtime-settings model-settings"
      data-generation={snapshot.generation}
    >
      <header className="runtime-settings-heading">
        <div>
          <h2 id="model-settings-title">Available models</h2>
          <p>
            Availability and capability evidence come from the latest native
            provider and runtime observations.
          </p>
        </div>
        <Button
          isDisabled={snapshot.writesDisabled}
          onPress={() => void actions.onRefresh()}
        >
          {snapshot.refresh.status === "pending" ? "Refreshing…" : "Refresh"}
        </Button>
      </header>

      {snapshot.operationError ? (
        <Notice title="Model change was not applied" tone="danger">
          <p>{snapshot.operationError}</p>
        </Notice>
      ) : null}

      <RefreshStatus refresh={snapshot.refresh} />

      {snapshot.models.length === 0 ? (
        <EmptyModels
          detail="Test and enable a provider to discover authoritative model routes."
          title="No models discovered"
        />
      ) : (
        <>
          <div className="model-toolbar" role="search">
            <label className="model-toolbar__search">
              <span>Search models</span>
              <input
                onChange={(event) => setQuery(event.currentTarget.value)}
                placeholder="Search provider or model identity"
                type="search"
                value={query}
              />
            </label>
            <label>
              <span>Provider</span>
              <select
                aria-label="Filter models by provider"
                onChange={(event) => setProviderId(event.currentTarget.value)}
                value={providerId}
              >
                <option value="all">All providers</option>
                {providers.map((provider) => (
                  <option key={provider.id} value={provider.id}>
                    {provider.name}
                  </option>
                ))}
              </select>
            </label>
            <label>
              <span>Capability</span>
              <select
                aria-label="Filter models by capability"
                onChange={(event) =>
                  setCapability(
                    event.currentTarget.value as "all" | ModelCapabilityKey,
                  )
                }
                value={capability}
              >
                <option value="all">All capabilities</option>
                {(Object.keys(CAPABILITY_LABELS) as ModelCapabilityKey[]).map(
                  (key) => (
                    <option key={key} value={key}>
                      {CAPABILITY_LABELS[key]}
                    </option>
                  ),
                )}
              </select>
            </label>
            <Button
              className="model-toolbar__bulk"
              isDisabled={
                snapshot.writesDisabled || actionableVisibleModels.length === 0
              }
              onPress={() =>
                void actions.onSetVisibleEnabled(
                  actionableVisibleModels.map((model) => model.id),
                  enableVisible,
                )
              }
            >
              {bulkLabel}
            </Button>
          </div>

          <StatusRegion className="model-result-count">
            {visibleModels.length === 1
              ? "1 visible model route"
              : `${visibleModels.length} visible model routes`}
          </StatusRegion>

          {visibleModels.length === 0 ? (
            <EmptyModels
              detail="Change the search or filters to review another authoritative route."
              title="No matching models"
            />
          ) : (
            <div aria-label="Model routes" className="model-list" role="list">
              {visibleModels.map((model) => (
                <ModelRow
                  actions={actions}
                  key={model.id}
                  model={model}
                  writesDisabled={snapshot.writesDisabled}
                />
              ))}
            </div>
          )}
        </>
      )}
    </section>
  );
}

function RefreshStatus({
  refresh,
}: {
  readonly refresh: Extract<
    ModelSettingsSnapshot,
    { status: "ready" }
  >["refresh"];
}) {
  if (refresh.status === "error") {
    return (
      <Notice title="Model refresh failed" tone="danger">
        <p>{refresh.message}</p>
      </Notice>
    );
  }
  if (refresh.status === "success") {
    return (
      <Notice title="Models refreshed" tone="success">
        <p>{refresh.message}</p>
      </Notice>
    );
  }
  return (
    <StatusRegion
      aria-busy={refresh.status === "pending"}
      className="runtime-operation-status"
      data-state={refresh.status}
    >
      {refresh.message}
    </StatusRegion>
  );
}

function ModelRow({
  actions,
  model,
  writesDisabled,
}: {
  readonly actions: ModelSettingsActions;
  readonly model: ModelRouteView;
  readonly writesDisabled: boolean;
}) {
  const effectiveCapabilities = model.capabilities.filter(
    (capability) =>
      capability.state === "supported" || capability.state === "degraded",
  );

  return (
    <article
      aria-busy={model.operationPending}
      className="model-row"
      data-available={model.available}
      role="listitem"
    >
      <div className="model-row__identity">
        <span>{model.providerName}</span>
        <ModelDetails model={model} />
        <small>
          {model.modelId} · {model.runtimeLabel}
        </small>
      </div>
      <div
        aria-label="Effective capabilities"
        className="model-capability-chips"
      >
        {effectiveCapabilities.length === 0 ? (
          <span data-state="unknown">No confirmed capabilities</span>
        ) : (
          effectiveCapabilities.map((capability) => (
            <span data-state={capability.state} key={capability.key}>
              {CAPABILITY_LABELS[capability.key]}
            </span>
          ))
        )}
      </div>
      <div className="model-row__context">
        <span>Context</span>
        <strong>{formatContextTokens(model.contextTokens)}</strong>
      </div>
      <Switch
        description={model.availabilityDetail}
        isDisabled={
          writesDisabled ||
          !model.available ||
          (model.operationPending ?? false)
        }
        isSelected={model.enabled}
        label={`${model.providerName} ${model.modelName} availability`}
        onChange={(enabled) => void actions.onSetEnabled(model.id, enabled)}
      />
    </article>
  );
}

function ModelDetails({ model }: { readonly model: ModelRouteView }) {
  return (
    <ModalDialog
      title={`${model.providerName} / ${model.modelName}`}
      triggerLabel={model.modelName}
      triggerVariant="quiet"
    >
      <div className="model-detail">
        <dl className="runtime-settings-facts">
          <Fact label="Provider" value={model.providerName} />
          <Fact label="Route identity" value={model.modelId} />
          <Fact label="Revision" value={model.revision} />
          <Fact label="Runtime" value={model.runtimeLabel} />
          <Fact
            label="Context"
            value={formatContextTokens(model.contextTokens)}
          />
          <Fact label="Availability" value={model.availabilityDetail} />
        </dl>
        <section aria-labelledby={`model-capabilities-${model.id}`}>
          <h3 id={`model-capabilities-${model.id}`}>
            Effective capabilities and evidence
          </h3>
          <div className="model-detail__capabilities">
            {model.capabilities.map((capability) => (
              <article data-state={capability.state} key={capability.key}>
                <header>
                  <strong>{CAPABILITY_LABELS[capability.key]}</strong>
                  <span>{capabilityStateLabel(capability.state)}</span>
                </header>
                <p>{capability.summary}</p>
                {capability.evidence.length === 0 ? (
                  <small>No current evidence supplied.</small>
                ) : (
                  <ul>
                    {capability.evidence.map((evidence, index) => (
                      <li
                        key={`${evidence.source}-${evidence.checkedAt}-${index}`}
                      >
                        <strong>{evidence.source}</strong>
                        <span>{evidence.detail}</span>
                        <time>{evidence.checkedAt}</time>
                      </li>
                    ))}
                  </ul>
                )}
              </article>
            ))}
          </div>
        </section>
      </div>
    </ModalDialog>
  );
}

function Fact({
  label,
  value,
}: {
  readonly label: string;
  readonly value: string;
}) {
  return (
    <div>
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

function EmptyModels({
  detail,
  title,
}: {
  readonly detail: string;
  readonly title: string;
}) {
  return (
    <section aria-label={title} className="runtime-settings-empty">
      <span aria-hidden="true">M</span>
      <h2>{title}</h2>
      <p>{detail}</p>
    </section>
  );
}

function capabilityStateLabel(state: ModelCapabilityState) {
  return state.charAt(0).toLocaleUpperCase() + state.slice(1);
}

function formatContextTokens(tokens: number | null) {
  if (tokens === null) return "Unknown";
  if (tokens >= 1_000_000 && tokens % 1_000_000 === 0) {
    return `${tokens / 1_000_000}M`;
  }
  if (tokens >= 1_000 && tokens % 1_000 === 0) return `${tokens / 1_000}K`;
  return String(tokens);
}

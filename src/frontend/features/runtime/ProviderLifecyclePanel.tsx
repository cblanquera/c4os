import { useState } from "react";

import type { DiscoveryScenario } from "./runtime-fixtures";
import { MODEL_ROUTES, PROVIDER_PROFILES } from "./runtime-fixtures";

const DISCOVERY_LABELS: Record<DiscoveryScenario, string> = {
  zero: "No usable models",
  one: "One usable model",
  many: "Many usable models",
};

/** Renders provider profiles and the accepted zero/one/many discovery matrix. */
export function ProviderLifecyclePanel() {
  const [discovery, setDiscovery] = useState<DiscoveryScenario>("many");
  const [enabledProfiles, setEnabledProfiles] = useState(
    () =>
      new Set(
        PROVIDER_PROFILES.filter((profile) => profile.isEnabled).map(
          (profile) => profile.id,
        ),
      ),
  );
  const [selectedModel, setSelectedModel] = useState(MODEL_ROUTES[0]?.id ?? "");
  const [connectionNotice, setConnectionNotice] = useState(
    "Latest test passed · 3 usable routes discovered",
  );

  const discoveredRoutes =
    discovery === "zero"
      ? []
      : discovery === "one"
        ? MODEL_ROUTES.slice(0, 1)
        : MODEL_ROUTES;

  /** Applies a deterministic discovery outcome and resets its explicit choice. */
  function handleDiscoveryChange(nextDiscovery: DiscoveryScenario) {
    const nextRoutes =
      nextDiscovery === "zero"
        ? []
        : nextDiscovery === "one"
          ? MODEL_ROUTES.slice(0, 1)
          : MODEL_ROUTES;

    setDiscovery(nextDiscovery);
    setSelectedModel(nextRoutes[0]?.id ?? "");
    setConnectionNotice(
      nextDiscovery === "zero"
        ? "Connection passed · no usable model routes were returned"
        : `Connection passed · ${nextRoutes.length} usable model ${
            nextRoutes.length === 1 ? "route" : "routes"
          } discovered`,
    );
  }

  /** Toggles profile availability without exposing its credential material. */
  function handleProfileToggle(profileId: string) {
    setEnabledProfiles((current) => {
      const next = new Set(current);

      if (next.has(profileId)) {
        next.delete(profileId);
      } else {
        next.add(profileId);
      }

      return next;
    });
  }

  return (
    <section
      className="runtime-panel"
      aria-labelledby="provider-lifecycle-title"
    >
      <header className="runtime-panel__header">
        <div>
          <p className="runtime-eyebrow">Provider lifecycle</p>
          <h2 id="provider-lifecycle-title">Profiles and discovery</h2>
          <p>
            Credentials remain opaque while profile availability and current
            connection evidence drive model readiness.
          </p>
        </div>
        <button
          type="button"
          className="runtime-button runtime-button--primary"
          onClick={() => handleDiscoveryChange(discovery)}
        >
          Test connection
        </button>
      </header>

      <div
        className="runtime-provider-grid"
        role="list"
        aria-label="Provider profiles"
      >
        {PROVIDER_PROFILES.map((profile) => {
          const isEnabled = enabledProfiles.has(profile.id);

          return (
            <article className="runtime-card" role="listitem" key={profile.id}>
              <header>
                <div>
                  <p className="runtime-card__meta">{profile.family}</p>
                  <h3>{profile.label}</h3>
                </div>
                <button
                  type="button"
                  className="runtime-switch"
                  role="switch"
                  aria-checked={isEnabled}
                  aria-label={`${profile.label} availability`}
                  onClick={() => handleProfileToggle(profile.id)}
                >
                  <span aria-hidden="true" />
                  {isEnabled ? "Available" : "Unavailable"}
                </button>
              </header>
              <p className="runtime-card__endpoint">{profile.endpoint}</p>
              <dl className="runtime-inline-facts">
                <div>
                  <dt>Models</dt>
                  <dd>{profile.modelCount}</dd>
                </div>
                <div>
                  <dt>Credential</dt>
                  <dd>Stored securely · ref only</dd>
                </div>
              </dl>
            </article>
          );
        })}
      </div>

      <section
        className="runtime-discovery"
        aria-labelledby="discovery-result-title"
      >
        <header>
          <div>
            <p className="runtime-eyebrow">Connection-test result</p>
            <h3 id="discovery-result-title">Usable model discovery</h3>
          </div>
          <span className="runtime-state" data-state="supported">
            Current evidence
          </span>
        </header>

        <div
          className="runtime-segmented"
          role="group"
          aria-label="Discovery result fixture"
        >
          {(Object.keys(DISCOVERY_LABELS) as DiscoveryScenario[]).map(
            (scenario) => (
              <button
                type="button"
                key={scenario}
                aria-pressed={discovery === scenario}
                onClick={() => handleDiscoveryChange(scenario)}
              >
                {DISCOVERY_LABELS[scenario]}
              </button>
            ),
          )}
        </div>

        <p className="runtime-notice" role="status">
          {connectionNotice}
        </p>

        {discoveredRoutes.length === 0 ? (
          <div className="runtime-empty" role="alert">
            <strong>Continue is blocked</strong>
            <span>
              The provider connection is valid, but it returned no usable model
              route.
            </span>
          </div>
        ) : (
          <fieldset className="runtime-choice-list">
            <legend>Explicit initial model route</legend>
            {discoveredRoutes.map((route, index) => (
              <label key={route.id}>
                <input
                  type="radio"
                  name="discovered-model"
                  checked={selectedModel === route.id}
                  onChange={() => setSelectedModel(route.id)}
                />
                <span>
                  <strong>{route.model}</strong>
                  <small>
                    {route.provider} · {route.contextWindow}
                  </small>
                </span>
                {index === 0 ? (
                  <span className="runtime-state" data-state="supported">
                    Recommended
                  </span>
                ) : null}
              </label>
            ))}
          </fieldset>
        )}

        {selectedModel ? (
          <p className="runtime-selection-summary">
            Selected explicitly:{" "}
            {MODEL_ROUTES.find((route) => route.id === selectedModel)?.provider}{" "}
            / {MODEL_ROUTES.find((route) => route.id === selectedModel)?.model}
          </p>
        ) : null}
      </section>
    </section>
  );
}

import { useState } from "react";

import type { CapabilityState } from "./runtime-fixtures";
import { getModelRoute, MODEL_ROUTES } from "./runtime-fixtures";

type AttachmentResolution =
  "blocked" | "compatible" | "converted" | "removed" | "cancelled";

const CAPABILITY_LABELS: Record<CapabilityState, string> = {
  supported: "Supported",
  unsupported: "Unsupported",
  unknown: "Unknown",
  degraded: "Degraded",
};

/** Renders route-scoped effective capability evidence and attachment preflight. */
export function CapabilityPreflightPanel() {
  const [selectedRouteId, setSelectedRouteId] = useState(
    "openrouter-kimi-opencode",
  );
  const [resolution, setResolution] = useState<AttachmentResolution>("blocked");
  const [notice, setNotice] = useState(
    "Submission blocked before runtime dispatch · Vision is required",
  );

  const selectedRoute = getModelRoute(selectedRouteId);
  const visionCapability = selectedRoute.capabilities.find(
    (capability) => capability.label === "Vision",
  );
  const reasoningCapability = selectedRoute.capabilities.find(
    (capability) => capability.label === "Reasoning",
  );

  /** Selects the complete route and recomputes dependent preflight state. */
  function handleRouteSelection(routeId: string) {
    const route = getModelRoute(routeId);
    const supportsVision = route.capabilities.some(
      (capability) =>
        capability.label === "Vision" && capability.state === "supported",
    );

    setSelectedRouteId(routeId);
    setResolution(supportsVision ? "compatible" : "blocked");
    setNotice(
      supportsVision
        ? `${route.model} selected · attachment ready for submission`
        : `Submission blocked before runtime dispatch · ${route.model} does not confirm Vision`,
    );
  }

  /** Applies one explicit conflict resolution without dropping the input. */
  function handleResolution(nextResolution: AttachmentResolution) {
    if (nextResolution === "compatible") {
      handleRouteSelection("openai-gpt5-opencode");
      return;
    }

    setResolution(nextResolution);
    setNotice(
      nextResolution === "converted"
        ? "Attachment converted to concept-board.txt · preflight ready"
        : nextResolution === "removed"
          ? "Attachment removed · text prompt remains ready"
          : "Submission cancelled · draft and attachment were preserved",
    );
  }

  return (
    <section className="runtime-panel" aria-labelledby="capability-route-title">
      <header className="runtime-panel__header">
        <div>
          <p className="runtime-eyebrow">Effective capabilities</p>
          <h2 id="capability-route-title">Model route preflight</h2>
          <p>
            Every choice replaces the complete provider, model revision,
            adapter, runtime, and dependent-control snapshot atomically.
          </p>
        </div>
      </header>

      <div
        className="runtime-route-selector"
        role="group"
        aria-label="Model routes"
      >
        {MODEL_ROUTES.map((route) => (
          <button
            type="button"
            key={route.id}
            aria-pressed={route.id === selectedRouteId}
            onClick={() => handleRouteSelection(route.id)}
          >
            <span>{route.provider}</span>
            <strong>{route.model}</strong>
            <small>
              {route.runtime === "opencode" ? "OpenCode" : "Pi"} ·{" "}
              {route.contextWindow}
            </small>
          </button>
        ))}
      </div>

      <article className="runtime-route-details">
        <header>
          <div>
            <p className="runtime-card__meta">Selected route</p>
            <h3>{selectedRoute.model}</h3>
          </div>
          <span className="runtime-state" data-state="supported">
            Explicit selection
          </span>
        </header>
        <dl className="runtime-route-identity">
          <div>
            <dt>Provider</dt>
            <dd>{selectedRoute.provider}</dd>
          </div>
          <div>
            <dt>Revision</dt>
            <dd>{selectedRoute.revision}</dd>
          </div>
          <div>
            <dt>Adapter</dt>
            <dd>{selectedRoute.adapter}</dd>
          </div>
          <div>
            <dt>Runtime</dt>
            <dd>{selectedRoute.runtime === "opencode" ? "OpenCode" : "Pi"}</dd>
          </div>
        </dl>

        <div
          className="runtime-capability-grid"
          role="list"
          aria-label="Capability evidence"
        >
          {selectedRoute.capabilities.map((capability) => (
            <section
              key={capability.label}
              role="listitem"
              data-capability={capability.state}
            >
              <header>
                <h4>{capability.label}</h4>
                <span className="runtime-state" data-state={capability.state}>
                  {CAPABILITY_LABELS[capability.state]}
                </span>
              </header>
              <p>{capability.reason}</p>
              <small>{capability.evidence}</small>
            </section>
          ))}
        </div>

        <dl className="runtime-dependent-controls">
          <div>
            <dt>Reasoning effort</dt>
            <dd>
              {reasoningCapability?.state === "supported"
                ? "High"
                : reasoningCapability?.state === "degraded"
                  ? "Unavailable · summaries only"
                  : "Unavailable · evidence unknown"}
            </dd>
          </div>
          <div>
            <dt>Attachment input</dt>
            <dd>
              {visionCapability?.state === "supported"
                ? "Images ready"
                : "Text only"}
            </dd>
          </div>
        </dl>
      </article>

      <section
        className="runtime-preflight"
        data-resolution={resolution}
        aria-labelledby="attachment-preflight-title"
      >
        <header>
          <div>
            <p className="runtime-eyebrow">Attachment preflight</p>
            <h3 id="attachment-preflight-title">
              {resolution === "converted"
                ? "concept-board.txt"
                : resolution === "removed"
                  ? "Attachment removed"
                  : "concept-board.png"}
            </h3>
          </div>
          <span
            className="runtime-state"
            data-state={resolution === "blocked" ? "unsupported" : "supported"}
          >
            {resolution === "blocked"
              ? "Needs Vision"
              : resolution === "cancelled"
                ? "Draft preserved"
                : "Ready"}
          </span>
        </header>
        <p className="runtime-notice" role="status">
          {notice}
        </p>

        {resolution === "blocked" ? (
          <div className="runtime-preflight__actions">
            <button
              type="button"
              className="runtime-button runtime-button--primary"
              onClick={() => handleResolution("compatible")}
            >
              Use compatible model
            </button>
            <button
              type="button"
              className="runtime-button"
              onClick={() => handleResolution("converted")}
            >
              Convert attachment
            </button>
            <button
              type="button"
              className="runtime-button"
              onClick={() => handleResolution("removed")}
            >
              Remove attachment
            </button>
            <button
              type="button"
              className="runtime-button runtime-button--quiet"
              onClick={() => handleResolution("cancelled")}
            >
              Cancel submission
            </button>
          </div>
        ) : null}
      </section>
    </section>
  );
}

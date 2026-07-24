import { useState } from "react";

import type { RuntimeKind } from "./runtime-fixtures";
import { getRuntime, RUNTIMES } from "./runtime-fixtures";

/** Renders peer runtime selection, exact pins, health, and restart state. */
export function RuntimeSupervisorPanel() {
  const [savedRuntime, setSavedRuntime] = useState<RuntimeKind>("opencode");
  const [draftRuntime, setDraftRuntime] = useState<RuntimeKind>("opencode");
  const [generations, setGenerations] = useState<Record<RuntimeKind, number>>({
    opencode: 17,
    pi: 9,
  });
  const [notice, setNotice] = useState(
    "OpenCode is the saved default for new Chats.",
  );

  const selectedRuntime = getRuntime(draftRuntime);
  const isDirty = savedRuntime !== draftRuntime;

  /** Saves the draft only as the default for future Chat bindings. */
  function handleSaveRuntime() {
    setSavedRuntime(draftRuntime);
    setNotice(
      `${selectedRuntime.label} ${selectedRuntime.version} saved for new Chats · existing bindings unchanged`,
    );
  }

  /** Advances one supervised process generation while retaining the exact pin. */
  function handleRestart() {
    setGenerations((current) => ({
      ...current,
      [draftRuntime]: current[draftRuntime] + 1,
    }));
    setNotice(
      `${selectedRuntime.label} restarted cleanly · health checked on generation ${generations[draftRuntime] + 1}`,
    );
  }

  return (
    <section
      className="runtime-panel"
      aria-labelledby="runtime-supervisor-title"
    >
      <header className="runtime-panel__header">
        <div>
          <p className="runtime-eyebrow">Runtime supervisor</p>
          <h2 id="runtime-supervisor-title">Peer runtimes</h2>
          <p>
            Exact native versions run behind separate C4OS adapters and one
            process-generation health boundary.
          </p>
        </div>
        <div className="runtime-panel__actions">
          <button
            type="button"
            className="runtime-button"
            onClick={handleRestart}
          >
            Restart {selectedRuntime.label}
          </button>
          <button
            type="button"
            className="runtime-button runtime-button--primary"
            disabled={!isDirty}
            onClick={handleSaveRuntime}
          >
            Save Runtime
          </button>
        </div>
      </header>

      <p className="runtime-notice" role="status">
        {notice}
      </p>

      <fieldset className="runtime-runtime-grid">
        <legend>Default runtime for new Chats</legend>
        {RUNTIMES.map((runtime) => (
          <label
            key={runtime.id}
            className="runtime-card runtime-card--choice"
            data-selected={draftRuntime === runtime.id}
          >
            <input
              type="radio"
              name="runtime-default"
              checked={draftRuntime === runtime.id}
              onChange={() => setDraftRuntime(runtime.id)}
            />
            <span className="runtime-card__body">
              <span className="runtime-card__heading">
                <span>
                  <span className="runtime-card__meta">Exact pin</span>
                  <strong>
                    {runtime.label} {runtime.version}
                  </strong>
                </span>
                <span
                  className="runtime-state"
                  data-state={
                    runtime.health === "Healthy" ? "supported" : "degraded"
                  }
                >
                  {runtime.health}
                </span>
              </span>
              <span>{runtime.healthDetail}</span>
              <small>{runtime.boundary}</small>
              <code>{runtime.digest}</code>
              <span className="runtime-generation">
                Process generation {generations[runtime.id]}
              </span>
            </span>
          </label>
        ))}
      </fieldset>

      <section
        className="runtime-health-strip"
        role="list"
        aria-label="Supervisor guarantees"
      >
        <div role="listitem">
          <strong>Compatibility</strong>
          <span>Exact adapter/native matrix</span>
        </div>
        <div role="listitem">
          <strong>Environment</strong>
          <span>Sanitized and isolated</span>
        </div>
        <div role="listitem">
          <strong>Shutdown</strong>
          <span>Process-group cleanup</span>
        </div>
        <div role="listitem">
          <strong>Recovery</strong>
          <span>Interrupted state retained</span>
        </div>
      </section>
    </section>
  );
}

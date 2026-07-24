import { useState } from "react";

import type { RuntimeKind } from "./runtime-fixtures";

type AttemptState = "Interrupted" | "Running" | "Cancelled" | "Completed";

interface AttemptFixture {
  readonly id: number;
  readonly output: string;
  readonly state: AttemptState;
}

interface SessionBindingFixture {
  readonly environment: "Local";
  readonly model: string;
  readonly runtime: RuntimeKind;
}

const INITIAL_ATTEMPTS: readonly AttemptFixture[] = [
  {
    id: 1,
    state: "Interrupted",
    output:
      "Partial output retained · 2 brokered reads · effect completion unknown",
  },
];

/** Renders first-submit binding and immutable turn/run recovery transitions. */
export function SessionLifecyclePanel() {
  const [newChatDefault, setNewChatDefault] = useState<RuntimeKind>("opencode");
  const [newChatBinding, setNewChatBinding] =
    useState<SessionBindingFixture | null>(null);
  const [attempts, setAttempts] =
    useState<readonly AttemptFixture[]>(INITIAL_ATTEMPTS);
  const [hasStaleSnapshot, setHasStaleSnapshot] = useState(true);
  const [hasUnknownEffect, setHasUnknownEffect] = useState(true);
  const [notice, setNotice] = useState(
    "Recovered after restart · immutable records and partial output retained",
  );

  const latestAttempt = attempts.at(-1);
  const canRetry =
    !hasStaleSnapshot &&
    !hasUnknownEffect &&
    latestAttempt?.state !== "Running";

  /** Captures defaults exactly once on the first valid provisional submission. */
  function handleFirstSubmission() {
    if (newChatBinding) return;

    setNewChatBinding({
      runtime: newChatDefault,
      model:
        newChatDefault === "opencode"
          ? "OpenAI - Work / openai/gpt-5"
          : "Hugging Face - Personal / deepseek-ai/DeepSeek-R1",
      environment: "Local",
    });
    setNotice(
      "Pending Chat promoted · runtime, route, environment, and capability baseline bound atomically",
    );
  }

  /** Creates a fresh attempt while keeping the User Turn and history immutable. */
  function handleRetry() {
    const nextAttempt = attempts.length + 1;

    setAttempts((current) => [
      ...current,
      {
        id: nextAttempt,
        state: "Running",
        output: "Fresh configuration snapshot · new single-use authorizations",
      },
    ]);
    setNotice(
      `Run Attempt ${nextAttempt} started · prior output and audit records remain intact`,
    );
  }

  /** Cancels only the active attempt and leaves the durable session available. */
  function handleCancelAttempt() {
    if (latestAttempt?.state !== "Running") return;

    setAttempts((current) =>
      current.map((attempt) =>
        attempt.id === latestAttempt.id
          ? {
              ...attempt,
              state: "Cancelled",
              output: "Cancellation acknowledged · no pending effect released",
            }
          : attempt,
      ),
    );
    setNotice(
      `Run Attempt ${latestAttempt.id} cancelled · Chat binding remains active`,
    );
  }

  return (
    <section
      className="runtime-panel"
      aria-labelledby="session-lifecycle-title"
    >
      <header className="runtime-panel__header">
        <div>
          <p className="runtime-eyebrow">Session lifecycle</p>
          <h2 id="session-lifecycle-title">Binding and recoverable runs</h2>
          <p>
            A pending Chat binds once. Retry creates a new Run Attempt beneath
            the same immutable User Turn.
          </p>
        </div>
      </header>

      <p className="runtime-notice" role="status">
        {notice}
      </p>

      <section
        className="runtime-session-card"
        aria-labelledby="pending-chat-title"
      >
        <header>
          <div>
            <p className="runtime-card__meta">Provisional Chat</p>
            <h3 id="pending-chat-title">Untitled Chat</h3>
          </div>
          <span
            className="runtime-state"
            data-state={newChatBinding ? "supported" : "unknown"}
          >
            {newChatBinding ? "Bound" : "Pending · not persisted"}
          </span>
        </header>

        <fieldset className="runtime-inline-choice">
          <legend>Default for new Chats</legend>
          <label>
            <input
              type="radio"
              name="new-chat-runtime"
              checked={newChatDefault === "opencode"}
              onChange={() => setNewChatDefault("opencode")}
            />
            OpenCode 1.18.3
          </label>
          <label>
            <input
              type="radio"
              name="new-chat-runtime"
              checked={newChatDefault === "pi"}
              onChange={() => setNewChatDefault("pi")}
            />
            Pi 0.80.10
          </label>
        </fieldset>

        {newChatBinding ? (
          <dl className="runtime-binding-grid">
            <div>
              <dt>Runtime binding</dt>
              <dd>
                {newChatBinding.runtime === "opencode"
                  ? "OpenCode 1.18.3"
                  : "Pi 0.80.10"}
              </dd>
            </div>
            <div>
              <dt>Model route</dt>
              <dd>{newChatBinding.model}</dd>
            </div>
            <div>
              <dt>Environment</dt>
              <dd>{newChatBinding.environment}</dd>
            </div>
            <div>
              <dt>Binding rule</dt>
              <dd>Immutable for this Chat</dd>
            </div>
          </dl>
        ) : (
          <button
            type="button"
            className="runtime-button runtime-button--primary"
            onClick={handleFirstSubmission}
          >
            Submit first valid prompt
          </button>
        )}
      </section>

      <section
        className="runtime-session-card"
        aria-labelledby="recovered-session-title"
      >
        <header>
          <div>
            <p className="runtime-card__meta">Recovered Chat Session</p>
            <h3 id="recovered-session-title">Investigate release pipeline</h3>
          </div>
          <span className="runtime-state" data-state="degraded">
            Review required
          </span>
        </header>

        <article
          className="runtime-turn"
          aria-labelledby="immutable-turn-title"
        >
          <header>
            <div>
              <p className="runtime-card__meta">User Turn · turn-0007</p>
              <h4 id="immutable-turn-title">Immutable prompt snapshot</h4>
            </div>
            <span className="runtime-state" data-state="supported">
              Frozen
            </span>
          </header>
          <blockquote>
            Check the release pipeline and update the failing validation.
          </blockquote>
          <small>
            Attachment: release-log.txt · sha256:82e1…fd03 · captured 05:09
          </small>
        </article>

        <div
          className="runtime-conflict-grid"
          role="list"
          aria-label="Recovery conflicts"
        >
          <article role="listitem" data-resolved={!hasStaleSnapshot}>
            <span
              className="runtime-state"
              data-state={hasStaleSnapshot ? "unsupported" : "supported"}
            >
              {hasStaleSnapshot ? "Stale snapshot" : "Fresh state loaded"}
            </span>
            <h4>Artifact changed after capture</h4>
            <p>
              The live release file no longer matches the turn snapshot. No
              effect may use the stale version.
            </p>
            {hasStaleSnapshot ? (
              <button
                type="button"
                className="runtime-button"
                onClick={() => {
                  setHasStaleSnapshot(false);
                  setNotice(
                    "Current artifact state loaded · immutable turn preserved",
                  );
                }}
              >
                Reload current state
              </button>
            ) : null}
          </article>
          <article role="listitem" data-resolved={!hasUnknownEffect}>
            <span
              className="runtime-state"
              data-state={hasUnknownEffect ? "degraded" : "supported"}
            >
              {hasUnknownEffect ? "Unknown effect" : "Effect reviewed"}
            </span>
            <h4>Completion could not be proven</h4>
            <p>
              Attempt 1 lost its worker after requesting a write. Retry waits
              for review instead of duplicating the effect.
            </p>
            {hasUnknownEffect ? (
              <button
                type="button"
                className="runtime-button"
                onClick={() => {
                  setHasUnknownEffect(false);
                  setNotice(
                    "Unknown effect reviewed · no completed write found",
                  );
                }}
              >
                Review effect
              </button>
            ) : null}
          </article>
        </div>

        <div
          className="runtime-attempts"
          role="list"
          aria-label="Run Attempt history"
        >
          {attempts.map((attempt) => (
            <article
              key={attempt.id}
              role="listitem"
              data-attempt-state={attempt.state.toLowerCase()}
            >
              <div>
                <span className="runtime-card__meta">
                  Run Attempt {attempt.id}
                </span>
                <strong>{attempt.state}</strong>
              </div>
              <p>{attempt.output}</p>
            </article>
          ))}
        </div>

        <div className="runtime-session-actions">
          {latestAttempt?.state === "Running" ? (
            <button
              type="button"
              className="runtime-button"
              onClick={handleCancelAttempt}
            >
              Cancel active attempt
            </button>
          ) : null}
          <button
            type="button"
            className="runtime-button runtime-button--primary"
            disabled={!canRetry}
            onClick={handleRetry}
          >
            Retry immutable turn
          </button>
        </div>
      </section>
    </section>
  );
}

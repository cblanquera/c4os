import { useMemo, useState } from "react";

import {
  createQaFixtureAdapter,
  QA_FIXTURE_GATE_ENV,
  QA_FIXTURE_GATE_VALUE,
  type QaFixtureAdapter,
  type QaFoundationSnapshot,
} from "./fixture";

export const QA_FOUNDATION_PATH = "/qa/foundation";

export interface QaFoundationRouteProps {
  readonly adapter?: QaFixtureAdapter;
}

export function QaFoundationRoute({ adapter }: QaFoundationRouteProps) {
  const fixture = useMemo(() => adapter ?? createQaFixtureAdapter(), [adapter]);
  const [snapshot, setSnapshot] = useState<QaFoundationSnapshot | null>(() =>
    fixture.enabled ? fixture.snapshot() : null,
  );
  const [resetMessage, setResetMessage] = useState("");

  if (!fixture.enabled || snapshot === null) {
    return (
      <main className="qa-foundation" data-qa-fixture-mode="disabled">
        <h1>C4OS QA Foundation</h1>
        <p role="status">Fixture mode unavailable</p>
        <p>
          This route contains no production state. Launch an explicit QA build
          with{" "}
          <code>
            {QA_FIXTURE_GATE_ENV}={QA_FIXTURE_GATE_VALUE}
          </code>{" "}
          to use deterministic fixtures.
        </p>
      </main>
    );
  }

  const resetFixture = () => {
    setSnapshot(fixture.reset());
    setResetMessage(`Fixture reset to ${fixture.now()}`);
  };

  return (
    <main className="qa-foundation" data-qa-fixture-mode="enabled">
      <header>
        <p className="qa-foundation__eyebrow" aria-label="Fixture mode">
          Fixture mode
        </p>
        <h1>C4OS QA Foundation</h1>
        <p>Deterministic QA data only — not production state.</p>
      </header>

      <section
        className="qa-foundation__state"
        aria-labelledby="qa-foundation-state"
      >
        <h2 id="qa-foundation-state">Foundation state</h2>
        <dl>
          <dt>Authority</dt>
          <dd>{snapshot.authority}</dd>
          <dt>Clock</dt>
          <dd>{snapshot.capturedAt}</dd>
          <dt>Workspace ID</dt>
          <dd>{snapshot.ids.workspace}</dd>
          <dt>Project ID</dt>
          <dd>{snapshot.ids.project}</dd>
          <dt>Chat ID</dt>
          <dd>{snapshot.ids.chat}</dd>
          <dt>Correlation ID</dt>
          <dd>{snapshot.ids.correlation}</dd>
        </dl>
      </section>

      <button
        className="qa-foundation__button"
        type="button"
        onClick={resetFixture}
      >
        Reset fixture
      </button>
      <p className="qa-foundation__status" aria-live="polite">
        {resetMessage}
      </p>
    </main>
  );
}

export default QaFoundationRoute;

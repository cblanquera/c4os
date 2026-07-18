export const QA_FIXTURE_GATE_ENV = "VITE_C4OS_QA_FIXTURES";
export const QA_FIXTURE_GATE_VALUE = "1";
export const QA_FIXTURE_CLOCK = "2026-07-18T00:00:00.000Z";

export const QA_FIXTURE_IDS = Object.freeze({
  workspace: "workspace-qa-0001",
  project: "project-qa-0001",
  chat: "chat-qa-0001",
  correlation: "correlation-qa-0001",
});

export interface QaFixtureBuildEnvironment {
  readonly VITE_C4OS_QA_FIXTURES?: string;
}

export interface QaFoundationSnapshot {
  readonly protocolVersion: 1;
  readonly authority: "qa-fixture-only";
  readonly fixtureMode: true;
  readonly route: "/qa/foundation";
  readonly capturedAt: typeof QA_FIXTURE_CLOCK;
  readonly generation: 1;
  readonly ids: typeof QA_FIXTURE_IDS;
}

export interface QaFixtureAdapter {
  readonly enabled: boolean;
  readonly authority: "qa-fixture-only";
  now(): typeof QA_FIXTURE_CLOCK;
  snapshot(): QaFoundationSnapshot;
  reset(): QaFoundationSnapshot;
}

export class QaFixtureDisabledError extends Error {
  constructor() {
    super(
      `Deterministic QA fixtures require ${QA_FIXTURE_GATE_ENV}=${QA_FIXTURE_GATE_VALUE}`,
    );
    this.name = "QaFixtureDisabledError";
  }
}

const FOUNDATION_BASELINE: QaFoundationSnapshot = Object.freeze({
  protocolVersion: 1,
  authority: "qa-fixture-only",
  fixtureMode: true,
  route: "/qa/foundation",
  capturedAt: QA_FIXTURE_CLOCK,
  generation: 1,
  ids: QA_FIXTURE_IDS,
});

function copySnapshot(): QaFoundationSnapshot {
  return {
    ...FOUNDATION_BASELINE,
    ids: { ...FOUNDATION_BASELINE.ids },
  };
}

function readViteEnvironment(): QaFixtureBuildEnvironment {
  return (
    (
      import.meta as ImportMeta & {
        readonly env?: QaFixtureBuildEnvironment;
      }
    ).env ?? {}
  );
}

export function isQaFixtureBuildEnabled(
  environment: QaFixtureBuildEnvironment = readViteEnvironment(),
): boolean {
  return environment.VITE_C4OS_QA_FIXTURES === QA_FIXTURE_GATE_VALUE;
}

/**
 * A deterministic renderer fixture boundary for browser-level QA only.
 *
 * This adapter deliberately has no persistence API and never reads or writes
 * localStorage. Production state must continue to arrive from the typed Rust
 * boundary; enabling this adapter does not grant production authority.
 */
export function createQaFixtureAdapter(options?: {
  readonly enabled?: boolean;
}): QaFixtureAdapter {
  const enabled = options?.enabled ?? isQaFixtureBuildEnabled();
  let state = copySnapshot();

  function requireEnabled(): void {
    if (!enabled) {
      throw new QaFixtureDisabledError();
    }
  }

  return {
    enabled,
    authority: "qa-fixture-only",
    now() {
      requireEnabled();
      return QA_FIXTURE_CLOCK;
    },
    snapshot() {
      requireEnabled();
      return { ...state, ids: { ...state.ids } };
    },
    reset() {
      requireEnabled();
      state = copySnapshot();
      return { ...state, ids: { ...state.ids } };
    },
  };
}

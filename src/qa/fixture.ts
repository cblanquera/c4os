import {
  APP_ROUTE_DEFINITIONS,
  type AppRoutePath,
} from "../app/route-contract";

export const QA_FIXTURE_GATE_ENV = "VITE_C4OS_QA_FIXTURES";
export const QA_FIXTURE_GATE_VALUE = "1";
export const QA_FIXTURE_CLOCK = "2026-07-18T00:00:00.000Z";
export const QA_FIXTURE_SCHEMA_VERSION = 1;
export const QA_FIXTURE_SCENARIO_ID = "spec-00003-integrated-r013";
export const QA_FIXTURE_DEFAULT_ISOLATION_ID = "qa-isolation-default";

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
  readonly schemaVersion: typeof QA_FIXTURE_SCHEMA_VERSION;
  readonly authority: "qa-fixture-only";
  readonly fixtureMode: true;
  readonly scenarioId: typeof QA_FIXTURE_SCENARIO_ID;
  readonly isolationId: string;
  readonly route: "/qa/foundation";
  readonly activeRoute: "/qa/foundation" | AppRoutePath;
  readonly capturedAt: typeof QA_FIXTURE_CLOCK;
  readonly generation: number;
  readonly ids: typeof QA_FIXTURE_IDS;
  readonly replay: readonly QaFixtureEvent[];
}

export interface QaFixtureEvent {
  readonly schemaVersion: typeof QA_FIXTURE_SCHEMA_VERSION;
  readonly sequence: number;
  readonly type: "route-launched";
  readonly route: AppRoutePath;
  readonly capturedAt: typeof QA_FIXTURE_CLOCK;
}

export interface QaDirectRouteLaunch {
  readonly route: AppRoutePath;
  readonly hashHref: `#${AppRoutePath}`;
  readonly snapshot: QaFoundationSnapshot;
}

export interface QaFixtureAdapter {
  readonly enabled: boolean;
  readonly authority: "qa-fixture-only";
  readonly isolationId: string;
  readonly routes: readonly AppRoutePath[];
  now(): typeof QA_FIXTURE_CLOCK;
  snapshot(): QaFoundationSnapshot;
  reset(): QaFoundationSnapshot;
  launchRoute(route: AppRoutePath): QaDirectRouteLaunch;
  replay(events: readonly QaFixtureEvent[]): QaFoundationSnapshot;
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
  schemaVersion: QA_FIXTURE_SCHEMA_VERSION,
  authority: "qa-fixture-only",
  fixtureMode: true,
  scenarioId: QA_FIXTURE_SCENARIO_ID,
  isolationId: QA_FIXTURE_DEFAULT_ISOLATION_ID,
  route: "/qa/foundation",
  activeRoute: "/qa/foundation",
  capturedAt: QA_FIXTURE_CLOCK,
  generation: 1,
  ids: QA_FIXTURE_IDS,
  replay: [],
});

const QA_DIRECT_ROUTES = Object.freeze(
  APP_ROUTE_DEFINITIONS.map(({ path }) => path),
) as readonly AppRoutePath[];

function copySnapshot(
  source: QaFoundationSnapshot = FOUNDATION_BASELINE,
): QaFoundationSnapshot {
  return {
    ...source,
    ids: { ...source.ids },
    replay: source.replay.map((event) => ({ ...event })),
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
  readonly isolationId?: string;
}): QaFixtureAdapter {
  const enabled = options?.enabled ?? isQaFixtureBuildEnabled();
  const isolationId =
    options?.isolationId?.trim() || QA_FIXTURE_DEFAULT_ISOLATION_ID;
  let state: QaFoundationSnapshot = {
    ...copySnapshot(),
    isolationId,
  };

  function requireEnabled(): void {
    if (!enabled) {
      throw new QaFixtureDisabledError();
    }
  }

  return {
    enabled,
    authority: "qa-fixture-only",
    isolationId,
    routes: QA_DIRECT_ROUTES,
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
      state = {
        ...copySnapshot(),
        isolationId,
      };
      return copySnapshot(state);
    },
    launchRoute(route) {
      requireEnabled();
      if (!QA_DIRECT_ROUTES.includes(route)) {
        throw new Error(`Unaccepted deterministic QA route: ${route}`);
      }
      const event: QaFixtureEvent = {
        schemaVersion: QA_FIXTURE_SCHEMA_VERSION,
        sequence: state.replay.length + 1,
        type: "route-launched",
        route,
        capturedAt: QA_FIXTURE_CLOCK,
      };
      state = {
        ...state,
        activeRoute: route,
        generation: state.generation + 1,
        replay: [...state.replay, event],
      };
      return {
        route,
        hashHref: `#${route}`,
        snapshot: copySnapshot(state),
      };
    },
    replay(events) {
      requireEnabled();
      state = {
        ...copySnapshot(),
        isolationId,
      };
      for (const [index, event] of events.entries()) {
        const expectedSequence = index + 1;
        if (
          event.schemaVersion !== QA_FIXTURE_SCHEMA_VERSION ||
          event.sequence !== expectedSequence ||
          event.type !== "route-launched" ||
          event.capturedAt !== QA_FIXTURE_CLOCK
        ) {
          throw new Error(
            `Invalid deterministic QA replay event at sequence ${expectedSequence}`,
          );
        }
        const replayed = this.launchRoute(event.route).snapshot.replay.at(-1);
        if (JSON.stringify(replayed) !== JSON.stringify(event)) {
          throw new Error(
            `Non-deterministic QA replay event at sequence ${expectedSequence}`,
          );
        }
      }
      return copySnapshot(state);
    },
  };
}

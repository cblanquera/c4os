import { describe, expect, it } from "vitest";

import {
  createQaFixtureAdapter,
  isQaFixtureBuildEnabled,
  QA_FIXTURE_CLOCK,
  QA_FIXTURE_IDS,
  QA_FIXTURE_SCHEMA_VERSION,
  QA_FIXTURE_SCENARIO_ID,
  QaFixtureDisabledError,
} from "../../../src/frontend/qa/fixture";

describe("deterministic QA fixture adapter", () => {
  it("requires the explicit build gate", () => {
    expect(isQaFixtureBuildEnabled({})).toBe(false);
    expect(isQaFixtureBuildEnabled({ VITE_C4OS_QA_FIXTURES: "true" })).toBe(
      false,
    );
    expect(isQaFixtureBuildEnabled({ VITE_C4OS_QA_FIXTURES: "1" })).toBe(true);
  });

  it("fails closed when fixture mode is disabled", () => {
    const adapter = createQaFixtureAdapter({ enabled: false });

    expect(adapter.enabled).toBe(false);
    expect(() => adapter.snapshot()).toThrow(QaFixtureDisabledError);
    expect(() => adapter.now()).toThrow(QaFixtureDisabledError);
    expect(() => adapter.reset()).toThrow(QaFixtureDisabledError);
    expect(() => adapter.launchRoute("/chat")).toThrow(QaFixtureDisabledError);
    expect(() => adapter.replay([])).toThrow(QaFixtureDisabledError);
  });

  it("uses fixed IDs and a fixed clock", () => {
    const adapter = createQaFixtureAdapter({ enabled: true });

    expect(adapter.now()).toBe(QA_FIXTURE_CLOCK);
    expect(adapter.snapshot()).toEqual(
      expect.objectContaining({
        schemaVersion: QA_FIXTURE_SCHEMA_VERSION,
        scenarioId: QA_FIXTURE_SCENARIO_ID,
        capturedAt: QA_FIXTURE_CLOCK,
        ids: QA_FIXTURE_IDS,
      }),
    );
  });

  it("resets to a fresh copy of the exact baseline", () => {
    const adapter = createQaFixtureAdapter({ enabled: true });
    const first = adapter.snapshot();
    const reset = adapter.reset();

    expect(reset).toEqual(first);
    expect(reset).not.toBe(first);
    expect(reset.ids).not.toBe(first.ids);
  });

  it("does not expose mutable adapter state through a snapshot", () => {
    const adapter = createQaFixtureAdapter({ enabled: true });
    const snapshot = adapter.snapshot();

    (snapshot.ids as { workspace: string }).workspace = "mutated";

    expect(adapter.snapshot().ids.workspace).toBe(QA_FIXTURE_IDS.workspace);
  });

  it("derives all 16 accepted direct routes from the application contract", () => {
    const adapter = createQaFixtureAdapter({ enabled: true });

    expect(adapter.routes).toHaveLength(16);
    expect(new Set(adapter.routes).size).toBe(16);
    expect(adapter.launchRoute("/settings/advanced-policies")).toMatchObject({
      route: "/settings/advanced-policies",
      hashHref: "#/settings/advanced-policies",
      snapshot: {
        activeRoute: "/settings/advanced-policies",
        generation: 2,
      },
    });
  });

  it("replays an exact route trace from the deterministic baseline", () => {
    const adapter = createQaFixtureAdapter({
      enabled: true,
      isolationId: "qa-replay",
    });
    adapter.launchRoute("/chat");
    const expected = adapter.launchRoute("/settings/models").snapshot;

    expect(adapter.replay(expected.replay)).toEqual(expected);
  });

  it("keeps independently created fixture adapters isolated", () => {
    const first = createQaFixtureAdapter({
      enabled: true,
      isolationId: "qa-first",
    });
    const second = createQaFixtureAdapter({
      enabled: true,
      isolationId: "qa-second",
    });

    first.launchRoute("/terminal");

    expect(first.snapshot()).toMatchObject({
      isolationId: "qa-first",
      activeRoute: "/terminal",
      generation: 2,
    });
    expect(second.snapshot()).toMatchObject({
      isolationId: "qa-second",
      activeRoute: "/qa/foundation",
      generation: 1,
      replay: [],
    });
  });
});

import { describe, expect, it } from "vitest";

import {
  createQaFixtureAdapter,
  isQaFixtureBuildEnabled,
  QA_FIXTURE_CLOCK,
  QA_FIXTURE_IDS,
  QaFixtureDisabledError,
} from "./fixture";

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
  });

  it("uses fixed IDs and a fixed clock", () => {
    const adapter = createQaFixtureAdapter({ enabled: true });

    expect(adapter.now()).toBe(QA_FIXTURE_CLOCK);
    expect(adapter.snapshot()).toEqual(
      expect.objectContaining({
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
});

import { describe, expect, it } from "vitest";

import { resolveBuildGatedQaRootEntry } from "../../../src/frontend/app/qa-entry";

describe("build-gated native QA root entry", () => {
  it("permits the production-composed Chat route only in a QA build", () => {
    expect(resolveBuildGatedQaRootEntry(true, "chat")).toBe("chat");
    expect(resolveBuildGatedQaRootEntry(false, "chat")).toBeNull();
  });

  it("permits the direct-route launcher only in a QA build", () => {
    expect(resolveBuildGatedQaRootEntry(true, "foundation")).toBe("foundation");
    expect(resolveBuildGatedQaRootEntry(false, "foundation")).toBeNull();
  });

  it("fails closed for absent and unknown entries", () => {
    expect(resolveBuildGatedQaRootEntry(true, undefined)).toBeNull();
    expect(resolveBuildGatedQaRootEntry(true, "detached-window")).toBeNull();
  });
});

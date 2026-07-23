import { describe, expect, it } from "vitest";

import { PROTOCOL_VERSION } from "./protocol";
import {
  POLICY_SETTING_KEYS,
  createPolicyAdapter,
  type PolicyDecision,
  type PolicySettingKey,
  type PolicyTransport,
} from "./policy-service";

const REQUEST_ID = "request:00000000-0000-4000-8000-000000000041";
const CORRELATION_ID = "correlation:00000000-0000-4000-8000-000000000042";

function categories(
  decision: PolicyDecision | null = null,
): Record<PolicySettingKey, PolicyDecision | null> {
  return Object.fromEntries(
    POLICY_SETTING_KEYS.map((key) => [key, decision]),
  ) as Record<PolicySettingKey, PolicyDecision | null>;
}

function payload(policyVersion = 1, coordinatorGeneration = 0) {
  return {
    authority: "rust-policy-service",
    coordinatorGeneration,
    policyVersion,
    revocationEpoch: policyVersion - 1,
    preset: policyVersion === 1 ? "ask-for-approval" : "custom",
    basePreset: "ask-for-approval",
    categoryValues: categories(policyVersion === 1 ? null : "ask"),
    effectiveCategoryValues: categories(policyVersion === 1 ? null : "ask"),
    exceptions: [
      {
        exceptionId: "exception-one",
        decision: "allow",
        action: "workspace.modify",
        scope: "README.md · workspace-one",
        source: "opencode-primary",
        duration: "Current Chat session",
      },
    ],
    maximumAuthorityRuleCount: 2,
    managedRequirementCount: 1,
  };
}

class FixtureTransport implements PolicyTransport {
  readonly calls: Array<{
    command: Parameters<PolicyTransport["invoke"]>[0];
    args: Readonly<Record<string, unknown>>;
  }> = [];

  async invoke(
    command: Parameters<PolicyTransport["invoke"]>[0],
    args: Readonly<Record<string, unknown>>,
  ): Promise<unknown> {
    this.calls.push({ command, args });
    const request = args.request as {
      requestId: string;
      correlationId: string;
    };
    const policyVersion = command === "policy_snapshot" ? 1 : this.calls.length;
    return {
      protocolVersion: PROTOCOL_VERSION,
      requestId: request.requestId,
      correlationId: request.correlationId,
      generation: policyVersion,
      payload: payload(policyVersion, policyVersion - 1),
    };
  }
}

describe("policy service adapter", () => {
  it("round-trips the exact 28-key schema with both CAS identities", async () => {
    const transport = new FixtureTransport();
    const adapter = createPolicyAdapter(transport, {
      requestIdFactory: () => REQUEST_ID as never,
      correlationIdFactory: () => CORRELATION_ID as never,
    });
    const snapshot = await adapter.readSnapshot();
    expect(Object.keys(snapshot.categoryValues)).toEqual(POLICY_SETTING_KEYS);
    expect(Object.keys(snapshot.effectiveCategoryValues)).toEqual(
      POLICY_SETTING_KEYS,
    );
    expect(snapshot.preset).toBe("ask-for-approval");

    await adapter.save(categories("ask"));
    expect(transport.calls[1]?.args.input).toEqual({
      expectedCoordinatorGeneration: 0,
      expectedPolicyVersion: 1,
      categoryValues: categories("ask"),
    });
  });

  it("uses the returned authority for an immediate exception revocation", async () => {
    const transport = new FixtureTransport();
    const adapter = createPolicyAdapter(transport, {
      requestIdFactory: () => REQUEST_ID as never,
      correlationIdFactory: () => CORRELATION_ID as never,
    });
    await adapter.readSnapshot();
    await adapter.save(categories("ask"));
    await adapter.revokeException("exception-one");
    expect(transport.calls[2]?.args.input).toEqual({
      expectedCoordinatorGeneration: 1,
      expectedPolicyVersion: 2,
      exceptionId: "exception-one",
    });
  });

  it("rejects an incomplete category payload before native invocation", async () => {
    const transport = new FixtureTransport();
    const adapter = createPolicyAdapter(transport, {
      requestIdFactory: () => REQUEST_ID as never,
      correlationIdFactory: () => CORRELATION_ID as never,
    });
    await adapter.readSnapshot();
    expect(() =>
      adapter.save({ "workspace.read": "allow" } as never),
    ).toThrowError(expect.objectContaining({ code: "invalidPayload" }));
    expect(transport.calls).toHaveLength(1);
  });
});

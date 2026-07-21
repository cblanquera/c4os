import { describe, expect, it } from "vitest";

import {
  createAttachmentDraft,
  reconcileAttachmentDraft,
  restoreAttachmentDraft,
  type AttachmentDraftInput,
} from "./attachment-draft";

type TestAttachment = AttachmentDraftInput & {
  readonly name: string;
};

/** Builds one authoritative input without supplying a display reference. */
function attachment(id: string): TestAttachment {
  return { id, name: `${id}.txt` };
}

describe("attachment draft references", () => {
  it("allocates one-based references without mutating the prior draft", () => {
    const empty = createAttachmentDraft<TestAttachment>();

    const populated = reconcileAttachmentDraft(empty, [
      attachment("one"),
      attachment("two"),
    ]);

    expect(empty).toEqual({ attachments: [], nextReferenceNumber: 1 });
    expect(
      populated.attachments.map(({ id, referenceNumber }) => ({
        id,
        referenceNumber,
      })),
    ).toEqual([
      { id: "one", referenceNumber: 1 },
      { id: "two", referenceNumber: 2 },
    ]);
    expect(populated.nextReferenceNumber).toBe(3);
  });

  it("preserves survivor references and never reuses a removed number", () => {
    const populated = reconcileAttachmentDraft(
      createAttachmentDraft<TestAttachment>(),
      [attachment("one"), attachment("two"), attachment("three")],
    );
    const afterRemoval = reconcileAttachmentDraft(populated, [
      attachment("one"),
      attachment("three"),
    ]);

    const afterAddition = reconcileAttachmentDraft(afterRemoval, [
      attachment("one"),
      attachment("three"),
      attachment("four"),
    ]);

    expect(
      afterRemoval.attachments.map(({ id, referenceNumber }) => ({
        id,
        referenceNumber,
      })),
    ).toEqual([
      { id: "one", referenceNumber: 1 },
      { id: "three", referenceNumber: 3 },
    ]);
    expect(
      afterAddition.attachments.map(({ id, referenceNumber }) => ({
        id,
        referenceNumber,
      })),
    ).toEqual([
      { id: "one", referenceNumber: 1 },
      { id: "three", referenceNumber: 3 },
      { id: "four", referenceNumber: 4 },
    ]);
    expect(afterAddition.nextReferenceNumber).toBe(5);
  });

  it("assigns a fresh reference when a removed identity is added again", () => {
    const first = reconcileAttachmentDraft(
      createAttachmentDraft<TestAttachment>(),
      [attachment("same")],
    );
    const removed = reconcileAttachmentDraft(first, []);

    const addedAgain = reconcileAttachmentDraft(removed, [attachment("same")]);

    expect(addedAgain.attachments[0]?.referenceNumber).toBe(2);
    expect(addedAgain.nextReferenceNumber).toBe(3);
  });

  it("continues from a persisted counter even when removed references are absent", () => {
    const restored = restoreAttachmentDraft(
      [{ ...attachment("one"), referenceNumber: 1 }],
      4,
    );

    const afterRestart = reconcileAttachmentDraft(restored, [
      attachment("one"),
      attachment("four"),
    ]);

    expect(
      afterRestart.attachments.map(({ id, referenceNumber }) => ({
        id,
        referenceNumber,
      })),
    ).toEqual([
      { id: "one", referenceNumber: 1 },
      { id: "four", referenceNumber: 4 },
    ]);
    expect(afterRestart.nextReferenceNumber).toBe(5);
  });

  it("rejects duplicate identities instead of assigning ambiguous references", () => {
    const empty = createAttachmentDraft<TestAttachment>();

    expect(() =>
      reconcileAttachmentDraft(empty, [attachment("same"), attachment("same")]),
    ).toThrow("Duplicate attachment identity: same");
  });
});

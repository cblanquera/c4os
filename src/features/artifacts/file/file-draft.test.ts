import { describe, expect, it } from "vitest";

import {
  commitFileDraft,
  createFileDraft,
  discardFileDraft,
  updateFileDraft,
} from "./file-draft";

describe("immutable file draft", () => {
  it("preserves its base while edits move between clean and dirty", () => {
    const original = createFileDraft("alpha\nbeta", 4);
    const dirty = updateFileDraft(original, "alpha\ngamma");
    const cleanAgain = updateFileDraft(dirty, "alpha\nbeta");

    expect(original).toEqual({
      baseContent: "alpha\nbeta",
      baseVersion: 4,
      content: "alpha\nbeta",
      dirty: false,
    });
    expect(dirty).toEqual({
      baseContent: "alpha\nbeta",
      baseVersion: 4,
      content: "alpha\ngamma",
      dirty: true,
    });
    expect(cleanAgain.dirty).toBe(false);
    expect(original).not.toBe(dirty);
    expect(Object.isFrozen(dirty)).toBe(true);
  });

  it("discards locally and advances its base only after broker confirmation", () => {
    const dirty = updateFileDraft(createFileDraft("one", 7), "two");

    expect(discardFileDraft(dirty)).toEqual({
      baseContent: "one",
      baseVersion: 7,
      content: "one",
      dirty: false,
    });
    expect(commitFileDraft(dirty, 8)).toEqual({
      baseContent: "two",
      baseVersion: 8,
      content: "two",
      dirty: false,
    });
    expect(() => commitFileDraft(dirty, 7)).toThrow(
      "must advance the draft base version",
    );
  });
});

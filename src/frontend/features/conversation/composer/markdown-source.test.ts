import { describe, expect, it } from "vitest";

import { linkSourceSelection, toggleSourceMarker } from "./markdown-source";

describe("Markdown source transformations", () => {
  it("wraps, unwraps, and inserts source markers without parsing the document", () => {
    expect(toggleSourceMarker("plain text", 6, 10, "**")).toEqual({
      rangeStart: 6,
      rangeEnd: 10,
      replacement: "**text**",
      selectionStart: 8,
      selectionEnd: 12,
    });
    expect(toggleSourceMarker("plain **text**", 8, 12, "**")).toEqual({
      rangeStart: 6,
      rangeEnd: 14,
      replacement: "text",
      selectionStart: 6,
      selectionEnd: 10,
    });
    expect(toggleSourceMarker("plain **text**", 6, 14, "**")).toEqual({
      rangeStart: 6,
      rangeEnd: 14,
      replacement: "text",
      selectionStart: 6,
      selectionEnd: 10,
    });
    expect(toggleSourceMarker("", 0, 0, "*")).toEqual({
      rangeStart: 0,
      rangeEnd: 0,
      replacement: "**",
      selectionStart: 1,
      selectionEnd: 1,
    });
  });

  it("accepts only HTTP URLs and requires selected source", () => {
    expect(
      linkSourceSelection("Open docs", 5, 9, " https://example.com "),
    ).toEqual({
      rangeStart: 5,
      rangeEnd: 9,
      replacement: "[docs](https://example.com)",
      selectionStart: 32,
      selectionEnd: 32,
    });
    expect(
      linkSourceSelection("Open docs", 5, 9, "javascript:alert(1)"),
    ).toBeNull();
    expect(
      linkSourceSelection("https://example.com", 0, 0, "https://example.com"),
    ).toBeNull();
  });
});

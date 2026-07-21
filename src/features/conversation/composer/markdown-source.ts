export interface SourceReplacement {
  readonly rangeEnd: number;
  readonly rangeStart: number;
  readonly replacement: string;
  readonly selectionEnd: number;
  readonly selectionStart: number;
}

/** Produces a source-marker toggle without interpreting or rewriting Markdown. */
export function toggleSourceMarker(
  source: string,
  selectionStart: number,
  selectionEnd: number,
  marker: "*" | "**",
): SourceReplacement {
  const selectedSource = source.slice(selectionStart, selectionEnd);
  const markerLength = marker.length;

  if (
    selectedSource.startsWith(marker) &&
    selectedSource.endsWith(marker) &&
    selectedSource.length >= markerLength * 2
  ) {
    const replacement = selectedSource.slice(markerLength, -markerLength);
    return {
      rangeEnd: selectionEnd,
      rangeStart: selectionStart,
      replacement,
      selectionEnd: selectionStart + replacement.length,
      selectionStart,
    };
  }

  const outerStart = selectionStart - markerLength;
  const outerEnd = selectionEnd + markerLength;
  if (
    outerStart >= 0 &&
    source.slice(outerStart, selectionStart) === marker &&
    source.slice(selectionEnd, outerEnd) === marker
  ) {
    return {
      rangeEnd: outerEnd,
      rangeStart: outerStart,
      replacement: selectedSource,
      selectionEnd: outerStart + selectedSource.length,
      selectionStart: outerStart,
    };
  }

  return {
    rangeEnd: selectionEnd,
    rangeStart: selectionStart,
    replacement: `${marker}${selectedSource}${marker}`,
    selectionEnd: selectionEnd + markerLength,
    selectionStart: selectionStart + markerLength,
  };
}

/** Produces a Markdown link only for a safe pasted web URL over selected source. */
export function linkSourceSelection(
  source: string,
  selectionStart: number,
  selectionEnd: number,
  pastedText: string,
): SourceReplacement | null {
  const selectedSource = source.slice(selectionStart, selectionEnd);
  const normalizedUrl = pastedText.trim();

  if (selectedSource.length === 0 || !isSafeWebUrl(normalizedUrl)) {
    return null;
  }

  const replacement = `[${selectedSource}](${normalizedUrl})`;
  return {
    rangeEnd: selectionEnd,
    rangeStart: selectionStart,
    replacement,
    selectionEnd: selectionStart + replacement.length,
    selectionStart: selectionStart + replacement.length,
  };
}

/** Restricts automatic link construction to ordinary HTTP and HTTPS addresses. */
function isSafeWebUrl(value: string): boolean {
  try {
    const parsedUrl = new URL(value);
    return parsedUrl.protocol === "http:" || parsedUrl.protocol === "https:";
  } catch {
    return false;
  }
}

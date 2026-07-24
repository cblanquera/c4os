export interface FileDraftSnapshot {
  readonly baseContent: string;
  readonly baseVersion: number;
  readonly content: string;
  readonly dirty: boolean;
}

/** Creates a clean immutable editor draft from one authoritative file version. */
export function createFileDraft(
  content: string,
  baseVersion: number,
): FileDraftSnapshot {
  validateVersion(baseVersion);
  return Object.freeze({
    baseContent: content,
    baseVersion,
    content,
    dirty: false,
  });
}

/** Returns a new draft while retaining the exact authoritative comparison base. */
export function updateFileDraft(
  draft: FileDraftSnapshot,
  content: string,
): FileDraftSnapshot {
  return Object.freeze({
    ...draft,
    content,
    dirty: content !== draft.baseContent,
  });
}

/** Discards local edits without changing the authoritative version cursor. */
export function discardFileDraft(draft: FileDraftSnapshot): FileDraftSnapshot {
  return Object.freeze({
    ...draft,
    content: draft.baseContent,
    dirty: false,
  });
}

/** Marks a broker-confirmed save as the next immutable comparison base. */
export function commitFileDraft(
  draft: FileDraftSnapshot,
  savedVersion: number,
): FileDraftSnapshot {
  validateVersion(savedVersion);
  if (savedVersion <= draft.baseVersion) {
    throw new Error(
      "A saved file version must advance the draft base version.",
    );
  }
  return Object.freeze({
    baseContent: draft.content,
    baseVersion: savedVersion,
    content: draft.content,
    dirty: false,
  });
}

/** Rejects invalid version cursors before they can enter renderer draft state. */
function validateVersion(version: number): void {
  if (!Number.isSafeInteger(version) || version < 0) {
    throw new Error("File draft versions must be nonnegative integers.");
  }
}

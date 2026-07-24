export type QaRootEntry =
  "chat" | "foundation" | "platform" | "policy" | "runtime";

const QA_ROOT_ENTRIES = new Set<QaRootEntry>([
  "chat",
  "foundation",
  "platform",
  "policy",
  "runtime",
]);

/**
 * Resolves an alternate native root only in a build that compiled QA fixtures.
 * Ordinary builds cannot enable the selector at runtime.
 */
export function resolveBuildGatedQaRootEntry(
  fixturesEnabled: boolean,
  candidate: string | undefined,
): QaRootEntry | null {
  if (!fixturesEnabled || !candidate) return null;
  return QA_ROOT_ENTRIES.has(candidate as QaRootEntry)
    ? (candidate as QaRootEntry)
    : null;
}

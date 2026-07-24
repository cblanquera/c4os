import type { ShellPreloadedState } from "../features/shell/state";

/** Production has no fixture state module in its dependency graph. */
export function createBuildGatedQaPreloadedState():
  ShellPreloadedState | undefined {
  return undefined;
}

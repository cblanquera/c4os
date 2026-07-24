import { useSyncExternalStore } from "react";

export type PlatformReviewState =
  | "Default"
  | "Hover"
  | "Pressed"
  | "Selected"
  | "Disabled"
  | "Dirty"
  | "Loading";

type ReviewStateListener = () => void;

const listeners = new Set<ReviewStateListener>();
let currentReviewState: PlatformReviewState = "Selected";

function subscribe(listener: ReviewStateListener): () => void {
  listeners.add(listener);
  return () => listeners.delete(listener);
}

function readReviewState(): PlatformReviewState {
  return currentReviewState;
}

export function setPlatformReviewState(state: PlatformReviewState): void {
  if (state === currentReviewState) return;

  currentReviewState = state;
  listeners.forEach((listener) => listener());
}

export function usePlatformReviewState(): PlatformReviewState {
  return useSyncExternalStore(subscribe, readReviewState, readReviewState);
}

/** Keeps each test independent without adding a persisted product override. */
export function resetPlatformReviewStateForTests(): void {
  currentReviewState = "Selected";
  listeners.forEach((listener) => listener());
}

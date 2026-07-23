import { useCallback, useEffect, useState } from "react";

import {
  readProviderSnapshot,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";

export type ProviderSnapshotState =
  | { readonly status: "error"; readonly message: string }
  | { readonly status: "loading" }
  | {
      readonly status: "ready";
      readonly snapshot: ProviderSettingsSnapshot;
    };

export type ProviderSnapshotController = {
  readonly state: ProviderSnapshotState;
  readonly publish: (snapshot: ProviderSettingsSnapshot) => void;
  readonly retry: () => void;
};

/** Loads and republishes the native Provider service snapshot. */
export function useProviderSnapshot(): ProviderSnapshotController {
  const [state, setState] = useState<ProviderSnapshotState>({
    status: "loading",
  });
  const [request, setRequest] = useState(0);

  const publish = useCallback((snapshot: ProviderSettingsSnapshot) => {
    setState({ status: "ready", snapshot });
  }, []);

  /** Requests a fresh native snapshot after a bounded failure. */
  function retry() {
    setState({ status: "loading" });
    setRequest((current) => current + 1);
  }

  useEffect(() => {
    let active = true;
    void readProviderSnapshot().then(
      (snapshot) => {
        if (active) publish(snapshot);
      },
      (error: unknown) => {
        if (!active) return;
        setState({
          status: "error",
          message:
            error instanceof Error
              ? error.message
              : "The native Provider service is unavailable.",
        });
      },
    );
    return () => {
      active = false;
    };
  }, [publish, request]);

  return { state, publish, retry };
}

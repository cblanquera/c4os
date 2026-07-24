import { useState } from "react";

import { Button, Notice } from "../../../components/accessible";
import {
  acceptProviderSessionCredentials,
  type ProviderSettingsSnapshot,
} from "../../../platform/provider-service";

export type ProviderCredentialFallbackNoticeProps = {
  readonly onSnapshot: (snapshot: ProviderSettingsSnapshot) => void;
  readonly snapshot: ProviderSettingsSnapshot;
};

/** Requires an explicit choice before a memory-only credential vault is used. */
export function ProviderCredentialFallbackNotice({
  onSnapshot,
  snapshot,
}: ProviderCredentialFallbackNoticeProps) {
  const [isAccepting, setIsAccepting] = useState(false);
  const [operationError, setOperationError] = useState<string | null>(null);

  if (snapshot.credentialProtection !== "session-only") return null;

  async function acceptSessionOnly() {
    setIsAccepting(true);
    setOperationError(null);
    try {
      onSnapshot(await acceptProviderSessionCredentials());
    } catch (error) {
      setOperationError(
        error instanceof Error
          ? error.message
          : "Session-only credential storage could not be enabled.",
      );
    } finally {
      setIsAccepting(false);
    }
  }

  if (!snapshot.credentialFallbackRequired) {
    return (
      <Notice title="Session-only credentials active" tone="warning">
        Credentials stay only in memory until C4OS quits. Provider profiles
        remain saved, but keys must be re-entered after restart.
      </Notice>
    );
  }

  return (
    <Notice
      title="macOS secure credential storage is unavailable"
      tone="warning"
    >
      <p>
        C4OS can keep provider keys only in memory for this app session. No key
        is written to configuration, Workspace files, logs, or plaintext
        fallback storage.
      </p>
      {operationError ? <p>{operationError}</p> : null}
      <Button isDisabled={isAccepting} onPress={() => void acceptSessionOnly()}>
        {isAccepting
          ? "Enabling session-only credentials…"
          : "Use session-only credentials"}
      </Button>
    </Notice>
  );
}

import type { McpVisibleState } from "./types";

export function stateLabel(state: McpVisibleState): string {
  return {
    connecting: "Connecting",
    denied: "Denied",
    disabled: "Disabled",
    executing: "Executing",
    failed: "Failed",
    ready: "Ready",
    restarting: "Restarting",
    revoked: "Revoked",
    testing: "Testing",
    timedOut: "Timed out",
  }[state];
}

import { invoke } from "@tauri-apps/api/core";

export type NativeCommand = "foundation_snapshot" | "workspace_start_snapshot";

/**
 * The only renderer import of Tauri's generic invoke primitive. Product
 * adapters expose allowlisted commands and validate every returned payload.
 */
export function invokeNative(
  command: NativeCommand,
  args: Readonly<Record<string, unknown>>,
): Promise<unknown> {
  return invoke(command, args);
}

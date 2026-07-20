import { invoke } from "@tauri-apps/api/core";

export type NativeCommand =
  | "foundation_snapshot"
  | "workspace_start_snapshot"
  | "runtime_core_snapshot"
  | "runtime_production_activate"
  | "runtime_production_shutdown"
  | "runtime_production_pump"
  | "runtime_production_answer_approval";

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

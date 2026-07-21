import { invoke } from "@tauri-apps/api/core";

export type NativeCommand =
  | "platform_snapshot"
  | "platform_reveal_main"
  | "platform_pick"
  | "foundation_snapshot"
  | "workspace_start_snapshot"
  | "conversation_snapshot"
  | "conversation_attachment_preview"
  | "conversation_request_branch"
  | "conversation_answer_branch_approval"
  | "conversation_begin_pending"
  | "conversation_cancel_pending"
  | "conversation_update_draft"
  | "conversation_submit"
  | "conversation_cancel_attempt"
  | "conversation_retry_attempt"
  | "conversation_activate_session"
  | "conversation_activate_project"
  | "conversation_add_project"
  | "conversation_relocate_project"
  | "conversation_rename_project"
  | "conversation_copy_project_path"
  | "conversation_reveal_project"
  | "conversation_reorder_projects"
  | "conversation_inactivate_project"
  | "conversation_inactivate_session"
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

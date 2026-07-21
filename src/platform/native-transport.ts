import { invoke } from "@tauri-apps/api/core";

export type NativeCommand =
  | "platform_snapshot"
  | "platform_reveal_main"
  | "platform_pick"
  | "foundation_snapshot"
  | "workspace_start_snapshot"
  | "conversation_snapshot"
  | "artifact_snapshot"
  | "artifact_run_terminal"
  | "artifact_terminal_stdin"
  | "artifact_terminal_resize"
  | "artifact_terminal_stop"
  | "artifact_terminal_ack_output"
  | "artifact_open_file"
  | "artifact_open_folder"
  | "artifact_focus"
  | "artifact_close_focus"
  | "artifact_begin_file_edit"
  | "artifact_update_file_draft"
  | "artifact_discard_file_draft"
  | "artifact_reject_file_proposal"
  | "artifact_resolve_file_conflict"
  | "artifact_save_file"
  | "artifact_answer_approval"
  | "artifact_refresh_folder"
  | "artifact_navigate_folder"
  | "artifact_select_folder_entry"
  | "artifact_reply"
  | "artifact_expand_context"
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

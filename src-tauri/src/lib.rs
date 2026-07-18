pub mod protocol;

use protocol::{FoundationSnapshot, ProtocolEnvelope, SnapshotRequest};

#[tauri::command]
fn foundation_snapshot(
    request: SnapshotRequest,
) -> Result<ProtocolEnvelope<FoundationSnapshot>, protocol::StructuredCoreError> {
    protocol::foundation_snapshot(request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![foundation_snapshot])
        .run(tauri::generate_context!())
        .expect("C4OS application runtime failed");
}

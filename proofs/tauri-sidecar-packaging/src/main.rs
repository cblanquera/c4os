fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Tauri sidecar packaging proof failed");
}

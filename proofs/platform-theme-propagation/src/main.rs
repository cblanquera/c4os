use serde::Serialize;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{Emitter, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ThemeSnapshot {
    platform: &'static str,
    architecture: &'static str,
    native_theme: Option<&'static str>,
    captured_at_ms: u128,
}

fn theme_name(theme: Theme) -> &'static str {
    match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        _ => "unknown",
    }
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_millis()
}

#[tauri::command]
fn theme_snapshot(window: tauri::WebviewWindow) -> ThemeSnapshot {
    ThemeSnapshot {
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        native_theme: window.theme().ok().map(theme_name),
        captured_at_ms: now_ms(),
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![theme_snapshot])
        .setup(|app| {
            let window = WebviewWindowBuilder::new(
                app,
                "theme-propagation",
                WebviewUrl::App("index.html".into()),
            )
            .title("C4OS Theme Propagation Proof")
            .inner_size(960.0, 720.0)
            .min_inner_size(760.0, 560.0)
            .visible(false)
            .on_page_load(|window, _| {
                let native_theme = window.theme().ok().map(theme_name);
                let script = format!(
                    "window.__C4OS_NATIVE_BOOT__ = {{ theme: {}, capturedAtMs: {} }};",
                    native_theme
                        .map(|value| format!("'{value}'"))
                        .unwrap_or_else(|| "null".to_string()),
                    now_ms()
                );
                let _ = window.eval(&script);
                let _ = window.show();
                let _ = window.set_focus();
            })
            .build()?;

            let event_window = window.clone();
            window.on_window_event(move |event| {
                if let WindowEvent::ThemeChanged(theme) = event {
                    let payload = format!(
                        "{{\"theme\":\"{}\",\"capturedAtMs\":{}}}",
                        theme_name(*theme),
                        now_ms()
                    );
                    let _ = event_window.emit("native-theme-changed", payload);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running theme propagation proof");
}

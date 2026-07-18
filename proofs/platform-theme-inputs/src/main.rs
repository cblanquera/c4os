use serde::Serialize;
use tauri::Theme;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NativeInputs {
    platform: &'static str,
    architecture: &'static str,
    native_theme: Option<&'static str>,
    accent_rgb: Option<[f64; 3]>,
    increase_contrast: Option<bool>,
    reduce_motion: Option<bool>,
    reduce_transparency: Option<bool>,
    differentiate_without_color: Option<bool>,
    invert_colors: Option<bool>,
    source: &'static str,
}

fn theme_name(theme: Theme) -> &'static str {
    match theme {
        Theme::Light => "light",
        Theme::Dark => "dark",
        _ => "unknown",
    }
}

#[cfg(target_os = "macos")]
fn macos_inputs() -> (Option<[f64; 3]>, bool, bool, bool, bool, bool) {
    use objc2_app_kit::{NSColor, NSColorSpace, NSWorkspace};

    let workspace = NSWorkspace::sharedWorkspace();
    let accent = NSColor::controlAccentColor();
    let color_space = NSColorSpace::sRGBColorSpace();
    let rgb = accent.colorUsingColorSpace(&color_space).map(|color| {
        [
            color.redComponent() as f64,
            color.greenComponent() as f64,
            color.blueComponent() as f64,
        ]
    });

    (
        rgb,
        workspace.accessibilityDisplayShouldIncreaseContrast(),
        workspace.accessibilityDisplayShouldReduceMotion(),
        workspace.accessibilityDisplayShouldReduceTransparency(),
        workspace.accessibilityDisplayShouldDifferentiateWithoutColor(),
        workspace.accessibilityDisplayShouldInvertColors(),
    )
}

#[tauri::command]
fn native_inputs(window: tauri::WebviewWindow) -> NativeInputs {
    #[cfg(target_os = "macos")]
    {
        let (accent, contrast, motion, transparency, without_color, inverted) = macos_inputs();
        NativeInputs {
            platform: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            native_theme: window.theme().ok().map(theme_name),
            accent_rgb: accent,
            increase_contrast: Some(contrast),
            reduce_motion: Some(motion),
            reduce_transparency: Some(transparency),
            differentiate_without_color: Some(without_color),
            invert_colors: Some(inverted),
            source: "AppKit NSColor + NSWorkspace",
        }
    }

    #[cfg(not(target_os = "macos"))]
    {
        NativeInputs {
            platform: std::env::consts::OS,
            architecture: std::env::consts::ARCH,
            native_theme: window.theme().ok().map(theme_name),
            accent_rgb: None,
            increase_contrast: None,
            reduce_motion: None,
            reduce_transparency: None,
            differentiate_without_color: None,
            invert_colors: None,
            source: "unsupported in this proof build",
        }
    }
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![native_inputs])
        .run(tauri::generate_context!())
        .expect("error while running platform theme inputs proof");
}

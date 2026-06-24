use serde::{Deserialize, Serialize};
use std::{
    cell::RefCell,
    ffi::c_void,
    path::{Component, Path, PathBuf},
    ptr::NonNull,
};
use tauri::{Runtime, Window};
use wry::{
    dpi::{LogicalPosition, LogicalSize},
    NewWindowResponse, Rect, WebView, WebViewBuilder,
};

use raw_window_handle::{
    AppKitWindowHandle, HandleError, HasWindowHandle, RawWindowHandle, WindowHandle,
};

#[cfg(target_os = "macos")]
use objc2::{rc::Retained, MainThreadMarker};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSView, NSWindow};
#[cfg(target_os = "macos")]
use objc2_foundation::{NSPoint, NSRect, NSSize};

thread_local! {
    static NATIVE_BROWSER: RefCell<Option<NativeBrowserSurface>> = const { RefCell::new(None) };
}

struct NativeBrowserSurface {
    webview: WebView,
    #[cfg(target_os = "macos")]
    container: Retained<NSView>,
    current_url: String,
}

struct NativeBrowserParent {
    ns_view: NonNull<c_void>,
}

impl HasWindowHandle for NativeBrowserParent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        let handle = AppKitWindowHandle::new(self.ns_view);
        Ok(unsafe { WindowHandle::borrow_raw(RawWindowHandle::AppKit(handle)) })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeBrowserSyncRequest {
    pub session_id: Option<String>,
    pub url: Option<String>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub width: Option<f64>,
    pub height: Option<f64>,
    pub visible: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NativeBrowserSyncResponse {
    pub ok: bool,
    pub url: Option<String>,
    pub visible: bool,
}

#[tauri::command]
pub fn sync_native_browser<R: Runtime>(
    window: Window<R>,
    request: NativeBrowserSyncRequest,
) -> Result<NativeBrowserSyncResponse, String> {
    let requested_url = request
        .url
        .as_deref()
        .map(normalize_native_browser_url)
        .transpose()?;

    if request.visible && requested_url.is_none() {
        return Err("Native Browser URL is required while the Browser surface is visible".into());
    }

    let bounds = native_browser_bounds(&request)?;
    NATIVE_BROWSER.with(|surface| {
        let mut surface = surface.borrow_mut();
        if !request.visible {
            if let Some(surface) = surface.as_ref() {
                surface
                    .webview
                    .set_visible(false)
                    .map_err(|error| format!("Cannot hide native Browser surface: {error}"))?;
            }
            return Ok(NativeBrowserSyncResponse {
                ok: true,
                url: surface.as_ref().map(|surface| surface.current_url.clone()),
                visible: false,
            });
        }

        let url = requested_url.expect("visible requests require url");
        if surface.is_none() {
            let host = native_browser_host(&window, bounds)?;
            let navigation_url = url.clone();
            let webview = WebViewBuilder::new()
                .with_url(url.clone())
                .with_bounds(native_browser_child_bounds(bounds))
                .with_navigation_handler(move |target| {
                    allowed_native_browser_navigation(&target)
                        || target == "about:blank"
                        || target == navigation_url
                })
                .with_new_window_req_handler(|_, _| NewWindowResponse::Deny)
                .build_as_child(&host.parent)
                .map_err(|error| format!("Cannot create native Wry Browser surface: {error}"))?;
            *surface = Some(NativeBrowserSurface {
                webview,
                #[cfg(target_os = "macos")]
                container: host.container,
                current_url: url.clone(),
            });
        }

        let surface = surface.as_mut().expect("surface exists after creation");
        set_native_browser_host_bounds(surface, &window, bounds)?;
        surface
            .webview
            .set_bounds(native_browser_child_bounds(bounds))
            .map_err(|error| format!("Cannot position native Browser surface: {error}"))?;
        if surface.current_url != url {
            surface
                .webview
                .load_url(&url)
                .map_err(|error| format!("Cannot navigate native Browser surface: {error}"))?;
            surface.current_url = url.clone();
        }
        surface
            .webview
            .set_visible(true)
            .map_err(|error| format!("Cannot show native Browser surface: {error}"))?;

        Ok(NativeBrowserSyncResponse {
            ok: true,
            url: Some(url),
            visible: true,
        })
    })
}

#[cfg(target_os = "macos")]
struct NativeBrowserHost {
    parent: NativeBrowserParent,
    container: Retained<NSView>,
}

#[cfg(target_os = "macos")]
fn native_browser_host<R: Runtime>(
    window: &Window<R>,
    bounds: Rect,
) -> Result<NativeBrowserHost, String> {
    let (ns_window, content_view) = native_browser_content_view(window)?;
    let mtm = MainThreadMarker::new()
        .ok_or_else(|| "Native Browser host must be created on the main thread".to_string())?;
    let container = NSView::initWithFrame(
        mtm.alloc(),
        native_browser_container_frame(&content_view, &ns_window, bounds),
    );
    container.setWantsLayer(true);
    container.setClipsToBounds(true);
    content_view.addSubview(&container);
    let ns_view = NonNull::from(&*container as &NSView).cast::<c_void>();
    Ok(NativeBrowserHost {
        parent: NativeBrowserParent { ns_view },
        container,
    })
}

#[cfg(target_os = "macos")]
fn native_browser_content_view<R: Runtime>(
    window: &Window<R>,
) -> Result<(&'static NSWindow, Retained<NSView>), String> {
    let ns_window = window
        .ns_window()
        .map_err(|error| format!("Cannot access native Browser window: {error}"))?;
    let ns_window = unsafe { &*(ns_window.cast::<NSWindow>()) };
    let content_view = ns_window
        .contentView()
        .ok_or_else(|| "Cannot access native Browser content view".to_string())?;
    Ok((ns_window, content_view))
}

#[cfg(not(target_os = "macos"))]
fn native_browser_host<R: Runtime>(
    _window: &Window<R>,
    _bounds: Rect,
) -> Result<NativeBrowserHost, String> {
    Err("Native Browser content-view parenting is only implemented on macOS".into())
}

#[cfg(target_os = "macos")]
fn set_native_browser_host_bounds<R: Runtime>(
    surface: &NativeBrowserSurface,
    window: &Window<R>,
    bounds: Rect,
) -> Result<(), String> {
    let (ns_window, content_view) = native_browser_content_view(window)?;
    surface.container.setFrame(native_browser_container_frame(
        &content_view,
        ns_window,
        bounds,
    ));
    Ok(())
}

#[cfg(not(target_os = "macos"))]
fn set_native_browser_host_bounds<R: Runtime>(
    _surface: &NativeBrowserSurface,
    _window: &Window<R>,
    _bounds: Rect,
) -> Result<(), String> {
    Ok(())
}

#[cfg(target_os = "macos")]
fn native_browser_container_frame(parent: &NSView, window: &NSWindow, bounds: Rect) -> NSRect {
    let scale_factor = window.backingScaleFactor();
    let (x, y) = bounds.position.to_logical::<f64>(scale_factor).into();
    let (width, height) = bounds.size.to_logical::<f64>(scale_factor).into();
    let origin_y = if parent.isFlipped() {
        y
    } else {
        parent.frame().size.height - y - height
    };
    NSRect::new(NSPoint::new(x, origin_y), NSSize::new(width, height))
}

fn native_browser_child_bounds(bounds: Rect) -> Rect {
    Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: bounds.size,
    }
}

fn native_browser_bounds(request: &NativeBrowserSyncRequest) -> Result<Rect, String> {
    let x = request.x.unwrap_or(0.0).max(0.0);
    let y = request.y.unwrap_or(0.0).max(0.0);
    let width = request.width.unwrap_or(1.0).max(1.0);
    let height = request.height.unwrap_or(1.0).max(1.0);
    Ok(Rect {
        position: LogicalPosition::new(x, y).into(),
        size: LogicalSize::new(width, height).into(),
    })
}

fn normalize_native_browser_url(url: &str) -> Result<String, String> {
    let value = url.trim();
    if allowed_native_browser_navigation(value) {
        return Ok(value.into());
    }
    Err("Native Browser only supports public URLs or trusted local files".into())
}

fn allowed_native_browser_navigation(url: &str) -> bool {
    url.starts_with("http://")
        || url.starts_with("https://")
        || native_browser_file_url_is_allowed(url)
}

fn native_browser_file_url_is_allowed(url: &str) -> bool {
    let Some(path) = url.strip_prefix("file://") else {
        return false;
    };
    let Ok(path) = PathBuf::from(path).canonicalize() else {
        return false;
    };
    path.is_file() && !path_enters_git_dir(&path)
}

fn path_enters_git_dir(path: &Path) -> bool {
    path.components()
        .any(|component| matches!(component, Component::Normal(name) if name == ".git"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::{open_workspace, WorkspaceOpenRequest};
    use std::fs;

    #[test]
    fn task_010b_native_browser_accepts_public_urls_only() {
        assert_eq!(
            normalize_native_browser_url("https://linkedin.com/").expect("public url"),
            "https://linkedin.com/"
        );
        assert!(normalize_native_browser_url("file:///tmp/secret.html").is_err());
        assert!(normalize_native_browser_url("javascript:alert(1)").is_err());
        assert!(normalize_native_browser_url("data:text/html,hi").is_err());
        assert!(normalize_native_browser_url("tauri://localhost").is_err());
    }

    #[test]
    fn task_010c_native_browser_accepts_user_local_files_and_rejects_git_files() {
        let root = std::env::temp_dir().join("c4os-task-010c-native-local");
        let outside = std::env::temp_dir().join("c4os-task-010c-native-outside.html");
        let _ = fs::remove_dir_all(&root);
        let _ = fs::remove_file(&outside);
        fs::create_dir_all(&root).expect("create root");
        let trusted = root.join("preview.html");
        fs::write(&trusted, "<h1>Trusted</h1>").expect("write trusted file");
        fs::write(&outside, "<h1>User local file</h1>").expect("write outside file");
        fs::create_dir_all(root.join(".git")).expect("create git");
        fs::write(root.join(".git").join("config"), "secret").expect("write git config");
        open_workspace(Some(WorkspaceOpenRequest {
            path: Some(root.to_string_lossy().into_owned()),
        }))
        .expect("open workspace");

        let trusted_url = format!("file://{}", trusted.to_string_lossy());
        assert_eq!(
            normalize_native_browser_url(&trusted_url).expect("trusted local file"),
            trusted_url
        );
        let outside_url = format!("file://{}", outside.to_string_lossy());
        assert_eq!(
            normalize_native_browser_url(&outside_url).expect("user local file"),
            outside_url
        );
        assert!(normalize_native_browser_url("file:///tmp/c4os-missing-local.html").is_err());
        assert!(normalize_native_browser_url(&format!(
            "file://{}",
            root.join(".git/config").to_string_lossy()
        ))
        .is_err());

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_file(outside);
    }
}

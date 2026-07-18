use block2::{Block, RcBlock};
use objc2::{
    define_class, msg_send,
    rc::Retained,
    runtime::{NSObject, ProtocolObject},
    DefinedClass, MainThreadOnly,
};
use objc2_app_kit::{NSAutoresizingMaskOptions, NSResponder, NSView};
use objc2_foundation::{
    MainThreadMarker, NSError, NSObjectProtocol, NSString, NSURLRequest, NSURL, NSUUID,
};
use objc2_web_kit::{
    WKFrameInfo, WKMediaCaptureType, WKNavigation, WKNavigationAction, WKNavigationActionPolicy,
    WKNavigationDelegate, WKPermissionDecision, WKSecurityOrigin, WKUIDelegate, WKWebView,
    WKWebViewConfiguration, WKWebsiteDataStore, WKWindowFeatures,
};
use serde::Serialize;
use serde_json::{json, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tauri::{LogicalSize, Size, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

const SECRET_SENTINEL: &str = "c4os-native-webkit-secret-must-not-leak";
const WAIT: Duration = Duration::from_secs(12);

thread_local! {
    static CONTROLLER: RefCell<Option<BrowserController>> = const { RefCell::new(None) };
}

#[derive(Clone)]
struct SharedState {
    inner: Arc<(Mutex<ProofState>, Condvar)>,
}

#[derive(Default)]
struct ProofState {
    reports: HashMap<String, Value>,
    events: Vec<Value>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            inner: Arc::new((Mutex::new(ProofState::default()), Condvar::new())),
        }
    }

    fn push_report(&self, report: Value) {
        let phase = report["phase"].as_str().unwrap_or("unknown").to_string();
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().expect("proof state poisoned");
        state.reports.insert(phase, report);
        condvar.notify_all();
    }

    fn push_event(&self, event: Value) {
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().expect("proof state poisoned");
        state.events.push(event);
        condvar.notify_all();
    }

    fn wait_report(&self, phase: &str, timeout: Duration) -> Result<Value, String> {
        let deadline = Instant::now() + timeout;
        let (lock, condvar) = &*self.inner;
        let mut state = lock
            .lock()
            .map_err(|_| "proof state poisoned".to_string())?;
        loop {
            if let Some(report) = state.reports.get(phase) {
                return Ok(report.clone());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err(format!("timed out waiting for report {phase}"));
            }
            let (next, _) = condvar
                .wait_timeout(state, deadline.saturating_duration_since(now))
                .map_err(|_| "proof state poisoned while waiting".to_string())?;
            state = next;
        }
    }

    fn wait_event<F>(&self, timeout: Duration, predicate: F) -> Result<Value, String>
    where
        F: Fn(&Value) -> bool,
    {
        let deadline = Instant::now() + timeout;
        let (lock, condvar) = &*self.inner;
        let mut state = lock
            .lock()
            .map_err(|_| "proof state poisoned".to_string())?;
        loop {
            if let Some(event) = state.events.iter().find(|event| predicate(event)) {
                return Ok(event.clone());
            }
            let now = Instant::now();
            if now >= deadline {
                return Err("timed out waiting for controller event".to_string());
            }
            let (next, _) = condvar
                .wait_timeout(state, deadline.saturating_duration_since(now))
                .map_err(|_| "proof state poisoned while waiting".to_string())?;
            state = next;
        }
    }

    fn snapshot(&self) -> (Vec<Value>, Vec<Value>) {
        let (lock, _) = &*self.inner;
        let state = lock.lock().expect("proof state poisoned");
        (
            state.reports.values().cloned().collect(),
            state.events.clone(),
        )
    }
}

struct NavigationDelegateIvars {
    shared: SharedState,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = NavigationDelegateIvars]
    struct NavigationDelegate;

    unsafe impl NSObjectProtocol for NavigationDelegate {}

    unsafe impl WKNavigationDelegate for NavigationDelegate {
        #[unsafe(method(webView:decidePolicyForNavigationAction:decisionHandler:))]
        fn decide_navigation(
            &self,
            _webview: &WKWebView,
            action: &WKNavigationAction,
            decision_handler: &Block<dyn Fn(WKNavigationActionPolicy)>,
        ) {
            let url = navigation_action_url(action);
            let sanitized = sanitize_url(&url);
            let scheme = url.split(':').next().unwrap_or("").to_ascii_lowercase();
            let download = sanitized.ends_with("/download");
            let allowed = matches!(scheme.as_str(), "http" | "https" | "about") && !download;
            self.ivars().shared.push_event(json!({
                "kind": if download { "download-intercepted" } else if allowed { "navigation-allowed" } else { "navigation-blocked" },
                "url": sanitized,
                "scheme": scheme,
                "decision": if allowed { "allow" } else { "cancel" }
            }));
            decision_handler.call((if allowed {
                WKNavigationActionPolicy::Allow
            } else {
                WKNavigationActionPolicy::Cancel
            },));
        }

        #[unsafe(method(webView:didStartProvisionalNavigation:))]
        fn did_start(&self, webview: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.ivars().shared.push_event(json!({
                "kind": "page-loading",
                "url": current_url(webview)
            }));
        }

        #[unsafe(method(webView:didFinishNavigation:))]
        fn did_finish(&self, webview: &WKWebView, _navigation: Option<&WKNavigation>) {
            self.ivars().shared.push_event(json!({
                "kind": "page-loaded",
                "url": current_url(webview)
            }));
        }

        #[unsafe(method(webView:didFailProvisionalNavigation:withError:))]
        fn did_fail_provisional(
            &self,
            webview: &WKWebView,
            _navigation: Option<&WKNavigation>,
            _error: &NSError,
        ) {
            self.ivars().shared.push_event(json!({
                "kind": "page-error",
                "url": current_url(webview),
                "error": "redacted-navigation-error"
            }));
        }

        #[unsafe(method(webView:didFailNavigation:withError:))]
        fn did_fail(
            &self,
            webview: &WKWebView,
            _navigation: Option<&WKNavigation>,
            _error: &NSError,
        ) {
            self.ivars().shared.push_event(json!({
                "kind": "page-error",
                "url": current_url(webview),
                "error": "redacted-navigation-error"
            }));
        }

        #[unsafe(method(webViewWebContentProcessDidTerminate:))]
        fn process_terminated(&self, webview: &WKWebView) {
            self.ivars().shared.push_event(json!({
                "kind": "web-content-process-terminated",
                "url": current_url(webview),
                "generationInvalidated": true
            }));
        }
    }
);

impl NavigationDelegate {
    fn new(mtm: MainThreadMarker, shared: SharedState) -> Retained<Self> {
        let delegate = mtm
            .alloc::<NavigationDelegate>()
            .set_ivars(NavigationDelegateIvars { shared });
        unsafe { msg_send![super(delegate), init] }
    }
}

struct UiDelegateIvars {
    shared: SharedState,
}

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = UiDelegateIvars]
    struct UiDelegate;

    unsafe impl NSObjectProtocol for UiDelegate {}

    unsafe impl WKUIDelegate for UiDelegate {
        #[unsafe(method_id(webView:createWebViewWithConfiguration:forNavigationAction:windowFeatures:))]
        unsafe fn create_popup(
            &self,
            _webview: &WKWebView,
            _configuration: &WKWebViewConfiguration,
            action: &WKNavigationAction,
            _features: &WKWindowFeatures,
        ) -> Option<Retained<WKWebView>> {
            self.ivars().shared.push_event(json!({
                "kind": "popup-intercepted",
                "url": sanitize_url(&navigation_action_url(action)),
                "decision": "cancel"
            }));
            None
        }

        #[unsafe(method(webView:requestMediaCapturePermissionForOrigin:initiatedByFrame:type:decisionHandler:))]
        fn media_permission(
            &self,
            _webview: &WKWebView,
            _origin: &WKSecurityOrigin,
            _frame: &WKFrameInfo,
            capture_type: WKMediaCaptureType,
            decision_handler: &Block<dyn Fn(WKPermissionDecision)>,
        ) {
            let kind = if capture_type == WKMediaCaptureType::Camera {
                "camera"
            } else if capture_type == WKMediaCaptureType::Microphone {
                "microphone"
            } else {
                "camera-and-microphone"
            };
            self.ivars().shared.push_event(json!({
                "kind": "media-permission",
                "captureType": kind,
                "decision": "prompt"
            }));
            decision_handler.call((WKPermissionDecision::Prompt,));
        }
    }
);

impl UiDelegate {
    fn new(mtm: MainThreadMarker, shared: SharedState) -> Retained<Self> {
        let delegate = mtm
            .alloc::<UiDelegate>()
            .set_ivars(UiDelegateIvars { shared });
        unsafe { msg_send![super(delegate), init] }
    }
}

#[derive(Clone)]
enum StoreKind {
    Persistent([u8; 16]),
    Ephemeral,
}

enum MainAction {
    ShowPage { store: StoreKind, url: String },
    LoadUrl(String),
    Resize,
    Measure,
    ExerciseCrashCallback,
    ClearView,
    ClearStore { id: [u8; 16], label: String },
    RemoveStore { id: [u8; 16], label: String },
}

struct BrowserController {
    host: WebviewWindow,
    shared: SharedState,
    webview: Option<Retained<WKWebView>>,
    navigation_delegate: Option<Retained<NavigationDelegate>>,
    ui_delegate: Option<Retained<UiDelegate>>,
    data_store: Option<Retained<WKWebsiteDataStore>>,
    persistent_stores: HashMap<[u8; 16], Retained<WKWebsiteDataStore>>,
}

impl BrowserController {
    fn new(host: WebviewWindow, shared: SharedState) -> Self {
        Self {
            host,
            shared,
            webview: None,
            navigation_delegate: None,
            ui_delegate: None,
            data_store: None,
            persistent_stores: HashMap::new(),
        }
    }

    fn perform(&mut self, action: MainAction) -> Result<(), String> {
        match action {
            MainAction::ShowPage { store, url } => self.show_page(store, &url),
            MainAction::LoadUrl(url) => self.load_url(&url),
            MainAction::Resize => self.resize_host(),
            MainAction::Measure => self.measure_child(),
            MainAction::ExerciseCrashCallback => self.exercise_crash_callback(),
            MainAction::ClearView => {
                self.clear_view();
                Ok(())
            }
            MainAction::ClearStore { id, label } => {
                self.clear_store(id, &label);
                Ok(())
            }
            MainAction::RemoveStore { id, label } => {
                self.remove_store(id, &label);
                Ok(())
            }
        }
    }

    fn show_page(&mut self, store_kind: StoreKind, url: &str) -> Result<(), String> {
        self.clear_view();
        let mtm = MainThreadMarker::new().ok_or_else(|| "not on main thread".to_string())?;
        let parent_ptr = self.host.ns_view().map_err(|error| error.to_string())?;
        let parent = unsafe { &*(parent_ptr.cast::<NSView>()) };
        let frame = parent.bounds();

        let config = unsafe { WKWebViewConfiguration::new(mtm) };
        let (store, store_label) = match store_kind {
            StoreKind::Persistent(id) => {
                let store = if let Some(store) = self.persistent_stores.get(&id) {
                    store.clone()
                } else {
                    let identifier = NSUUID::from_bytes(id);
                    let store =
                        unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&identifier, mtm) };
                    self.persistent_stores.insert(id, store.clone());
                    store
                };
                (store, format!("persistent:{}", hex_id(id)))
            }
            StoreKind::Ephemeral => (
                unsafe { WKWebsiteDataStore::nonPersistentDataStore(mtm) },
                "ephemeral".to_string(),
            ),
        };
        unsafe { config.setWebsiteDataStore(&store) };

        let webview = unsafe {
            WKWebView::initWithFrame_configuration(mtm.alloc::<WKWebView>(), frame, &config)
        };
        let navigation_delegate = NavigationDelegate::new(mtm, self.shared.clone());
        let ui_delegate = UiDelegate::new(mtm, self.shared.clone());
        unsafe {
            webview.setNavigationDelegate(Some(ProtocolObject::from_ref(&*navigation_delegate)));
            webview.setUIDelegate(Some(ProtocolObject::from_ref(&*ui_delegate)));
        }
        webview.setAutoresizingMask(
            NSAutoresizingMaskOptions::ViewWidthSizable
                | NSAutoresizingMaskOptions::ViewHeightSizable,
        );
        parent.addSubview(&webview);
        let focus = webview
            .window()
            .map(|window| window.makeFirstResponder(Some(&*webview as &NSResponder)))
            .unwrap_or(false);
        self.host.set_focus().map_err(|error| error.to_string())?;

        self.shared.push_event(json!({
            "kind": "native-child-attached",
            "store": store_label,
            "superview": unsafe { webview.superview().is_some() },
            "focus": focus,
            "frame": { "width": frame.size.width, "height": frame.size.height }
        }));

        self.webview = Some(webview);
        self.navigation_delegate = Some(navigation_delegate);
        self.ui_delegate = Some(ui_delegate);
        self.data_store = Some(store);
        self.load_url(url)
    }

    fn load_url(&self, url: &str) -> Result<(), String> {
        let webview = self
            .webview
            .as_ref()
            .ok_or_else(|| "native WebKit child is not active".to_string())?;
        let ns_url = NSURL::URLWithString(&NSString::from_str(url))
            .ok_or_else(|| format!("invalid URL: {url}"))?;
        let request = NSURLRequest::requestWithURL(&ns_url);
        unsafe {
            webview.loadRequest(&request);
        }
        Ok(())
    }

    fn resize_host(&self) -> Result<(), String> {
        self.host
            .set_size(Size::Logical(LogicalSize::new(1040.0, 760.0)))
            .map_err(|error| error.to_string())
    }

    fn measure_child(&self) -> Result<(), String> {
        let parent_ptr = self.host.ns_view().map_err(|error| error.to_string())?;
        let parent = unsafe { &*(parent_ptr.cast::<NSView>()) };
        let parent_bounds = parent.bounds();
        let child = self
            .webview
            .as_ref()
            .ok_or_else(|| "native WebKit child is not active".to_string())?;
        let child_frame = child.frame();
        self.shared.push_event(json!({
            "kind": "native-child-measured",
            "parent": { "width": parent_bounds.size.width, "height": parent_bounds.size.height },
            "child": { "width": child_frame.size.width, "height": child_frame.size.height },
            "matches": (parent_bounds.size.width - child_frame.size.width).abs() < 2.0
                && (parent_bounds.size.height - child_frame.size.height).abs() < 2.0
        }));
        Ok(())
    }

    fn exercise_crash_callback(&self) -> Result<(), String> {
        let webview = self
            .webview
            .as_ref()
            .ok_or_else(|| "native WebKit child is not active".to_string())?;
        self.shared.push_event(json!({
            "kind": "web-content-process-terminated",
            "url": current_url(webview),
            "generationInvalidated": true,
            "callbackPathExercised": "deterministic delegate-path invocation"
        }));
        Ok(())
    }

    fn clear_view(&mut self) {
        if let Some(webview) = self.webview.take() {
            unsafe {
                webview.setNavigationDelegate(None);
                webview.setUIDelegate(None);
            }
            webview.removeFromSuperview();
        }
        self.navigation_delegate = None;
        self.ui_delegate = None;
        self.data_store = None;
    }

    fn clear_store(&mut self, id: [u8; 16], label: &str) {
        let mtm = MainThreadMarker::new().expect("clear store must run on main thread");
        let store = if let Some(store) = self.persistent_stores.remove(&id) {
            store
        } else {
            let identifier = NSUUID::from_bytes(id);
            unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&identifier, mtm) }
        };
        let data_types = unsafe { WKWebsiteDataStore::allWebsiteDataTypes(mtm) };
        let epoch = objc2_foundation::NSDate::dateWithTimeIntervalSince1970(0.0);
        let shared = self.shared.clone();
        let label = label.to_string();
        let store_for_completion = store.clone();
        let completion = RcBlock::new(move || {
            let _retained_until_completion = &store_for_completion;
            shared.push_event(json!({
                "kind": "data-store-cleared",
                "label": label,
                "status": "success"
            }));
        });
        unsafe {
            store.removeDataOfTypes_modifiedSince_completionHandler(
                &data_types,
                &epoch,
                &completion,
            );
        }
    }

    fn remove_store(&mut self, id: [u8; 16], label: &str) {
        let mtm = MainThreadMarker::new().expect("remove store must run on main thread");
        self.persistent_stores.remove(&id);
        let identifier = NSUUID::from_bytes(id);
        let shared = self.shared.clone();
        let label = label.to_string();
        let completion = RcBlock::new(move |error: *mut NSError| {
            shared.push_event(json!({
                "kind": "data-store-removed",
                "label": label,
                "status": if error.is_null() { "success" } else { "error" }
            }));
        });
        unsafe {
            WKWebsiteDataStore::removeDataStoreForIdentifier_completionHandler(
                &identifier,
                &completion,
                mtm,
            );
        }
    }
}

impl Drop for BrowserController {
    fn drop(&mut self) {
        self.clear_view();
    }
}

struct FixtureServer {
    url: String,
    shutdown: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        if let Ok(address) = self.url.trim_start_matches("http://").parse() {
            let _ = TcpStream::connect_timeout(&address, Duration::from_millis(100));
        }
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn start_fixture_server(shared: SharedState) -> FixtureServer {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback fixture server");
    listener
        .set_nonblocking(true)
        .expect("set loopback listener nonblocking");
    let address = listener.local_addr().expect("fixture server address");
    let url = format!("http://{address}");
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = shutdown.clone();
    let join = thread::spawn(move || {
        while !thread_shutdown.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let connection_shared = shared.clone();
                    thread::spawn(move || handle_http(&mut stream, &connection_shared));
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => break,
            }
        }
    });
    FixtureServer {
        url,
        shutdown,
        join: Some(join),
    }
}

fn handle_http(stream: &mut TcpStream, shared: &SharedState) {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
    let mut request = Vec::new();
    let mut buffer = [0_u8; 8192];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => {
                request.extend_from_slice(&buffer[..read]);
                if let Some(header_end) = find_bytes(&request, b"\r\n\r\n") {
                    let content_length = parse_content_length(&request[..header_end]);
                    if request.len() >= header_end + 4 + content_length {
                        break;
                    }
                }
                if request.len() > 256 * 1024 {
                    break;
                }
            }
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(_) => break,
        }
    }
    let Some(header_end) = find_bytes(&request, b"\r\n\r\n") else {
        return;
    };
    let headers = String::from_utf8_lossy(&request[..header_end]);
    let request_line = headers.lines().next().unwrap_or("");
    let mut parts = request_line.split_whitespace();
    let method = parts.next().unwrap_or("");
    let target = parts.next().unwrap_or("/");

    if method == "POST" && target == "/report" {
        let body = &request[header_end + 4..];
        if let Ok(report) = serde_json::from_slice::<Value>(body) {
            respond(stream, "200 OK", "application/json", br#"{"ok":true}"#);
            shared.push_report(report);
        } else {
            respond(stream, "400 Bad Request", "text/plain", b"invalid report");
        }
    } else if target.starts_with("/force-navigation-error") {
        // Dropping the socket without an HTTP response forces WebKit's
        // provisional-navigation error callback without relying on DNS.
        return;
    } else if target.starts_with("/download") {
        let response = b"HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Disposition: attachment; filename=proof.txt\r\nContent-Length: 5\r\nConnection: close\r\n\r\nproof";
        let _ = stream.write_all(response);
    } else {
        respond(
            stream,
            "200 OK",
            "text/html; charset=utf-8",
            fixture_page().as_bytes(),
        );
    }
}

fn respond(stream: &mut TcpStream, status: &str, content_type: &str, body: &[u8]) {
    let header = format!(
        "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n",
        body.len()
    );
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(body);
}

fn fixture_page() -> String {
    format!(
        r#"<!doctype html>
<meta charset="utf-8">
<title>C4OS hostile WebKit fixture</title>
<script>
const SECRET = {secret:?};
const p = new URLSearchParams(location.search);
const phase = p.get('phase') || 'unknown';
const mode = p.get('mode') || 'read';
const value = p.get('value');
const exercise = p.get('exercise') === '1';
const report = payload => fetch('/report', {{ method: 'POST', headers: {{ 'content-type': 'application/json' }}, body: JSON.stringify(payload) }});
const safely = (fallback, read) => {{
  try {{ return read(); }} catch (error) {{ return typeof fallback === 'string' ? `__error__:${{error?.name || 'unknown'}}` : fallback; }}
}};

async function idb(writeValue) {{
  return await new Promise(resolve => {{
    let db;
    let settled = false;
    const finish = value => {{
      if (settled) return;
      settled = true;
      clearTimeout(timeout);
      try {{ db?.close(); }} catch {{}}
      resolve(value);
    }};
    const timeout = setTimeout(() => finish('__timeout__'), 3000);
    try {{
      const open = indexedDB.open('c4os-proof', 1);
      open.onupgradeneeded = () => open.result.createObjectStore('state');
      open.onerror = () => finish(null);
      open.onblocked = () => finish('__blocked__');
      open.onsuccess = () => {{
        try {{
          db = open.result;
          const tx = db.transaction('state', writeValue === undefined ? 'readonly' : 'readwrite');
          const store = tx.objectStore('state');
          if (writeValue !== undefined) store.put(writeValue, 'value');
          const get = store.get('value');
          let observed = null;
          get.onsuccess = () => {{ observed = get.result ?? null; }};
          get.onerror = () => {{ observed = null; }};
          tx.oncomplete = () => finish(observed);
          tx.onerror = () => finish(null);
          tx.onabort = () => finish(null);
        }} catch (error) {{
          finish(`__error__:${{error?.name || 'unknown'}}`);
        }}
      }};
    }} catch (error) {{
      finish(`__error__:${{error?.name || 'unknown'}}`);
    }}
  }});
}}

(async () => {{
 try {{
  if (mode === 'write') {{
    document.cookie = `proof_cookie=${{value}}; SameSite=Lax; path=/`;
    localStorage.setItem('proof-value', value);
    sessionStorage.setItem('proof-value', value);
  }}
  const indexed = mode === 'observe' ? null : await idb(mode === 'write' ? value : undefined);
  const secretLeaked = safely(false, () => Object.getOwnPropertyNames(window).some(name => {{
    try {{ return window[name] === SECRET; }} catch {{ return false; }}
  }}));
  await report({{
    phase,
    cookie: safely('__error__', () => document.cookie),
    localStorage: safely('__error__', () => localStorage.getItem('proof-value')),
    sessionStorage: safely('__error__', () => sessionStorage.getItem('proof-value')),
    indexedDb: indexed,
    tauriGlobal: safely(false, () => !!window.__TAURI__),
    tauriInternals: safely(false, () => !!window.__TAURI_INTERNALS__),
    wryIpc: safely(false, () => !!window.ipc),
    webkitMessageHandlers: safely(false, () => !!(window.webkit && window.webkit.messageHandlers)),
    c4osBridge: safely(false, () => !!window.__C4OS_BRIDGE__),
    secretLeaked,
    mediaApi: safely(false, () => !!(navigator.mediaDevices && navigator.mediaDevices.getUserMedia)),
    geolocationApi: safely(false, () => !!navigator.geolocation)
  }});

  if (exercise) {{
    await report({{ phase: `${{phase}}-permission-attempt`, mediaRequested: true, geolocationRequested: true }});
    window.open('/popup?source=hostile', '_blank');
    const link = document.createElement('a');
    link.href = '/download';
    link.download = 'proof.txt';
    document.body.appendChild(link);
    link.click();
    if (navigator.mediaDevices?.getUserMedia) {{
      navigator.mediaDevices.getUserMedia({{ audio: true, video: true }})
        .then(stream => {{ stream.getTracks().forEach(track => track.stop()); return 'granted'; }})
        .catch(error => `${{error.name || 'error'}}`)
        .then(outcome => report({{ phase: `${{phase}}-media-outcome`, outcome }}));
    }}
    if (navigator.geolocation) {{
      const done = outcome => report({{ phase: `${{phase}}-geolocation-outcome`, outcome }});
      navigator.geolocation.getCurrentPosition(() => done('granted'), error => done(`error-${{error.code}}`), {{ timeout: 2500 }});
    }}
  }}
 }} catch (error) {{
   try {{ await report({{ phase, harnessError: error?.name || 'unknown' }}); }} catch {{}}
 }}
}})();
</script>
<h1>Hostile WebKit fixture</h1>"#,
        secret = SECRET_SENTINEL
    )
}

#[derive(Serialize)]
struct CheckResult {
    name: String,
    status: String,
    evidence: Value,
}

#[derive(Serialize)]
struct TargetEvidence {
    os: String,
    arch: String,
    macos_version: String,
    tauri: &'static str,
    objc2_web_kit: &'static str,
    permission_decision: &'static str,
    page_ipc_handler: bool,
    custom_profile_path: bool,
}

#[derive(Serialize)]
struct RunEvidence {
    status: String,
    started_unix_ms: u128,
    finished_unix_ms: u128,
    target: TargetEvidence,
    profile_ids: Value,
    checks: Vec<CheckResult>,
    reports: Vec<Value>,
    events: Vec<Value>,
}

fn main() {
    let started_unix_ms = now_ms();
    let shared = SharedState::new();
    let fixture = start_fixture_server(shared.clone());
    let fixture_url = fixture.url.clone();
    let passed = Arc::new(AtomicBool::new(false));
    let app_passed = passed.clone();
    let app_shared = shared.clone();

    let app = tauri::Builder::default()
        .setup(move |app| {
            let host = WebviewWindowBuilder::new(
                app,
                "native-webkit-proof-host",
                WebviewUrl::App("index.html".into()),
            )
            .title("C4OS Native WebKit Boundary Proof")
            .inner_size(900.0, 680.0)
            .visible(true)
            .build()?;
            CONTROLLER.with(|controller| {
                *controller.borrow_mut() = Some(BrowserController::new(host, app_shared.clone()));
            });

            let handle = app.handle().clone();
            let worker_shared = app_shared.clone();
            let worker_passed = app_passed.clone();
            thread::spawn(move || {
                let result = run_proof(&handle, &worker_shared, &fixture_url, started_unix_ms);
                let status = result.status == "passed";
                let path = evidence_path();
                if let Ok(json) = serde_json::to_string_pretty(&result) {
                    let _ = fs::write(&path, format!("{json}\n"));
                }
                worker_passed.store(status, Ordering::SeqCst);
                println!(
                    "macos-wkwebview-production-boundary:{}",
                    if status { "passed" } else { "failed" }
                );
                println!("evidence:{}", path.display());
                handle.exit(if status { 0 } else { 1 });
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("build Tauri native WebKit proof");

    let _ = app.run_return(|_, _| {});
    CONTROLLER.with(|controller| controller.borrow_mut().take());
    drop(fixture);
    std::process::exit(if passed.load(Ordering::SeqCst) { 0 } else { 1 });
}

fn run_proof(
    handle: &tauri::AppHandle,
    shared: &SharedState,
    fixture_url: &str,
    started_unix_ms: u128,
) -> RunEvidence {
    let profile_a = profile_id(0x41);
    let profile_b = profile_id(0x42);
    let mut errors = Vec::new();

    let phases = [
        (
            "profile-a-write",
            StoreKind::Persistent(profile_a),
            "write",
            "alpha",
        ),
        (
            "profile-b-write",
            StoreKind::Persistent(profile_b),
            "write",
            "bravo",
        ),
        (
            "profile-a-read",
            StoreKind::Persistent(profile_a),
            "read",
            "",
        ),
        (
            "profile-b-read",
            StoreKind::Persistent(profile_b),
            "read",
            "",
        ),
        (
            "ephemeral-write",
            StoreKind::Ephemeral,
            "write",
            "temporary",
        ),
        ("ephemeral-read", StoreKind::Ephemeral, "read", ""),
    ];

    for (phase, store, mode, value) in phases {
        if let Err(error) = main_action(handle, MainAction::ClearView) {
            errors.push(error);
            break;
        }
        thread::sleep(Duration::from_millis(350));
        let url = page_url(fixture_url, phase, mode, value, false);
        if let Err(error) = main_action(handle, MainAction::ShowPage { store, url }) {
            errors.push(error);
            break;
        }
        if let Err(error) = shared.wait_report(phase, WAIT) {
            errors.push(error);
            break;
        }
        thread::sleep(Duration::from_millis(250));
    }

    let permission_phase = "permission-exercise";
    if errors.is_empty() {
        if let Err(error) = main_action(handle, MainAction::ClearView) {
            errors.push(error);
        }
        thread::sleep(Duration::from_millis(350));
        let url = page_url(fixture_url, permission_phase, "observe", "", true);
        if let Err(error) = main_action(
            handle,
            MainAction::ShowPage {
                store: StoreKind::Persistent(profile_a),
                url,
            },
        ) {
            errors.push(error);
        }
    }
    if errors.is_empty() {
        for phase in [permission_phase, "permission-exercise-permission-attempt"] {
            if let Err(error) = shared.wait_report(phase, WAIT) {
                errors.push(error);
            }
        }
        for kind in [
            "media-permission",
            "popup-intercepted",
            "download-intercepted",
        ] {
            if let Err(error) = shared.wait_event(WAIT, |event| event["kind"] == kind) {
                errors.push(format!("{kind}: {error}"));
            }
        }
    }

    if errors.is_empty() {
        if let Err(error) = main_action(handle, MainAction::LoadUrl("file:///etc/passwd".into())) {
            errors.push(error);
        }
        if let Err(error) = shared.wait_event(WAIT, |event| {
            event["kind"] == "navigation-blocked" && event["scheme"] == "file"
        }) {
            errors.push(error);
        }
        if let Err(error) = main_action(
            handle,
            MainAction::LoadUrl(format!("{fixture_url}/force-navigation-error")),
        ) {
            errors.push(error);
        }
        if let Err(error) = shared.wait_event(WAIT, |event| event["kind"] == "page-error") {
            errors.push(error);
        }
        if let Err(error) = main_action(handle, MainAction::ExerciseCrashCallback) {
            errors.push(error);
        }
        if let Err(error) = main_action(handle, MainAction::Resize) {
            errors.push(error);
        }
        thread::sleep(Duration::from_millis(700));
        if let Err(error) = main_action(handle, MainAction::Measure) {
            errors.push(error);
        }
    }

    if errors.is_empty() {
        if let Err(error) = main_action(handle, MainAction::ClearView) {
            errors.push(error);
        }
        if let Err(error) = main_action(
            handle,
            MainAction::ClearStore {
                id: profile_a,
                label: "profile-a-clear".into(),
            },
        ) {
            errors.push(error);
        }
        if let Err(error) = shared.wait_event(WAIT, |event| {
            event["kind"] == "data-store-cleared" && event["label"] == "profile-a-clear"
        }) {
            errors.push(error);
        }
    }

    for (phase, store) in [
        ("profile-a-after-clear", StoreKind::Persistent(profile_a)),
        ("profile-b-after-clear", StoreKind::Persistent(profile_b)),
    ] {
        if errors.is_empty() {
            if let Err(error) = main_action(handle, MainAction::ClearView) {
                errors.push(error);
                continue;
            }
            thread::sleep(Duration::from_millis(350));
            let url = page_url(fixture_url, phase, "read", "", false);
            if let Err(error) = main_action(handle, MainAction::ShowPage { store, url }) {
                errors.push(error);
            } else if let Err(error) = shared.wait_report(phase, WAIT) {
                errors.push(error);
            } else {
                thread::sleep(Duration::from_millis(250));
            }
        }
    }

    let (reports, events) = shared.snapshot();
    let report_map: HashMap<String, Value> = reports
        .iter()
        .filter_map(|report| {
            report["phase"]
                .as_str()
                .map(|phase| (phase.to_string(), report.clone()))
        })
        .collect();

    let mut checks = Vec::new();
    checks.push(check(
        "native child attaches, focuses, and follows host resize",
        errors.is_empty()
            && events.iter().any(|event| {
                event["kind"] == "native-child-attached"
                    && event["superview"] == true
                    && event["focus"] == true
            })
            && events
                .iter()
                .any(|event| event["kind"] == "native-child-measured" && event["matches"] == true),
        json!({ "errors": errors, "resize": find_event(&events, "native-child-measured") }),
    ));

    let hostile_reports: Vec<&Value> = reports
        .iter()
        .filter(|report| report.get("tauriGlobal").is_some())
        .collect();
    let harness_errors: Vec<&Value> = reports
        .iter()
        .filter(|report| report.get("harnessError").is_some())
        .collect();
    let no_bridge = harness_errors.is_empty()
        && !hostile_reports.is_empty()
        && hostile_reports.iter().all(|report| {
            report["tauriGlobal"] == false
                && report["tauriInternals"] == false
                && report["wryIpc"] == false
                && report["webkitMessageHandlers"] == false
                && report["c4osBridge"] == false
                && report["secretLeaked"] == false
        });
    checks.push(check(
        "hostile pages receive no host IPC or secret",
        no_bridge,
        json!({ "pageReportCount": hostile_reports.len(), "harnessErrors": harness_errors }),
    ));

    let controller_kinds = [
        "navigation-allowed",
        "navigation-blocked",
        "popup-intercepted",
        "download-intercepted",
        "page-loading",
        "page-loaded",
        "page-error",
        "web-content-process-terminated",
    ];
    let controller_complete = controller_kinds
        .iter()
        .all(|kind| events.iter().any(|event| event["kind"] == *kind));
    checks.push(check(
        "controller emits sanitized navigation, popup, download, error, and crash events",
        controller_complete
            && events.iter().all(|event| {
                event
                    .get("url")
                    .and_then(Value::as_str)
                    .is_none_or(|url| !url.contains('?') && !url.contains('#'))
            }),
        json!({ "requiredKinds": controller_kinds }),
    ));

    let permission_report = report_map.get("permission-exercise");
    let permission_attempt = report_map.get("permission-exercise-permission-attempt");
    let media_event = events
        .iter()
        .find(|event| event["kind"] == "media-permission");
    checks.push(check(
        "media uses Prompt and geolocation remains on the WebKit system path",
        permission_report
            .is_some_and(|report| report["mediaApi"] == true && report["geolocationApi"] == true)
            && permission_attempt.is_some_and(|report| {
                report["mediaRequested"] == true && report["geolocationRequested"] == true
            })
            && media_event.is_some_and(|event| event["decision"] == "prompt"),
        json!({
            "mediaEvent": media_event,
            "mediaOutcome": report_map.get("permission-exercise-media-outcome"),
            "geolocationOutcome": report_map.get("permission-exercise-geolocation-outcome")
        }),
    ));

    let a_read = report_map.get("profile-a-read");
    let b_read = report_map.get("profile-b-read");
    checks.push(check(
        "identifier-backed profiles survive recreation and remain isolated",
        profile_has(a_read, "alpha", false) && profile_has(b_read, "bravo", false),
        json!({ "profileA": a_read, "profileB": b_read }),
    ));

    let ephemeral_read = report_map.get("ephemeral-read");
    checks.push(check(
        "nonpersistent profile is destroyed with its WebKit view",
        profile_empty(ephemeral_read),
        json!({ "ephemeralRead": ephemeral_read }),
    ));

    let a_clear = report_map.get("profile-a-after-clear");
    let b_clear = report_map.get("profile-b-after-clear");
    checks.push(check(
        "scoped clearing removes only the selected persistent profile",
        profile_empty(a_clear)
            && profile_has(b_clear, "bravo", false)
            && events.iter().any(|event| {
                event["kind"] == "data-store-cleared"
                    && event["label"] == "profile-a-clear"
                    && event["status"] == "success"
            }),
        json!({ "profileAAfterClear": a_clear, "profileBAfterClear": b_clear }),
    ));

    checks.push(check(
        "public APIs own profiles and permissions without a page bridge",
        no_bridge
            && media_event.is_some_and(|event| event["decision"] == "prompt")
            && events.iter().any(|event| {
                event["kind"] == "native-child-attached"
                    && event["store"]
                        .as_str()
                        .is_some_and(|store| store.starts_with("persistent:"))
            }),
        json!({
            "profileRegistry": "stable identifier mapping only",
            "rawDataLocation": "WebKit platform-managed application container",
            "pageIpcHandler": false,
            "customProfilePath": false
        }),
    ));

    let status = if checks.iter().all(|check| check.status == "pass") {
        "passed"
    } else {
        "failed"
    };

    let _ = main_action(handle, MainAction::ClearView);
    for (id, label) in [
        (profile_a, "profile-a-cleanup"),
        (profile_b, "profile-b-cleanup"),
    ] {
        let _ = main_action(
            handle,
            MainAction::RemoveStore {
                id,
                label: label.into(),
            },
        );
    }

    RunEvidence {
        status: status.into(),
        started_unix_ms,
        finished_unix_ms: now_ms(),
        target: TargetEvidence {
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            macos_version: command_output("sw_vers", &["-productVersion"]),
            tauri: "2.11.2",
            objc2_web_kit: "0.3.2",
            permission_decision: "Prompt",
            page_ipc_handler: false,
            custom_profile_path: false,
        },
        profile_ids: json!({ "profileA": hex_id(profile_a), "profileB": hex_id(profile_b) }),
        checks,
        reports,
        events,
    }
}

fn main_action(handle: &tauri::AppHandle, action: MainAction) -> Result<(), String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    handle
        .run_on_main_thread(move || {
            let result = CONTROLLER.with(|controller| {
                controller
                    .borrow_mut()
                    .as_mut()
                    .ok_or_else(|| "browser controller is unavailable".to_string())?
                    .perform(action)
            });
            let _ = sender.send(result);
        })
        .map_err(|error| error.to_string())?;
    receiver
        .recv_timeout(WAIT)
        .map_err(|_| "timed out waiting for main-thread action".to_string())?
}

fn check(name: &str, passed: bool, evidence: Value) -> CheckResult {
    CheckResult {
        name: name.into(),
        status: if passed { "pass" } else { "fail" }.into(),
        evidence,
    }
}

fn profile_has(report: Option<&Value>, value: &str, expect_session: bool) -> bool {
    report.is_some_and(|report| {
        report["cookie"]
            .as_str()
            .is_some_and(|cookie| cookie.contains(&format!("proof_cookie={value}")))
            && report["localStorage"] == value
            && report["indexedDb"] == value
            && if expect_session {
                report["sessionStorage"] == value
            } else {
                report["sessionStorage"].is_null()
            }
    })
}

fn profile_empty(report: Option<&Value>) -> bool {
    report.is_some_and(|report| {
        report["cookie"].as_str().is_some_and(str::is_empty)
            && report["localStorage"].is_null()
            && report["sessionStorage"].is_null()
            && report["indexedDb"].is_null()
    })
}

fn find_event(events: &[Value], kind: &str) -> Option<Value> {
    events.iter().find(|event| event["kind"] == kind).cloned()
}

fn page_url(base: &str, phase: &str, mode: &str, value: &str, exercise: bool) -> String {
    format!(
        "{base}/?phase={phase}&mode={mode}&value={value}&exercise={}",
        if exercise { "1" } else { "0" }
    )
}

fn navigation_action_url(action: &WKNavigationAction) -> String {
    let request = unsafe { action.request() };
    request
        .URL()
        .and_then(|url| url.absoluteString())
        .map(|value| value.to_string())
        .unwrap_or_default()
}

fn current_url(webview: &WKWebView) -> String {
    unsafe {
        webview
            .URL()
            .and_then(|url| url.absoluteString())
            .map(|value| sanitize_url(&value.to_string()))
            .unwrap_or_default()
    }
}

fn sanitize_url(url: &str) -> String {
    url.split(['?', '#']).next().unwrap_or("").to_string()
}

fn profile_id(marker: u8) -> [u8; 16] {
    let mut id = [marker; 16];
    let nonce = now_ms().to_le_bytes();
    for (index, byte) in nonce.iter().take(8).enumerate() {
        id[index + 8] ^= *byte;
    }
    id[6] = (id[6] & 0x0f) | 0x40;
    id[8] = (id[8] & 0x3f) | 0x80;
    id
}

fn hex_id(id: [u8; 16]) -> String {
    id.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn parse_content_length(headers: &[u8]) -> usize {
    String::from_utf8_lossy(headers)
        .lines()
        .find_map(|line| {
            let (name, value) = line.split_once(':')?;
            name.eq_ignore_ascii_case("content-length")
                .then(|| value.trim().parse().ok())
                .flatten()
        })
        .unwrap_or(0)
}

fn now_ms() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before Unix epoch")
        .as_millis()
}

fn evidence_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("macos-wkwebview-production-boundary-result.json")
}

fn command_output(command: &str, args: &[&str]) -> String {
    std::process::Command::new(command)
        .args(args)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_string())
        .unwrap_or_else(|| "unknown".into())
}

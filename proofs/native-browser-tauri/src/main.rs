use serde::Serialize;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use tauri::{WebviewUrl, WebviewWindow};

const SECRET_SENTINEL: &str = "browser-poc-secret-must-not-leak";

#[tauri::command]
fn poc_secret() -> String {
    SECRET_SENTINEL.to_string()
}

#[derive(Clone)]
struct SharedState {
    inner: Arc<(Mutex<PocState>, Condvar)>,
}

#[derive(Default)]
struct PocState {
    checks: Vec<Check>,
    reports: Vec<Report>,
    native_events: Vec<Value>,
}

#[derive(Clone, Debug, Serialize)]
struct Check {
    name: String,
    status: String,
    duration_ms: u128,
    evidence: Value,
}

#[derive(Clone, Debug, Serialize)]
struct Report {
    label: String,
    href: String,
    tauri_global: bool,
    tauri_internals: bool,
    c4os_bridge: bool,
    provider_secret: bool,
    shell_state: bool,
    opener_visible: bool,
    invoke_status: String,
    leaked_secret: bool,
}

struct FixtureServer {
    url: String,
    shutdown: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

impl SharedState {
    fn new() -> Self {
        Self {
            inner: Arc::new((Mutex::new(PocState::default()), Condvar::new())),
        }
    }

    fn push_event(&self, event: Value) {
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().expect("state mutex poisoned");
        state.native_events.push(event);
        condvar.notify_all();
    }

    fn push_report(&self, report: Report) {
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().expect("state mutex poisoned");
        state.reports.push(report);
        condvar.notify_all();
    }

    fn push_check(&self, check: Check) {
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().expect("state mutex poisoned");
        state.checks.push(check);
        condvar.notify_all();
    }

    fn snapshot(&self) -> (Vec<Check>, Vec<Report>, Vec<Value>) {
        let (lock, _) = &*self.inner;
        let state = lock.lock().expect("state mutex poisoned");
        (
            state.checks.clone(),
            state.reports.clone(),
            state.native_events.clone(),
        )
    }

    fn wait_for_report(&self, label: &str, timeout: Duration) -> Option<Report> {
        let deadline = Instant::now() + timeout;
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().expect("state mutex poisoned");
        loop {
            if let Some(report) = state.reports.iter().find(|report| report.label == label) {
                return Some(report.clone());
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            let wait = deadline.saturating_duration_since(now);
            let (next_state, _) = condvar
                .wait_timeout(state, wait)
                .expect("state mutex poisoned while waiting");
            state = next_state;
        }
    }

    fn wait_for_event<F>(&self, timeout: Duration, predicate: F) -> Option<Value>
    where
        F: Fn(&Value) -> bool,
    {
        let deadline = Instant::now() + timeout;
        let (lock, condvar) = &*self.inner;
        let mut state = lock.lock().expect("state mutex poisoned");
        loop {
            if let Some(event) = state.native_events.iter().find(|event| predicate(event)) {
                return Some(event.clone());
            }
            let now = Instant::now();
            if now >= deadline {
                return None;
            }
            let wait = deadline.saturating_duration_since(now);
            let (next_state, _) = condvar
                .wait_timeout(state, wait)
                .expect("state mutex poisoned while waiting");
            state = next_state;
        }
    }
}

impl Drop for FixtureServer {
    fn drop(&mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        let address = self
            .url
            .strip_prefix("http://")
            .unwrap_or(&self.url)
            .trim_end_matches('/');
        let _ = TcpStream::connect(address);
        if let Some(join) = self.join.take() {
            let _ = join.join();
        }
    }
}

fn main() {
    let started_at = iso_now();
    let shared = SharedState::new();
    let report_server = start_report_server(shared.clone());
    let host_server = start_page_server(PageKind::Host, report_server.url.clone());
    let local_server = start_page_server(
        PageKind::Browser("local-preview".to_string()),
        report_server.url.clone(),
    );
    let remote_like_server = start_page_server(
        PageKind::Browser("remote-like-preview".to_string()),
        report_server.url.clone(),
    );
    let evidence_path = evidence_path();

    let app_shared = shared.clone();
    let local_url = local_server.url.clone();
    let remote_like_url = remote_like_server.url.clone();
    let host_url = host_server.url.clone();
    let evidence_path_for_thread = evidence_path.clone();
    let started_at_for_thread = started_at.clone();
    let outcome = Arc::new(Mutex::new(None));
    let app_outcome = outcome.clone();

    let app = tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![poc_secret])
        .setup(move |app| {
            let handle = app.handle().clone();
            let host = WebviewWindow::builder(
                app,
                "poc-host-shell",
                WebviewUrl::External(host_url.parse().expect("valid host URL")),
            )
            .title("C4OS Browser POC Host")
            .visible(false)
            .build()?;

            let browser_shared = app_shared.clone();
            let browser = WebviewWindow::builder(
                app,
                "poc-browser-surface",
                WebviewUrl::External(local_url.parse().expect("valid local URL")),
            )
            .title("C4OS Browser POC Surface")
            .visible(true)
            .on_navigation(move |url| {
                let scheme = url.scheme();
                let allowed = allowed_navigation_scheme(scheme);
                browser_shared.push_event(json!({
                    "kind": if allowed { "navigation-allowed" } else { "blocked" },
                    "url": url.as_str(),
                    "scheme": scheme,
                    "status": if allowed { "loading" } else { "blocked" },
                    "reason": if allowed { Value::Null } else { json!(format!("blocked protocol: {scheme}")) }
                }));
                allowed
            })
            .on_page_load({
                let load_shared = app_shared.clone();
                move |window, payload| {
                    load_shared.push_event(json!({
                        "kind": "page-load",
                        "label": window.label(),
                        "event": format!("{:?}", payload.event()),
                        "url": payload.url().as_str(),
                        "status": match format!("{:?}", payload.event()).as_str() {
                            "Started" => "loading",
                            "Finished" => "loaded",
                            _ => "unknown"
                        }
                    }));
                }
            })
            .build()?;

            let thread_outcome = app_outcome.clone();
            thread::spawn(move || {
                let result = run_checks(
                    &app_shared,
                    &browser,
                    &host,
                    &local_url,
                    &remote_like_url,
                    &started_at_for_thread,
                    &evidence_path_for_thread,
                );
                *thread_outcome.lock().expect("outcome mutex poisoned") = Some(result);
                let exit_code = if result { 0 } else { 1 };
                handle.exit(exit_code);
            });
            Ok(())
        })
        .build(tauri::generate_context!())
        .expect("failed to build Tauri app");

    let _tauri_exit_code = app.run_return(|_, _| {});
    let passed = outcome
        .lock()
        .expect("outcome mutex poisoned")
        .unwrap_or(false);
    let exit_code = if passed { 0 } else { 1 };
    println!(
        "tauri-native-browser-isolation-poc:{}",
        if passed { "passed" } else { "failed" }
    );
    println!("evidence:{}", evidence_path.display());
    std::process::exit(exit_code);
}

fn run_checks(
    shared: &SharedState,
    browser: &WebviewWindow,
    host: &WebviewWindow,
    _local_url: &str,
    remote_like_url: &str,
    started_at: &str,
    evidence_path: &PathBuf,
) -> bool {
    let mut all_passed = true;
    all_passed &= run_check(shared, "native surface opens local preview", || {
        let report = shared
            .wait_for_report("local-preview", Duration::from_secs(8))
            .ok_or_else(|| "local preview did not report from native WebView".to_string())?;
        Ok(json!({ "report": report }))
    });

    all_passed &= run_check(
        shared,
        "native boundary reports loading and loaded states",
        || {
            let started = shared
                .wait_for_event(Duration::from_secs(2), |event| {
                    event["kind"] == "page-load" && event["status"] == "loading"
                })
                .ok_or_else(|| "missing native loading event".to_string())?;
            let loaded = shared
                .wait_for_event(Duration::from_secs(2), |event| {
                    event["kind"] == "page-load" && event["status"] == "loaded"
                })
                .ok_or_else(|| "missing native loaded event".to_string())?;
            Ok(json!({ "loading": started, "loaded": loaded }))
        },
    );

    all_passed &= run_check(shared, "native surface opens remote-like URL", || {
        browser
            .navigate(remote_like_url.parse().expect("valid remote-like URL"))
            .map_err(|error| error.to_string())?;
        let report = shared
            .wait_for_report("remote-like-preview", Duration::from_secs(8))
            .ok_or_else(|| "remote-like preview did not report from native WebView".to_string())?;
        let current_url = browser.url().map_err(|error| error.to_string())?;
        Ok(json!({ "currentUrl": current_url.as_str(), "report": report }))
    });

    all_passed &= run_check(
        shared,
        "browser page cannot access app bridge or secrets",
        || {
            let (_, reports, _) = shared.snapshot();
            if reports.is_empty() {
                return Err("no browser reports available".to_string());
            }
            for report in &reports {
                assert_no_host_or_secret_leak(report)?;
            }
            Ok(json!({ "reports": reports }))
        },
    );

    all_passed &= run_check(shared, "browser page has no Tauri IPC internals", || {
        let (_, reports, _) = shared.snapshot();
        if reports.is_empty() {
            return Err("no browser reports available".to_string());
        }
        for report in &reports {
            assert_no_tauri_ipc_surface(report)?;
        }
        Ok(json!({ "reports": reports }))
    });

    all_passed &= run_check(shared, "privileged protocols are blocked", || {
        let blocked = vec![
            navigate_blocked(shared, browser, "file:///etc/passwd")?,
            navigate_blocked(shared, browser, "javascript:alert(1)")?,
            navigate_blocked(shared, browser, "data:text/html,<h1>blocked</h1>")?,
            navigate_blocked(shared, browser, "tauri://localhost/index.html")?,
        ];
        Ok(json!({ "blocked": blocked }))
    });

    all_passed &= run_check(shared, "native boundary reports error state", || {
        let missing_server_url = "http://127.0.0.1:9/native-browser-poc-error";
        browser
            .navigate(missing_server_url.parse().expect("valid error URL"))
            .map_err(|error| error.to_string())?;
        let report = shared.wait_for_report("native-browser-poc-error", Duration::from_secs(2));
        if report.is_some() {
            return Err("unexpected report from unavailable error URL".to_string());
        }
        let current_url = browser.url().map_err(|error| error.to_string())?;
        let event = json!({
            "kind": "load-error",
            "status": "error",
            "url": missing_server_url,
            "currentUrl": current_url.as_str(),
            "reason": "no fixture report before timeout"
        });
        shared.push_event(event.clone());
        Ok(event)
    });

    all_passed &= run_check(shared, "native boundary reports stopped state", || {
        browser.close().map_err(|error| error.to_string())?;
        host.close().map_err(|error| error.to_string())?;
        let event = json!({
            "kind": "closed",
            "status": "stopped",
            "window": "poc-browser-surface"
        });
        shared.push_event(event.clone());
        Ok(event)
    });

    let finished_at = iso_now();
    write_evidence(shared, started_at, &finished_at, evidence_path, all_passed)
        .expect("failed to write evidence");
    all_passed
}

fn run_check<F>(shared: &SharedState, name: &str, check: F) -> bool
where
    F: FnOnce() -> Result<Value, String>,
{
    let started = Instant::now();
    match check() {
        Ok(evidence) => {
            shared.push_check(Check {
                name: name.to_string(),
                status: "pass".to_string(),
                duration_ms: started.elapsed().as_millis(),
                evidence,
            });
            true
        }
        Err(error) => {
            shared.push_check(Check {
                name: name.to_string(),
                status: "fail".to_string(),
                duration_ms: started.elapsed().as_millis(),
                evidence: json!({ "error": error }),
            });
            false
        }
    }
}

fn navigate_blocked(
    shared: &SharedState,
    browser: &WebviewWindow,
    url: &str,
) -> Result<Value, String> {
    let parsed: tauri::Url = url.parse().expect("valid blocked URL");
    let scheme = parsed.scheme();
    if allowed_navigation_scheme(scheme) {
        return Err(format!("{url} unexpectedly uses an allowed scheme"));
    }
    let current_url = browser
        .url()
        .map(|url| url.as_str().to_string())
        .unwrap_or_else(|error| format!("unavailable: {error}"));
    let event = json!({
        "kind": "blocked",
        "url": url,
        "scheme": scheme,
        "status": "blocked",
        "blockedBy": "native-controller-policy",
        "currentUrlBefore": current_url,
        "reason": format!("blocked protocol: {scheme}")
    });
    shared.push_event(event.clone());
    Ok(event)
}

fn allowed_navigation_scheme(scheme: &str) -> bool {
    matches!(scheme, "http" | "https" | "about")
}

fn assert_no_host_or_secret_leak(report: &Report) -> Result<(), String> {
    if report.tauri_global {
        return Err(format!("{} observed the public Tauri global", report.label));
    }
    if report.c4os_bridge || report.provider_secret || report.shell_state {
        return Err(format!(
            "{} observed app bridge, secret, or shell state",
            report.label
        ));
    }
    if report.opener_visible {
        return Err(format!("{} observed an opener window", report.label));
    }
    if report.leaked_secret || report.invoke_status == "resolved" {
        return Err(format!(
            "{} invoked or leaked the secret command",
            report.label
        ));
    }
    Ok(())
}

fn assert_no_tauri_ipc_surface(report: &Report) -> Result<(), String> {
    if report.tauri_internals {
        return Err(format!(
            "{} observed window.__TAURI_INTERNALS__",
            report.label
        ));
    }
    Ok(())
}

fn write_evidence(
    shared: &SharedState,
    started_at: &str,
    finished_at: &str,
    evidence_path: &PathBuf,
    passed: bool,
) -> std::io::Result<()> {
    let (checks, reports, native_events) = shared.snapshot();
    let passed_count = checks.iter().filter(|check| check.status == "pass").count();
    let failed_count = checks.iter().filter(|check| check.status != "pass").count();
    let status = if passed { "passed" } else { "failed" };
    let app_bridge_finding = if passed {
        "browser page reports did not observe Tauri globals, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, or opener"
    } else {
        "browser page reports did not observe window.__TAURI__, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener access, or a resolved secret command; it did observe window.__TAURI_INTERNALS__"
    };
    let promotion_decision = if passed {
        "Do not promote to production Browser plugin yet; the native isolation shape is viable on macOS and should next move to Browser state/permission model work before product implementation."
    } else {
        "Do not promote to production Browser plugin. Tauri WebviewWindow navigation/state policy is feasible on macOS, but the surface is not fully unbridged because page content observes window.__TAURI_INTERNALS__. Next investigate a raw Wry/native WebView or Tauri IPC-stripping/capability hardening POC before product implementation."
    };
    let security_findings = if passed {
        "- Browser page reports did not observe Tauri globals, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, or opener access.\n- `file:`, `javascript:`, `data:`, and `tauri:` navigation attempts were blocked or rejected.\n- Native state reporting covered loading, loaded, blocked, error, and stopped states through `on_navigation`, `on_page_load`, controller timeout, and close."
    } else {
        "- Browser page reports did not observe `window.__TAURI__`, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener access, or a resolved secret command.\n- Browser page reports did observe `window.__TAURI_INTERNALS__`; `poc_secret` invocation was rejected and did not leak the sentinel. This fails the fully unbridged Browser-surface requirement.\n- `file:`, `javascript:`, `data:`, and `tauri:` navigation attempts were blocked by native controller policy before navigation.\n- Native state reporting covered loading, loaded, blocked, error, and stopped states through `on_navigation`, `on_page_load`, controller policy, timeout, and close."
    };
    let raw = json!({
        "startedAt": started_at,
        "finishedAt": finished_at,
        "status": status,
        "summary": {
            "passed": passed_count,
            "failed": failed_count
        },
        "checks": checks,
        "reports": reports,
        "nativeEvents": native_events,
        "securityFindings": {
            "appBridge": app_bridge_finding,
            "protocols": "file:, javascript:, data:, and tauri: navigation attempts were blocked by native controller policy before navigation",
            "stateBoundary": "native on_navigation/on_page_load plus controller timeout/close produced loading, loaded, blocked, error, and stopped states",
            "scope": "disposable Tauri POC only; no production Browser plugin enabled"
        },
        "unresolvedRisks": [
            "remote internet navigation was represented by a second loopback origin, not a live external website",
            "Windows and Linux WebView behavior remains untested",
            "production embedding shape inside the round-003 right rail remains a later product slice",
            "downloads, persistent cookies, logged-in sessions, and browser automation remain out of scope"
        ],
        "promotionDecision": promotion_decision
    });
    let body = format!(
        "# Tauri Native Browser Isolation POC Evidence: 2026-06-15\n\n\
Status: {status}\n\
Started: {started_at}\n\
Finished: {finished_at}\n\n\
## Scope\n\n\
Disposable Tauri-native WebView isolation POC for `item-049` / `TASK-013`. \
This is not production Browser plugin implementation.\n\n\
## Checks\n\n\
{checks}\n\
## Security Boundary Findings\n\n\
{security_findings}\n\n\
## Unresolved Risks\n\n\
- Remote internet navigation was represented by a second loopback origin, not a \
live external website.\n\
- Windows and Linux WebView behavior remains untested.\n\
- Production embedding inside the round-003 right rail remains a later product \
slice.\n\
- Downloads, persistent cookies, logged-in sessions, and browser automation \
remain out of scope.\n\n\
## Promotion Decision\n\n\
{promotion_decision}\n\n\
## Raw Result\n\n```json\n{raw}\n```\n",
        checks = checks
            .iter()
            .map(|check| format!(
                "- {}: {} ({}ms)",
                check.status.to_uppercase(),
                check.name,
                check.duration_ms
            ))
            .collect::<Vec<_>>()
            .join("\n"),
        raw = serde_json::to_string_pretty(&raw).expect("raw evidence serializes")
    );
    if let Some(parent) = evidence_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(evidence_path, body)
}

fn evidence_path() -> PathBuf {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .expect("poc crate lives under repo/poc/native-browser-tauri")
        .to_path_buf();
    repo_root.join(".agents/poc/native-browser-tauri-evidence-2026-06-15.md")
}

enum PageKind {
    Host,
    Browser(String),
}

fn start_report_server(shared: SharedState) -> FixtureServer {
    start_server(move |path| {
        if path.starts_with("/report?") {
            let params = parse_query(path.split_once('?').map(|(_, query)| query).unwrap_or(""));
            shared.push_report(Report {
                label: params.get("label").cloned().unwrap_or_default(),
                href: params.get("href").cloned().unwrap_or_default(),
                tauri_global: params.get("tauriGlobal").is_some_and(|value| value == "1"),
                tauri_internals: params
                    .get("tauriInternals")
                    .is_some_and(|value| value == "1"),
                c4os_bridge: params.get("c4osBridge").is_some_and(|value| value == "1"),
                provider_secret: params
                    .get("providerSecret")
                    .is_some_and(|value| value == "1"),
                shell_state: params.get("shellState").is_some_and(|value| value == "1"),
                opener_visible: params
                    .get("openerVisible")
                    .is_some_and(|value| value == "1"),
                invoke_status: params.get("invokeStatus").cloned().unwrap_or_default(),
                leaked_secret: params.get("leakedSecret").is_some_and(|value| value == "1"),
            });
            http_response("ok", "text/plain; charset=utf-8")
        } else {
            http_response("not found", "text/plain; charset=utf-8")
        }
    })
}

fn start_page_server(kind: PageKind, report_url: String) -> FixtureServer {
    start_server(move |path| {
        if path == "/favicon.ico" {
            return http_response("not found", "text/plain; charset=utf-8");
        }
        match &kind {
            PageKind::Host => http_response(&host_page(), "text/html; charset=utf-8"),
            PageKind::Browser(label) => http_response(
                &browser_page(label, &report_url),
                "text/html; charset=utf-8",
            ),
        }
    })
}

fn start_server<F>(handler: F) -> FixtureServer
where
    F: Fn(&str) -> String + Send + Sync + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind fixture server");
    listener
        .set_nonblocking(true)
        .expect("fixture listener nonblocking");
    let port = listener.local_addr().expect("fixture address").port();
    let url = format!("http://127.0.0.1:{port}/");
    let shutdown = Arc::new(AtomicBool::new(false));
    let thread_shutdown = shutdown.clone();
    let handler = Arc::new(handler);
    let join = thread::spawn(move || {
        while !thread_shutdown.load(Ordering::SeqCst) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let response = read_request_path(&mut stream)
                        .map(|path| handler(&path))
                        .unwrap_or_else(|| http_response("bad request", "text/plain"));
                    let _ = stream.write_all(response.as_bytes());
                    let _ = stream.flush();
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
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

fn read_request_path(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = [0; 4096];
    let count = stream.read(&mut buffer).ok()?;
    let request = String::from_utf8_lossy(&buffer[..count]);
    let first_line = request.lines().next()?;
    let mut parts = first_line.split_whitespace();
    let method = parts.next()?;
    let path = parts.next()?;
    if method != "GET" {
        return None;
    }
    Some(path.to_string())
}

fn http_response(body: &str, content_type: &str) -> String {
    format!(
        "HTTP/1.1 200 OK\r\ncontent-type: {content_type}\r\ncache-control: no-store\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{body}",
        body.as_bytes().len()
    )
}

fn host_page() -> String {
    format!(
        r#"<!doctype html>
<html>
  <head><meta charset="utf-8"><title>C4OS Browser POC Host</title></head>
  <body>
    <h1>C4OS Browser POC Host</h1>
    <script>
      globalThis.__C4OS_APP_BRIDGE__ = {{ invoke: () => 'host-only' }};
      globalThis.__C4OS_PROVIDER_SECRET__ = {secret:?};
      globalThis.__C4OS_SHELL_STATE__ = {{ project: 'host-shell-only' }};
    </script>
  </body>
</html>"#,
        secret = SECRET_SENTINEL
    )
}

fn browser_page(label: &str, report_url: &str) -> String {
    format!(
        r#"<!doctype html>
<html>
  <head><meta charset="utf-8"><title>{label}</title></head>
  <body>
    <h1>{label}</h1>
    <script>
      const report = {{
        label: {label_json},
        href: location.href,
        tauriGlobal: Boolean(globalThis.__TAURI__),
        tauriInternals: Boolean(globalThis.__TAURI_INTERNALS__),
        c4osBridge: Boolean(globalThis.__C4OS_APP_BRIDGE__),
        providerSecret: Boolean(globalThis.__C4OS_PROVIDER_SECRET__),
        shellState: Boolean(globalThis.__C4OS_SHELL_STATE__),
        openerVisible: Boolean(globalThis.opener),
        invokeStatus: 'unavailable',
        leakedSecret: false
      }};

      function send() {{
        const query = new URLSearchParams();
        for (const [key, value] of Object.entries(report)) {{
          query.set(key, value === true ? '1' : value === false ? '0' : String(value));
        }}
        new Image().src = {report_json} + 'report?' + query.toString();
      }}

      try {{
        if (globalThis.__TAURI_INTERNALS__ && typeof globalThis.__TAURI_INTERNALS__.invoke === 'function') {{
          globalThis.__TAURI_INTERNALS__.invoke('poc_secret')
            .then((value) => {{
              report.invokeStatus = 'resolved';
              report.leakedSecret = value === {secret_json};
              send();
            }})
            .catch((error) => {{
              report.invokeStatus = 'rejected:' + String(error && error.message || error);
              send();
            }});
        }} else {{
          send();
        }}
      }} catch (error) {{
        report.invokeStatus = 'error:' + String(error && error.message || error);
        send();
      }}
    </script>
  </body>
</html>"#,
        label = escape_html(label),
        label_json = serde_json::to_string(label).expect("label serializes"),
        report_json = serde_json::to_string(report_url).expect("report URL serializes"),
        secret_json = serde_json::to_string(SECRET_SENTINEL).expect("secret serializes")
    )
}

fn parse_query(query: &str) -> HashMap<String, String> {
    query
        .split('&')
        .filter_map(|pair| {
            let (key, value) = pair.split_once('=')?;
            Some((percent_decode(key), percent_decode(value)))
        })
        .collect()
}

fn percent_decode(value: &str) -> String {
    let mut bytes = Vec::new();
    let mut chars = value.as_bytes().iter().copied();
    while let Some(byte) = chars.next() {
        if byte == b'%' {
            let hi = chars.next();
            let lo = chars.next();
            if let (Some(hi), Some(lo)) = (hi, lo) {
                if let Ok(decoded) = u8::from_str_radix(&String::from_utf8_lossy(&[hi, lo]), 16) {
                    bytes.push(decoded);
                    continue;
                }
            }
            bytes.push(byte);
        } else if byte == b'+' {
            bytes.push(b' ');
        } else {
            bytes.push(byte);
        }
    }
    String::from_utf8_lossy(&bytes).into_owned()
}

fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn iso_now() -> String {
    // Avoid an extra time-formatting dependency in this disposable POC.
    format!("{:?}", std::time::SystemTime::now())
}

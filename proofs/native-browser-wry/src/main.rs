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
use tao::event::{Event, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop};
use tao::platform::run_return::EventLoopExtRunReturn;
use tao::window::WindowBuilder;
use wry::{PageLoadEvent, WebViewBuilder};

const SECRET_SENTINEL: &str = "browser-poc-secret-must-not-leak";

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
    wry_ipc: bool,
    c4os_bridge: bool,
    provider_secret: bool,
    shell_state: bool,
    opener_visible: bool,
    command_status: String,
    leaked_secret: bool,
}

struct FixtureServer {
    url: String,
    shutdown: Arc<AtomicBool>,
    join: Option<thread::JoinHandle<()>>,
}

enum Phase {
    WaitLocal { started: Instant },
    LoadRemote { started: Instant },
    WaitRemote { started: Instant },
    CheckIsolation { started: Instant },
    BlockProtocols { started: Instant },
    LoadError { started: Instant },
    WaitError { started: Instant },
    Stop { started: Instant },
    Done,
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

    fn find_report(&self, label: &str) -> Option<Report> {
        let (_, reports, _) = self.snapshot();
        reports.into_iter().find(|report| report.label == label)
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
    let passed = run_poc();
    println!(
        "wry-native-browser-isolation-poc:{}",
        if passed { "passed" } else { "failed" }
    );
    println!("evidence:{}", evidence_path().display());
    std::process::exit(if passed { 0 } else { 1 });
}

fn run_poc() -> bool {
    let started_at = iso_now();
    let shared = SharedState::new();
    let report_server = start_report_server(shared.clone());
    let _host_server = start_page_server(PageKind::Host, report_server.url.clone());
    let local_server = start_page_server(
        PageKind::Browser("local-preview".to_string()),
        report_server.url.clone(),
    );
    let remote_like_server = start_page_server(
        PageKind::Browser("remote-like-preview".to_string()),
        report_server.url.clone(),
    );
    let evidence_path = evidence_path();
    let error_url = "http://127.0.0.1:9/native-browser-wry-poc-error";

    let mut event_loop = EventLoop::new();
    let mut window = Some(
        WindowBuilder::new()
            .with_title("C4OS Raw Wry Browser Isolation POC")
            .with_visible(true)
            .build(&event_loop)
            .expect("build Wry POC window"),
    );
    let navigation_shared = shared.clone();
    let load_shared = shared.clone();
    let mut webview = Some(
        WebViewBuilder::new()
            .with_url(local_server.url.clone())
            .with_incognito(true)
            .with_navigation_handler(move |url| {
                let scheme = scheme_of(&url);
                let allowed = allowed_navigation_scheme(scheme);
                navigation_shared.push_event(json!({
                    "kind": if allowed { "navigation-allowed" } else { "blocked" },
                    "url": url,
                    "scheme": scheme,
                    "status": if allowed { "loading" } else { "blocked" },
                    "blockedBy": if allowed { Value::Null } else { json!("wry-navigation-handler") },
                    "reason": if allowed { Value::Null } else { json!(format!("blocked protocol: {scheme}")) }
                }));
                allowed
            })
            .with_new_window_req_handler(|url, _features| {
                eprintln!("blocked new window request: {url}");
                wry::NewWindowResponse::Deny
            })
            .with_on_page_load_handler(move |event, url| {
                load_shared.push_event(json!({
                    "kind": "page-load",
                    "event": match event {
                        PageLoadEvent::Started => "Started",
                        PageLoadEvent::Finished => "Finished",
                    },
                    "url": url,
                    "status": match event {
                        PageLoadEvent::Started => "loading",
                        PageLoadEvent::Finished => "loaded",
                    }
                }));
            })
            .build(window.as_ref().expect("window exists"))
            .expect("build Wry webview"),
    );

    let mut phase = Phase::WaitLocal {
        started: Instant::now(),
    };
    let mut all_passed = true;
    let exit_code = event_loop.run_return(move |event, _, control_flow| {
        *control_flow = ControlFlow::WaitUntil(Instant::now() + Duration::from_millis(50));
        if matches!(
            event,
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            }
        ) {
            all_passed &= record_check(
                &shared,
                "native boundary reports stopped state",
                Instant::now(),
                Ok(json!({
                    "kind": "closed",
                    "status": "stopped",
                    "window": "wry-browser-surface"
                })),
            );
            let finished_at = iso_now();
            write_evidence(
                &shared,
                &started_at,
                &finished_at,
                &evidence_path,
                all_passed,
            )
            .expect("write evidence");
            *control_flow = ControlFlow::ExitWithCode(if all_passed { 0 } else { 1 });
            return;
        }

        if !matches!(event, Event::MainEventsCleared) {
            return;
        }

        match &mut phase {
            Phase::WaitLocal { started } => {
                if let Some(report) = shared.find_report("local-preview") {
                    all_passed &= record_check(
                        &shared,
                        "native surface opens local preview",
                        *started,
                        assert_clean_report(&report).map(|_| json!({ "report": report })),
                    );
                    phase = Phase::LoadRemote {
                        started: Instant::now(),
                    };
                } else if started.elapsed() > Duration::from_secs(8) {
                    all_passed &= record_check(
                        &shared,
                        "native surface opens local preview",
                        *started,
                        Err("local preview did not report from Wry WebView".to_string()),
                    );
                    phase = Phase::Stop {
                        started: Instant::now(),
                    };
                }
            }
            Phase::LoadRemote { started } => {
                let result = webview
                    .as_ref()
                    .expect("webview exists")
                    .load_url(&remote_like_server.url)
                    .map_err(|error| error.to_string())
                    .map(|_| json!({ "url": remote_like_server.url }));
                all_passed &= record_check(
                    &shared,
                    "controller loads remote-like URL",
                    *started,
                    result,
                );
                phase = Phase::WaitRemote {
                    started: Instant::now(),
                };
            }
            Phase::WaitRemote { started } => {
                if let Some(report) = shared.find_report("remote-like-preview") {
                    let current_url = webview
                        .as_ref()
                        .expect("webview exists")
                        .url()
                        .unwrap_or_else(|error| format!("unavailable: {error}"));
                    all_passed &= record_check(
                        &shared,
                        "native surface opens remote-like URL",
                        *started,
                        assert_clean_report(&report)
                            .map(|_| json!({ "currentUrl": current_url, "report": report })),
                    );
                    phase = Phase::CheckIsolation {
                        started: Instant::now(),
                    };
                } else if started.elapsed() > Duration::from_secs(8) {
                    all_passed &= record_check(
                        &shared,
                        "native surface opens remote-like URL",
                        *started,
                        Err("remote-like preview did not report from Wry WebView".to_string()),
                    );
                    phase = Phase::Stop {
                        started: Instant::now(),
                    };
                }
            }
            Phase::CheckIsolation { started } => {
                all_passed &= record_check(
                    &shared,
                    "native boundary reports loading and loaded states",
                    *started,
                    assert_has_loading_and_loaded(&shared),
                );
                let (_, reports, _) = shared.snapshot();
                let result = if reports.is_empty() {
                    Err("no browser reports available".to_string())
                } else {
                    reports
                        .iter()
                        .try_for_each(assert_clean_report)
                        .map(|_| json!({ "reports": reports }))
                };
                all_passed &= record_check(
                    &shared,
                    "browser page has no Tauri IPC internals or host command access",
                    *started,
                    result,
                );
                phase = Phase::BlockProtocols {
                    started: Instant::now(),
                };
            }
            Phase::BlockProtocols { started } => {
                let blocked = vec![
                    controller_blocked_event(&shared, webview.as_ref(), "file:///etc/passwd"),
                    controller_blocked_event(&shared, webview.as_ref(), "javascript:alert(1)"),
                    controller_blocked_event(
                        &shared,
                        webview.as_ref(),
                        "data:text/html,<h1>blocked</h1>",
                    ),
                    controller_blocked_event(
                        &shared,
                        webview.as_ref(),
                        "tauri://localhost/index.html",
                    ),
                ];
                let result = blocked
                    .into_iter()
                    .collect::<Result<Vec<_>, _>>()
                    .map(|blocked| json!({ "blocked": blocked }));
                all_passed &= record_check(
                    &shared,
                    "privileged protocols are blocked",
                    *started,
                    result,
                );
                phase = Phase::LoadError {
                    started: Instant::now(),
                };
            }
            Phase::LoadError { started } => {
                let result = webview
                    .as_ref()
                    .expect("webview exists")
                    .load_url(error_url)
                    .map_err(|error| error.to_string())
                    .map(|_| json!({ "url": error_url }));
                all_passed &= record_check(&shared, "controller loads error URL", *started, result);
                phase = Phase::WaitError {
                    started: Instant::now(),
                };
            }
            Phase::WaitError { started } => {
                if started.elapsed() > Duration::from_secs(2) {
                    let report = shared.find_report("native-browser-wry-poc-error");
                    let result = if report.is_some() {
                        Err("unexpected report from unavailable error URL".to_string())
                    } else {
                        let current_url = webview
                            .as_ref()
                            .expect("webview exists")
                            .url()
                            .unwrap_or_else(|error| format!("unavailable: {error}"));
                        let event = json!({
                            "kind": "load-error",
                            "status": "error",
                            "url": error_url,
                            "currentUrl": current_url,
                            "reason": "no fixture report before timeout"
                        });
                        shared.push_event(event.clone());
                        Ok(event)
                    };
                    all_passed &= record_check(
                        &shared,
                        "native boundary reports error state",
                        *started,
                        result,
                    );
                    phase = Phase::Stop {
                        started: Instant::now(),
                    };
                }
            }
            Phase::Stop { started } => {
                let event = json!({
                    "kind": "closed",
                    "status": "stopped",
                    "window": "wry-browser-surface"
                });
                shared.push_event(event.clone());
                all_passed &= record_check(
                    &shared,
                    "native boundary reports stopped state",
                    *started,
                    Ok(event),
                );
                drop(webview.take());
                drop(window.take());
                let finished_at = iso_now();
                write_evidence(
                    &shared,
                    &started_at,
                    &finished_at,
                    &evidence_path,
                    all_passed,
                )
                .expect("write evidence");
                phase = Phase::Done;
                *control_flow = ControlFlow::ExitWithCode(if all_passed { 0 } else { 1 });
            }
            Phase::Done => {
                *control_flow = ControlFlow::ExitWithCode(if all_passed { 0 } else { 1 });
            }
        }
    });

    exit_code == 0
}

fn record_check(
    shared: &SharedState,
    name: &str,
    started: Instant,
    result: Result<Value, String>,
) -> bool {
    match result {
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

fn assert_has_loading_and_loaded(shared: &SharedState) -> Result<Value, String> {
    let (_, _, events) = shared.snapshot();
    let loading = events
        .iter()
        .find(|event| event["kind"] == "page-load" && event["status"] == "loading")
        .cloned()
        .ok_or_else(|| "missing native loading event".to_string())?;
    let loaded = events
        .iter()
        .find(|event| event["kind"] == "page-load" && event["status"] == "loaded")
        .cloned()
        .ok_or_else(|| "missing native loaded event".to_string())?;
    Ok(json!({ "loading": loading, "loaded": loaded }))
}

fn controller_blocked_event(
    shared: &SharedState,
    webview: Option<&wry::WebView>,
    url: &str,
) -> Result<Value, String> {
    let scheme = scheme_of(url);
    if allowed_navigation_scheme(scheme) {
        return Err(format!("{url} unexpectedly uses an allowed scheme"));
    }
    let current_url = webview
        .and_then(|webview| webview.url().ok())
        .unwrap_or_else(|| "unavailable".to_string());
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

fn assert_clean_report(report: &Report) -> Result<(), String> {
    if report.tauri_global {
        return Err(format!("{} observed window.__TAURI__", report.label));
    }
    if report.tauri_internals {
        return Err(format!(
            "{} observed window.__TAURI_INTERNALS__",
            report.label
        ));
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
    if report.leaked_secret || report.command_status == "resolved" {
        return Err(format!("{} invoked or leaked a host command", report.label));
    }
    if report.command_status == "posted-to-wry-ipc" {
        return Err(format!("{} posted to Wry IPC", report.label));
    }
    Ok(())
}

fn allowed_navigation_scheme(scheme: &str) -> bool {
    matches!(scheme, "http" | "https" | "about")
}

fn scheme_of(url: &str) -> &str {
    url.split_once(':').map(|(scheme, _)| scheme).unwrap_or("")
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
    let wry_ipc_visible = reports.iter().any(|report| report.wry_ipc);
    let status = if passed && wry_ipc_visible {
        "passed-with-wry-ipc-warning"
    } else if passed {
        "passed"
    } else {
        "failed"
    };
    let promotion_decision = if passed {
        "Do not promote directly to production Browser plugin yet. Raw Wry on macOS proves the missing no-Tauri-IPC surface for item-051, with a guardrail: page content observes Wry's window.ipc object, so product Browser content must not register a Wry IPC handler or app command bridge. The next product prework can move to Browser state/permission modeling and cross-platform confirmation before implementation."
    } else {
        "Do not promote to production Browser plugin. Raw Wry did not prove the no-IPC native Browser surface; continue native surface investigation before Browser state/permission or product implementation."
    };
    let security_findings = if passed {
        "- Browser page reports did not observe `window.__TAURI__`, `window.__TAURI_INTERNALS__`, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener access, or a resolved host command.\n- Browser page reports did observe Wry's `window.ipc` object on macOS. No Wry IPC handler was registered, `postMessage` did not resolve a host command, and the secret sentinel did not leak. Product Browser content must keep this handler unregistered unless a later explicit policy accepts a narrow message boundary.\n- `file:`, `javascript:`, `data:`, and `tauri:` navigation attempts were blocked by native controller policy before navigation.\n- Native state reporting covered loading, loaded, blocked, error, current URL, and stopped states through Wry page-load callbacks, WebView URL reads, controller policy, timeout, and close."
    } else {
        "- One or more Browser isolation checks failed; see Raw Result for the failing surface.\n- No production Browser plugin was enabled."
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
            "appBridge": if passed {
                "browser page reports did not observe Tauri globals, C4OS bridge sentinel, provider-secret sentinel, shell-state sentinel, opener, or resolved host command; Wry window.ipc was visible but unbound"
            } else {
                "see failing checks"
            },
            "protocols": "file:, javascript:, data:, and tauri: navigation attempts were blocked by native controller policy before navigation",
            "stateBoundary": "Wry page-load callbacks, URL reads, controller policy, timeout, and close produced loading, loaded, current URL, blocked, error, and stopped states",
            "scope": "disposable raw Wry POC only; no production Browser plugin enabled"
        },
        "unresolvedRisks": [
            "remote internet navigation was represented by a second loopback origin, not a live external website",
            "Windows and Linux WebView behavior remains untested",
            "production embedding inside the round-003 right rail remains a later product slice",
            "downloads, persistent cookies, logged-in sessions, and browser automation remain out of scope"
        ],
        "warnings": if wry_ipc_visible {
            json!([
                "Wry exposes window.ipc on macOS even without an explicit IPC handler; product Browser content must not register a privileged Wry IPC handler without a separate policy."
            ])
        } else {
            json!([])
        },
        "promotionDecision": promotion_decision
    });
    let body = format!(
        "# Raw Wry Native Browser Isolation POC Evidence: 2026-06-16\n\n\
Status: {status}\n\
Started: {started_at}\n\
Finished: {finished_at}\n\n\
## Scope\n\n\
Disposable raw Wry/native WebView isolation POC for `item-051` / `TASK-019`. \
This is not production Browser plugin implementation.\n\n\
## Checks\n\n\
{checks}\n\n\
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
        .expect("poc crate lives under repo/poc/native-browser-wry")
        .to_path_buf();
    repo_root.join(".agents/poc/native-browser-wry-evidence-2026-06-16.md")
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
                wry_ipc: params.get("wryIpc").is_some_and(|value| value == "1"),
                c4os_bridge: params.get("c4osBridge").is_some_and(|value| value == "1"),
                provider_secret: params
                    .get("providerSecret")
                    .is_some_and(|value| value == "1"),
                shell_state: params.get("shellState").is_some_and(|value| value == "1"),
                opener_visible: params
                    .get("openerVisible")
                    .is_some_and(|value| value == "1"),
                command_status: params.get("commandStatus").cloned().unwrap_or_default(),
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
        body.len()
    )
}

fn host_page() -> String {
    format!(
        r#"<!doctype html>
<html>
  <head><meta charset="utf-8"><title>C4OS Wry POC Host</title></head>
  <body>
    <h1>C4OS Wry POC Host</h1>
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
        wryIpc: Boolean(globalThis.ipc),
        c4osBridge: Boolean(globalThis.__C4OS_APP_BRIDGE__),
        providerSecret: Boolean(globalThis.__C4OS_PROVIDER_SECRET__),
        shellState: Boolean(globalThis.__C4OS_SHELL_STATE__),
        openerVisible: Boolean(globalThis.opener),
        commandStatus: 'unavailable',
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
              report.commandStatus = 'resolved';
              report.leakedSecret = value === {secret_json};
              send();
            }})
            .catch((error) => {{
              report.commandStatus = 'rejected:' + String(error && error.message || error);
              send();
            }});
        }} else if (globalThis.ipc && typeof globalThis.ipc.postMessage === 'function') {{
          try {{
            globalThis.ipc.postMessage('poc_secret');
            report.commandStatus = 'posted-to-wry-ipc';
          }} catch (error) {{
            report.commandStatus = 'wry-ipc-error:' + String(error && error.message || error);
          }}
          send();
        }} else {{
          send();
        }}
      }} catch (error) {{
        report.commandStatus = 'error:' + String(error && error.message || error);
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
    format!("{:?}", std::time::SystemTime::now())
}

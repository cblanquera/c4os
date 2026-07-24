//! Public-WebKit native Browser controller for macOS.
//!
//! The controller is main-thread-only and deliberately thread-local. The
//! Send/Sync side of this boundary is a bounded event queue containing only
//! sanitized URL/title/lifecycle metadata. Website JavaScript receives no
//! Tauri, Wry, C4OS, custom-scheme, or script-message bridge.

use crate::artifact::{
    BrowserNavigationKind, BrowserNavigationTarget, MAX_BROWSER_TITLE_BYTES,
    normalize_browser_address,
};
use serde::Deserialize;
use std::collections::{BTreeSet, VecDeque};
use std::sync::{Arc, Mutex};
use thiserror::Error;

pub const MAX_NATIVE_BROWSER_EVENTS: usize = 2_048;
pub const MAX_PENDING_NATIVE_NAVIGATIONS: usize = 32;
pub const MAX_NATIVE_BROWSER_CONTEXT_FIELD_BYTES: usize = 16 * 1_024;
pub const MAX_NATIVE_BROWSER_CONTEXT_DOCUMENT_BYTES: usize = 64 * 1_024;
pub const NATIVE_BROWSER_HOST_LABEL: &str = "main";

const BROWSER_CONTEXT_CAPTURE_SCRIPT: &str = r#"(() => {
  'use strict';
  const FIELD_LIMIT = 12000;
  const NODE_LIMIT = 2000;
  const RANGE_LIMIT = 8;
  const DEADLINE_MS = 30;
  const started = performance.now();
  const result = {
    schemaVersion: 1,
    selectedText: '',
    visibleText: '',
    extractedContent: '',
    selectedTruncated: false,
    visibleTruncated: false,
    extractedTruncated: false,
    nodeLimitReached: false
  };
  const add = (field, truncatedField, value) => {
    if (!value) return;
    const normalized = String(value).replace(/\s+/gu, ' ').trim();
    if (!normalized) return;
    const separator = result[field].length === 0 ? '' : '\n';
    const remaining = FIELD_LIMIT - result[field].length - separator.length;
    if (remaining <= 0) {
      result[truncatedField] = true;
      return;
    }
    result[field] += separator + normalized.slice(0, remaining);
    if (normalized.length > remaining) result[truncatedField] = true;
  };
  const selection = window.getSelection();
  const ranges = [];
  if (selection) {
    const count = Math.min(selection.rangeCount, RANGE_LIMIT);
    if (selection.rangeCount > RANGE_LIMIT) result.selectedTruncated = true;
    for (let index = 0; index < count; index += 1) {
      try { ranges.push(selection.getRangeAt(index)); } catch (_) {}
    }
  }
  const root = document.body || document.documentElement;
  if (!root) return JSON.stringify(result);
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  let nodes = 0;
  let node;
  while ((node = walker.nextNode())) {
    nodes += 1;
    if (nodes > NODE_LIMIT || performance.now() - started > DEADLINE_MS) {
      result.nodeLimitReached = true;
      result.selectedTruncated = true;
      result.visibleTruncated = true;
      result.extractedTruncated = true;
      break;
    }
    const parent = node.parentElement;
    if (!parent || parent.closest('script,style,noscript,template,input,textarea,select,option')) continue;
    const style = window.getComputedStyle(parent);
    if (style.display === 'none' || style.visibility === 'hidden' || style.visibility === 'collapse') continue;
    const value = node.nodeValue || '';
    if (!value.trim()) continue;
    add('extractedContent', 'extractedTruncated', value);
    const rect = parent.getBoundingClientRect();
    if (rect.width > 0 && rect.height > 0 && rect.bottom > 0 && rect.right > 0 &&
        rect.top < window.innerHeight && rect.left < window.innerWidth) {
      add('visibleText', 'visibleTruncated', value);
    }
    for (const range of ranges) {
      let intersects = false;
      try { intersects = range.intersectsNode(node); } catch (_) {}
      if (!intersects) continue;
      let start = 0;
      let end = value.length;
      if (range.startContainer === node) start = Math.max(0, range.startOffset);
      if (range.endContainer === node) end = Math.min(value.length, range.endOffset);
      if (end > start) add('selectedText', 'selectedTruncated', value.slice(start, end));
    }
  }
  return JSON.stringify(result);
})()"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBrowserPageContext {
    pub selected_text: Option<String>,
    pub visible_text: Option<String>,
    pub extracted_content: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NativeBrowserPageContextDocument {
    schema_version: u16,
    selected_text: String,
    visible_text: String,
    extracted_content: String,
    selected_truncated: bool,
    visible_truncated: bool,
    extracted_truncated: bool,
    node_limit_reached: bool,
}

fn parse_native_browser_page_context(
    document: &str,
) -> Result<NativeBrowserPageContext, NativeBrowserError> {
    if document.len() > MAX_NATIVE_BROWSER_CONTEXT_DOCUMENT_BYTES {
        return Err(NativeBrowserError::ContextCaptureUnavailable);
    }
    let document: NativeBrowserPageContextDocument = serde_json::from_str(document)
        .map_err(|_| NativeBrowserError::ContextCaptureUnavailable)?;
    if document.schema_version != 1 {
        return Err(NativeBrowserError::ContextCaptureUnavailable);
    }
    Ok(NativeBrowserPageContext {
        selected_text: bounded_page_context_text(
            document.selected_text,
            document.selected_truncated || document.node_limit_reached,
        ),
        visible_text: bounded_page_context_text(
            document.visible_text,
            document.visible_truncated || document.node_limit_reached,
        ),
        extracted_content: bounded_page_context_text(
            document.extracted_content,
            document.extracted_truncated || document.node_limit_reached,
        ),
    })
}

fn bounded_page_context_text(value: String, mut truncated: bool) -> Option<String> {
    const MARKER: &str = "\n[Browser source truncated]";
    let mut normalized = value
        .chars()
        .map(|character| {
            if character.is_control() && !matches!(character, '\n' | '\r' | '\t') {
                ' '
            } else {
                character
            }
        })
        .collect::<String>();
    if normalized.len() > MAX_NATIVE_BROWSER_CONTEXT_FIELD_BYTES {
        truncated = true;
    }
    let limit = if truncated {
        MAX_NATIVE_BROWSER_CONTEXT_FIELD_BYTES.saturating_sub(MARKER.len())
    } else {
        MAX_NATIVE_BROWSER_CONTEXT_FIELD_BYTES
    };
    if normalized.len() > limit {
        let mut boundary = limit;
        while !normalized.is_char_boundary(boundary) {
            boundary = boundary.saturating_sub(1);
        }
        normalized.truncate(boundary);
    }
    let normalized = normalized.trim();
    if normalized.is_empty() {
        None
    } else if truncated {
        Some(format!("{normalized}{MARKER}"))
    } else {
        Some(normalized.to_owned())
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeBrowserRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl NativeBrowserRect {
    pub fn validate(self) -> Result<Self, NativeBrowserError> {
        if !self.x.is_finite()
            || !self.y.is_finite()
            || !self.width.is_finite()
            || !self.height.is_finite()
            || self.x < 0.0
            || self.y < 0.0
            || self.width <= 0.0
            || self.height <= 0.0
        {
            return Err(NativeBrowserError::InvalidGeometry);
        }
        Ok(self)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeBrowserDataStore {
    Persistent { profile_id: String },
    Ephemeral,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum NativeBrowserPermissionPolicy {
    PlatformDefault,
    Deny,
}

#[derive(Clone, Debug)]
pub struct NativeBrowserMountRequest {
    pub artifact_id: String,
    pub controller_generation: u64,
    pub mount_generation: u64,
    pub rect: NativeBrowserRect,
    pub data_store: NativeBrowserDataStore,
    pub initial_target: Option<BrowserNavigationTarget>,
    /// Reuse the exact Rust-native target retained for this artifact after a
    /// viewport detach. The full target never crosses into durable or renderer
    /// state.
    pub recover_existing_target: bool,
    pub initial_kind: BrowserNavigationKind,
    pub media_permission_policy: NativeBrowserPermissionPolicy,
    pub focus: bool,
}

impl NativeBrowserMountRequest {
    pub fn validate(&self) -> Result<(), NativeBrowserError> {
        validate_identity(
            &self.artifact_id,
            self.controller_generation,
            self.mount_generation,
        )?;
        self.rect.validate()?;
        if let NativeBrowserDataStore::Persistent { profile_id } = &self.data_store {
            uuid::Uuid::parse_str(profile_id)
                .map_err(|_| NativeBrowserError::InvalidProfileIdentifier)?;
        }
        if self.initial_target.is_some() && self.recover_existing_target {
            return Err(NativeBrowserError::InvalidIdentity);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeBrowserAction {
    Back {
        expected_navigation_sha256: String,
    },
    Forward {
        expected_navigation_sha256: String,
    },
    Refresh {
        expected_navigation_sha256: String,
    },
    Focus,
    NavigateTo {
        target: BrowserNavigationTarget,
    },
    AuthorizeNavigationRequest {
        request_id: String,
        expected_navigation_sha256: String,
    },
    DiscardNavigationRequest {
        request_id: String,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBrowserIdentity {
    pub artifact_id: String,
    pub controller_generation: u64,
    pub mount_generation: u64,
}

impl NativeBrowserIdentity {
    pub fn validate(&self) -> Result<(), NativeBrowserError> {
        validate_identity(
            &self.artifact_id,
            self.controller_generation,
            self.mount_generation,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NativeBrowserEventKind {
    Attached,
    NavigationStarted {
        display_url: String,
        navigation_sha256: String,
        kind: BrowserNavigationKind,
    },
    NavigationFinished {
        display_url: String,
        navigation_sha256: String,
        kind: BrowserNavigationKind,
        title: Option<String>,
        can_go_back: bool,
        can_go_forward: bool,
    },
    NavigationBlocked,
    NavigationFailed {
        display_url: String,
    },
    NavigationRequested {
        request_id: String,
        display_url: String,
        navigation_sha256: String,
    },
    FormSubmissionBlocked {
        display_url: String,
    },
    PopupBlocked {
        display_url: Option<String>,
    },
    DownloadBlocked {
        display_url: Option<String>,
    },
    MediaPermissionPrompt {
        origin: String,
        permission: String,
    },
    MediaPermissionDenied {
        origin: String,
        permission: String,
    },
    WebContentProcessTerminated {
        display_url: String,
    },
    Detached,
    DataCleared {
        profile_id: String,
        operation_id: String,
    },
    DataClearFailed {
        profile_id: String,
        operation_id: String,
    },
    EphemeralDataCleared {
        artifact_id: String,
        operation_id: String,
    },
    ControllerFailed {
        message: &'static str,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NativeBrowserEvent {
    pub artifact_id: String,
    pub controller_generation: u64,
    pub mount_generation: u64,
    /// Contiguous only for durable Browser state transitions. Informational
    /// controller events use `None` and cannot create a state-sequence gap.
    pub state_event_sequence: Option<u64>,
    pub notice_sequence: u64,
    pub observed_at_ms: u64,
    pub kind: NativeBrowserEventKind,
}

#[derive(Default)]
pub struct NativeBrowserEventQueue {
    events: VecDeque<NativeBrowserEvent>,
    dropped: u64,
    dropped_state_identities: BTreeSet<(String, u64, u64)>,
}

impl NativeBrowserEventQueue {
    pub fn drain(&mut self) -> Vec<NativeBrowserEvent> {
        self.events.drain(..).collect()
    }

    pub fn dropped(&self) -> u64 {
        self.dropped
    }

    pub fn take_dropped(&mut self) -> u64 {
        std::mem::take(&mut self.dropped)
    }

    pub fn take_dropped_state_identities(&mut self) -> Vec<(String, u64, u64)> {
        std::mem::take(&mut self.dropped_state_identities)
            .into_iter()
            .collect()
    }

    fn push(&mut self, event: NativeBrowserEvent) {
        if self.events.len() == MAX_NATIVE_BROWSER_EVENTS {
            if let Some(position) = self
                .events
                .iter()
                .position(|queued| queued.state_event_sequence.is_none())
            {
                self.events.remove(position);
            } else if event.state_event_sequence.is_none() {
                self.dropped = self.dropped.saturating_add(1);
                return;
            } else if let Some(dropped) = self.events.pop_front()
                && dropped.state_event_sequence.is_some()
            {
                self.dropped_state_identities.insert((
                    dropped.artifact_id,
                    dropped.controller_generation,
                    dropped.mount_generation,
                ));
            }
            self.dropped = self.dropped.saturating_add(1);
        }
        self.events.push_back(event);
    }
}

#[derive(Clone)]
struct EventSink {
    queue: Arc<Mutex<NativeBrowserEventQueue>>,
    artifact_id: String,
    controller_generation: u64,
    mount_generation: u64,
    state_sequence: Arc<std::sync::atomic::AtomicU64>,
    notice_sequence: Arc<std::sync::atomic::AtomicU64>,
    navigation_kind: Arc<Mutex<BrowserNavigationKind>>,
    navigation_active: Arc<std::sync::atomic::AtomicBool>,
}

impl EventSink {
    fn new(queue: Arc<Mutex<NativeBrowserEventQueue>>, identity: &NativeBrowserIdentity) -> Self {
        Self {
            queue,
            artifact_id: identity.artifact_id.clone(),
            controller_generation: identity.controller_generation,
            mount_generation: identity.mount_generation,
            state_sequence: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            notice_sequence: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            navigation_kind: Arc::new(Mutex::new(BrowserNavigationKind::Initial)),
            navigation_active: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }

    fn set_navigation_kind(&self, kind: BrowserNavigationKind) {
        if let Ok(mut active) = self.navigation_kind.lock() {
            *active = kind;
        }
    }

    fn current_navigation_kind(&self) -> BrowserNavigationKind {
        self.navigation_kind
            .lock()
            .map(|kind| *kind)
            .unwrap_or(BrowserNavigationKind::New)
    }

    fn navigation_is_active(&self) -> bool {
        self.navigation_active
            .load(std::sync::atomic::Ordering::SeqCst)
    }

    fn navigation_started(&self, target: &BrowserNavigationTarget) {
        use std::sync::atomic::Ordering;
        if self.navigation_active.swap(true, Ordering::SeqCst) {
            return;
        }
        let kind = self.current_navigation_kind();
        self.push_state(NativeBrowserEventKind::NavigationStarted {
            display_url: target.display_url().to_owned(),
            navigation_sha256: target.navigation_sha256(),
            kind,
        });
    }

    fn navigation_finished(&self, kind: NativeBrowserEventKind) {
        use std::sync::atomic::Ordering;
        if !self.navigation_active.swap(false, Ordering::SeqCst) {
            return;
        }
        self.push_state(kind);
        self.set_navigation_kind(BrowserNavigationKind::New);
    }

    fn push_state(&self, kind: NativeBrowserEventKind) {
        use std::sync::atomic::Ordering;
        let sequence = self.state_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        self.push(kind, Some(sequence));
    }

    fn push_notice(&self, kind: NativeBrowserEventKind) {
        self.push(kind, None);
    }

    fn push(&self, kind: NativeBrowserEventKind, state_event_sequence: Option<u64>) {
        use std::sync::atomic::Ordering;
        let notice_sequence = self.notice_sequence.fetch_add(1, Ordering::SeqCst) + 1;
        if let Ok(mut queue) = self.queue.lock() {
            queue.push(NativeBrowserEvent {
                artifact_id: self.artifact_id.clone(),
                controller_generation: self.controller_generation,
                mount_generation: self.mount_generation,
                state_event_sequence,
                notice_sequence,
                observed_at_ms: now_ms(),
                kind,
            });
        }
    }
}

#[derive(Debug, Error)]
pub enum NativeBrowserError {
    #[error("native Browser identity is invalid")]
    InvalidIdentity,
    #[error("native Browser geometry is invalid")]
    InvalidGeometry,
    #[error("native Browser profile identifier is invalid")]
    InvalidProfileIdentifier,
    #[error("native Browser surface is unavailable")]
    SurfaceUnavailable,
    #[error("native Browser host is unavailable")]
    HostUnavailable,
    #[error("native Browser main-thread dispatch failed")]
    MainThreadDispatch,
    #[error("native Browser page context capture is unavailable")]
    ContextCaptureUnavailable,
    #[error("native Browser clear watchdog is unavailable")]
    ClearWatchdogUnavailable,
    #[error("native Browser is not supported by this target")]
    UnsupportedTarget,
}

fn validate_identity(
    artifact_id: &str,
    controller_generation: u64,
    mount_generation: u64,
) -> Result<(), NativeBrowserError> {
    if artifact_id.is_empty()
        || artifact_id.len() > crate::artifact::MAX_BROWSER_REFERENCE_ID_BYTES
        || !artifact_id.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@')
        })
        || controller_generation == 0
        || mount_generation == 0
    {
        return Err(NativeBrowserError::InvalidIdentity);
    }
    Ok(())
}

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_millis() as u64)
        .unwrap_or(1)
        .max(1)
}

fn content_disposition_is_attachment(value: &str) -> bool {
    value
        .split(';')
        .next()
        .is_some_and(|disposition| disposition.trim().eq_ignore_ascii_case("attachment"))
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use block2::{Block, RcBlock};
    use objc2::{
        DefinedClass, MainThreadOnly, define_class, msg_send,
        rc::Retained,
        runtime::{AnyObject, Bool, NSObject, ProtocolObject},
    };
    use objc2_app_kit::{NSResponder, NSView, NSWindowOrderingMode};
    use objc2_foundation::{
        MainThreadMarker, NSError, NSHTTPURLResponse, NSObjectProtocol, NSPoint, NSRect, NSSize,
        NSString, NSURL, NSURLRequest, NSURLResponse, NSUUID,
    };
    use objc2_web_kit::{
        WKBackForwardListItem, WKContentWorld, WKFrameInfo, WKMediaCaptureType, WKNavigation,
        WKNavigationAction, WKNavigationActionPolicy, WKNavigationDelegate, WKNavigationResponse,
        WKNavigationResponsePolicy, WKPermissionDecision, WKSecurityOrigin, WKUIDelegate,
        WKWebView, WKWebViewConfiguration, WKWebsiteDataRecord, WKWebsiteDataStore,
        WKWindowFeatures,
    };
    use std::cell::{Cell, RefCell};
    use std::collections::HashMap;
    use std::ptr::NonNull;
    use std::rc::Rc;
    use std::sync::atomic::Ordering;
    use tauri::{Manager, WebviewWindow};

    thread_local! {
        static CONTROLLER: RefCell<MacBrowserController> = RefCell::new(MacBrowserController::default());
    }

    struct PendingMainFrameNavigation {
        target: BrowserNavigationTarget,
        effect: PendingMainFrameNavigationEffect,
        created_at_ms: u64,
    }

    enum PendingMainFrameNavigationEffect {
        Request(Retained<NSURLRequest>),
        HistoryItem(Retained<WKBackForwardListItem>),
    }

    #[derive(Default)]
    struct NavigationAuthorityState {
        authorized_once: Option<BrowserNavigationTarget>,
        pending: HashMap<String, PendingMainFrameNavigation>,
        current: Option<BrowserNavigationTarget>,
        committed: Option<BrowserNavigationTarget>,
        suppress_next_failure: bool,
    }

    impl NavigationAuthorityState {
        fn authorize_target(&mut self, target: BrowserNavigationTarget) {
            self.authorized_once = Some(target);
        }

        fn consume_authorization(&mut self, target: &BrowserNavigationTarget) -> bool {
            if self.authorized_once.as_ref() == Some(target) {
                self.authorized_once = None;
                self.current = Some(target.clone());
                true
            } else {
                false
            }
        }

        fn authorization_matches(&self, target: &BrowserNavigationTarget) -> bool {
            self.authorized_once.as_ref() == Some(target)
        }

        fn remember_current(&mut self, target: BrowserNavigationTarget) {
            self.current = Some(target);
        }

        fn remember_committed(&mut self, target: BrowserNavigationTarget) {
            self.current = Some(target.clone());
            self.committed = Some(target);
        }

        fn restore_committed_after_block(&mut self) -> bool {
            let Some(committed) = self.committed.clone() else {
                return false;
            };
            self.current = Some(committed);
            self.suppress_next_failure = true;
            true
        }

        fn consume_suppressed_failure(&mut self) -> bool {
            std::mem::take(&mut self.suppress_next_failure)
        }

        fn enqueue_request(
            &mut self,
            target: BrowserNavigationTarget,
            request: Retained<NSURLRequest>,
        ) -> Option<String> {
            self.enqueue(target, PendingMainFrameNavigationEffect::Request(request))
        }

        fn enqueue_history_item(
            &mut self,
            target: BrowserNavigationTarget,
            item: Retained<WKBackForwardListItem>,
        ) -> Option<String> {
            self.enqueue(target, PendingMainFrameNavigationEffect::HistoryItem(item))
        }

        fn enqueue(
            &mut self,
            target: BrowserNavigationTarget,
            effect: PendingMainFrameNavigationEffect,
        ) -> Option<String> {
            let cutoff = now_ms().saturating_sub(120_000);
            self.pending
                .retain(|_, pending| pending.created_at_ms >= cutoff);
            if self
                .pending
                .values()
                .any(|pending| pending.target == target)
            {
                return None;
            }
            if self.pending.len() == MAX_PENDING_NATIVE_NAVIGATIONS {
                let oldest_key = self.pending.keys().min().cloned();
                if let Some(oldest_key) = oldest_key {
                    self.pending.remove(&oldest_key);
                }
            }
            let request_id = uuid::Uuid::new_v4().as_simple().to_string();
            self.pending.insert(
                request_id.clone(),
                PendingMainFrameNavigation {
                    target,
                    effect,
                    created_at_ms: now_ms(),
                },
            );
            Some(request_id)
        }

        fn authorize_request(
            &mut self,
            request_id: &str,
            expected_navigation_sha256: &str,
        ) -> Option<PendingMainFrameNavigation> {
            let pending = self.pending.remove(request_id)?;
            if pending.created_at_ms < now_ms().saturating_sub(120_000)
                || pending.target.navigation_sha256() != expected_navigation_sha256
            {
                return None;
            }
            self.authorized_once = Some(pending.target.clone());
            Some(pending)
        }
    }

    struct NavigationDelegateIvars {
        sink: EventSink,
        authority: Rc<RefCell<NavigationAuthorityState>>,
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
                let target = navigation_action_target(action);
                let download = unsafe { action.shouldPerformDownload() };
                if download {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::DownloadBlocked {
                            display_url: target
                                .as_ref()
                                .map(|target| target.display_url().to_owned()),
                        });
                    decision_handler.call((WKNavigationActionPolicy::Cancel,));
                    return;
                }
                let Some(target) = target else {
                    decision_handler.call((WKNavigationActionPolicy::Cancel,));
                    return;
                };
                let target_frame = unsafe { action.targetFrame() };
                let Some(target_frame) = target_frame else {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::PopupBlocked {
                            display_url: Some(target.display_url().to_owned()),
                        });
                    decision_handler.call((WKNavigationActionPolicy::Cancel,));
                    return;
                };
                if !unsafe { target_frame.isMainFrame() } {
                    // Ordinary site subframes stay inside WebKit's sandbox. Only
                    // top-level Browser artifact navigation is an app effect.
                    decision_handler.call((WKNavigationActionPolicy::Allow,));
                    return;
                }
                let request = unsafe { action.request() };
                if !navigation_request_is_get(&request) {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::FormSubmissionBlocked {
                            display_url: target.display_url().to_owned(),
                        });
                    decision_handler.call((WKNavigationActionPolicy::Cancel,));
                    return;
                }
                if self
                    .ivars()
                    .authority
                    .borrow_mut()
                    .consume_authorization(&target)
                {
                    decision_handler.call((WKNavigationActionPolicy::Allow,));
                    return;
                }
                let request_id = self
                    .ivars()
                    .authority
                    .borrow_mut()
                    .enqueue_request(target.clone(), request);
                if let Some(request_id) = request_id {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::NavigationRequested {
                            request_id,
                            display_url: target.display_url().to_owned(),
                            navigation_sha256: target.navigation_sha256(),
                        });
                }
                if self.ivars().sink.navigation_is_active()
                    && self
                        .ivars()
                        .authority
                        .borrow_mut()
                        .restore_committed_after_block()
                {
                    self.ivars()
                        .sink
                        .navigation_finished(NativeBrowserEventKind::NavigationBlocked);
                }
                decision_handler.call((WKNavigationActionPolicy::Cancel,));
            }

            #[unsafe(method(webView:shouldGoToBackForwardListItem:willUseInstantBack:completionHandler:))]
            fn should_go_to_history_item(
                &self,
                _webview: &WKWebView,
                item: &WKBackForwardListItem,
                _will_use_instant_back: bool,
                completion_handler: &Block<dyn Fn(Bool)>,
            ) {
                let Some(target) = back_forward_item_target(item) else {
                    completion_handler.call((Bool::NO,));
                    return;
                };
                if self
                    .ivars()
                    .authority
                    .borrow()
                    .authorization_matches(&target)
                {
                    // WebKit follows this history-item preflight with the
                    // ordinary navigation-action policy callback. Preserve the
                    // one-shot authorization for that effect boundary so one
                    // approved Back/Forward operation cannot prompt twice.
                    completion_handler.call((Bool::YES,));
                    return;
                }
                let retained = unsafe { Retained::retain(NonNull::from(item).as_ptr()) };
                let request_id = retained.and_then(|item| {
                    self.ivars()
                        .authority
                        .borrow_mut()
                        .enqueue_history_item(target.clone(), item)
                });
                if let Some(request_id) = request_id {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::NavigationRequested {
                            request_id,
                            display_url: target.display_url().to_owned(),
                            navigation_sha256: target.navigation_sha256(),
                        });
                }
                completion_handler.call((Bool::NO,));
            }

            #[unsafe(method(webView:decidePolicyForNavigationResponse:decisionHandler:))]
            fn decide_navigation_response(
                &self,
                _webview: &WKWebView,
                navigation_response: &WKNavigationResponse,
                decision_handler: &Block<dyn Fn(WKNavigationResponsePolicy)>,
            ) {
                let response = unsafe { navigation_response.response() };
                let target = response.URL().as_deref().and_then(nsurl_target);
                let unsupported_target = target.is_none();
                let blocked_download = unsafe { !navigation_response.canShowMIMEType() }
                    || response_is_attachment(&response);
                if blocked_download {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::DownloadBlocked {
                            display_url: target
                                .as_ref()
                                .map(|target| target.display_url().to_owned()),
                        });
                }
                if unsupported_target || blocked_download {
                    if unsafe { navigation_response.isForMainFrame() }
                        && self
                            .ivars()
                            .authority
                            .borrow_mut()
                            .restore_committed_after_block()
                    {
                        self.ivars()
                            .sink
                            .navigation_finished(NativeBrowserEventKind::NavigationBlocked);
                    }
                    decision_handler.call((WKNavigationResponsePolicy::Cancel,));
                } else {
                    decision_handler.call((WKNavigationResponsePolicy::Allow,));
                }
            }

            #[unsafe(method(webView:didStartProvisionalNavigation:))]
            fn did_start(&self, webview: &WKWebView, _navigation: Option<&WKNavigation>) {
                let target = current_navigation_target(webview)
                    .or_else(|| self.ivars().authority.borrow().current.clone())
                    .unwrap_or_else(invalid_browser_target);
                self.ivars()
                    .authority
                    .borrow_mut()
                    .remember_current(target.clone());
                self.ivars().sink.navigation_started(&target);
            }

            #[unsafe(method(webView:didReceiveServerRedirectForProvisionalNavigation:))]
            fn did_receive_redirect(
                &self,
                webview: &WKWebView,
                _navigation: Option<&WKNavigation>,
            ) {
                if let Some(target) = current_navigation_target(webview) {
                    self.ivars().authority.borrow_mut().remember_current(target);
                }
            }

            #[unsafe(method(webView:didFinishNavigation:))]
            fn did_finish(&self, webview: &WKWebView, _navigation: Option<&WKNavigation>) {
                let target = current_navigation_target(webview)
                    .or_else(|| self.ivars().authority.borrow().current.clone())
                    .unwrap_or_else(invalid_browser_target);
                self.ivars()
                    .authority
                    .borrow_mut()
                    .remember_committed(target.clone());
                self.ivars()
                    .sink
                    .navigation_finished(NativeBrowserEventKind::NavigationFinished {
                        display_url: target.display_url().to_owned(),
                        navigation_sha256: target.navigation_sha256(),
                        kind: self.ivars().sink.current_navigation_kind(),
                        title: current_title(webview),
                        can_go_back: unsafe { webview.canGoBack() },
                        can_go_forward: unsafe { webview.canGoForward() },
                    });
            }

            #[unsafe(method(webView:didFailProvisionalNavigation:withError:))]
            fn did_fail_provisional(
                &self,
                webview: &WKWebView,
                _navigation: Option<&WKNavigation>,
                _error: &NSError,
            ) {
                if self
                    .ivars()
                    .authority
                    .borrow_mut()
                    .consume_suppressed_failure()
                {
                    return;
                }
                self.ivars()
                    .sink
                    .navigation_finished(NativeBrowserEventKind::NavigationFailed {
                        display_url: current_display_url(webview),
                    });
            }

            #[unsafe(method(webView:didFailNavigation:withError:))]
            fn did_fail(
                &self,
                webview: &WKWebView,
                _navigation: Option<&WKNavigation>,
                _error: &NSError,
            ) {
                if self
                    .ivars()
                    .authority
                    .borrow_mut()
                    .consume_suppressed_failure()
                {
                    return;
                }
                self.ivars()
                    .sink
                    .navigation_finished(NativeBrowserEventKind::NavigationFailed {
                        display_url: current_display_url(webview),
                    });
            }

            #[unsafe(method(webViewWebContentProcessDidTerminate:))]
            fn process_terminated(&self, webview: &WKWebView) {
                self.ivars()
                    .sink
                    .navigation_active
                    .store(false, Ordering::SeqCst);
                self.ivars()
                    .sink
                    .push_state(NativeBrowserEventKind::WebContentProcessTerminated {
                        display_url: current_display_url(webview),
                    });
            }
        }
    );

    impl NavigationDelegate {
        fn new(
            mtm: MainThreadMarker,
            sink: EventSink,
            authority: Rc<RefCell<NavigationAuthorityState>>,
        ) -> Retained<Self> {
            let delegate = mtm
                .alloc::<Self>()
                .set_ivars(NavigationDelegateIvars { sink, authority });
            unsafe { msg_send![super(delegate), init] }
        }
    }

    struct UiDelegateIvars {
        sink: EventSink,
        media_permission_policy: NativeBrowserPermissionPolicy,
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
                let display_url =
                    navigation_action_target(action).map(|target| target.display_url().to_owned());
                self.ivars()
                    .sink
                    .push_notice(NativeBrowserEventKind::PopupBlocked { display_url });
                None
            }

            #[unsafe(method(webView:requestMediaCapturePermissionForOrigin:initiatedByFrame:type:decisionHandler:))]
            fn media_permission(
                &self,
                _webview: &WKWebView,
                origin: &WKSecurityOrigin,
                _frame: &WKFrameInfo,
                capture_type: WKMediaCaptureType,
                decision_handler: &Block<dyn Fn(WKPermissionDecision)>,
            ) {
                let permission = if capture_type == WKMediaCaptureType::Camera {
                    "Camera"
                } else if capture_type == WKMediaCaptureType::Microphone {
                    "Microphone"
                } else {
                    "Camera and microphone"
                };
                let origin = security_origin(origin);
                if self.ivars().media_permission_policy == NativeBrowserPermissionPolicy::Deny {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::MediaPermissionDenied {
                            origin,
                            permission: permission.into(),
                        });
                    decision_handler.call((WKPermissionDecision::Deny,));
                } else {
                    self.ivars()
                        .sink
                        .push_notice(NativeBrowserEventKind::MediaPermissionPrompt {
                            origin,
                            permission: permission.into(),
                        });
                    // Preserve WebKit/macOS default mediation; never translate
                    // a C4OS Allow label into an unconditional device grant.
                    decision_handler.call((WKPermissionDecision::Prompt,));
                }
            }
        }
    );

    impl UiDelegate {
        fn new(
            mtm: MainThreadMarker,
            sink: EventSink,
            media_permission_policy: NativeBrowserPermissionPolicy,
        ) -> Retained<Self> {
            let delegate = mtm.alloc::<Self>().set_ivars(UiDelegateIvars {
                sink,
                media_permission_policy,
            });
            unsafe { msg_send![super(delegate), init] }
        }
    }

    struct ActiveSurface {
        identity: NativeBrowserIdentity,
        sink: EventSink,
        authority: Rc<RefCell<NavigationAuthorityState>>,
        data_store: NativeBrowserDataStore,
        webview: Retained<WKWebView>,
        capture_active: Rc<Cell<bool>>,
        capture_in_flight: Rc<Cell<bool>>,
        _navigation_delegate: Retained<NavigationDelegate>,
        _ui_delegate: Retained<UiDelegate>,
        _store: Retained<WKWebsiteDataStore>,
    }

    struct NativeBrowserRecoveryTarget {
        target: BrowserNavigationTarget,
        data_store: NativeBrowserDataStore,
    }

    #[derive(Default)]
    struct MacBrowserController {
        active: Option<ActiveSurface>,
        stores: HashMap<String, Retained<WKWebsiteDataStore>>,
        ephemeral_stores: HashMap<String, Retained<WKWebsiteDataStore>>,
        recovery_targets: HashMap<String, NativeBrowserRecoveryTarget>,
    }

    impl MacBrowserController {
        fn mount(
            &mut self,
            host: WebviewWindow,
            queue: Arc<Mutex<NativeBrowserEventQueue>>,
            request: NativeBrowserMountRequest,
        ) -> Result<(), NativeBrowserError> {
            request.validate()?;
            let identity = NativeBrowserIdentity {
                artifact_id: request.artifact_id.clone(),
                controller_generation: request.controller_generation,
                mount_generation: request.mount_generation,
            };
            if self.active.as_ref().is_some_and(|active| {
                active.identity.artifact_id == identity.artifact_id
                    && active.identity.controller_generation == identity.controller_generation
                    && active.identity.mount_generation == identity.mount_generation
            }) {
                let surface = self.active.as_ref().expect("active Browser checked above");
                set_frame(&host, &surface.webview, request.rect)?;
                if request.focus {
                    focus(&host, &surface.webview)?;
                }
                return Ok(());
            }

            let recovery_target = if request.recover_existing_target {
                Some(
                    self.recovery_targets
                        .get(&request.artifact_id)
                        .filter(|retained| retained.data_store == request.data_store)
                        .map(|retained| retained.target.clone())
                        .ok_or(NativeBrowserError::SurfaceUnavailable)?,
                )
            } else {
                None
            };
            self.detach();
            let mtm = MainThreadMarker::new().ok_or(NativeBrowserError::MainThreadDispatch)?;
            let store = match &request.data_store {
                NativeBrowserDataStore::Persistent { profile_id } => {
                    if let Some(store) = self.stores.get(profile_id) {
                        store.clone()
                    } else {
                        let uuid = uuid::Uuid::parse_str(profile_id)
                            .map_err(|_| NativeBrowserError::InvalidProfileIdentifier)?;
                        let ns_uuid = NSUUID::from_bytes(*uuid.as_bytes());
                        let store =
                            unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&ns_uuid, mtm) };
                        self.stores.insert(profile_id.clone(), store.clone());
                        store
                    }
                }
                NativeBrowserDataStore::Ephemeral => {
                    if let Some(store) = self.ephemeral_stores.get(&request.artifact_id) {
                        store.clone()
                    } else {
                        let store = unsafe { WKWebsiteDataStore::nonPersistentDataStore(mtm) };
                        self.ephemeral_stores
                            .insert(request.artifact_id.clone(), store.clone());
                        store
                    }
                }
            };
            let parent = host_view(&host)?;
            let frame = native_rect(parent, request.rect)?;
            let configuration = unsafe { WKWebViewConfiguration::new(mtm) };
            unsafe { configuration.setWebsiteDataStore(&store) };
            let webview = unsafe {
                WKWebView::initWithFrame_configuration(
                    mtm.alloc::<WKWebView>(),
                    frame,
                    &configuration,
                )
            };
            let sink = EventSink::new(queue, &identity);
            let authority = Rc::new(RefCell::new(NavigationAuthorityState::default()));
            let navigation_delegate =
                NavigationDelegate::new(mtm, sink.clone(), Rc::clone(&authority));
            let ui_delegate = UiDelegate::new(mtm, sink.clone(), request.media_permission_policy);
            unsafe {
                webview
                    .setNavigationDelegate(Some(ProtocolObject::from_ref(&*navigation_delegate)));
                webview.setUIDelegate(Some(ProtocolObject::from_ref(&*ui_delegate)));
            }
            parent.addSubview_positioned_relativeTo(&webview, NSWindowOrderingMode::Above, None);
            if request.focus {
                focus(&host, &webview)?;
            }
            sink.push_notice(NativeBrowserEventKind::Attached);
            self.active = Some(ActiveSurface {
                identity,
                sink: sink.clone(),
                authority: Rc::clone(&authority),
                data_store: request.data_store,
                webview,
                capture_active: Rc::new(Cell::new(true)),
                capture_in_flight: Rc::new(Cell::new(false)),
                _navigation_delegate: navigation_delegate,
                _ui_delegate: ui_delegate,
                _store: store,
            });
            if let Some(target) = request.initial_target.or(recovery_target) {
                sink.set_navigation_kind(request.initial_kind);
                authority.borrow_mut().authorize_target(target.clone());
                load_target(
                    &self.active.as_ref().expect("surface was installed").webview,
                    &target,
                )?;
            }
            Ok(())
        }

        fn capture_context(
            &mut self,
            identity: &NativeBrowserIdentity,
            expected_navigation_sha256: &str,
            sender: std::sync::mpsc::SyncSender<
                Result<NativeBrowserPageContext, NativeBrowserError>,
            >,
        ) -> Result<(), NativeBrowserError> {
            let active = self.require_active(identity)?;
            if active.sink.navigation_active.load(Ordering::SeqCst)
                || active.capture_in_flight.replace(true)
            {
                return Err(NativeBrowserError::ContextCaptureUnavailable);
            }
            let current_matches = active
                .authority
                .borrow()
                .current
                .as_ref()
                .is_some_and(|target| target.navigation_sha256() == expected_navigation_sha256);
            if !current_matches {
                active.capture_in_flight.set(false);
                return Err(NativeBrowserError::ContextCaptureUnavailable);
            }
            let mtm = MainThreadMarker::new().ok_or_else(|| {
                active.capture_in_flight.set(false);
                NativeBrowserError::MainThreadDispatch
            })?;
            let capture_active = Rc::clone(&active.capture_active);
            let capture_in_flight = Rc::clone(&active.capture_in_flight);
            let authority = Rc::clone(&active.authority);
            let expected_navigation_sha256 = expected_navigation_sha256.to_owned();
            let completion = RcBlock::new(move |result: *mut AnyObject, error: *mut NSError| {
                capture_in_flight.set(false);
                let captured = (|| {
                    if !capture_active.get() || !error.is_null() {
                        return Err(NativeBrowserError::ContextCaptureUnavailable);
                    }
                    let target_matches =
                        authority.borrow().current.as_ref().is_some_and(|target| {
                            target.navigation_sha256() == expected_navigation_sha256
                        });
                    if !target_matches {
                        return Err(NativeBrowserError::ContextCaptureUnavailable);
                    }
                    let object = unsafe { result.as_ref() }
                        .ok_or(NativeBrowserError::ContextCaptureUnavailable)?;
                    let document = object
                        .downcast_ref::<NSString>()
                        .ok_or(NativeBrowserError::ContextCaptureUnavailable)?
                        .to_string();
                    parse_native_browser_page_context(&document)
                })();
                let _ = sender.try_send(captured);
            });
            let world = unsafe { WKContentWorld::defaultClientWorld(mtm) };
            let script = NSString::from_str(BROWSER_CONTEXT_CAPTURE_SCRIPT);
            unsafe {
                active
                    .webview
                    .evaluateJavaScript_inFrame_inContentWorld_completionHandler(
                        &script,
                        None,
                        &world,
                        Some(&completion),
                    );
            }
            Ok(())
        }

        fn geometry(
            &mut self,
            host: &WebviewWindow,
            identity: &NativeBrowserIdentity,
            rect: NativeBrowserRect,
        ) -> Result<(), NativeBrowserError> {
            let active = self.require_active(identity)?;
            set_frame(host, &active.webview, rect)
        }

        fn navigate(
            &mut self,
            identity: &NativeBrowserIdentity,
            action: NativeBrowserAction,
        ) -> Result<(), NativeBrowserError> {
            let active = self.require_active(identity)?;
            match action {
                NativeBrowserAction::Back {
                    expected_navigation_sha256,
                } => {
                    let item = unsafe { active.webview.backForwardList().backItem() }
                        .ok_or(NativeBrowserError::SurfaceUnavailable)?;
                    let target = back_forward_item_target(&item)
                        .ok_or(NativeBrowserError::SurfaceUnavailable)?;
                    require_navigation_digest(&target, &expected_navigation_sha256)?;
                    active.authority.borrow_mut().authorize_target(target);
                    active.sink.set_navigation_kind(BrowserNavigationKind::Back);
                    unsafe { active.webview.goBack() };
                }
                NativeBrowserAction::Forward {
                    expected_navigation_sha256,
                } => {
                    let item = unsafe { active.webview.backForwardList().forwardItem() }
                        .ok_or(NativeBrowserError::SurfaceUnavailable)?;
                    let target = back_forward_item_target(&item)
                        .ok_or(NativeBrowserError::SurfaceUnavailable)?;
                    require_navigation_digest(&target, &expected_navigation_sha256)?;
                    active.authority.borrow_mut().authorize_target(target);
                    active
                        .sink
                        .set_navigation_kind(BrowserNavigationKind::Forward);
                    unsafe { active.webview.goForward() };
                }
                NativeBrowserAction::Refresh {
                    expected_navigation_sha256,
                } => {
                    let target = current_navigation_target(&active.webview)
                        .or_else(|| active.authority.borrow().current.clone())
                        .ok_or(NativeBrowserError::SurfaceUnavailable)?;
                    require_navigation_digest(&target, &expected_navigation_sha256)?;
                    active.authority.borrow_mut().authorize_target(target);
                    active
                        .sink
                        .set_navigation_kind(BrowserNavigationKind::Refresh);
                    unsafe { active.webview.reload() };
                }
                NativeBrowserAction::Focus => return Err(NativeBrowserError::HostUnavailable),
                NativeBrowserAction::NavigateTo { target } => {
                    active.sink.set_navigation_kind(BrowserNavigationKind::New);
                    active
                        .authority
                        .borrow_mut()
                        .authorize_target(target.clone());
                    load_target(&active.webview, &target)?;
                }
                NativeBrowserAction::AuthorizeNavigationRequest {
                    request_id,
                    expected_navigation_sha256,
                } => {
                    let pending = active
                        .authority
                        .borrow_mut()
                        .authorize_request(&request_id, &expected_navigation_sha256)
                        .ok_or(NativeBrowserError::SurfaceUnavailable)?;
                    match pending.effect {
                        PendingMainFrameNavigationEffect::Request(request) => {
                            active.sink.set_navigation_kind(BrowserNavigationKind::New);
                            unsafe { active.webview.loadRequest(&request) };
                        }
                        PendingMainFrameNavigationEffect::HistoryItem(item) => {
                            let kind = native_history_item_kind(&active.webview, &item);
                            active.sink.set_navigation_kind(kind);
                            unsafe { active.webview.goToBackForwardListItem(&item) };
                        }
                    }
                }
                NativeBrowserAction::DiscardNavigationRequest { request_id } => {
                    active.authority.borrow_mut().pending.remove(&request_id);
                }
            }
            Ok(())
        }

        fn focus(
            &mut self,
            host: &WebviewWindow,
            identity: &NativeBrowserIdentity,
        ) -> Result<(), NativeBrowserError> {
            let active = self.require_active(identity)?;
            focus(host, &active.webview)
        }

        fn detach_identity(&mut self, identity: &NativeBrowserIdentity) {
            if self.active.as_ref().is_some_and(|active| {
                active.identity.artifact_id == identity.artifact_id
                    && active.identity.controller_generation == identity.controller_generation
                    && active.identity.mount_generation == identity.mount_generation
            }) {
                self.detach();
            }
        }

        fn detach(&mut self) {
            if let Some(active) = self.active.take() {
                active.capture_active.set(false);
                active.capture_in_flight.set(false);
                if let Some(target) = active.authority.borrow().current.clone() {
                    self.recovery_targets.insert(
                        active.identity.artifact_id.clone(),
                        NativeBrowserRecoveryTarget {
                            target,
                            data_store: active.data_store.clone(),
                        },
                    );
                }
                unsafe {
                    active.webview.setNavigationDelegate(None);
                    active.webview.setUIDelegate(None);
                }
                active.webview.removeFromSuperview();
                active.sink.push_notice(NativeBrowserEventKind::Detached);
            }
        }

        fn clear_profile(
            &mut self,
            profile_id: String,
            operation_id: String,
            queue: Arc<Mutex<NativeBrowserEventQueue>>,
        ) -> Result<(), NativeBrowserError> {
            let uuid = uuid::Uuid::parse_str(&profile_id)
                .map_err(|_| NativeBrowserError::InvalidProfileIdentifier)?;
            if self.active.as_ref().is_some_and(|active| {
                matches!(&active.data_store, NativeBrowserDataStore::Persistent { profile_id: active_id } if active_id == &profile_id)
            }) {
                self.detach();
            }
            self.recovery_targets.retain(|_, retained| {
                !matches!(
                    &retained.data_store,
                    NativeBrowserDataStore::Persistent { profile_id: retained_id }
                        if retained_id == &profile_id
                )
            });
            let mtm = MainThreadMarker::new().ok_or(NativeBrowserError::MainThreadDispatch)?;
            let store = self.stores.remove(&profile_id).unwrap_or_else(|| {
                let ns_uuid = NSUUID::from_bytes(*uuid.as_bytes());
                unsafe { WKWebsiteDataStore::dataStoreForIdentifier(&ns_uuid, mtm) }
            });
            let types = unsafe { WKWebsiteDataStore::allWebsiteDataTypes(mtm) };
            let epoch = objc2_foundation::NSDate::dateWithTimeIntervalSince1970(0.0);
            let verification_store = store.clone();
            let verification_types = types.clone();
            let verification_queue = Arc::clone(&queue);
            let profile_for_verification = profile_id.clone();
            let operation_for_verification = operation_id.clone();
            let resolved = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let verification_resolved = Arc::clone(&resolved);
            let verification = block2::RcBlock::new(
                move |records: NonNull<objc2_foundation::NSArray<WKWebsiteDataRecord>>| {
                    let cleared = unsafe { records.as_ref() }.count() == 0;
                    if !verification_resolved.swap(true, Ordering::SeqCst) {
                        push_profile_clear_event(
                            &verification_queue,
                            profile_for_verification.clone(),
                            operation_for_verification.clone(),
                            cleared,
                        );
                    }
                },
            );
            let completion = block2::RcBlock::new(move || unsafe {
                verification_store
                    .fetchDataRecordsOfTypes_completionHandler(&verification_types, &verification);
            });
            unsafe {
                store.removeDataOfTypes_modifiedSince_completionHandler(
                    &types,
                    &epoch,
                    &completion,
                );
            }
            std::thread::Builder::new()
                .name("c4os-browser-clear-watchdog".into())
                .spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(10));
                    if !resolved.swap(true, Ordering::SeqCst) {
                        push_profile_clear_event(&queue, profile_id, operation_id, false);
                    }
                })
                .map_err(|_| NativeBrowserError::ClearWatchdogUnavailable)?;
            Ok(())
        }

        fn destroy_ephemeral(
            &mut self,
            artifact_id: String,
            operation_id: String,
            queue: Arc<Mutex<NativeBrowserEventQueue>>,
        ) -> Result<(), NativeBrowserError> {
            validate_identity(&artifact_id, 1, 1)?;
            uuid::Uuid::parse_str(&operation_id)
                .map_err(|_| NativeBrowserError::InvalidProfileIdentifier)?;
            if self
                .active
                .as_ref()
                .is_some_and(|active| active.identity.artifact_id == artifact_id)
            {
                self.detach();
            }
            self.ephemeral_stores.remove(&artifact_id);
            self.recovery_targets.remove(&artifact_id);
            if let Ok(mut queue) = queue.lock() {
                queue.push(NativeBrowserEvent {
                    artifact_id: artifact_id.clone(),
                    controller_generation: 1,
                    mount_generation: 1,
                    state_event_sequence: None,
                    notice_sequence: 1,
                    observed_at_ms: now_ms(),
                    kind: NativeBrowserEventKind::EphemeralDataCleared {
                        artifact_id,
                        operation_id,
                    },
                });
            }
            Ok(())
        }

        fn require_active(
            &self,
            identity: &NativeBrowserIdentity,
        ) -> Result<&ActiveSurface, NativeBrowserError> {
            self.active
                .as_ref()
                .filter(|active| {
                    active.identity.artifact_id == identity.artifact_id
                        && active.identity.controller_generation == identity.controller_generation
                        && active.identity.mount_generation == identity.mount_generation
                })
                .ok_or(NativeBrowserError::SurfaceUnavailable)
        }
    }

    fn push_profile_clear_event(
        queue: &Arc<Mutex<NativeBrowserEventQueue>>,
        profile_id: String,
        operation_id: String,
        cleared: bool,
    ) {
        if let Ok(mut queue) = queue.lock() {
            queue.push(NativeBrowserEvent {
                artifact_id: "browser-profile-clear".into(),
                controller_generation: 1,
                mount_generation: 1,
                state_event_sequence: None,
                notice_sequence: 1,
                observed_at_ms: now_ms(),
                kind: if cleared {
                    NativeBrowserEventKind::DataCleared {
                        profile_id,
                        operation_id,
                    }
                } else {
                    NativeBrowserEventKind::DataClearFailed {
                        profile_id,
                        operation_id,
                    }
                },
            });
        }
    }

    fn host_view(host: &WebviewWindow) -> Result<&NSView, NativeBrowserError> {
        let pointer = host
            .ns_view()
            .map_err(|_| NativeBrowserError::HostUnavailable)?;
        Ok(unsafe { &*(pointer.cast::<NSView>()) })
    }

    fn native_rect(parent: &NSView, rect: NativeBrowserRect) -> Result<NSRect, NativeBrowserError> {
        rect.validate()?;
        let bounds = parent.bounds();
        let content_top_inset = parent
            .window()
            .map(|window| {
                let content = window.contentLayoutRect();
                (bounds.origin.y + bounds.size.height - content.origin.y - content.size.height)
                    .clamp(0.0, bounds.size.height.max(0.0))
            })
            .unwrap_or(0.0);
        Ok(native_rect_in_bounds(
            rect,
            bounds,
            content_top_inset,
            parent.isFlipped(),
        ))
    }

    fn native_rect_in_bounds(
        rect: NativeBrowserRect,
        bounds: NSRect,
        content_top_inset: f64,
        flipped: bool,
    ) -> NSRect {
        let width = rect.width.min(bounds.size.width.max(0.0));
        let height = rect.height.min(bounds.size.height.max(0.0));
        let x = rect.x.min((bounds.size.width - width).max(0.0));
        let top = (rect.y + content_top_inset).min((bounds.size.height - height).max(0.0));
        let y = if flipped {
            top
        } else {
            (bounds.size.height - top - height).max(0.0)
        };
        NSRect::new(NSPoint::new(x, y), NSSize::new(width, height))
    }

    fn set_frame(
        host: &WebviewWindow,
        webview: &WKWebView,
        rect: NativeBrowserRect,
    ) -> Result<(), NativeBrowserError> {
        let parent = host_view(host)?;
        webview.setFrame(native_rect(parent, rect)?);
        Ok(())
    }

    fn focus(host: &WebviewWindow, webview: &WKWebView) -> Result<(), NativeBrowserError> {
        host.set_focus()
            .map_err(|_| NativeBrowserError::HostUnavailable)?;
        let accepted = webview
            .window()
            .map(|window| window.makeFirstResponder(Some(webview as &NSResponder)))
            .unwrap_or(false);
        if accepted {
            Ok(())
        } else {
            Err(NativeBrowserError::SurfaceUnavailable)
        }
    }

    fn load_target(
        webview: &WKWebView,
        target: &BrowserNavigationTarget,
    ) -> Result<(), NativeBrowserError> {
        let url = NSURL::URLWithString(&NSString::from_str(target.navigation_url_str()))
            .ok_or(NativeBrowserError::SurfaceUnavailable)?;
        let request = NSURLRequest::requestWithURL(&url);
        unsafe { webview.loadRequest(&request) };
        Ok(())
    }

    fn navigation_action_target(action: &WKNavigationAction) -> Option<BrowserNavigationTarget> {
        let request = unsafe { action.request() };
        let url = request.URL()?;
        nsurl_target(&url)
    }

    fn navigation_request_is_get(request: &NSURLRequest) -> bool {
        request
            .HTTPMethod()
            .map(|method| method.to_string().eq_ignore_ascii_case("GET"))
            .unwrap_or(true)
    }

    fn back_forward_item_target(item: &WKBackForwardListItem) -> Option<BrowserNavigationTarget> {
        let url = unsafe { item.URL() };
        nsurl_target(&url)
    }

    fn native_history_item_kind(
        webview: &WKWebView,
        item: &WKBackForwardListItem,
    ) -> BrowserNavigationKind {
        let Some(target) = back_forward_item_target(item) else {
            return BrowserNavigationKind::New;
        };
        let list = unsafe { webview.backForwardList() };
        if unsafe { list.backItem() }
            .as_deref()
            .and_then(back_forward_item_target)
            .as_ref()
            == Some(&target)
        {
            BrowserNavigationKind::Back
        } else if unsafe { list.forwardItem() }
            .as_deref()
            .and_then(back_forward_item_target)
            .as_ref()
            == Some(&target)
        {
            BrowserNavigationKind::Forward
        } else {
            BrowserNavigationKind::New
        }
    }

    fn nsurl_target(url: &NSURL) -> Option<BrowserNavigationTarget> {
        let absolute = url.absoluteString()?;
        normalize_browser_address(&absolute.to_string()).ok()
    }

    fn response_is_attachment(response: &NSURLResponse) -> bool {
        let object: &AnyObject = response;
        let Some(response) = object.downcast_ref::<NSHTTPURLResponse>() else {
            return false;
        };
        response
            .valueForHTTPHeaderField(&NSString::from_str("Content-Disposition"))
            .is_some_and(|value| content_disposition_is_attachment(&value.to_string()))
    }

    fn current_navigation_target(webview: &WKWebView) -> Option<BrowserNavigationTarget> {
        unsafe { webview.URL() }.and_then(|url| nsurl_target(&url))
    }

    fn invalid_browser_target() -> BrowserNavigationTarget {
        BrowserNavigationTarget::parse("https://invalid.local/")
            .expect("the native Browser fallback target is valid")
    }

    fn require_navigation_digest(
        target: &BrowserNavigationTarget,
        expected_navigation_sha256: &str,
    ) -> Result<(), NativeBrowserError> {
        if target.navigation_sha256() == expected_navigation_sha256 {
            Ok(())
        } else {
            Err(NativeBrowserError::SurfaceUnavailable)
        }
    }

    fn current_display_url(webview: &WKWebView) -> String {
        current_navigation_target(webview)
            .map(|target| target.display_url().to_owned())
            .unwrap_or_else(|| "https://invalid.local/".into())
    }

    fn current_title(webview: &WKWebView) -> Option<String> {
        unsafe { webview.title() }.and_then(|title| {
            let title = title.to_string();
            let title = title.trim();
            (!title.is_empty()
                && title.len() <= MAX_BROWSER_TITLE_BYTES
                && !title.chars().any(char::is_control))
            .then(|| title.to_owned())
        })
    }

    fn security_origin(origin: &WKSecurityOrigin) -> String {
        let scheme = unsafe { origin.protocol() }.to_string();
        let host = unsafe { origin.host() }.to_string();
        let port = unsafe { origin.port() };
        if port > 0 {
            format!("{scheme}://{host}:{port}")
        } else {
            format!("{scheme}://{host}")
        }
    }

    fn report_dispatch_failure(
        queue: &Arc<Mutex<NativeBrowserEventQueue>>,
        identity: &NativeBrowserIdentity,
        message: &'static str,
    ) {
        EventSink::new(Arc::clone(queue), identity)
            .push_notice(NativeBrowserEventKind::ControllerFailed { message });
    }

    pub fn dispatch_mount(
        app: &tauri::AppHandle,
        queue: Arc<Mutex<NativeBrowserEventQueue>>,
        request: NativeBrowserMountRequest,
    ) -> Result<(), NativeBrowserError> {
        request.validate()?;
        let identity = NativeBrowserIdentity {
            artifact_id: request.artifact_id.clone(),
            controller_generation: request.controller_generation,
            mount_generation: request.mount_generation,
        };
        let handle = app.clone();
        let failure_queue = Arc::clone(&queue);
        app.run_on_main_thread(move || {
            let Some(host) = handle.get_webview_window(NATIVE_BROWSER_HOST_LABEL) else {
                report_dispatch_failure(&failure_queue, &identity, "host-unavailable");
                return;
            };
            CONTROLLER.with_borrow_mut(|controller| {
                if controller.mount(host, queue, request).is_err() {
                    report_dispatch_failure(&failure_queue, &identity, "mount-failed");
                }
            });
        })
        .map_err(|_| NativeBrowserError::MainThreadDispatch)
    }

    pub fn dispatch_geometry(
        app: &tauri::AppHandle,
        identity: NativeBrowserIdentity,
        rect: NativeBrowserRect,
    ) -> Result<(), NativeBrowserError> {
        identity.validate()?;
        rect.validate()?;
        let handle = app.clone();
        app.run_on_main_thread(move || {
            let Some(host) = handle.get_webview_window(NATIVE_BROWSER_HOST_LABEL) else {
                return;
            };
            CONTROLLER.with_borrow_mut(|controller| {
                let _ = controller.geometry(&host, &identity, rect);
            });
        })
        .map_err(|_| NativeBrowserError::MainThreadDispatch)
    }

    pub fn dispatch_action(
        app: &tauri::AppHandle,
        queue: Arc<Mutex<NativeBrowserEventQueue>>,
        identity: NativeBrowserIdentity,
        action: NativeBrowserAction,
    ) -> Result<(), NativeBrowserError> {
        identity.validate()?;
        let handle = app.clone();
        app.run_on_main_thread(move || {
            let Some(host) = handle.get_webview_window(NATIVE_BROWSER_HOST_LABEL) else {
                report_dispatch_failure(&queue, &identity, "host-unavailable");
                return;
            };
            CONTROLLER.with_borrow_mut(|controller| {
                let result = if action == NativeBrowserAction::Focus {
                    controller.focus(&host, &identity)
                } else {
                    controller.navigate(&identity, action)
                };
                if result.is_err() {
                    report_dispatch_failure(&queue, &identity, "action-failed");
                }
            });
        })
        .map_err(|_| NativeBrowserError::MainThreadDispatch)
    }

    pub fn capture_context(
        app: &tauri::AppHandle,
        identity: NativeBrowserIdentity,
        expected_navigation_sha256: String,
    ) -> Result<NativeBrowserPageContext, NativeBrowserError> {
        identity.validate()?;
        let digest = expected_navigation_sha256
            .strip_prefix("sha256:")
            .ok_or(NativeBrowserError::InvalidIdentity)?;
        if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(NativeBrowserError::InvalidIdentity);
        }
        // A synchronous Tauri command may wait for the isolated-world callback
        // only when it is not already executing on the AppKit main thread.
        if MainThreadMarker::new().is_some() {
            return Err(NativeBrowserError::MainThreadDispatch);
        }
        let (sender, receiver) = std::sync::mpsc::sync_channel(1);
        let failure_sender = sender.clone();
        app.run_on_main_thread(move || {
            CONTROLLER.with_borrow_mut(|controller| {
                if let Err(error) =
                    controller.capture_context(&identity, &expected_navigation_sha256, sender)
                {
                    let _ = failure_sender.try_send(Err(error));
                }
            });
        })
        .map_err(|_| NativeBrowserError::MainThreadDispatch)?;
        receiver
            .recv_timeout(std::time::Duration::from_secs(2))
            .map_err(|_| NativeBrowserError::ContextCaptureUnavailable)?
    }

    pub fn dispatch_detach(
        app: &tauri::AppHandle,
        identity: NativeBrowserIdentity,
    ) -> Result<(), NativeBrowserError> {
        identity.validate()?;
        app.run_on_main_thread(move || {
            CONTROLLER.with_borrow_mut(|controller| controller.detach_identity(&identity));
        })
        .map_err(|_| NativeBrowserError::MainThreadDispatch)
    }

    pub fn dispatch_clear_profile(
        app: &tauri::AppHandle,
        queue: Arc<Mutex<NativeBrowserEventQueue>>,
        profile_id: String,
        operation_id: String,
    ) -> Result<(), NativeBrowserError> {
        uuid::Uuid::parse_str(&profile_id)
            .map_err(|_| NativeBrowserError::InvalidProfileIdentifier)?;
        uuid::Uuid::parse_str(&operation_id)
            .map_err(|_| NativeBrowserError::InvalidProfileIdentifier)?;
        app.run_on_main_thread(move || {
            CONTROLLER.with_borrow_mut(|controller| {
                let failure_profile = profile_id.clone();
                let failure_operation = operation_id.clone();
                let failure_queue = Arc::clone(&queue);
                if controller
                    .clear_profile(profile_id, operation_id, queue)
                    .is_err()
                {
                    push_profile_clear_event(
                        &failure_queue,
                        failure_profile,
                        failure_operation,
                        false,
                    );
                }
            });
        })
        .map_err(|_| NativeBrowserError::MainThreadDispatch)
    }

    pub fn dispatch_destroy_ephemeral(
        app: &tauri::AppHandle,
        queue: Arc<Mutex<NativeBrowserEventQueue>>,
        artifact_id: String,
        operation_id: String,
    ) -> Result<(), NativeBrowserError> {
        validate_identity(&artifact_id, 1, 1)?;
        uuid::Uuid::parse_str(&operation_id)
            .map_err(|_| NativeBrowserError::InvalidProfileIdentifier)?;
        app.run_on_main_thread(move || {
            CONTROLLER.with_borrow_mut(|controller| {
                let _ = controller.destroy_ephemeral(artifact_id, operation_id, queue);
            });
        })
        .map_err(|_| NativeBrowserError::MainThreadDispatch)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn history_preflight_preserves_one_shot_navigation_authorization() {
            let target = BrowserNavigationTarget::parse("https://example.com/history?private=1")
                .expect("history target");
            let mut authority = NavigationAuthorityState::default();

            authority.authorize_target(target.clone());

            assert!(authority.authorization_matches(&target));
            assert!(authority.authorization_matches(&target));
            assert!(authority.consume_authorization(&target));
            assert!(!authority.authorization_matches(&target));
            assert!(!authority.consume_authorization(&target));

            authority.remember_committed(target.clone());
            authority.remember_current(
                BrowserNavigationTarget::parse("https://example.com/provisional")
                    .expect("provisional target"),
            );
            assert!(authority.restore_committed_after_block());
            assert_eq!(authority.current.as_ref(), Some(&target));
            assert!(authority.consume_suppressed_failure());
            assert!(!authority.consume_suppressed_failure());
        }

        #[test]
        fn decorated_content_inset_offsets_dom_geometry_below_the_titlebar() {
            let bounds = NSRect::new(NSPoint::new(0.0, 0.0), NSSize::new(792.0, 761.0));
            let rect = NativeBrowserRect {
                x: 0.0,
                y: 146.0,
                width: 792.0,
                height: 418.0,
            };

            let frame = native_rect_in_bounds(rect, bounds, 28.0, false);

            assert_eq!(frame.origin.x, 0.0);
            assert_eq!(frame.origin.y, 169.0);
            assert_eq!(frame.size.width, 792.0);
            assert_eq!(frame.size.height, 418.0);
        }
    }
}

#[cfg(target_os = "macos")]
pub use macos::{
    capture_context, dispatch_action, dispatch_clear_profile, dispatch_destroy_ephemeral,
    dispatch_detach, dispatch_geometry, dispatch_mount,
};

#[cfg(not(target_os = "macos"))]
pub fn dispatch_mount(
    _app: &tauri::AppHandle,
    _queue: Arc<Mutex<NativeBrowserEventQueue>>,
    _request: NativeBrowserMountRequest,
) -> Result<(), NativeBrowserError> {
    Err(NativeBrowserError::UnsupportedTarget)
}

#[cfg(not(target_os = "macos"))]
pub fn dispatch_geometry(
    _app: &tauri::AppHandle,
    _identity: NativeBrowserIdentity,
    _rect: NativeBrowserRect,
) -> Result<(), NativeBrowserError> {
    Err(NativeBrowserError::UnsupportedTarget)
}

#[cfg(not(target_os = "macos"))]
pub fn capture_context(
    _app: &tauri::AppHandle,
    _identity: NativeBrowserIdentity,
    _expected_navigation_sha256: String,
) -> Result<NativeBrowserPageContext, NativeBrowserError> {
    Err(NativeBrowserError::UnsupportedTarget)
}

#[cfg(not(target_os = "macos"))]
pub fn dispatch_action(
    _app: &tauri::AppHandle,
    _queue: Arc<Mutex<NativeBrowserEventQueue>>,
    _identity: NativeBrowserIdentity,
    _action: NativeBrowserAction,
) -> Result<(), NativeBrowserError> {
    Err(NativeBrowserError::UnsupportedTarget)
}

#[cfg(not(target_os = "macos"))]
pub fn dispatch_detach(
    _app: &tauri::AppHandle,
    _identity: NativeBrowserIdentity,
) -> Result<(), NativeBrowserError> {
    Err(NativeBrowserError::UnsupportedTarget)
}

#[cfg(not(target_os = "macos"))]
pub fn dispatch_clear_profile(
    _app: &tauri::AppHandle,
    _queue: Arc<Mutex<NativeBrowserEventQueue>>,
    _profile_id: String,
    _operation_id: String,
) -> Result<(), NativeBrowserError> {
    Err(NativeBrowserError::UnsupportedTarget)
}

#[cfg(not(target_os = "macos"))]
pub fn dispatch_destroy_ephemeral(
    _app: &tauri::AppHandle,
    _queue: Arc<Mutex<NativeBrowserEventQueue>>,
    _artifact_id: String,
    _operation_id: String,
) -> Result<(), NativeBrowserError> {
    Err(NativeBrowserError::UnsupportedTarget)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn queued_event(
        artifact_id: &str,
        controller_generation: u64,
        mount_generation: u64,
        sequence: u64,
        durable: bool,
    ) -> NativeBrowserEvent {
        NativeBrowserEvent {
            artifact_id: artifact_id.into(),
            controller_generation,
            mount_generation,
            state_event_sequence: durable.then_some(sequence),
            notice_sequence: sequence,
            observed_at_ms: sequence,
            kind: if durable {
                NativeBrowserEventKind::NavigationFailed {
                    display_url: "https://example.com/".into(),
                }
            } else {
                NativeBrowserEventKind::Attached
            },
        }
    }

    #[test]
    fn geometry_and_identity_reject_nonfinite_or_generationless_input() {
        assert!(
            NativeBrowserRect {
                x: 0.0,
                y: 0.0,
                width: f64::NAN,
                height: 300.0,
            }
            .validate()
            .is_err()
        );
        assert!(
            NativeBrowserIdentity {
                artifact_id: "artifact-browser-1".into(),
                controller_generation: 0,
                mount_generation: 1,
            }
            .validate()
            .is_err()
        );
    }

    #[test]
    fn queue_is_bounded_and_records_drops() {
        let mut queue = NativeBrowserEventQueue::default();
        for sequence in 1..=(MAX_NATIVE_BROWSER_EVENTS as u64 + 3) {
            queue.push(NativeBrowserEvent {
                artifact_id: "artifact-browser-1".into(),
                controller_generation: 1,
                mount_generation: 1,
                state_event_sequence: None,
                notice_sequence: sequence,
                observed_at_ms: sequence,
                kind: NativeBrowserEventKind::Attached,
            });
        }
        assert_eq!(queue.drain().len(), MAX_NATIVE_BROWSER_EVENTS);
        assert_eq!(queue.dropped(), 3);
    }

    #[test]
    fn queue_records_identity_when_a_durable_state_event_is_lost() {
        let mut queue = NativeBrowserEventQueue::default();
        for sequence in 1..=(MAX_NATIVE_BROWSER_EVENTS as u64 + 1) {
            queue.push(NativeBrowserEvent {
                artifact_id: "artifact-browser-overflow".into(),
                controller_generation: 4,
                mount_generation: 2,
                state_event_sequence: Some(sequence),
                notice_sequence: sequence,
                observed_at_ms: sequence,
                kind: NativeBrowserEventKind::NavigationFailed {
                    display_url: "https://example.com/".into(),
                },
            });
        }
        assert_eq!(queue.dropped(), 1);
        assert_eq!(
            queue.take_dropped_state_identities(),
            vec![("artifact-browser-overflow".into(), 4, 2)]
        );
    }

    #[test]
    fn queue_preserves_durable_events_before_notices_and_resets_drop_evidence() {
        let mut mixed = NativeBrowserEventQueue::default();
        mixed.push(queued_event("notice", 1, 1, 1, false));
        for sequence in 2..=MAX_NATIVE_BROWSER_EVENTS as u64 {
            mixed.push(queued_event("durable", 1, 1, sequence, true));
        }
        mixed.push(queued_event("durable", 1, 1, 10_000, true));
        let retained = mixed.drain();
        assert_eq!(retained.len(), MAX_NATIVE_BROWSER_EVENTS);
        assert!(
            retained
                .iter()
                .all(|event| event.state_event_sequence.is_some())
        );
        assert!(
            retained
                .iter()
                .any(|event| event.state_event_sequence == Some(10_000))
        );
        assert_eq!(mixed.take_dropped(), 1);
        assert!(mixed.take_dropped_state_identities().is_empty());

        let mut durable_only = NativeBrowserEventQueue::default();
        for sequence in 1..=MAX_NATIVE_BROWSER_EVENTS as u64 {
            durable_only.push(queued_event("durable", 2, 3, sequence, true));
        }
        durable_only.push(queued_event("notice", 2, 3, 20_000, false));
        let retained = durable_only.drain();
        assert_eq!(retained.len(), MAX_NATIVE_BROWSER_EVENTS);
        assert_eq!(retained[0].state_event_sequence, Some(1));
        assert_eq!(
            retained.last().and_then(|event| event.state_event_sequence),
            Some(MAX_NATIVE_BROWSER_EVENTS as u64)
        );
        assert_eq!(durable_only.take_dropped(), 1);
        assert!(durable_only.take_dropped_state_identities().is_empty());

        for sequence in 1..=MAX_NATIVE_BROWSER_EVENTS as u64 {
            durable_only.push(queued_event("durable-loss", 4, 5, sequence, true));
        }
        durable_only.push(queued_event("durable-loss", 4, 5, 30_000, true));
        let retained = durable_only.drain();
        assert_eq!(retained[0].state_event_sequence, Some(2));
        assert_eq!(
            retained.last().and_then(|event| event.state_event_sequence),
            Some(30_000)
        );
        assert_eq!(durable_only.take_dropped(), 1);
        assert_eq!(
            durable_only.take_dropped_state_identities(),
            vec![("durable-loss".into(), 4, 5)]
        );
        assert_eq!(durable_only.take_dropped(), 0);
        assert!(durable_only.take_dropped_state_identities().is_empty());
    }

    #[test]
    fn attachment_disposition_matching_is_case_insensitive_and_token_exact() {
        for value in [
            "attachment",
            " Attachment ",
            "ATTACHMENT; filename=private.bin",
            "attachment ; filename=private.bin",
        ] {
            assert!(content_disposition_is_attachment(value));
        }
        for value in [
            "inline",
            "inline; filename=page.html",
            "attachmentish; filename=page.html",
            "",
        ] {
            assert!(!content_disposition_is_attachment(value));
        }
    }

    #[test]
    fn page_context_parser_is_strict_bounded_and_marks_source_truncation() {
        let valid = serde_json::json!({
            "schemaVersion": 1,
            "selectedText": " selected ",
            "visibleText": "visible\u{0007} text",
            "extractedContent": "extracted",
            "selectedTruncated": false,
            "visibleTruncated": false,
            "extractedTruncated": true,
            "nodeLimitReached": false,
        })
        .to_string();
        let parsed = parse_native_browser_page_context(&valid).unwrap();
        assert_eq!(parsed.selected_text.as_deref(), Some("selected"));
        assert_eq!(parsed.visible_text.as_deref(), Some("visible  text"));
        assert_eq!(
            parsed.extracted_content.as_deref(),
            Some("extracted\n[Browser source truncated]")
        );

        let unknown = valid.strip_suffix('}').unwrap().to_owned() + ",\"cookie\":\"secret\"}";
        assert!(parse_native_browser_page_context(&unknown).is_err());
        assert!(
            parse_native_browser_page_context(
                &"x".repeat(MAX_NATIVE_BROWSER_CONTEXT_DOCUMENT_BYTES + 1)
            )
            .is_err()
        );
        assert!(!BROWSER_CONTEXT_CAPTURE_SCRIPT.contains("messageHandlers"));
        assert!(!BROWSER_CONTEXT_CAPTURE_SCRIPT.contains("__TAURI__"));
        assert!(!BROWSER_CONTEXT_CAPTURE_SCRIPT.contains("localStorage"));
        assert!(!BROWSER_CONTEXT_CAPTURE_SCRIPT.contains("cookie"));
    }
}

//! Bounded, Rust-owned OpenCode 1.18.3 workspace event subscriber.
//!
//! The pinned generated SDK defines `Event.subscribe` as `GET /event` with an
//! optional `directory` query and a `text/event-stream` response. This module
//! owns the incremental HTTP/SSE decoding and worker lifecycle. Authentication
//! and socket creation remain in the native loopback boundary.

use std::fmt;
use std::io::Read;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, SyncSender, TryRecvError, TrySendError};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::Value;
use thiserror::Error;

use crate::runtime::opencode::OPENCODE_NATIVE_VERSION;

pub const OPENCODE_WORKSPACE_EVENT_PATH: &str = "/event";
const MAX_HTTP_HEADER_BYTES: usize = 32 * 1024;
const MAX_HTTP_HEADERS: usize = 128;
const MAX_EVENT_PATH_BYTES: usize = 4 * 1024;
const MAX_SSE_FRAME_BYTES: usize = 512 * 1024;
const MAX_SSE_FIELDS: usize = 64;
const MAX_PENDING_FRAMES: usize = 4_096;
const MAX_TOTAL_FRAMES: u64 = 4_096;
const MAX_CHUNK_LINE_BYTES: usize = 128;
const MAX_DECODE_BUFFER_BYTES: usize = MAX_SSE_FRAME_BYTES + 64 * 1024;
const MIN_POLL_TIMEOUT: Duration = Duration::from_millis(10);
const MAX_POLL_TIMEOUT: Duration = Duration::from_millis(500);
const MAX_STARTUP_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_IDLE_TIMEOUT: Duration = Duration::from_secs(30);
const MIN_RECONNECT_BACKOFF: Duration = Duration::from_millis(10);
const MAX_RECONNECT_BACKOFF: Duration = Duration::from_secs(2);
const MAX_RECONNECT_ATTEMPTS: u32 = 8;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenCodeEventSubscription {
    pub workspace_directory: String,
    pub native_version: String,
    pub process_generation: u64,
}

impl OpenCodeEventSubscription {
    pub fn validate(&self) -> Result<(), OpenCodeStreamError> {
        if self.native_version != OPENCODE_NATIVE_VERSION
            || self.process_generation == 0
            || self.workspace_directory.is_empty()
            || self.workspace_directory.len() > MAX_EVENT_PATH_BYTES
            || !self.workspace_directory.starts_with('/')
            || self
                .workspace_directory
                .bytes()
                .any(|byte| byte == 0 || byte.is_ascii_control())
        {
            return Err(OpenCodeStreamError::InvalidConfiguration);
        }
        Ok(())
    }

    pub fn request_path(&self) -> Result<String, OpenCodeStreamError> {
        self.validate()?;
        let encoded = percent_encode_query(&self.workspace_directory);
        let path = format!("{OPENCODE_WORKSPACE_EVENT_PATH}?directory={encoded}");
        if path.len() > MAX_EVENT_PATH_BYTES {
            return Err(OpenCodeStreamError::InvalidConfiguration);
        }
        Ok(path)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenCodeStreamBounds {
    pub startup_timeout: Duration,
    pub idle_timeout: Duration,
    pub read_poll_timeout: Duration,
    pub maximum_frame_bytes: usize,
    pub maximum_pending_frames: usize,
    pub maximum_total_frames: u64,
    pub reconnect_backoff: Duration,
    pub maximum_reconnect_attempts: u32,
}

impl Default for OpenCodeStreamBounds {
    fn default() -> Self {
        Self {
            startup_timeout: Duration::from_secs(5),
            idle_timeout: Duration::from_secs(30),
            read_poll_timeout: Duration::from_millis(100),
            maximum_frame_bytes: MAX_SSE_FRAME_BYTES,
            maximum_pending_frames: 256,
            maximum_total_frames: MAX_TOTAL_FRAMES,
            reconnect_backoff: Duration::from_millis(100),
            maximum_reconnect_attempts: 4,
        }
    }
}

impl OpenCodeStreamBounds {
    pub fn validate(&self) -> Result<(), OpenCodeStreamError> {
        if self.startup_timeout.is_zero()
            || self.startup_timeout > MAX_STARTUP_TIMEOUT
            || self.idle_timeout.is_zero()
            || self.idle_timeout > MAX_IDLE_TIMEOUT
            || self.read_poll_timeout < MIN_POLL_TIMEOUT
            || self.read_poll_timeout > MAX_POLL_TIMEOUT
            || self.maximum_frame_bytes == 0
            || self.maximum_frame_bytes > MAX_SSE_FRAME_BYTES
            || self.maximum_pending_frames == 0
            || self.maximum_pending_frames > MAX_PENDING_FRAMES
            || self.maximum_total_frames == 0
            || self.maximum_total_frames > MAX_TOTAL_FRAMES
            || self.reconnect_backoff < MIN_RECONNECT_BACKOFF
            || self.reconnect_backoff > MAX_RECONNECT_BACKOFF
            || self.maximum_reconnect_attempts == 0
            || self.maximum_reconnect_attempts > MAX_RECONNECT_ATTEMPTS
        {
            return Err(OpenCodeStreamError::InvalidConfiguration);
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenCodeStreamFrame {
    pub native_session_id: String,
    pub frame: Vec<u8>,
    pub received_at_ms: u64,
}

/// A connector must bind the request to its preconfigured loopback endpoint,
/// resolve authentication internally, write the exact request, and configure
/// reads to return within `read_poll_timeout` so cancellation remains bounded.
pub trait OpenCodeStreamConnector: Send + 'static {
    type Connection: Read + Send + 'static;

    fn connect(
        &mut self,
        subscription: &OpenCodeEventSubscription,
        read_poll_timeout: Duration,
    ) -> Result<Self::Connection, OpenCodeStreamError>;
}

#[derive(Clone, Debug, Error, Eq, PartialEq)]
pub enum OpenCodeStreamError {
    #[error("OpenCode event stream configuration is invalid")]
    InvalidConfiguration,
    #[error("OpenCode event stream authentication was rejected")]
    AuthenticationRejected,
    #[error("OpenCode event stream is unavailable")]
    Unavailable,
    #[error("OpenCode event stream timed out")]
    Timeout,
    #[error("OpenCode event stream protocol is malformed")]
    Protocol,
    #[error("OpenCode event stream exceeded a fixed bound")]
    BoundExceeded,
    #[error("OpenCode event stream substituted the pinned native version")]
    VersionSubstituted,
    #[error("OpenCode event stream backpressure was exceeded")]
    Backpressure,
    #[error("OpenCode event stream worker is stopped")]
    Stopped,
    #[error("OpenCode event stream reached its bounded connection lifetime")]
    RotationRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum WorkerState {
    Starting,
    Running,
    Reconnecting,
    Failed(OpenCodeStreamError),
    Stopped,
}

/// One bounded worker per workspace/runtime process. It has no renderer-facing
/// API and yields only validated, session-routable frames to Rust core.
pub struct OpenCodeStreamWorker {
    receiver: Receiver<OpenCodeStreamFrame>,
    cancelled: Arc<AtomicBool>,
    state: Arc<Mutex<WorkerState>>,
    handle: Option<JoinHandle<()>>,
    drain_limit: usize,
}

impl OpenCodeStreamWorker {
    pub fn start<C: OpenCodeStreamConnector>(
        connector: C,
        subscription: OpenCodeEventSubscription,
        bounds: OpenCodeStreamBounds,
    ) -> Result<Self, OpenCodeStreamError> {
        subscription.validate()?;
        bounds.validate()?;
        let (sender, receiver) = mpsc::sync_channel(bounds.maximum_pending_frames);
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let cancelled = Arc::new(AtomicBool::new(false));
        let state = Arc::new(Mutex::new(WorkerState::Starting));
        let worker_cancelled = Arc::clone(&cancelled);
        let worker_state = Arc::clone(&state);
        let worker_subscription = subscription.clone();
        let handle = thread::Builder::new()
            .name(format!(
                "opencode-event-g{}",
                subscription.process_generation
            ))
            .spawn(move || {
                run_worker(
                    connector,
                    &worker_subscription,
                    bounds,
                    sender,
                    ready_sender,
                    &worker_cancelled,
                    &worker_state,
                );
            })
            .map_err(|_| OpenCodeStreamError::Unavailable)?;

        let startup = ready_receiver.recv_timeout(bounds.startup_timeout);
        match startup {
            Ok(Ok(())) => Ok(Self {
                receiver,
                cancelled,
                state,
                handle: Some(handle),
                drain_limit: bounds.maximum_pending_frames,
            }),
            Ok(Err(error)) => {
                cancelled.store(true, Ordering::Release);
                let _ = handle.join();
                Err(error)
            }
            Err(_) => {
                cancelled.store(true, Ordering::Release);
                let _ = handle.join();
                Err(OpenCodeStreamError::Timeout)
            }
        }
    }

    pub fn is_ready(&self) -> bool {
        self.state
            .lock()
            .is_ok_and(|state| matches!(*state, WorkerState::Running))
    }

    pub fn drain(&mut self) -> Result<Vec<OpenCodeStreamFrame>, OpenCodeStreamError> {
        self.ensure_running()?;
        let mut frames = Vec::new();
        for _ in 0..self.drain_limit {
            match self.receiver.try_recv() {
                Ok(frame) => frames.push(frame),
                Err(TryRecvError::Empty) => break,
                Err(TryRecvError::Disconnected) => {
                    return Err(self.failure_or_stopped());
                }
            }
        }
        self.ensure_running()?;
        Ok(frames)
    }

    pub fn shutdown(&mut self) -> Result<(), OpenCodeStreamError> {
        self.cancelled.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            handle
                .join()
                .map_err(|_| OpenCodeStreamError::Unavailable)?;
        }
        if let Ok(mut state) = self.state.lock() {
            *state = WorkerState::Stopped;
        }
        Ok(())
    }

    fn ensure_running(&self) -> Result<(), OpenCodeStreamError> {
        match self.state.lock().as_deref() {
            Ok(WorkerState::Running | WorkerState::Reconnecting) => Ok(()),
            Ok(WorkerState::Failed(error)) => Err(error.clone()),
            _ => Err(OpenCodeStreamError::Stopped),
        }
    }

    fn failure_or_stopped(&self) -> OpenCodeStreamError {
        match self.state.lock().as_deref() {
            Ok(WorkerState::Failed(error)) => error.clone(),
            _ => OpenCodeStreamError::Stopped,
        }
    }
}

impl fmt::Debug for OpenCodeStreamWorker {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("OpenCodeStreamWorker")
            .field("ready", &self.is_ready())
            .field("credentials", &"<opaque>")
            .finish()
    }
}

impl Drop for OpenCodeStreamWorker {
    fn drop(&mut self) {
        let _ = self.shutdown();
    }
}

fn run_worker<C: OpenCodeStreamConnector>(
    mut connector: C,
    subscription: &OpenCodeEventSubscription,
    bounds: OpenCodeStreamBounds,
    sender: SyncSender<OpenCodeStreamFrame>,
    ready: SyncSender<Result<(), OpenCodeStreamError>>,
    cancelled: &AtomicBool,
    state: &Mutex<WorkerState>,
) {
    let startup_deadline = Instant::now() + bounds.startup_timeout;
    let mut ready = Some(ready);
    let mut reconnect_attempts = 0u32;
    loop {
        if cancelled.load(Ordering::Acquire) {
            finish_worker(state, &mut ready, OpenCodeStreamError::Stopped, true);
            return;
        }
        if let Err(error) = subscription.validate() {
            finish_worker(state, &mut ready, error, false);
            return;
        }

        let mut connection = match connector.connect(subscription, bounds.read_poll_timeout) {
            Ok(connection) => connection,
            Err(error) => {
                if schedule_reconnect(
                    error.clone(),
                    &mut reconnect_attempts,
                    ready.is_some(),
                    startup_deadline,
                    bounds,
                    cancelled,
                    state,
                ) {
                    continue;
                }
                finish_worker(state, &mut ready, error, false);
                return;
            }
        };
        let header_deadline = if ready.is_some() {
            startup_deadline
        } else {
            Instant::now() + bounds.startup_timeout
        };
        let (body_mode, initial) = match read_stream_head(
            &mut connection,
            header_deadline,
            cancelled,
            bounds.maximum_frame_bytes,
        ) {
            Ok(value) => value,
            Err(error) => {
                if schedule_reconnect(
                    error.clone(),
                    &mut reconnect_attempts,
                    ready.is_some(),
                    startup_deadline,
                    bounds,
                    cancelled,
                    state,
                ) {
                    continue;
                }
                finish_worker(state, &mut ready, error, false);
                return;
            }
        };

        if let Ok(mut state) = state.lock() {
            *state = WorkerState::Running;
        }
        if let Some(ready) = ready.take() {
            let _ = ready.send(Ok(()));
        }

        let mut cycle_frames = 0u64;
        let mut cycle = StreamCycle {
            subscription,
            bounds,
            sender: &sender,
            cancelled,
            total_frames: &mut cycle_frames,
        };
        let result = stream_frames(&mut connection, body_mode, initial, &mut cycle);
        if cancelled.load(Ordering::Acquire) {
            finish_worker(state, &mut ready, OpenCodeStreamError::Stopped, true);
            return;
        }
        if cycle_frames != 0 {
            reconnect_attempts = 0;
        }
        let error = result.err().unwrap_or(OpenCodeStreamError::Unavailable);
        if schedule_reconnect(
            error.clone(),
            &mut reconnect_attempts,
            false,
            startup_deadline,
            bounds,
            cancelled,
            state,
        ) {
            continue;
        }
        finish_worker(state, &mut ready, error, false);
        return;
    }
}

fn finish_worker(
    state: &Mutex<WorkerState>,
    ready: &mut Option<SyncSender<Result<(), OpenCodeStreamError>>>,
    error: OpenCodeStreamError,
    stopped: bool,
) {
    if let Ok(mut state) = state.lock() {
        *state = if stopped {
            WorkerState::Stopped
        } else {
            WorkerState::Failed(error.clone())
        };
    }
    if let Some(ready) = ready.take() {
        let _ = ready.send(Err(error));
    }
}

#[allow(clippy::too_many_arguments)]
fn schedule_reconnect(
    error: OpenCodeStreamError,
    reconnect_attempts: &mut u32,
    starting: bool,
    startup_deadline: Instant,
    bounds: OpenCodeStreamBounds,
    cancelled: &AtomicBool,
    state: &Mutex<WorkerState>,
) -> bool {
    if !matches!(
        error,
        OpenCodeStreamError::Unavailable
            | OpenCodeStreamError::Timeout
            | OpenCodeStreamError::RotationRequired
    ) || *reconnect_attempts >= bounds.maximum_reconnect_attempts
        || (starting && Instant::now() >= startup_deadline)
    {
        return false;
    }
    *reconnect_attempts += 1;
    if let Ok(mut state) = state.lock() {
        *state = WorkerState::Reconnecting;
    }
    let exponent = reconnect_attempts.saturating_sub(1).min(4);
    let multiplier = 1u32 << exponent;
    let delay = bounds
        .reconnect_backoff
        .saturating_mul(multiplier)
        .min(MAX_RECONNECT_BACKOFF);
    wait_for_reconnect(cancelled, delay) && (!starting || Instant::now() < startup_deadline)
}

fn wait_for_reconnect(cancelled: &AtomicBool, delay: Duration) -> bool {
    let deadline = Instant::now() + delay;
    while Instant::now() < deadline {
        if cancelled.load(Ordering::Acquire) {
            return false;
        }
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        thread::sleep(remaining.min(Duration::from_millis(10)));
    }
    !cancelled.load(Ordering::Acquire)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum HttpBodyMode {
    Identity,
    Chunked,
}

fn read_stream_head<R: Read>(
    connection: &mut R,
    deadline: Instant,
    cancelled: &AtomicBool,
    maximum_frame_bytes: usize,
) -> Result<(HttpBodyMode, Vec<u8>), OpenCodeStreamError> {
    let mut wire = Vec::new();
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        if cancelled.load(Ordering::Acquire) {
            return Err(OpenCodeStreamError::Stopped);
        }
        if let Some(header_end) = find_bytes(&wire, b"\r\n\r\n") {
            if header_end > MAX_HTTP_HEADER_BYTES {
                return Err(OpenCodeStreamError::BoundExceeded);
            }
            let mode = validate_stream_head(&wire[..header_end])?;
            let initial = wire[header_end + 4..].to_vec();
            if initial.len() > maximum_frame_bytes + 64 * 1024 {
                return Err(OpenCodeStreamError::BoundExceeded);
            }
            return Ok((mode, initial));
        }
        if wire.len() >= MAX_HTTP_HEADER_BYTES || Instant::now() >= deadline {
            return Err(OpenCodeStreamError::Timeout);
        }
        match connection.read(&mut buffer) {
            Ok(0) => return Err(OpenCodeStreamError::Unavailable),
            Ok(read) => {
                if wire.len().saturating_add(read) > MAX_DECODE_BUFFER_BYTES {
                    return Err(OpenCodeStreamError::BoundExceeded);
                }
                wire.extend_from_slice(&buffer[..read]);
            }
            Err(error) if retryable_io(&error) => continue,
            Err(_) => return Err(OpenCodeStreamError::Unavailable),
        }
    }
}

fn validate_stream_head(header: &[u8]) -> Result<HttpBodyMode, OpenCodeStreamError> {
    let header = std::str::from_utf8(header).map_err(|_| OpenCodeStreamError::Protocol)?;
    let mut lines = header.split("\r\n");
    let status = lines.next().ok_or(OpenCodeStreamError::Protocol)?;
    if !status.starts_with("HTTP/1.1 ") {
        return Err(OpenCodeStreamError::Protocol);
    }
    let code = status
        .split_ascii_whitespace()
        .nth(1)
        .and_then(|value| value.parse::<u16>().ok())
        .ok_or(OpenCodeStreamError::Protocol)?;
    if matches!(code, 401 | 403) {
        return Err(OpenCodeStreamError::AuthenticationRejected);
    }
    if code != 200 {
        return Err(OpenCodeStreamError::Unavailable);
    }

    let mut content_type = None;
    let mut transfer_encoding = None;
    let mut content_length = None;
    let mut content_encoding = None;
    let mut count = 0usize;
    for line in lines {
        count += 1;
        if count > MAX_HTTP_HEADERS || line.starts_with([' ', '\t']) {
            return Err(OpenCodeStreamError::BoundExceeded);
        }
        let (name, value) = line.split_once(':').ok_or(OpenCodeStreamError::Protocol)?;
        let name = name.to_ascii_lowercase();
        let value = value.trim();
        if !name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
            || value
                .bytes()
                .any(|byte| !byte.is_ascii() || byte.is_ascii_control())
        {
            return Err(OpenCodeStreamError::Protocol);
        }
        let destination = match name.as_str() {
            "content-type" => &mut content_type,
            "transfer-encoding" => &mut transfer_encoding,
            "content-length" => &mut content_length,
            "content-encoding" => &mut content_encoding,
            _ => continue,
        };
        if destination.replace(value.to_owned()).is_some() {
            return Err(OpenCodeStreamError::Protocol);
        }
    }
    if !content_type.is_some_and(|value| {
        value
            .split(';')
            .next()
            .is_some_and(|media| media.trim().eq_ignore_ascii_case("text/event-stream"))
    }) || content_length.is_some()
        || content_encoding.is_some_and(|value| !value.eq_ignore_ascii_case("identity"))
    {
        return Err(OpenCodeStreamError::Protocol);
    }
    match transfer_encoding {
        None => Ok(HttpBodyMode::Identity),
        Some(value) if value.eq_ignore_ascii_case("chunked") => Ok(HttpBodyMode::Chunked),
        Some(_) => Err(OpenCodeStreamError::Protocol),
    }
}

struct StreamCycle<'a> {
    subscription: &'a OpenCodeEventSubscription,
    bounds: OpenCodeStreamBounds,
    sender: &'a SyncSender<OpenCodeStreamFrame>,
    cancelled: &'a AtomicBool,
    total_frames: &'a mut u64,
}

fn stream_frames<R: Read>(
    connection: &mut R,
    body_mode: HttpBodyMode,
    initial: Vec<u8>,
    cycle: &mut StreamCycle<'_>,
) -> Result<(), OpenCodeStreamError> {
    let mut chunked = (body_mode == HttpBodyMode::Chunked).then(ChunkDecoder::default);
    let mut sse = SseDecoder::new(cycle.bounds.maximum_frame_bytes);
    let mut last_read = Instant::now();
    let mut buffer = [0_u8; 8 * 1024];
    process_body_bytes(initial, chunked.as_mut(), &mut sse, cycle)?;

    loop {
        if cycle.cancelled.load(Ordering::Acquire) {
            return Ok(());
        }
        if Instant::now().duration_since(last_read) >= cycle.bounds.idle_timeout {
            return Err(OpenCodeStreamError::Timeout);
        }
        match connection.read(&mut buffer) {
            Ok(0) => return Err(OpenCodeStreamError::Unavailable),
            Ok(read) => {
                last_read = Instant::now();
                process_body_bytes(buffer[..read].to_vec(), chunked.as_mut(), &mut sse, cycle)?;
            }
            Err(error) if retryable_io(&error) => continue,
            Err(_) => return Err(OpenCodeStreamError::Unavailable),
        }
    }
}

fn process_body_bytes(
    input: Vec<u8>,
    chunked: Option<&mut ChunkDecoder>,
    sse: &mut SseDecoder,
    cycle: &mut StreamCycle<'_>,
) -> Result<(), OpenCodeStreamError> {
    let decoded = if let Some(chunked) = chunked {
        chunked.push(&input)?
    } else {
        input
    };
    for frame in sse.push(&decoded)? {
        *cycle.total_frames = cycle
            .total_frames
            .checked_add(1)
            .ok_or(OpenCodeStreamError::BoundExceeded)?;
        let Some(native_session_id) = route_frame(&frame, cycle.subscription)? else {
            if *cycle.total_frames == cycle.bounds.maximum_total_frames {
                return Err(OpenCodeStreamError::RotationRequired);
            }
            continue;
        };
        let event = OpenCodeStreamFrame {
            native_session_id,
            frame,
            received_at_ms: unix_time_ms()?,
        };
        match cycle.sender.try_send(event) {
            Ok(()) => {}
            Err(TrySendError::Full(_)) => return Err(OpenCodeStreamError::Backpressure),
            Err(TrySendError::Disconnected(_)) => return Err(OpenCodeStreamError::Stopped),
        }
        if *cycle.total_frames == cycle.bounds.maximum_total_frames {
            return Err(OpenCodeStreamError::RotationRequired);
        }
        if *cycle.total_frames > cycle.bounds.maximum_total_frames {
            return Err(OpenCodeStreamError::BoundExceeded);
        }
    }
    Ok(())
}

#[derive(Default)]
struct ChunkDecoder {
    pending: Vec<u8>,
    remaining: Option<usize>,
    terminal: bool,
}

impl ChunkDecoder {
    fn push(&mut self, bytes: &[u8]) -> Result<Vec<u8>, OpenCodeStreamError> {
        if self.terminal && !bytes.is_empty() {
            return Err(OpenCodeStreamError::Protocol);
        }
        if self.pending.len().saturating_add(bytes.len()) > MAX_DECODE_BUFFER_BYTES {
            return Err(OpenCodeStreamError::BoundExceeded);
        }
        self.pending.extend_from_slice(bytes);
        let mut decoded = Vec::new();
        loop {
            if self.terminal {
                if self.pending == b"\r\n" {
                    self.pending.clear();
                }
                if !self.pending.is_empty() {
                    return Err(OpenCodeStreamError::Protocol);
                }
                break;
            }
            let length = if let Some(remaining) = self.remaining {
                remaining
            } else {
                let Some(end) = find_bytes(&self.pending, b"\r\n") else {
                    if self.pending.len() > MAX_CHUNK_LINE_BYTES {
                        return Err(OpenCodeStreamError::BoundExceeded);
                    }
                    break;
                };
                if end == 0 || end > MAX_CHUNK_LINE_BYTES {
                    return Err(OpenCodeStreamError::Protocol);
                }
                let line = std::str::from_utf8(&self.pending[..end])
                    .map_err(|_| OpenCodeStreamError::Protocol)?;
                let raw_length = line
                    .split(';')
                    .next()
                    .ok_or(OpenCodeStreamError::Protocol)?;
                let length = usize::from_str_radix(raw_length.trim(), 16)
                    .map_err(|_| OpenCodeStreamError::Protocol)?;
                self.pending.drain(..end + 2);
                if length == 0 {
                    self.terminal = true;
                    continue;
                }
                if length > MAX_DECODE_BUFFER_BYTES {
                    return Err(OpenCodeStreamError::BoundExceeded);
                }
                self.remaining = Some(length);
                length
            };
            if self.pending.len() < length.saturating_add(2) {
                break;
            }
            if &self.pending[length..length + 2] != b"\r\n" {
                return Err(OpenCodeStreamError::Protocol);
            }
            decoded.extend_from_slice(&self.pending[..length]);
            self.pending.drain(..length + 2);
            self.remaining = None;
        }
        Ok(decoded)
    }
}

struct SseDecoder {
    pending: Vec<u8>,
    maximum_frame_bytes: usize,
}

impl SseDecoder {
    fn new(maximum_frame_bytes: usize) -> Self {
        Self {
            pending: Vec::new(),
            maximum_frame_bytes,
        }
    }

    fn push(&mut self, bytes: &[u8]) -> Result<Vec<Vec<u8>>, OpenCodeStreamError> {
        if self.pending.len().saturating_add(bytes.len()) > self.maximum_frame_bytes {
            return Err(OpenCodeStreamError::BoundExceeded);
        }
        self.pending.extend_from_slice(bytes);
        let mut frames = Vec::new();
        while let Some((end, delimiter)) = find_sse_delimiter(&self.pending) {
            if end > self.maximum_frame_bytes {
                return Err(OpenCodeStreamError::BoundExceeded);
            }
            let frame = self.pending[..end].to_vec();
            self.pending.drain(..end + delimiter);
            if frame.iter().all(u8::is_ascii_whitespace) {
                continue;
            }
            frames.push(frame);
        }
        Ok(frames)
    }
}

fn route_frame(
    frame: &[u8],
    subscription: &OpenCodeEventSubscription,
) -> Result<Option<String>, OpenCodeStreamError> {
    let text = std::str::from_utf8(frame).map_err(|_| OpenCodeStreamError::Protocol)?;
    let mut data = None;
    let mut field_count = 0usize;
    for line in text.lines() {
        field_count += 1;
        if field_count > MAX_SSE_FIELDS {
            return Err(OpenCodeStreamError::BoundExceeded);
        }
        if line.starts_with(':') || line.starts_with("event:") || line.starts_with("id:") {
            continue;
        }
        if let Some(value) = line.strip_prefix("data:") {
            if data.replace(value.trim_start()).is_some() {
                return Err(OpenCodeStreamError::Protocol);
            }
        } else if !line.trim().is_empty() {
            return Err(OpenCodeStreamError::Protocol);
        }
    }
    let Some(data) = data else {
        return Ok(None);
    };
    let event: Value = serde_json::from_str(data).map_err(|_| OpenCodeStreamError::Protocol)?;
    let event_type = event
        .get("type")
        .and_then(Value::as_str)
        .ok_or(OpenCodeStreamError::Protocol)?;
    let properties = event
        .get("properties")
        .and_then(Value::as_object)
        .ok_or(OpenCodeStreamError::Protocol)?;

    if matches!(
        event_type,
        "installation.updated" | "installation.update-available"
    ) {
        let version = properties
            .get("version")
            .and_then(Value::as_str)
            .ok_or(OpenCodeStreamError::Protocol)?;
        if version != subscription.native_version {
            return Err(OpenCodeStreamError::VersionSubstituted);
        }
        return Ok(None);
    }

    let native_session_id = properties
        .get("sessionID")
        .and_then(Value::as_str)
        .or_else(|| {
            properties
                .get("part")
                .and_then(Value::as_object)
                .and_then(|part| part.get("sessionID"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            properties
                .get("info")
                .and_then(Value::as_object)
                .and_then(|info| info.get("sessionID"))
                .and_then(Value::as_str)
        })
        .or_else(|| {
            matches!(
                event_type,
                "session.created" | "session.updated" | "session.deleted"
            )
            .then(|| {
                properties
                    .get("info")
                    .and_then(Value::as_object)
                    .and_then(|info| info.get("id"))
                    .and_then(Value::as_str)
            })
            .flatten()
        });
    match native_session_id {
        Some(value) if valid_native_id(value) => Ok(Some(value.to_owned())),
        Some(_) => Err(OpenCodeStreamError::Protocol),
        None => Ok(None),
    }
}

fn valid_native_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 192
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':' | b'@' | b'/')
        })
}

fn percent_encode_query(value: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(value.len());
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~') {
            encoded.push(char::from(byte));
        } else {
            encoded.push('%');
            encoded.push(char::from(HEX[usize::from(byte >> 4)]));
            encoded.push(char::from(HEX[usize::from(byte & 0x0f)]));
        }
    }
    encoded
}

fn find_sse_delimiter(bytes: &[u8]) -> Option<(usize, usize)> {
    let lf = find_bytes(bytes, b"\n\n").map(|index| (index, 2));
    let crlf = find_bytes(bytes, b"\r\n\r\n").map(|index| (index, 4));
    match (lf, crlf) {
        (Some(left), Some(right)) => Some(if left.0 <= right.0 { left } else { right }),
        (Some(value), None) | (None, Some(value)) => Some(value),
        (None, None) => None,
    }
}

fn find_bytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    haystack
        .windows(needle.len())
        .position(|window| window == needle)
}

fn retryable_io(error: &std::io::Error) -> bool {
    matches!(
        error.kind(),
        std::io::ErrorKind::TimedOut
            | std::io::ErrorKind::WouldBlock
            | std::io::ErrorKind::Interrupted
    )
}

fn unix_time_ms() -> Result<u64, OpenCodeStreamError> {
    let milliseconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OpenCodeStreamError::Unavailable)?
        .as_millis();
    u64::try_from(milliseconds)
        .ok()
        .filter(|value| *value != 0)
        .ok_or(OpenCodeStreamError::Unavailable)
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io::{Cursor, Read, Write};
    use std::net::{Ipv4Addr, TcpListener};
    use std::sync::atomic::AtomicUsize;
    use std::sync::mpsc;
    use std::thread;

    use crate::runtime::opencode::{LoopbackEndpoint, RandomSecretReference};
    use crate::runtime::opencode_native::{LoopbackEventStreamConnector, VaultCredentialResolver};
    use crate::security::credentials::CredentialVault;

    use super::*;

    const SECRET: &str = "opencode-stream-secret-0123456789abcdef";

    fn reference() -> RandomSecretReference {
        RandomSecretReference::new("opencode-stream-secret", 256).unwrap()
    }

    fn resolver(reference: &RandomSecretReference) -> VaultCredentialResolver {
        let vault = CredentialVault::session_only().unwrap();
        let credential = vault.store("opencode-stream", SECRET.as_bytes()).unwrap();
        let resolver = VaultCredentialResolver::new(vault);
        resolver.register(reference, credential).unwrap();
        resolver
    }

    fn subscription() -> OpenCodeEventSubscription {
        OpenCodeEventSubscription {
            workspace_directory: "/private/tmp/C4OS Workspace".into(),
            native_version: OPENCODE_NATIVE_VERSION.into(),
            process_generation: 7,
        }
    }

    fn test_bounds() -> OpenCodeStreamBounds {
        OpenCodeStreamBounds {
            startup_timeout: Duration::from_secs(2),
            idle_timeout: Duration::from_secs(2),
            read_poll_timeout: Duration::from_millis(20),
            maximum_frame_bytes: 8 * 1024,
            maximum_pending_frames: 8,
            maximum_total_frames: 8,
            reconnect_backoff: Duration::from_millis(10),
            maximum_reconnect_attempts: 2,
        }
    }

    struct StreamFixture {
        endpoint: LoopbackEndpoint,
        request: Receiver<Vec<u8>>,
        release: SyncSender<()>,
        handle: JoinHandle<()>,
    }

    fn serve_chunked(parts: Vec<Vec<u8>>) -> StreamFixture {
        serve_with(move |stream| {
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream; charset=utf-8\r\nTransfer-Encoding: chunked\r\nConnection: close\r\n\r\n",
                )
                .unwrap();
            for part in parts {
                write!(stream, "{:X}\r\n", part.len()).unwrap();
                stream.write_all(&part).unwrap();
                stream.write_all(b"\r\n").unwrap();
                stream.flush().unwrap();
            }
        })
    }

    fn serve_identity(body: Vec<u8>, content_type: &'static str) -> StreamFixture {
        serve_with(move |stream| {
            write!(
                stream,
                "HTTP/1.1 200 OK\r\nContent-Type: {content_type}\r\nConnection: close\r\n\r\n"
            )
            .unwrap();
            stream.write_all(&body).unwrap();
            stream.flush().unwrap();
        })
    }

    fn serve_with(
        response: impl FnOnce(&mut std::net::TcpStream) + Send + 'static,
    ) -> StreamFixture {
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = listener.local_addr().unwrap();
        let endpoint = LoopbackEndpoint::new(address.ip(), address.port()).unwrap();
        let (request_sender, request) = mpsc::sync_channel(1);
        let (release, release_receiver) = mpsc::sync_channel(1);
        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            stream
                .set_read_timeout(Some(Duration::from_secs(2)))
                .unwrap();
            let mut request_bytes = Vec::new();
            let mut buffer = [0_u8; 1024];
            loop {
                let read = stream.read(&mut buffer).unwrap();
                request_bytes.extend_from_slice(&buffer[..read]);
                if request_bytes.windows(4).any(|window| window == b"\r\n\r\n") {
                    break;
                }
            }
            request_sender.send(request_bytes).unwrap();
            response(&mut stream);
            let _ = release_receiver.recv_timeout(Duration::from_secs(3));
        });
        StreamFixture {
            endpoint,
            request,
            release,
            handle,
        }
    }

    fn connector(
        endpoint: LoopbackEndpoint,
        reference: &RandomSecretReference,
    ) -> LoopbackEventStreamConnector {
        LoopbackEventStreamConnector::new(
            endpoint,
            resolver(reference),
            "opencode".into(),
            reference.clone(),
            Duration::from_secs(2),
        )
        .unwrap()
    }

    fn wait_for_frame(worker: &mut OpenCodeStreamWorker) -> OpenCodeStreamFrame {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let mut frames = worker.drain().unwrap();
            if let Some(frame) = frames.pop() {
                return frame;
            }
            assert!(Instant::now() < deadline, "stream frame timed out");
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn wait_for_failure(worker: &mut OpenCodeStreamWorker) -> OpenCodeStreamError {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            if let Err(error) = worker.drain() {
                return error;
            }
            assert!(Instant::now() < deadline, "stream failure timed out");
            thread::sleep(Duration::from_millis(10));
        }
    }

    struct ScriptedConnection {
        bytes: Cursor<Vec<u8>>,
        stay_open: bool,
    }

    impl ScriptedConnection {
        fn response(frame: Option<&[u8]>, stay_open: bool) -> Self {
            let mut bytes =
                b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nConnection: close\r\n\r\n"
                    .to_vec();
            if let Some(frame) = frame {
                bytes.extend_from_slice(frame);
            }
            Self {
                bytes: Cursor::new(bytes),
                stay_open,
            }
        }
    }

    impl Read for ScriptedConnection {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            let read = self.bytes.read(buffer)?;
            if read != 0 || !self.stay_open {
                return Ok(read);
            }
            thread::sleep(Duration::from_millis(2));
            Err(std::io::Error::from(std::io::ErrorKind::WouldBlock))
        }
    }

    struct ScriptedConnector {
        scripts: VecDeque<Result<ScriptedConnection, OpenCodeStreamError>>,
        connects: Arc<AtomicUsize>,
    }

    impl OpenCodeStreamConnector for ScriptedConnector {
        type Connection = ScriptedConnection;

        fn connect(
            &mut self,
            _subscription: &OpenCodeEventSubscription,
            _read_poll_timeout: Duration,
        ) -> Result<Self::Connection, OpenCodeStreamError> {
            self.connects.fetch_add(1, Ordering::SeqCst);
            self.scripts
                .pop_front()
                .unwrap_or(Err(OpenCodeStreamError::Unavailable))
        }
    }

    #[test]
    fn pinned_subscription_and_incremental_decoders_preserve_exact_sdk_contract() {
        assert_eq!(
            subscription().request_path().unwrap(),
            "/event?directory=%2Fprivate%2Ftmp%2FC4OS%20Workspace"
        );
        let header =
            b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nTransfer-Encoding: chunked";
        assert_eq!(validate_stream_head(header), Ok(HttpBodyMode::Chunked));

        let payload = b"data: {\"type\":\"message.part.updated\",\"properties\":{\"part\":{\"id\":\"part-1\",\"sessionID\":\"native-session-1\",\"messageID\":\"message-1\",\"type\":\"text\",\"text\":\"hi\"},\"delta\":\"hi\"}}\n\n";
        let split = payload.len() / 2;
        let mut encoded = Vec::new();
        for part in [&payload[..split], &payload[split..]] {
            write!(&mut encoded, "{:X}\r\n", part.len()).unwrap();
            encoded.extend_from_slice(part);
            encoded.extend_from_slice(b"\r\n");
        }
        let mut chunks = ChunkDecoder::default();
        let first = chunks.push(&encoded[..23]).unwrap();
        let second = chunks.push(&encoded[23..]).unwrap();
        let mut sse = SseDecoder::new(MAX_SSE_FRAME_BYTES);
        assert!(sse.push(&first).unwrap().is_empty());
        let frames = sse.push(&second).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(
            route_frame(&frames[0], &subscription()).unwrap(),
            Some("native-session-1".into())
        );
    }

    #[test]
    fn malformed_headers_frames_bounds_and_version_substitution_fail_closed_without_io() {
        assert_eq!(
            validate_stream_head(b"HTTP/1.1 200 OK\r\nContent-Type: application/json"),
            Err(OpenCodeStreamError::Protocol)
        );
        assert_eq!(
            route_frame(b"data: not-json", &subscription()),
            Err(OpenCodeStreamError::Protocol)
        );
        assert_eq!(
            route_frame(
                b"data: {\"type\":\"installation.updated\",\"properties\":{\"version\":\"9.9.9\"}}",
                &subscription(),
            ),
            Err(OpenCodeStreamError::VersionSubstituted)
        );
        let mut sse = SseDecoder::new(32);
        assert_eq!(
            sse.push(&[b'x'; 33]),
            Err(OpenCodeStreamError::BoundExceeded)
        );
    }

    #[test]
    fn bounded_reconnect_recovers_initial_unavailability_disconnect_and_lifetime_rotation() {
        let first = br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-1","sessionID":"native-session-1","messageID":"message-1","type":"text","text":"one"},"delta":"one"}}

"#;
        let second = br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-2","sessionID":"native-session-1","messageID":"message-1","type":"text","text":"onetwo"},"delta":"two"}}

"#;
        let connects = Arc::new(AtomicUsize::new(0));
        let connector = ScriptedConnector {
            scripts: VecDeque::from([
                Err(OpenCodeStreamError::Unavailable),
                Ok(ScriptedConnection::response(Some(first), false)),
                Ok(ScriptedConnection::response(Some(second), false)),
                Ok(ScriptedConnection::response(None, true)),
            ]),
            connects: Arc::clone(&connects),
        };
        let mut bounds = test_bounds();
        bounds.maximum_total_frames = 1;
        bounds.maximum_reconnect_attempts = 4;
        let mut worker = OpenCodeStreamWorker::start(connector, subscription(), bounds).unwrap();
        let deadline = Instant::now() + Duration::from_secs(2);
        let mut observed = Vec::new();
        while observed.len() < 2 {
            observed.extend(worker.drain().unwrap());
            assert!(Instant::now() < deadline, "reconnect frames timed out");
            thread::sleep(Duration::from_millis(5));
        }
        assert_eq!(observed[0].frame, first[..first.len() - 2]);
        assert_eq!(observed[1].frame, second[..second.len() - 2]);
        while connects.load(Ordering::SeqCst) < 4 || !worker.is_ready() {
            assert!(Instant::now() < deadline, "stable reconnect timed out");
            thread::sleep(Duration::from_millis(5));
        }
        worker.shutdown().unwrap();
    }

    #[test]
    fn version_substitution_is_terminal_and_never_retried() {
        let substituted =
            br#"data: {"type":"installation.updated","properties":{"version":"9.9.9"}}

"#;
        let connects = Arc::new(AtomicUsize::new(0));
        let connector = ScriptedConnector {
            scripts: VecDeque::from([
                Ok(ScriptedConnection::response(Some(substituted), false)),
                Ok(ScriptedConnection::response(None, true)),
            ]),
            connects: Arc::clone(&connects),
        };
        let mut worker =
            OpenCodeStreamWorker::start(connector, subscription(), test_bounds()).unwrap();
        assert_eq!(
            wait_for_failure(&mut worker),
            OpenCodeStreamError::VersionSubstituted
        );
        assert_eq!(connects.load(Ordering::SeqCst), 1);
    }

    #[test]
    #[ignore = "local-loopback tier: requires permission to bind an authenticated fixture"]
    fn authenticated_chunked_loopback_stream_routes_incremental_frames_and_shuts_down() {
        let mut frame = br#"data: {"type":"message.part.updated","properties":{"part":{"id":"part-1","sessionID":"native-session-1","messageID":"message-1","type":"text","text":"hi"},"delta":"hi"}}"#.to_vec();
        frame.extend_from_slice(b"\n\n");
        let fixture = serve_chunked(vec![frame[..37].to_vec(), frame[37..].to_vec()]);
        let reference = reference();
        let stream_connector = connector(fixture.endpoint.clone(), &reference);
        let connector_debug = format!("{stream_connector:?}");
        let mut worker =
            OpenCodeStreamWorker::start(stream_connector, subscription(), test_bounds()).unwrap();
        let request = String::from_utf8(fixture.request.recv().unwrap()).unwrap();
        assert!(
            request.starts_with(
                "GET /event?directory=%2Fprivate%2Ftmp%2FC4OS%20Workspace HTTP/1.1\r\n"
            )
        );
        assert!(request.contains("\r\nAccept: text/event-stream\r\n"));
        assert!(request.contains("\r\nAuthorization: Basic "));
        assert!(!request.contains(SECRET));
        assert!(!connector_debug.contains(SECRET));
        assert!(!format!("{worker:?}").contains(SECRET));

        let observed = wait_for_frame(&mut worker);
        assert_eq!(observed.native_session_id, "native-session-1");
        assert_eq!(observed.frame, frame[..frame.len() - 2]);
        assert_ne!(observed.received_at_ms, 0);
        worker.shutdown().unwrap();
        fixture.release.send(()).unwrap();
        fixture.handle.join().unwrap();
    }

    #[test]
    #[ignore = "local-loopback tier: requires permission to bind an authenticated fixture"]
    fn malformed_oversized_and_version_substituted_streams_fail_closed() {
        let cases = [
            (
                b"data: not-json\n\n".to_vec(),
                OpenCodeStreamError::Protocol,
                test_bounds(),
            ),
            (
                b"data: {\"type\":\"installation.updated\",\"properties\":{\"version\":\"9.9.9\"}}\n\n".to_vec(),
                OpenCodeStreamError::VersionSubstituted,
                test_bounds(),
            ),
            (
                vec![b'x'; 257],
                OpenCodeStreamError::BoundExceeded,
                OpenCodeStreamBounds {
                    maximum_frame_bytes: 256,
                    ..test_bounds()
                },
            ),
        ];
        for (body, expected, bounds) in cases {
            let fixture = serve_identity(body, "text/event-stream");
            let reference = reference();
            let mut worker = OpenCodeStreamWorker::start(
                connector(fixture.endpoint.clone(), &reference),
                subscription(),
                bounds,
            )
            .unwrap();
            fixture.request.recv().unwrap();
            assert_eq!(wait_for_failure(&mut worker), expected);
            fixture.release.send(()).unwrap();
            fixture.handle.join().unwrap();
        }
    }

    #[test]
    #[ignore = "local-loopback tier: requires permission to bind an authenticated fixture"]
    fn substituted_content_type_and_unmapped_credential_never_form_a_stream() {
        let fixture = serve_identity(Vec::new(), "application/json");
        let mapped_reference = reference();
        let error = OpenCodeStreamWorker::start(
            connector(fixture.endpoint.clone(), &mapped_reference),
            subscription(),
            test_bounds(),
        )
        .unwrap_err();
        assert_eq!(error, OpenCodeStreamError::Protocol);
        fixture.request.recv().unwrap();
        fixture.release.send(()).unwrap();
        fixture.handle.join().unwrap();

        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        listener.set_nonblocking(true).unwrap();
        let address = listener.local_addr().unwrap();
        let endpoint = LoopbackEndpoint::new(address.ip(), address.port()).unwrap();
        let unmapped = reference();
        let resolver = VaultCredentialResolver::new(CredentialVault::session_only().unwrap());
        let connector = LoopbackEventStreamConnector::new(
            endpoint,
            resolver,
            "opencode".into(),
            unmapped,
            Duration::from_millis(50),
        )
        .unwrap();
        assert!(matches!(
            OpenCodeStreamWorker::start(connector, subscription(), test_bounds()),
            Err(OpenCodeStreamError::AuthenticationRejected)
        ));
        assert!(matches!(
            listener.accept(),
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock
        ));
    }
}

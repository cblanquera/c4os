//! Supervised MCP client transports.
//!
//! The SDK is deliberately kept behind this module.  Callers provide only a
//! validated definition and operation-scoped resolved values; no credential is
//! retained in a durable definition or projected back to the renderer.
#![allow(deprecated)]

use std::{
    borrow::Cow,
    collections::{BTreeMap, BTreeSet},
    future::Future,
    net::{IpAddr, SocketAddr},
    path::{Path, PathBuf},
    pin::Pin,
    process::Stdio,
    sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
    },
    time::Duration,
};

use bytes::{Bytes, BytesMut};
use futures::{SinkExt, StreamExt, TryStreamExt, stream::BoxStream};
use http::{HeaderName, HeaderValue, header::WWW_AUTHENTICATE};
use reqwest::header::ACCEPT;
use rmcp::{
    ClientHandler, ServiceExt,
    model::{
        CallToolRequestParams, ClientCapabilities, ClientInfo, ClientJsonRpcMessage, ClientRequest,
        CreateMessageRequestParams, CreateMessageResult, Implementation, PaginatedRequestParams,
        ProtocolVersion, ReadResourceRequestParams, Request, RequestId, SamplingCapability,
        SamplingMessageContentBlock, ServerJsonRpcMessage, ServerResult,
    },
    service::{
        NotificationContext, PeerRequestOptions, RoleClient, RunningService, RxJsonRpcMessage,
        TxJsonRpcMessage,
    },
    transport::{
        StreamableHttpClientTransport, Transport,
        async_rw::JsonRpcMessageCodec,
        common::http_header::{
            EVENT_STREAM_MIME_TYPE, HEADER_LAST_EVENT_ID, HEADER_SESSION_ID, JSON_MIME_TYPE,
        },
        streamable_http_client::{
            AuthRequiredError, InsufficientScopeError, StreamableHttpClient,
            StreamableHttpClientTransportConfig, StreamableHttpError, StreamableHttpPostResponse,
        },
    },
};
use serde_json::{Map, Value};
use sha2::{Digest, Sha256};
use sse_stream::{Error as SseError, Sse, SseStream};
use thiserror::Error;
use tokio::{
    process::{Child, ChildStdin, ChildStdout},
    sync::{Mutex as AsyncMutex, Notify},
};
use tokio_util::codec::{FramedRead, FramedWrite};
use url::{Host, Url};
use zeroize::{Zeroize, Zeroizing};

use super::{
    MAX_MCP_RESOURCES, MAX_MCP_TEXT_BYTES, MAX_MCP_TOOLS, MCP_PROTOCOL_VERSION,
    McpCapabilitySnapshot, McpEnvironmentSource, McpError, McpHeaderSource, McpResourceSnapshot,
    McpToolSnapshot, McpTransportDefinition, McpWorkingDirectory,
};

pub type McpFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;

/// The protocol frame cap is deliberately independent from a server's
/// configured result cap. It leaves bounded JSON-RPC envelope room around the
/// largest accepted result while preventing a hostile peer from forcing an
/// unbounded pre-deserialization allocation.
const MAX_MCP_WIRE_FRAME_BYTES: usize = 17 * 1_024 * 1_024;
const MAX_MCP_HTTP_SESSION_ID_BYTES: usize = 1_024;

/// Operation-scoped values resolved by the Rust authority boundary.
///
/// Values in this structure must never be serialized, logged, or persisted.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct McpSamplingContext {
    pub server_id: String,
    pub authority_id: String,
    pub lifecycle_generation: u64,
    pub definition_sha256: String,
    pub timeout_ms: u64,
    pub max_output_bytes: u64,
}

#[derive(Default)]
pub struct ResolvedMcpLaunch {
    /// Required only when an opt-in sampling broker is installed.
    pub sampling_context: Option<McpSamplingContext>,
    pub c4os_home: PathBuf,
    /// Private writable directory for one supervised worker generation. The
    /// STDIO sandbox denies every other host write target.
    pub scratch_root: PathBuf,
    pub active_project: Option<PathBuf>,
    pub trusted_roots: Vec<PathBuf>,
    pub environment: BTreeMap<String, String>,
    pub bearer: Option<String>,
    pub headers: BTreeMap<String, String>,
}

impl Drop for ResolvedMcpLaunch {
    fn drop(&mut self) {
        if let Some(value) = self.bearer.as_mut() {
            value.zeroize();
        }
        for value in self.environment.values_mut() {
            value.zeroize();
        }
        for value in self.headers.values_mut() {
            value.zeroize();
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct McpHandshakeSnapshot {
    pub protocol_version: String,
    pub server_name: String,
    pub server_version: String,
    pub instructions_present: bool,
    pub capabilities: McpCapabilitySnapshot,
}

#[derive(Clone, Debug, PartialEq)]
pub struct McpRawResult {
    pub value: Value,
    pub is_error: bool,
    pub output_bytes: u64,
}

/// A cooperative cancellation signal. The request method converts it to the
/// protocol's `notifications/cancelled` message before returning.
#[derive(Clone, Default)]
pub struct McpCancellation {
    inner: Arc<McpCancellationInner>,
}

#[derive(Default)]
struct McpCancellationInner {
    cancelled: AtomicBool,
    notify: Notify,
}

impl McpCancellation {
    pub fn cancel(&self) {
        if !self.inner.cancelled.swap(true, Ordering::SeqCst) {
            self.inner.notify.notify_waiters();
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.inner.cancelled.load(Ordering::SeqCst)
    }

    pub async fn cancelled(&self) {
        if self.is_cancelled() {
            return;
        }
        self.inner.notify.notified().await;
    }
}

type BoundedChildReader =
    FramedRead<ChildStdout, JsonRpcMessageCodec<RxJsonRpcMessage<RoleClient>>>;
type BoundedChildWriter =
    FramedWrite<ChildStdin, JsonRpcMessageCodec<TxJsonRpcMessage<RoleClient>>>;

/// STDIO transport whose decoder refuses an over-limit line before serde sees
/// it. The official child helper currently builds the same codec without a
/// maximum, so C4OS owns the bounded process/codec composition directly.
struct BoundedChildTransport {
    reader: BoundedChildReader,
    writer: Arc<AsyncMutex<Option<BoundedChildWriter>>>,
    child: Option<Child>,
    process_group: Option<i32>,
}

impl BoundedChildTransport {
    fn spawn(mut command: tokio::process::Command) -> Result<Self, McpError> {
        let mut child = command
            .spawn()
            .map_err(|error| McpError::Transport(safe_transport_error(&error)))?;
        #[cfg(unix)]
        let process_group = child.id().and_then(|id| i32::try_from(id).ok());
        #[cfg(not(unix))]
        let process_group = None;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| McpError::Transport("supervised process stdout unavailable".into()))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| McpError::Transport("supervised process stdin unavailable".into()))?;
        Ok(Self {
            reader: FramedRead::new(
                stdout,
                JsonRpcMessageCodec::new_with_max_length(MAX_MCP_WIRE_FRAME_BYTES),
            ),
            writer: Arc::new(AsyncMutex::new(Some(FramedWrite::new(
                stdin,
                JsonRpcMessageCodec::new_with_max_length(MAX_MCP_WIRE_FRAME_BYTES),
            )))),
            child: Some(child),
            process_group,
        })
    }

    async fn stop(&mut self) {
        terminate_process_group(self.process_group.take());
        if let Some(mut child) = self.child.take() {
            let _ = child.start_kill();
            let _ = child.wait().await;
        }
    }
}

impl Transport<RoleClient> for BoundedChildTransport {
    type Error = std::io::Error;

    fn send(
        &mut self,
        item: TxJsonRpcMessage<RoleClient>,
    ) -> impl Future<Output = Result<(), Self::Error>> + Send + 'static {
        let writer = Arc::clone(&self.writer);
        async move {
            let encoded = serde_json::to_vec(&item).map_err(std::io::Error::other)?;
            if encoded.len().saturating_add(1) > MAX_MCP_WIRE_FRAME_BYTES {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "MCP frame exceeded its bound",
                ));
            }
            let mut writer = writer.lock().await;
            let writer = writer.as_mut().ok_or_else(|| {
                std::io::Error::new(std::io::ErrorKind::NotConnected, "MCP transport is closed")
            })?;
            writer.send(item).await.map_err(Into::into)
        }
    }

    async fn receive(&mut self) -> Option<RxJsonRpcMessage<RoleClient>> {
        match self.reader.next().await {
            Some(Ok(message)) => Some(message),
            Some(Err(_)) => {
                self.stop().await;
                None
            }
            None => None,
        }
    }

    async fn close(&mut self) -> Result<(), Self::Error> {
        self.writer.lock().await.take();
        self.stop().await;
        Ok(())
    }
}

impl Drop for BoundedChildTransport {
    fn drop(&mut self) {
        terminate_process_group(self.process_group.take());
        if let Some(child) = self.child.as_mut() {
            let _ = child.start_kill();
        }
    }
}

#[derive(Clone)]
struct BoundedReqwestClient {
    inner: reqwest::Client,
    session_id: Arc<StdMutex<Option<String>>>,
}

#[derive(Debug, Error)]
enum BoundedHttpClientError {
    #[error("HTTP request failed")]
    Request(#[source] reqwest::Error),
    #[error("MCP frame exceeded its bound")]
    FrameBound,
    #[error("MCP response was invalid")]
    InvalidResponse,
}

impl BoundedReqwestClient {
    fn new(inner: reqwest::Client, session_id: Arc<StdMutex<Option<String>>>) -> Self {
        Self { inner, session_id }
    }

    fn apply_headers(
        mut request: reqwest::RequestBuilder,
        headers: std::collections::HashMap<HeaderName, HeaderValue>,
    ) -> Result<reqwest::RequestBuilder, StreamableHttpError<BoundedHttpClientError>> {
        for (name, value) in headers {
            let reserved = name == ACCEPT
                || name.as_str().eq_ignore_ascii_case(HEADER_SESSION_ID)
                || name.as_str().eq_ignore_ascii_case(HEADER_LAST_EVENT_ID);
            if reserved {
                return Err(StreamableHttpError::ReservedHeaderConflict(
                    name.to_string(),
                ));
            }
            request = request.header(name, value);
        }
        Ok(request)
    }

    fn request_error(error: reqwest::Error) -> StreamableHttpError<BoundedHttpClientError> {
        StreamableHttpError::Client(BoundedHttpClientError::Request(error))
    }
}

struct SseFrameLimiter {
    limit: usize,
    frame_bytes: usize,
    line_has_data: bool,
    previous_was_cr: bool,
}

impl SseFrameLimiter {
    fn new(limit: usize) -> Self {
        Self {
            limit,
            frame_bytes: 0,
            line_has_data: false,
            previous_was_cr: false,
        }
    }

    fn inspect(&mut self, bytes: &Bytes) -> Result<(), std::io::Error> {
        for byte in bytes {
            self.frame_bytes = self.frame_bytes.saturating_add(1);
            if self.frame_bytes > self.limit {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "MCP SSE event exceeded its bound",
                ));
            }
            match *byte {
                b'\r' => {
                    if !self.line_has_data {
                        self.frame_bytes = 0;
                    }
                    self.line_has_data = false;
                    self.previous_was_cr = true;
                }
                b'\n' if self.previous_was_cr => {
                    self.previous_was_cr = false;
                }
                b'\n' => {
                    if !self.line_has_data {
                        self.frame_bytes = 0;
                    }
                    self.line_has_data = false;
                    self.previous_was_cr = false;
                }
                _ => {
                    self.line_has_data = true;
                    self.previous_was_cr = false;
                }
            }
        }
        Ok(())
    }
}

impl Default for SseFrameLimiter {
    fn default() -> Self {
        Self::new(MAX_MCP_WIRE_FRAME_BYTES)
    }
}

async fn bounded_response_bytes(
    response: reqwest::Response,
) -> Result<Bytes, StreamableHttpError<BoundedHttpClientError>> {
    if response
        .content_length()
        .is_some_and(|length| length > MAX_MCP_WIRE_FRAME_BYTES as u64)
    {
        return Err(StreamableHttpError::Client(
            BoundedHttpClientError::FrameBound,
        ));
    }
    let mut bytes = BytesMut::new();
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(BoundedReqwestClient::request_error)?;
        if bytes.len().saturating_add(chunk.len()) > MAX_MCP_WIRE_FRAME_BYTES {
            return Err(StreamableHttpError::Client(
                BoundedHttpClientError::FrameBound,
            ));
        }
        bytes.extend_from_slice(&chunk);
    }
    Ok(bytes.freeze())
}

fn bounded_sse_stream(response: reqwest::Response) -> BoxStream<'static, Result<Sse, SseError>> {
    let bytes = response
        .bytes_stream()
        .map_err(|_| std::io::Error::other("MCP SSE transport failed"))
        .scan(
            (SseFrameLimiter::default(), false),
            |(limiter, failed), item| {
                if *failed {
                    return futures::future::ready(None);
                }
                let item = item.and_then(|bytes| {
                    limiter.inspect(&bytes)?;
                    Ok(bytes)
                });
                *failed = item.is_err();
                futures::future::ready(Some(item))
            },
        );
    SseStream::from_bytes_stream(bytes).boxed()
}

impl StreamableHttpClient for BoundedReqwestClient {
    type Error = BoundedHttpClientError;

    async fn get_stream(
        &self,
        uri: Arc<str>,
        session_id: Arc<str>,
        last_event_id: Option<String>,
        auth_header: Option<String>,
        custom_headers: std::collections::HashMap<HeaderName, HeaderValue>,
    ) -> Result<BoxStream<'static, Result<Sse, SseError>>, StreamableHttpError<Self::Error>> {
        let mut request = self
            .inner
            .get(uri.as_ref())
            .header(ACCEPT, [EVENT_STREAM_MIME_TYPE, JSON_MIME_TYPE].join(", "))
            .header(HEADER_SESSION_ID, session_id.as_ref());
        if let Some(last_event_id) = last_event_id {
            request = request.header(HEADER_LAST_EVENT_ID, last_event_id);
        }
        if let Some(auth_header) = auth_header {
            request = request.bearer_auth(auth_header);
        }
        let response = Self::apply_headers(request, custom_headers)?
            .send()
            .await
            .map_err(Self::request_error)?;
        if response.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
            return Err(StreamableHttpError::ServerDoesNotSupportSse);
        }
        let response = response.error_for_status().map_err(Self::request_error)?;
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .map(|value| String::from_utf8_lossy(value.as_bytes()).to_string());
        if content_type.as_deref().is_none_or(|value| {
            !value
                .as_bytes()
                .starts_with(EVENT_STREAM_MIME_TYPE.as_bytes())
                && !value.as_bytes().starts_with(JSON_MIME_TYPE.as_bytes())
        }) {
            return Err(StreamableHttpError::UnexpectedContentType(content_type));
        }
        Ok(bounded_sse_stream(response))
    }

    async fn delete_session(
        &self,
        uri: Arc<str>,
        session_id: Arc<str>,
        auth_header: Option<String>,
        custom_headers: std::collections::HashMap<HeaderName, HeaderValue>,
    ) -> Result<(), StreamableHttpError<Self::Error>> {
        let mut request = self
            .inner
            .delete(uri.as_ref())
            .header(HEADER_SESSION_ID, session_id.as_ref());
        if let Some(auth_header) = auth_header {
            request = request.bearer_auth(auth_header);
        }
        let response = Self::apply_headers(request, custom_headers)?
            .send()
            .await
            .map_err(Self::request_error)?;
        if response.status() == reqwest::StatusCode::METHOD_NOT_ALLOWED {
            return Ok(());
        }
        response.error_for_status().map_err(Self::request_error)?;
        Ok(())
    }

    async fn post_message(
        &self,
        uri: Arc<str>,
        message: ClientJsonRpcMessage,
        session_id: Option<Arc<str>>,
        auth_header: Option<String>,
        custom_headers: std::collections::HashMap<HeaderName, HeaderValue>,
    ) -> Result<StreamableHttpPostResponse, StreamableHttpError<Self::Error>> {
        let encoded = serde_json::to_vec(&message)
            .map_err(|_| StreamableHttpError::Client(BoundedHttpClientError::InvalidResponse))?;
        if encoded.len() > MAX_MCP_WIRE_FRAME_BYTES {
            return Err(StreamableHttpError::Client(
                BoundedHttpClientError::FrameBound,
            ));
        }
        let mut request = self
            .inner
            .post(uri.as_ref())
            .header(ACCEPT, [EVENT_STREAM_MIME_TYPE, JSON_MIME_TYPE].join(", "));
        if let Some(auth_header) = auth_header {
            request = request.bearer_auth(auth_header);
        }
        let session_was_attached = session_id.is_some();
        if let Some(session_id) = session_id {
            request = request.header(HEADER_SESSION_ID, session_id.as_ref());
        }
        let response = Self::apply_headers(request, custom_headers)?
            .body(encoded)
            .header(reqwest::header::CONTENT_TYPE, JSON_MIME_TYPE)
            .send()
            .await
            .map_err(Self::request_error)?;
        let status = response.status();
        if status == reqwest::StatusCode::UNAUTHORIZED
            && let Some(value) = response.headers().get(WWW_AUTHENTICATE)
        {
            let value = value.to_str().map_err(|_| {
                StreamableHttpError::Client(BoundedHttpClientError::InvalidResponse)
            })?;
            return Err(StreamableHttpError::AuthRequired(AuthRequiredError::new(
                value.to_owned(),
            )));
        }
        if status == reqwest::StatusCode::FORBIDDEN
            && let Some(value) = response.headers().get(WWW_AUTHENTICATE)
        {
            let value = value.to_str().map_err(|_| {
                StreamableHttpError::Client(BoundedHttpClientError::InvalidResponse)
            })?;
            return Err(StreamableHttpError::InsufficientScope(
                InsufficientScopeError::new(value.to_owned(), None),
            ));
        }
        if matches!(
            status,
            reqwest::StatusCode::ACCEPTED | reqwest::StatusCode::NO_CONTENT
        ) {
            return Ok(StreamableHttpPostResponse::Accepted);
        }
        if status == reqwest::StatusCode::NOT_FOUND && session_was_attached {
            return Err(StreamableHttpError::SessionExpired);
        }
        let content_type = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .map(|value| String::from_utf8_lossy(value.as_bytes()).to_string());
        let content_length = response.content_length();
        let response_session_id = match response.headers().get(HEADER_SESSION_ID) {
            Some(value)
                if !value.as_bytes().is_empty()
                    && value.as_bytes().len() <= MAX_MCP_HTTP_SESSION_ID_BYTES
                    && value
                        .as_bytes()
                        .iter()
                        .all(|byte| matches!(byte, 0x21..=0x7e)) =>
            {
                Some(
                    value
                        .to_str()
                        .map_err(|_| {
                            StreamableHttpError::Client(BoundedHttpClientError::InvalidResponse)
                        })?
                        .to_owned(),
                )
            }
            Some(_) => {
                return Err(StreamableHttpError::Client(
                    BoundedHttpClientError::InvalidResponse,
                ));
            }
            None => None,
        };
        if let Some(session_id) = response_session_id.as_ref() {
            *self.session_id.lock().map_err(|_| {
                StreamableHttpError::Client(BoundedHttpClientError::InvalidResponse)
            })? = Some(session_id.clone());
        }
        if status.is_success() && content_length == Some(0) {
            return Ok(StreamableHttpPostResponse::Accepted);
        }
        if !status.is_success() {
            let body = bounded_response_bytes(response).await?;
            if content_type
                .as_deref()
                .is_some_and(|value| value.as_bytes().starts_with(JSON_MIME_TYPE.as_bytes()))
                && let Ok(message) = serde_json::from_slice::<ServerJsonRpcMessage>(&body)
            {
                return Ok(StreamableHttpPostResponse::Json(
                    message,
                    response_session_id,
                ));
            }
            return Err(StreamableHttpError::UnexpectedServerResponse(Cow::Owned(
                format!("HTTP {status}"),
            )));
        }
        match content_type.as_deref() {
            Some(value)
                if value
                    .as_bytes()
                    .starts_with(EVENT_STREAM_MIME_TYPE.as_bytes()) =>
            {
                Ok(StreamableHttpPostResponse::Sse(
                    bounded_sse_stream(response),
                    response_session_id,
                ))
            }
            Some(value) if value.as_bytes().starts_with(JSON_MIME_TYPE.as_bytes()) => {
                let body = bounded_response_bytes(response).await?;
                let message =
                    serde_json::from_slice::<ServerJsonRpcMessage>(&body).map_err(|_| {
                        StreamableHttpError::Client(BoundedHttpClientError::InvalidResponse)
                    })?;
                Ok(StreamableHttpPostResponse::Json(
                    message,
                    response_session_id,
                ))
            }
            _ => Err(StreamableHttpError::UnexpectedContentType(content_type)),
        }
    }
}

pub trait McpConnection: Send + Sync {
    fn handshake(&self) -> &McpHandshakeSnapshot;
    fn notification_epoch(&self) -> u64;
    fn list_tools<'a>(
        &'a self,
        timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<McpToolSnapshot>, McpError>>;
    fn list_resources<'a>(
        &'a self,
        timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<McpResourceSnapshot>, McpError>>;
    fn call_tool<'a>(
        &'a self,
        name: &'a str,
        arguments: Value,
        timeout_ms: u64,
        max_output_bytes: u64,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>>;
    fn read_resource<'a>(
        &'a self,
        uri: &'a str,
        timeout_ms: u64,
        max_output_bytes: u64,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>>;
    fn close(self: Box<Self>, timeout_ms: u64) -> McpFuture<'static, Result<(), McpError>>;
}

pub trait McpTransportFactory: Send + Sync {
    fn connect<'a>(
        &'a self,
        definition: &'a McpTransportDefinition,
        launch: ResolvedMcpLaunch,
        timeout_ms: u64,
    ) -> McpFuture<'a, Result<Box<dyn McpConnection>, McpError>>;
}

/// Core sampling seam. Implementations perform model-capability preflight,
/// policy/human review, bounded runtime dispatch, cancellation, and redaction.
pub trait McpSamplingBroker: Send + Sync {
    fn sample<'a>(
        &'a self,
        context: &'a McpSamplingContext,
        request: CreateMessageRequestParams,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<CreateMessageResult, McpError>>;
}

pub async fn broker_sampling_request(
    broker: &dyn McpSamplingBroker,
    context: &McpSamplingContext,
    request: CreateMessageRequestParams,
    cancellation: McpCancellation,
) -> Result<CreateMessageResult, McpError> {
    if context.server_id.is_empty()
        || context.server_id.len() > 255
        || context.authority_id.is_empty()
        || context.lifecycle_generation == 0
        || context.timeout_ms == 0
        || context.max_output_bytes == 0
    {
        return Err(McpError::InvalidInput);
    }
    validate_sampling_request(&request)?;
    if cancellation.is_cancelled() {
        return Err(McpError::Cancelled);
    }
    let result = broker.sample(context, request, cancellation).await?;
    validate_sampling_result(&result)?;
    Ok(result)
}

#[derive(Clone, Default)]
pub struct RmcpTransportFactory {
    sampling: Option<Arc<dyn McpSamplingBroker>>,
}

impl RmcpTransportFactory {
    pub fn with_sampling(broker: Arc<dyn McpSamplingBroker>) -> Self {
        Self {
            sampling: Some(broker),
        }
    }
}

impl McpTransportFactory for RmcpTransportFactory {
    fn connect<'a>(
        &'a self,
        definition: &'a McpTransportDefinition,
        launch: ResolvedMcpLaunch,
        timeout_ms: u64,
    ) -> McpFuture<'a, Result<Box<dyn McpConnection>, McpError>> {
        Box::pin(async move {
            let client = McpTransportClient::connect_with_sampling(
                definition,
                launch,
                timeout_ms,
                self.sampling.clone(),
            )
            .await?;
            Ok(Box::new(client) as Box<dyn McpConnection>)
        })
    }
}

#[derive(Clone, Default)]
struct C4osClientHandler {
    notification_epoch: Arc<AtomicU64>,
    sampling_context: Option<McpSamplingContext>,
    sampling: Option<Arc<dyn McpSamplingBroker>>,
}

impl ClientHandler for C4osClientHandler {
    fn get_info(&self) -> ClientInfo {
        let mut capabilities = ClientCapabilities::default();
        if self.sampling.is_some() && self.sampling_context.is_some() {
            // Basic sampling only. Tool loops and context inclusion remain
            // unadvertised until the production broker can honor those exact
            // optional capabilities.
            capabilities.sampling = Some(SamplingCapability::default());
        }
        ClientInfo::new(
            capabilities,
            Implementation::new("c4os", env!("CARGO_PKG_VERSION")),
        )
        .with_protocol_version(ProtocolVersion::V_2025_11_25)
    }

    async fn on_resource_updated(
        &self,
        _params: rmcp::model::ResourceUpdatedNotificationParam,
        _context: NotificationContext<RoleClient>,
    ) {
        self.notification_epoch.fetch_add(1, Ordering::SeqCst);
    }

    async fn on_resource_list_changed(&self, _context: NotificationContext<RoleClient>) {
        self.notification_epoch.fetch_add(1, Ordering::SeqCst);
    }

    async fn on_tool_list_changed(&self, _context: NotificationContext<RoleClient>) {
        self.notification_epoch.fetch_add(1, Ordering::SeqCst);
    }

    async fn create_message(
        &self,
        params: CreateMessageRequestParams,
        context: rmcp::service::RequestContext<RoleClient>,
    ) -> Result<CreateMessageResult, rmcp::ErrorData> {
        let (Some(broker), Some(sampling_context)) =
            (self.sampling.as_ref(), self.sampling_context.as_ref())
        else {
            return Err(rmcp::ErrorData::method_not_found::<
                rmcp::model::CreateMessageRequestMethod,
            >());
        };
        let cancellation = McpCancellation::default();
        let result = tokio::select! {
            result = broker_sampling_request(
                broker.as_ref(),
                sampling_context,
                params,
                cancellation.clone(),
            ) => result,
            _ = context.ct.cancelled() => {
                cancellation.cancel();
                return Err(sampling_protocol_error(McpError::Cancelled));
            }
        }
        .map_err(sampling_protocol_error)?;
        validate_sampling_result(&result).map_err(sampling_protocol_error)?;
        Ok(result)
    }
}

/// Exact-version rmcp client used by production and hostile transport tests.
pub struct McpTransportClient {
    running: Option<RunningService<RoleClient, C4osClientHandler>>,
    handshake: McpHandshakeSnapshot,
    notification_epoch: Arc<AtomicU64>,
    process_group: Option<i32>,
    http_cancellation: Option<HttpCancellationSender>,
}

#[derive(Clone)]
struct HttpCancellationSender {
    client: reqwest::Client,
    uri: String,
    bearer: Arc<Zeroizing<String>>,
    headers: Arc<BTreeMap<String, Zeroizing<String>>>,
    session_id: Arc<StdMutex<Option<String>>>,
}

impl HttpCancellationSender {
    async fn cancel(
        &self,
        request_id: RequestId,
        reason: &str,
        timeout: Duration,
    ) -> Result<(), McpError> {
        let session_id = self
            .session_id
            .lock()
            .map_err(|_| McpError::Transport("HTTP session state unavailable".into()))?
            .clone()
            .ok_or(McpError::NotReady)?;
        let encoded = serde_json::to_vec(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "notifications/cancelled",
            "params": {
                "requestId": request_id,
                "reason": reason,
            },
        }))
        .map_err(|_| McpError::InvalidState)?;
        if encoded.len() > MAX_MCP_WIRE_FRAME_BYTES {
            return Err(McpError::BoundExceeded);
        }
        let mut request = self
            .client
            .post(&self.uri)
            .header(ACCEPT, [EVENT_STREAM_MIME_TYPE, JSON_MIME_TYPE].join(", "))
            .header(reqwest::header::CONTENT_TYPE, JSON_MIME_TYPE)
            .header(HEADER_SESSION_ID, session_id)
            .header("MCP-Protocol-Version", MCP_PROTOCOL_VERSION)
            .bearer_auth(self.bearer.as_str())
            .body(encoded);
        for (name, value) in self.headers.iter() {
            request = request.header(name, value.as_str());
        }
        let response = tokio::time::timeout(timeout, request.send())
            .await
            .map_err(|_| McpError::TimedOut)?
            .map_err(|error| McpError::Transport(safe_transport_error(&error)))?;
        if !matches!(
            response.status(),
            reqwest::StatusCode::ACCEPTED | reqwest::StatusCode::NO_CONTENT
        ) {
            return Err(McpError::Transport(
                "HTTP cancellation notification was rejected".into(),
            ));
        }
        Ok(())
    }
}

impl McpTransportClient {
    pub async fn connect(
        definition: &McpTransportDefinition,
        launch: ResolvedMcpLaunch,
        timeout_ms: u64,
    ) -> Result<Self, McpError> {
        Self::connect_with_sampling(definition, launch, timeout_ms, None).await
    }

    pub async fn connect_with_sampling(
        definition: &McpTransportDefinition,
        mut launch: ResolvedMcpLaunch,
        timeout_ms: u64,
        sampling: Option<Arc<dyn McpSamplingBroker>>,
    ) -> Result<Self, McpError> {
        let handler = C4osClientHandler {
            notification_epoch: Arc::default(),
            sampling_context: launch.sampling_context.take(),
            sampling,
        };
        let notification_epoch = Arc::clone(&handler.notification_epoch);
        let timeout = Duration::from_millis(timeout_ms);

        let (running, process_group, http_cancellation) = match definition {
            McpTransportDefinition::Stdio {
                command,
                arguments,
                environment,
                working_directory,
                executable_sha256,
                ..
            } => {
                validate_resolved_environment(environment, &launch.environment)?;
                let executable = validate_executable(command, executable_sha256.as_deref())?;
                let directory = resolve_working_directory(working_directory, &launch)?;
                let scratch_root = validate_private_scratch(&launch.scratch_root)?;
                let profile = build_macos_sandbox_profile(
                    &executable,
                    launch
                        .active_project
                        .iter()
                        .chain(launch.trusted_roots.iter()),
                    &scratch_root,
                )?;
                let mut command = tokio::process::Command::new("/usr/bin/sandbox-exec");
                command
                    .arg("-p")
                    .arg(profile)
                    .arg(&executable)
                    .args(arguments)
                    .current_dir(directory)
                    .env_clear()
                    .env("HOME", &scratch_root)
                    .env("PATH", "/usr/bin:/bin")
                    .env("TMPDIR", &scratch_root)
                    .envs(std::mem::take(&mut launch.environment))
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::null())
                    .kill_on_drop(true);
                #[cfg(unix)]
                command.process_group(0);

                let transport = BoundedChildTransport::spawn(command)?;
                let running = tokio::time::timeout(timeout, handler.serve(transport))
                    .await
                    .map_err(|_| McpError::TimedOut)?
                    .map_err(|error| McpError::Transport(safe_transport_error(&error)))?;
                (running, None, None)
            }
            McpTransportDefinition::StreamableHttp { url, .. } => {
                let (resolved_host, resolved_addresses) =
                    validate_http_endpoint(url, timeout).await?;
                validate_resolved_headers(definition, &launch)?;
                let mut config = StreamableHttpClientTransportConfig::with_uri(url.clone());
                config.allow_stateless = false;
                config.reinit_on_expired_session = false;
                config.channel_buffer_capacity = 4;
                let bearer = launch.bearer.take().ok_or(McpError::Credential)?;
                config.auth_header = Some(bearer.clone());
                let resolved_headers = std::mem::take(&mut launch.headers);
                for (name, value) in &resolved_headers {
                    let name = name.parse().map_err(|_| McpError::InvalidInput)?;
                    let value = value.parse().map_err(|_| McpError::InvalidInput)?;
                    config.custom_headers.insert(name, value);
                }
                let http_client = reqwest::Client::builder()
                    .pool_max_idle_per_host(0)
                    .redirect(reqwest::redirect::Policy::none())
                    .no_proxy()
                    .connect_timeout(timeout)
                    .resolve_to_addrs(&resolved_host, &resolved_addresses)
                    .build()
                    .map_err(|error| McpError::Transport(safe_transport_error(&error)))?;
                let session_id = Arc::new(StdMutex::new(None));
                let http_cancellation = HttpCancellationSender {
                    client: http_client.clone(),
                    uri: url.clone(),
                    bearer: Arc::new(Zeroizing::new(bearer)),
                    headers: Arc::new(
                        resolved_headers
                            .into_iter()
                            .map(|(name, value)| (name, Zeroizing::new(value)))
                            .collect(),
                    ),
                    session_id: Arc::clone(&session_id),
                };
                let transport = StreamableHttpClientTransport::with_client(
                    BoundedReqwestClient::new(http_client, session_id),
                    config,
                );
                let running = tokio::time::timeout(timeout, handler.serve(transport))
                    .await
                    .map_err(|_| McpError::TimedOut)?
                    .map_err(|error| McpError::Transport(safe_transport_error(&error)))?;
                (running, None, Some(http_cancellation))
            }
        };

        let peer = running.peer_info().ok_or(McpError::UnsupportedProtocol)?;
        if peer.protocol_version != ProtocolVersion::V_2025_11_25
            || peer.protocol_version.as_str() != MCP_PROTOCOL_VERSION
        {
            drop(running);
            terminate_process_group(process_group);
            return Err(McpError::UnsupportedProtocol);
        }
        let capabilities = project_capabilities(&peer.capabilities);
        let handshake = McpHandshakeSnapshot {
            protocol_version: peer.protocol_version.to_string(),
            server_name: bounded_required_text(&peer.server_info.name)?,
            server_version: bounded_required_text(&peer.server_info.version)?,
            instructions_present: peer
                .instructions
                .as_deref()
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false),
            capabilities,
        };
        Ok(Self {
            running: Some(running),
            handshake,
            notification_epoch,
            process_group,
            http_cancellation,
        })
    }

    pub fn handshake(&self) -> &McpHandshakeSnapshot {
        &self.handshake
    }

    pub fn notification_epoch(&self) -> u64 {
        self.notification_epoch.load(Ordering::SeqCst)
    }

    pub async fn list_tools(&self, timeout_ms: u64) -> Result<Vec<McpToolSnapshot>, McpError> {
        if !self.handshake.capabilities.tools {
            return Ok(Vec::new());
        }
        let running = self.running.as_ref().ok_or(McpError::NotReady)?;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
        let mut cursor = None;
        let mut cursors = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut projected = Vec::new();
        loop {
            let result = tokio::time::timeout(
                remaining_until(deadline)?,
                running.list_tools(Some(paginated_request(cursor.clone()))),
            )
            .await
            .map_err(|_| McpError::TimedOut)?
            .map_err(map_service_error)?;
            if projected.len().saturating_add(result.tools.len()) > MAX_MCP_TOOLS {
                return Err(McpError::BoundExceeded);
            }
            for tool in result.tools {
                let name = bounded_required_text(tool.name.as_ref())?;
                if !seen.insert(name.clone()) {
                    return Err(McpError::InvalidState);
                }
                projected.push(McpToolSnapshot {
                    name,
                    title: bounded_optional_text(tool.title.as_deref()),
                    description: bounded_optional_text(tool.description.as_deref()),
                    input_schema: Value::Object(tool.input_schema.as_ref().clone()),
                    input_schema_sha256: sha256_json(tool.input_schema.as_ref())?,
                    output_schema: tool
                        .output_schema
                        .as_deref()
                        .map(|schema| Value::Object(schema.clone())),
                    output_schema_sha256: tool
                        .output_schema
                        .as_deref()
                        .map(sha256_json)
                        .transpose()?,
                });
            }
            cursor = checked_next_cursor(cursor.as_deref(), result.next_cursor)?;
            if cursor
                .as_ref()
                .is_some_and(|cursor| !cursors.insert(cursor.clone()))
            {
                return Err(McpError::BoundExceeded);
            }
            if cursors.len() > MAX_MCP_TOOLS {
                return Err(McpError::BoundExceeded);
            }
            if cursor.is_none() {
                break;
            }
        }
        projected.sort_by(|left, right| left.name.cmp(&right.name));
        Ok(projected)
    }

    pub async fn list_resources(
        &self,
        timeout_ms: u64,
    ) -> Result<Vec<McpResourceSnapshot>, McpError> {
        if !self.handshake.capabilities.resources {
            return Ok(Vec::new());
        }
        let running = self.running.as_ref().ok_or(McpError::NotReady)?;
        let deadline = tokio::time::Instant::now() + Duration::from_millis(timeout_ms);
        let mut cursor = None;
        let mut cursors = BTreeSet::new();
        let mut seen = BTreeSet::new();
        let mut projected = Vec::new();
        loop {
            let result = tokio::time::timeout(
                remaining_until(deadline)?,
                running.list_resources(Some(paginated_request(cursor.clone()))),
            )
            .await
            .map_err(|_| McpError::TimedOut)?
            .map_err(map_service_error)?;
            if projected.len().saturating_add(result.resources.len()) > MAX_MCP_RESOURCES {
                return Err(McpError::BoundExceeded);
            }
            for resource in result.resources {
                let uri = bounded_required_text(&resource.uri)?;
                if !seen.insert(uri.clone()) {
                    return Err(McpError::InvalidState);
                }
                projected.push(McpResourceSnapshot {
                    uri,
                    name: bounded_required_text(&resource.name)?,
                    title: bounded_optional_text(resource.title.as_deref()),
                    description: bounded_optional_text(resource.description.as_deref()),
                    mime_type: bounded_optional_text(resource.mime_type.as_deref()),
                    size: resource.size,
                });
            }
            cursor = checked_next_cursor(cursor.as_deref(), result.next_cursor)?;
            if cursor
                .as_ref()
                .is_some_and(|cursor| !cursors.insert(cursor.clone()))
            {
                return Err(McpError::BoundExceeded);
            }
            if cursors.len() > MAX_MCP_RESOURCES {
                return Err(McpError::BoundExceeded);
            }
            if cursor.is_none() {
                break;
            }
        }
        projected.sort_by(|left, right| left.uri.cmp(&right.uri));
        Ok(projected)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Value,
        timeout_ms: u64,
        max_output_bytes: u64,
        cancellation: McpCancellation,
    ) -> Result<McpRawResult, McpError> {
        let arguments = match arguments {
            Value::Object(arguments) => arguments,
            _ => return Err(McpError::InvalidInput),
        };
        let request = ClientRequest::CallToolRequest(Request::new(
            CallToolRequestParams::new(name.to_owned()).with_arguments(arguments),
        ));
        let result = self
            .request_cancellable(request, timeout_ms, cancellation)
            .await?;
        let ServerResult::CallToolResult(result) = result else {
            return Err(McpError::Transport("unexpected response kind".into()));
        };
        bounded_raw_result(
            serde_json::to_value(&result).map_err(|_| McpError::InvalidState)?,
            result.is_error.unwrap_or(false),
            max_output_bytes,
        )
    }

    pub async fn read_resource(
        &self,
        uri: &str,
        timeout_ms: u64,
        max_output_bytes: u64,
        cancellation: McpCancellation,
    ) -> Result<McpRawResult, McpError> {
        let request = ClientRequest::ReadResourceRequest(Request::new(
            ReadResourceRequestParams::new(uri.to_owned()),
        ));
        let result = self
            .request_cancellable(request, timeout_ms, cancellation)
            .await?;
        let ServerResult::ReadResourceResult(result) = result else {
            return Err(McpError::Transport("unexpected response kind".into()));
        };
        bounded_raw_result(
            serde_json::to_value(&result).map_err(|_| McpError::InvalidState)?,
            false,
            max_output_bytes,
        )
    }

    async fn request_cancellable(
        &self,
        request: ClientRequest,
        timeout_ms: u64,
        cancellation: McpCancellation,
    ) -> Result<ServerResult, McpError> {
        if cancellation.is_cancelled() {
            return Err(McpError::Cancelled);
        }
        let running = self.running.as_ref().ok_or(McpError::NotReady)?;
        let mut handle = running
            .send_cancellable_request(request, PeerRequestOptions::no_options())
            .await
            .map_err(map_service_error)?;
        tokio::select! {
            response = &mut handle.rx => {
                response
                    .map_err(|_| McpError::Transport("response channel closed".into()))?
                    .map_err(map_service_error)
            }
            _ = cancellation.cancelled() => {
                self.cancel_request(handle, "C4OS operation cancelled", timeout_ms).await?;
                Err(McpError::Cancelled)
            }
            _ = tokio::time::sleep(Duration::from_millis(timeout_ms)) => {
                self.cancel_request(handle, "C4OS operation timed out", timeout_ms).await?;
                Err(McpError::TimedOut)
            }
        }
    }

    async fn cancel_request(
        &self,
        handle: rmcp::service::RequestHandle<RoleClient>,
        reason: &str,
        timeout_ms: u64,
    ) -> Result<(), McpError> {
        let timeout = Duration::from_millis(timeout_ms.min(1_000));
        if let Some(sender) = &self.http_cancellation {
            return sender.cancel(handle.id.clone(), reason, timeout).await;
        }
        tokio::time::timeout(timeout, handle.cancel(Some(reason.into())))
            .await
            .map_err(|_| McpError::TimedOut)?
            .map_err(map_service_error)
    }

    pub async fn shutdown(mut self, timeout_ms: u64) -> Result<(), McpError> {
        let process_group = self.process_group.take();
        if let Some(mut running) = self.running.take() {
            let timeout = Duration::from_millis(timeout_ms.min(3_000));
            let closed = running
                .close_with_timeout(timeout)
                .await
                .map_err(|_| McpError::Transport("MCP worker join failed".into()))?;
            if closed.is_none() {
                terminate_process_group(process_group);
                return Err(McpError::TimedOut);
            }
        }
        terminate_process_group(process_group);
        Ok(())
    }
}

impl Drop for McpTransportClient {
    fn drop(&mut self) {
        terminate_process_group(self.process_group.take());
    }
}

impl McpConnection for McpTransportClient {
    fn handshake(&self) -> &McpHandshakeSnapshot {
        self.handshake()
    }

    fn notification_epoch(&self) -> u64 {
        self.notification_epoch()
    }

    fn list_tools<'a>(
        &'a self,
        timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<McpToolSnapshot>, McpError>> {
        Box::pin(async move { self.list_tools(timeout_ms).await })
    }

    fn list_resources<'a>(
        &'a self,
        timeout_ms: u64,
    ) -> McpFuture<'a, Result<Vec<McpResourceSnapshot>, McpError>> {
        Box::pin(async move { self.list_resources(timeout_ms).await })
    }

    fn call_tool<'a>(
        &'a self,
        name: &'a str,
        arguments: Value,
        timeout_ms: u64,
        max_output_bytes: u64,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>> {
        Box::pin(async move {
            self.call_tool(name, arguments, timeout_ms, max_output_bytes, cancellation)
                .await
        })
    }

    fn read_resource<'a>(
        &'a self,
        uri: &'a str,
        timeout_ms: u64,
        max_output_bytes: u64,
        cancellation: McpCancellation,
    ) -> McpFuture<'a, Result<McpRawResult, McpError>> {
        Box::pin(async move {
            self.read_resource(uri, timeout_ms, max_output_bytes, cancellation)
                .await
        })
    }

    fn close(self: Box<Self>, timeout_ms: u64) -> McpFuture<'static, Result<(), McpError>> {
        Box::pin(async move { self.shutdown(timeout_ms).await })
    }
}

fn project_capabilities(value: &rmcp::model::ServerCapabilities) -> McpCapabilitySnapshot {
    let tools = value.tools.as_ref();
    let resources = value.resources.as_ref();
    let mut experimental_keys = value
        .experimental
        .as_ref()
        .into_iter()
        .flat_map(|items| items.keys().cloned())
        .chain(
            value
                .extensions
                .as_ref()
                .into_iter()
                .flat_map(|items| items.keys().map(|key| format!("extension:{key}"))),
        )
        .filter_map(|value| bounded_optional_text(Some(&value)))
        .collect::<Vec<_>>();
    experimental_keys.sort();
    experimental_keys.truncate(128);
    McpCapabilitySnapshot {
        tools: tools.is_some(),
        tool_list_changed: tools.and_then(|value| value.list_changed).unwrap_or(false),
        resources: resources.is_some(),
        resource_list_changed: resources
            .and_then(|value| value.list_changed)
            .unwrap_or(false),
        resource_subscribe: resources.and_then(|value| value.subscribe).unwrap_or(false),
        prompts: value.prompts.is_some(),
        logging: value.logging.is_some(),
        completions: value.completions.is_some(),
        tasks: value.tasks.is_some(),
        experimental_keys,
    }
}

fn resolve_working_directory(
    descriptor: &McpWorkingDirectory,
    launch: &ResolvedMcpLaunch,
) -> Result<PathBuf, McpError> {
    let requested = match descriptor {
        McpWorkingDirectory::C4osHome => &launch.scratch_root,
        McpWorkingDirectory::ActiveProject => launch
            .active_project
            .as_ref()
            .ok_or(McpError::InvalidInput)?,
        McpWorkingDirectory::TrustedRoot { path } => {
            let requested = Path::new(path);
            launch
                .trusted_roots
                .iter()
                .find(|candidate| candidate.as_path() == requested)
                .ok_or(McpError::Denied)?
        }
    };
    let canonical = requested
        .canonicalize()
        .map_err(|_| McpError::InvalidInput)?;
    if !canonical.is_dir() {
        return Err(McpError::InvalidInput);
    }
    let allowed = std::iter::once(&launch.c4os_home)
        .chain(launch.active_project.iter())
        .chain(launch.trusted_roots.iter())
        .filter_map(|path| path.canonicalize().ok())
        .any(|root| canonical.starts_with(root));
    if !allowed {
        return Err(McpError::Denied);
    }
    Ok(canonical)
}

fn validate_executable(command: &str, expected_digest: Option<&str>) -> Result<PathBuf, McpError> {
    let path = Path::new(command);
    if !path.is_absolute() {
        return Err(McpError::InvalidInput);
    }
    let canonical = path.canonicalize().map_err(|_| McpError::InvalidInput)?;
    if canonical != path || !canonical.is_file() {
        return Err(McpError::Denied);
    }
    let expected = expected_digest.ok_or(McpError::Denied)?;
    let bytes = std::fs::read(&canonical).map_err(|_| McpError::InvalidInput)?;
    let actual = format!("sha256:{}", hex_digest(&Sha256::digest(bytes)));
    if !actual.eq_ignore_ascii_case(expected) {
        return Err(McpError::Denied);
    }
    Ok(canonical)
}

#[cfg(target_os = "macos")]
fn build_macos_sandbox_profile<'a>(
    executable: &Path,
    read_roots: impl Iterator<Item = &'a PathBuf>,
    scratch_root: &Path,
) -> Result<String, McpError> {
    let executable_parent = executable.parent().ok_or(McpError::InvalidInput)?;
    let mut readable_roots = vec![executable_parent.to_owned(), scratch_root.to_owned()];
    readable_roots.extend(
        read_roots
            .map(PathBuf::as_path)
            .map(Path::canonicalize)
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| McpError::Denied)?,
    );
    readable_roots.sort();
    readable_roots.dedup();

    let mut profile = vec![
        "(version 1)".to_owned(),
        "(import \"system.sb\")".to_owned(),
        "(deny network*)".to_owned(),
        "(deny file-write*)".to_owned(),
        "(deny process-fork)".to_owned(),
        "(deny process-exec)".to_owned(),
        format!(
            "(allow process-exec (literal \"{}\"))",
            escape_sandbox_path(executable)?
        ),
        format!(
            "(allow file-read* (subpath \"{}\"))",
            escape_sandbox_path(executable_parent)?
        ),
        format!(
            "(allow file-read* (subpath \"{}\"))",
            escape_sandbox_path(scratch_root)?
        ),
        format!(
            "(allow file-write* (subpath \"{}\"))",
            escape_sandbox_path(scratch_root)?
        ),
    ];
    let mut metadata_ancestors = BTreeSet::new();
    for root in &readable_roots {
        metadata_ancestors.extend(
            root.ancestors()
                .skip(1)
                .filter(|ancestor| *ancestor != Path::new("/"))
                .map(Path::to_owned),
        );
    }
    for ancestor in metadata_ancestors {
        profile.push(format!(
            "(allow file-read-metadata (literal \"{}\"))",
            escape_sandbox_path(&ancestor)?
        ));
    }
    for root in readable_roots {
        profile.push(format!(
            "(allow file-read* (subpath \"{}\"))",
            escape_sandbox_path(&root)?
        ));
    }
    Ok(profile.join("\n"))
}

#[cfg(not(target_os = "macos"))]
fn build_macos_sandbox_profile<'a>(
    _executable: &Path,
    _read_roots: impl Iterator<Item = &'a PathBuf>,
    _scratch_root: &Path,
) -> Result<String, McpError> {
    Err(McpError::Denied)
}

fn validate_private_scratch(path: &Path) -> Result<PathBuf, McpError> {
    let metadata = std::fs::symlink_metadata(path).map_err(|_| McpError::Denied)?;
    let canonical = path.canonicalize().map_err(|_| McpError::Denied)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() || canonical != path {
        return Err(McpError::Denied);
    }
    Ok(canonical)
}

fn escape_sandbox_path(path: &Path) -> Result<String, McpError> {
    let value = path.to_str().ok_or(McpError::InvalidInput)?;
    if value.contains(['\n', '\r', '\0']) {
        return Err(McpError::InvalidInput);
    }
    Ok(value.replace('\\', "\\\\").replace('"', "\\\""))
}

async fn validate_http_endpoint(
    value: &str,
    timeout: Duration,
) -> Result<(String, Vec<SocketAddr>), McpError> {
    let url = Url::parse(value).map_err(|_| McpError::InvalidInput)?;
    if url.username() != "" || url.password().is_some() || url.fragment().is_some() {
        return Err(McpError::InvalidInput);
    }
    let host = url.host().ok_or(McpError::InvalidInput)?;
    let loopback = match host {
        Host::Domain(name) => name.eq_ignore_ascii_case("localhost"),
        Host::Ipv4(address) => address.is_loopback(),
        Host::Ipv6(address) => address.is_loopback(),
    };
    if url.scheme() != "https" && !(url.scheme() == "http" && loopback) {
        return Err(McpError::Denied);
    }
    let host = url.host_str().ok_or(McpError::InvalidInput)?.to_owned();
    let port = url.port_or_known_default().ok_or(McpError::InvalidInput)?;
    let addresses = tokio::time::timeout(timeout, tokio::net::lookup_host((host.as_str(), port)))
        .await
        .map_err(|_| McpError::TimedOut)?
        .map_err(|_| McpError::Transport("endpoint resolution failed".into()))?
        .collect::<BTreeSet<_>>();
    if addresses.is_empty()
        || addresses.iter().any(|address| {
            if loopback {
                !address.ip().is_loopback()
            } else {
                is_non_public_address(&address.ip())
            }
        })
    {
        return Err(McpError::Denied);
    }
    Ok((host, addresses.into_iter().collect::<Vec<_>>()))
}

fn is_non_public_address(address: &IpAddr) -> bool {
    match address {
        IpAddr::V4(value) => {
            value.is_private()
                || value.is_loopback()
                || value.is_link_local()
                || value.is_multicast()
                || value.is_broadcast()
                || value.is_unspecified()
                || value.octets()[0] == 0
                || value.octets()[0] >= 240
                || (value.octets()[0] == 100 && (64..=127).contains(&value.octets()[1]))
                || (value.octets()[0] == 169 && value.octets()[1] == 254)
                || (value.octets()[0] == 192
                    && matches!((value.octets()[1], value.octets()[2]), (0, _) | (88, 99)))
                || (value.octets()[0] == 198
                    && ((18..=19).contains(&value.octets()[1])
                        || (value.octets()[1] == 51 && value.octets()[2] == 100)))
                || (value.octets()[0] == 203 && value.octets()[1] == 0 && value.octets()[2] == 113)
        }
        IpAddr::V6(value) => {
            value
                .to_ipv4_mapped()
                .is_some_and(|value| is_non_public_address(&IpAddr::V4(value)))
                || value.is_loopback()
                || value.is_multicast()
                || value.is_unspecified()
                || (value.segments()[0] & 0xfe00) == 0xfc00
                || (value.segments()[0] & 0xffc0) == 0xfe80
                || (value.segments()[0] & 0xe000) != 0x2000
                || (value.segments()[0] == 0x2001
                    && (value.segments()[1] == 0
                        || (value.segments()[1] == 2 && value.segments()[2] == 0)
                        || (value.segments()[1] & 0xfff0) == 0x0010
                        || (value.segments()[1] & 0xfff0) == 0x0020
                        || value.segments()[1] == 0x0db8))
                || value.segments()[0] == 0x2002
                || (value.segments()[0] == 0x3fff && value.segments()[1] < 0x1000)
        }
    }
}

fn validate_resolved_headers(
    definition: &McpTransportDefinition,
    launch: &ResolvedMcpLaunch,
) -> Result<(), McpError> {
    let McpTransportDefinition::StreamableHttp {
        bearer, headers, ..
    } = definition
    else {
        return Err(McpError::InvalidInput);
    };
    if bearer.is_none()
        || launch.bearer.as_deref().is_none_or(str::is_empty)
        || headers.len() != launch.headers.len()
    {
        return Err(McpError::Credential);
    }
    for header in headers {
        let Some(value) = launch.headers.get(&header.name) else {
            return Err(McpError::Credential);
        };
        if value.is_empty() || value.contains(['\r', '\n']) {
            return Err(McpError::InvalidInput);
        }
        if let McpHeaderSource::Literal { value: expected } = &header.source
            && value != expected
        {
            return Err(McpError::Credential);
        }
    }
    Ok(())
}

fn validate_resolved_environment(
    definitions: &[super::McpEnvironmentBinding],
    resolved: &BTreeMap<String, String>,
) -> Result<(), McpError> {
    if definitions.len() != resolved.len() {
        return Err(McpError::Credential);
    }
    for binding in definitions {
        let value = resolved.get(&binding.name).ok_or(McpError::Credential)?;
        if value.contains('\0') {
            return Err(McpError::InvalidInput);
        }
        if let McpEnvironmentSource::Literal { value: expected } = &binding.source
            && value != expected
        {
            return Err(McpError::Credential);
        }
    }
    Ok(())
}

fn validate_sampling_request(request: &CreateMessageRequestParams) -> Result<(), McpError> {
    request.validate().map_err(|_| McpError::InvalidInput)?;
    if request.messages.is_empty()
        || request.messages.len() > 128
        || request.max_tokens == 0
        || request.max_tokens > 1_000_000
        || request
            .temperature
            .is_some_and(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
    {
        return Err(McpError::BoundExceeded);
    }
    if request.meta.is_some()
        || request.task.is_some()
        || request.include_context.is_some()
        || request.stop_sequences.is_some()
        || request.metadata.is_some()
        || request.tools.is_some()
        || request.tool_choice.is_some()
    {
        return Err(McpError::InvalidInput);
    }
    if request
        .system_prompt
        .as_deref()
        .is_some_and(|value| value.len() > 64 * 1_024)
        || request.stop_sequences.as_ref().is_some_and(|values| {
            values.len() > 64 || values.iter().any(|value| value.len() > 4_096)
        })
        || request
            .tools
            .as_ref()
            .is_some_and(|tools| tools.len() > 128)
    {
        return Err(McpError::BoundExceeded);
    }
    if let Some(preferences) = request.model_preferences.as_ref()
        && ([
            preferences.cost_priority,
            preferences.speed_priority,
            preferences.intelligence_priority,
        ]
        .into_iter()
        .flatten()
        .any(|value| !value.is_finite() || !(0.0..=1.0).contains(&value))
            || preferences.hints.as_ref().is_some_and(|hints| {
                hints.len() > 32
                    || hints.iter().any(|hint| {
                        hint.name
                            .as_deref()
                            .is_some_and(|name| name.is_empty() || name.len() > 255)
                    })
            }))
    {
        return Err(McpError::InvalidInput);
    }
    sampling_text_messages(request)?;
    let value = serde_json::to_value(request).map_err(|_| McpError::InvalidInput)?;
    validate_hostile_json(&value, 0)?;
    let bytes = serde_json::to_vec(&value).map_err(|_| McpError::InvalidInput)?;
    if bytes.len() > 512 * 1_024 {
        return Err(McpError::BoundExceeded);
    }
    Ok(())
}

pub(crate) fn sampling_text_messages(
    request: &CreateMessageRequestParams,
) -> Result<Vec<(String, String)>, McpError> {
    let mut output = Vec::with_capacity(request.messages.len());
    for message in &request.messages {
        if message.meta.is_some() || message.content.is_empty() || message.content.len() > 32 {
            return Err(McpError::InvalidInput);
        }
        let mut text = String::new();
        for content in message.content.iter() {
            let SamplingMessageContentBlock::Text(content) = content else {
                return Err(McpError::InvalidInput);
            };
            if content.meta.is_some()
                || content.annotations.is_some()
                || content.text.is_empty()
                || content.text.len() > 64 * 1_024
                || content.text.contains('\0')
            {
                return Err(McpError::InvalidInput);
            }
            if text.len().saturating_add(content.text.len()) > 64 * 1_024 {
                return Err(McpError::BoundExceeded);
            }
            text.push_str(&content.text);
        }
        output.push((
            match message.role {
                rmcp::model::Role::User => "user",
                rmcp::model::Role::Assistant => "assistant",
            }
            .into(),
            text,
        ));
    }
    Ok(output)
}

pub(crate) fn validate_sampling_result(result: &CreateMessageResult) -> Result<(), McpError> {
    result.validate().map_err(|_| McpError::InvalidState)?;
    bounded_required_text(&result.model)?;
    if !matches!(
        result.stop_reason.as_deref(),
        Some(
            CreateMessageResult::STOP_REASON_END_TURN
                | CreateMessageResult::STOP_REASON_END_MAX_TOKEN
        )
    ) || result.message.meta.is_some()
        || result.message.content.is_empty()
        || result.message.content.len() > 32
    {
        return Err(McpError::InvalidState);
    }
    for content in result.message.content.iter() {
        let SamplingMessageContentBlock::Text(content) = content else {
            return Err(McpError::InvalidState);
        };
        if content.meta.is_some()
            || content.annotations.is_some()
            || content.text.is_empty()
            || content.text.len() > 64 * 1_024
            || content.text.contains('\0')
        {
            return Err(McpError::InvalidState);
        }
    }
    let value = serde_json::to_value(result).map_err(|_| McpError::InvalidState)?;
    validate_hostile_json(&value, 0)?;
    if serde_json::to_vec(&value)
        .map_err(|_| McpError::InvalidState)?
        .len()
        > 1024 * 1024
    {
        return Err(McpError::BoundExceeded);
    }
    Ok(())
}

fn validate_hostile_json(value: &Value, depth: usize) -> Result<(), McpError> {
    if depth > 32 {
        return Err(McpError::BoundExceeded);
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
        Value::String(value) => {
            if value.len() > 64 * 1_024 || value.contains('\0') {
                Err(McpError::BoundExceeded)
            } else {
                Ok(())
            }
        }
        Value::Array(values) => {
            if values.len() > 1_024 {
                return Err(McpError::BoundExceeded);
            }
            for value in values {
                validate_hostile_json(value, depth + 1)?;
            }
            Ok(())
        }
        Value::Object(values) => {
            if values.len() > 1_024 {
                return Err(McpError::BoundExceeded);
            }
            for (key, value) in values {
                if key.len() > 4_096 || key.contains('\0') {
                    return Err(McpError::BoundExceeded);
                }
                validate_hostile_json(value, depth + 1)?;
            }
            Ok(())
        }
    }
}

fn sampling_protocol_error(error: McpError) -> rmcp::ErrorData {
    let message = match error {
        McpError::Denied | McpError::Untrusted | McpError::Revoked => "sampling request denied",
        McpError::Cancelled => "sampling request cancelled",
        McpError::TimedOut => "sampling request timed out",
        McpError::BoundExceeded => "sampling request exceeded a C4OS bound",
        _ => "sampling request failed",
    };
    rmcp::ErrorData::invalid_request(message, None)
}

fn bounded_raw_result(
    value: Value,
    is_error: bool,
    max_output_bytes: u64,
) -> Result<McpRawResult, McpError> {
    let bytes = serde_json::to_vec(&value).map_err(|_| McpError::InvalidState)?;
    let output_bytes = u64::try_from(bytes.len()).map_err(|_| McpError::BoundExceeded)?;
    if output_bytes > max_output_bytes {
        return Err(McpError::BoundExceeded);
    }
    Ok(McpRawResult {
        value,
        is_error,
        output_bytes,
    })
}

fn checked_next_cursor(
    previous: Option<&str>,
    next: Option<String>,
) -> Result<Option<String>, McpError> {
    if let Some(next) = next.as_deref()
        && (next.is_empty() || next.len() > 4_096 || Some(next) == previous)
    {
        return Err(McpError::BoundExceeded);
    }
    Ok(next)
}

fn paginated_request(cursor: Option<String>) -> PaginatedRequestParams {
    let mut request = PaginatedRequestParams::default();
    request.cursor = cursor;
    request
}

fn remaining_until(deadline: tokio::time::Instant) -> Result<Duration, McpError> {
    deadline
        .checked_duration_since(tokio::time::Instant::now())
        .filter(|remaining| !remaining.is_zero())
        .ok_or(McpError::TimedOut)
}

fn bounded_required_text(value: &str) -> Result<String, McpError> {
    let value = bounded_optional_text(Some(value)).ok_or(McpError::InvalidInput)?;
    if value.is_empty() {
        return Err(McpError::InvalidInput);
    }
    Ok(value)
}

fn bounded_optional_text(value: Option<&str>) -> Option<String> {
    let value = value?.trim();
    if value.is_empty() {
        return None;
    }
    let mut output = String::with_capacity(value.len().min(MAX_MCP_TEXT_BYTES));
    for character in value.chars() {
        let character = if character.is_control() && character != '\n' && character != '\t' {
            '\u{fffd}'
        } else {
            character
        };
        if output.len() + character.len_utf8() > MAX_MCP_TEXT_BYTES {
            break;
        }
        output.push(character);
    }
    Some(output)
}

fn sha256_json(value: &Map<String, Value>) -> Result<String, McpError> {
    let bytes = serde_json::to_vec(value).map_err(|_| McpError::InvalidState)?;
    Ok(format!("sha256:{}", hex_digest(&Sha256::digest(bytes))))
}

fn hex_digest(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn map_service_error(error: rmcp::ServiceError) -> McpError {
    match error {
        rmcp::ServiceError::Timeout { .. } => McpError::TimedOut,
        other => McpError::Transport(safe_transport_error(&other)),
    }
}

fn safe_transport_error(error: &impl std::fmt::Display) -> String {
    let _ = error;
    "transport operation failed".into()
}

fn terminate_process_group(process_group: Option<i32>) {
    #[cfg(unix)]
    if let Some(process_group) = process_group {
        // The command is launched as the leader of its own group. A negative
        // pid targets the complete group and prevents orphaned descendants.
        unsafe {
            libc::kill(-process_group, libc::SIGKILL);
        }
    }
    #[cfg(not(unix))]
    let _ = process_group;
}

#[cfg(test)]
mod tests {
    use bytes::{Bytes, BytesMut};
    use rmcp::{
        service::{RoleClient, RxJsonRpcMessage},
        transport::async_rw::{JsonRpcMessageCodec, JsonRpcMessageCodecError},
    };
    use tokio_util::codec::Decoder;

    use super::{SseFrameLimiter, is_non_public_address};

    #[test]
    fn stdio_codec_rejects_a_frame_before_unbounded_deserialization() {
        let mut codec =
            JsonRpcMessageCodec::<RxJsonRpcMessage<RoleClient>>::new_with_max_length(32);
        let mut bytes = BytesMut::from(&vec![b'x'; 33][..]);

        let error = codec
            .decode(&mut bytes)
            .expect_err("an unterminated over-limit frame must fail");

        assert!(matches!(
            error,
            JsonRpcMessageCodecError::MaxLineLengthExceeded
        ));
        assert_eq!(bytes.len(), 33);
    }

    #[test]
    fn sse_limiter_bounds_one_event_across_chunks_and_resets_at_blank_lines() {
        let mut limiter = SseFrameLimiter::new(18);
        limiter
            .inspect(&Bytes::from_static(b"data: first\n\n"))
            .expect("first bounded event");
        limiter
            .inspect(&Bytes::from_static(b"data: second"))
            .expect("partial second event");
        assert!(
            limiter
                .inspect(&Bytes::from_static(b"-past-limit"))
                .is_err(),
            "one SSE event must not grow past the cap across chunks"
        );
    }

    #[test]
    fn http_destination_policy_denies_special_purpose_addresses() {
        for value in [
            "0.0.0.0",
            "10.0.0.1",
            "100.64.0.1",
            "127.0.0.1",
            "169.254.1.1",
            "192.0.2.1",
            "192.88.99.1",
            "198.18.0.1",
            "198.51.100.1",
            "203.0.113.1",
            "224.0.0.1",
            "::",
            "::1",
            "fc00::1",
            "fe80::1",
            "2001:db8::1",
            "2001:2::1",
            "2002::1",
            "3fff::1",
        ] {
            let address = value.parse().expect("fixture IP address");
            assert!(
                is_non_public_address(&address),
                "accepted special IP {value}"
            );
        }
        for value in ["8.8.8.8", "1.1.1.1", "2606:4700:4700::1111"] {
            let address = value.parse().expect("fixture public IP address");
            assert!(!is_non_public_address(&address), "denied public IP {value}");
        }
    }
}

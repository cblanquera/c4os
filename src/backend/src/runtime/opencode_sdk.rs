//! Integrity-pinned OpenCode SDK plugin and authenticated C4OS broker boundary.
//!
//! The JavaScript plugin is deliberately effectless: it can only submit a
//! proposal over a preconnected Unix descriptor. This module authenticates
//! that descriptor by possession, validates and correlates every frame, and
//! returns bounded decisions owned by C4OS.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::os::fd::{AsRawFd, FromRawFd, RawFd};
use std::os::unix::ffi::OsStrExt;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::os::unix::net::UnixStream;
use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use thiserror::Error;
use uuid::Uuid;

use crate::runtime::opencode::{
    C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL, OPENCODE_SDK_VERSION,
};
use crate::runtime::supervisor::sha256_file;

pub const OPENCODE_SDK_PLUGIN_VERSION: &str = "1.18.3";
pub const OPENCODE_SDK_TOOL_IDS: [&str; 2] = [C4OS_ACTION_PROPOSAL_TOOL, C4OS_RESOURCE_READ_TOOL];
pub const OPENCODE_BROKER_FD_ENV: &str = "C4OS_OPENCODE_BROKER_FD";
pub const OPENCODE_PROCESS_GENERATION_ENV: &str = "C4OS_OPENCODE_PROCESS_GENERATION";
#[doc(hidden)]
pub const OPENCODE_TEST_TLS_TRUST_FD_ENV: &str = "C4OS_OPENCODE_TEST_TLS_TRUST_FD";

const BROKER_SCHEMA_VERSION: u32 = 1;
const MAX_FRAME_BYTES: usize = 512 * 1024;
const MAX_TREE_FILES: usize = 8_192;
const MAX_TREE_BYTES: u64 = 128 * 1024 * 1024;
const MAX_DEPENDENCY_FILE_BYTES: u64 = 16 * 1024 * 1024;
const MAX_IO_TIMEOUT: Duration = Duration::from_secs(30);
const MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES: usize = 256 * 1024;
const MIN_BROKER_FD: RawFd = 64;
const MAX_BROKER_FD: RawFd = 1_023;
const SDK_INTEGRITY: &str = "sha512-Mevo4e6kQwbvto9E+42KSIVMhp+JBu+SwQhC5AomAvrV6Xkio3U249T+xDILDCXhl5Z/Hi/DlAuVLzpGnuh0gg==";
const PLUGIN_INTEGRITY: &str = "sha512-hAm/hZkSCsMSNHVy9DKmFtZAve/qYoIWpduNPcvXnnKK7NMlJEOQitA6CQC67Ym5DFKypDW1TC9YO6S1WdGtpg==";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenCodeSdkIntegrity;

impl OpenCodeSdkIntegrity {
    const PINNED_FILES: [(&'static str, &'static str, u64); 6] = [
        (
            "package.json",
            "sha256:39aa05e5633457a1d3335cb9026f92cef3bea570d553f4692d0d573e5040d030",
            64 * 1024,
        ),
        (
            "package-lock.json",
            "sha256:295f7d56e55da283d11a11d756f129e9671a9c6937a637571c6f302aed426338",
            8 * 1024 * 1024,
        ),
        (
            "broker-channel.mjs",
            "sha256:5b65f8375ceb2737cdb86ecd74f271eb47eb17147bb3bc992a534f5a52fcd5c3",
            512 * 1024,
        ),
        (
            "credential-channel.mjs",
            "sha256:931b230c20707b6206c38182d9fc9fe2cf9414877cbaa756d80f930c244c58b8",
            512 * 1024,
        ),
        (
            "c4os-tools-plugin.mjs",
            "sha256:ec264f15e7a0bf7b1d55883faed6c0883564f7edf17b9898f1c8c0bdc8211e03",
            256 * 1024,
        ),
        (
            "sdk-client.mjs",
            "sha256:4524f30f0f9c29ccc2530e89a5a6a45d955778d129f30222954c6277f16466d3",
            256 * 1024,
        ),
    ];
    const NODE_MODULES_TREE_SHA256: &'static str =
        "sha256:747f554f7f533c29c61d4da3034478a46869c70f10053209d5b82464b36a96e8";

    /// Verifies every production sidecar source, exact lock records, and every
    /// installed dependency artifact before a plugin or descriptor exists.
    pub fn verify(sidecar_root: &Path) -> Result<OpenCodeSdkIntegrityReceipt, OpenCodeSdkError> {
        if OPENCODE_SDK_VERSION != OPENCODE_SDK_PLUGIN_VERSION {
            return Err(OpenCodeSdkError::InvalidIntegrity);
        }
        let root = canonical_directory(sidecar_root)?;
        for (relative, expected, maximum_bytes) in Self::PINNED_FILES {
            verify_pinned_file(&root, relative, expected, maximum_bytes)?;
        }
        verify_exact_lock(&root)?;
        let dependency_tree_sha256 = dependency_tree_sha256(&root.join("node_modules"))?;
        if dependency_tree_sha256 != Self::NODE_MODULES_TREE_SHA256 {
            return Err(OpenCodeSdkError::InvalidIntegrity);
        }
        Ok(OpenCodeSdkIntegrityReceipt {
            sidecar_root: root,
            dependency_tree_sha256,
        })
    }

    pub fn pinned_file_names() -> impl Iterator<Item = &'static str> {
        Self::PINNED_FILES.into_iter().map(|(name, _, _)| name)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenCodeSdkIntegrityReceipt {
    sidecar_root: PathBuf,
    dependency_tree_sha256: String,
}

impl OpenCodeSdkIntegrityReceipt {
    pub fn sidecar_root(&self) -> &Path {
        &self.sidecar_root
    }

    pub fn dependency_tree_sha256(&self) -> &str {
        &self.dependency_tree_sha256
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OpenCodeSdkMaterialization {
    config_home: PathBuf,
    plugin_path: PathBuf,
    plugin_sha256: String,
    dependency_tree_sha256: String,
}

impl OpenCodeSdkMaterialization {
    pub fn config_home(&self) -> &Path {
        &self.config_home
    }

    pub fn plugin_path(&self) -> &Path {
        &self.plugin_path
    }

    pub fn plugin_sha256(&self) -> &str {
        &self.plugin_sha256
    }

    pub fn dependency_tree_sha256(&self) -> &str {
        &self.dependency_tree_sha256
    }

    pub fn sdk_version(&self) -> &'static str {
        OPENCODE_SDK_VERSION
    }

    pub fn tool_ids(&self) -> [&'static str; 2] {
        OPENCODE_SDK_TOOL_IDS
    }
}

pub fn materialize_opencode_sdk_plugin(
    sidecar_root: &Path,
    config_home: &Path,
) -> Result<OpenCodeSdkMaterialization, OpenCodeSdkError> {
    if !config_home.is_absolute() {
        return Err(OpenCodeSdkError::InvalidConfiguration);
    }
    let receipt = OpenCodeSdkIntegrity::verify(sidecar_root)?;
    fs::create_dir_all(config_home).map_err(OpenCodeSdkError::Io)?;
    let config_home = canonical_directory(config_home)?;
    let plugins = config_home.join("opencode/plugins");
    fs::create_dir_all(&plugins).map_err(OpenCodeSdkError::Io)?;
    fs::set_permissions(&plugins, fs::Permissions::from_mode(0o700))
        .map_err(OpenCodeSdkError::Io)?;
    let plugins = canonical_directory(&plugins)?;
    if !plugins.starts_with(&config_home) {
        return Err(OpenCodeSdkError::InvalidConfiguration);
    }

    let source = receipt.sidecar_root.join("c4os-tools-plugin.mjs");
    let source_url = file_url(&source)?;
    let wrapper = format!(
        "export {{ C4osBrokerToolsPlugin }} from {};\n",
        serde_json::to_string(&source_url).map_err(|_| OpenCodeSdkError::InvalidConfiguration)?
    );
    if wrapper.len() > 16 * 1024 {
        return Err(OpenCodeSdkError::InvalidConfiguration);
    }
    let plugin_path = plugins.join("c4os-tools.js");
    atomic_private_write(&plugin_path, wrapper.as_bytes())?;
    let plugin_path = plugin_path
        .canonicalize()
        .map_err(|_| OpenCodeSdkError::InvalidConfiguration)?;
    if !plugin_path.starts_with(&plugins)
        || !plugin_path
            .metadata()
            .map_err(OpenCodeSdkError::Io)?
            .is_file()
    {
        return Err(OpenCodeSdkError::InvalidConfiguration);
    }
    let plugin_sha256 =
        sha256_file(&plugin_path).map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    Ok(OpenCodeSdkMaterialization {
        config_home,
        plugin_path,
        plugin_sha256,
        dependency_tree_sha256: receipt.dependency_tree_sha256,
    })
}

#[derive(Debug)]
pub struct OpenCodeSdkLaunchBinding {
    worker_channel: UnixStream,
    test_tls_trust: Option<TestTlsTrustLaunchCapability>,
    materialization: OpenCodeSdkMaterialization,
    process_generation: u64,
}

#[derive(Debug)]
struct TestTlsTrustLaunchCapability {
    c4os_channel: UnixStream,
    worker_channel: UnixStream,
    descriptor: Vec<u8>,
}

impl OpenCodeSdkLaunchBinding {
    pub fn child_fd(&self) -> u32 {
        u32::try_from(self.worker_channel.as_raw_fd()).expect("broker descriptor is positive")
    }

    pub fn environment(&self) -> BTreeMap<String, String> {
        let mut environment = BTreeMap::from([
            (OPENCODE_BROKER_FD_ENV.into(), self.child_fd().to_string()),
            (
                OPENCODE_PROCESS_GENERATION_ENV.into(),
                self.process_generation.to_string(),
            ),
            (
                "XDG_CONFIG_HOME".into(),
                self.materialization.config_home.display().to_string(),
            ),
        ]);
        if let Some(capability) = &self.test_tls_trust {
            environment.insert(
                OPENCODE_TEST_TLS_TRUST_FD_ENV.into(),
                capability.worker_channel.as_raw_fd().to_string(),
            );
        }
        environment
    }

    pub fn materialization(&self) -> &OpenCodeSdkMaterialization {
        &self.materialization
    }

    #[doc(hidden)]
    pub fn test_tls_trust_child_fd(&self) -> Option<u32> {
        self.test_tls_trust.as_ref().map(|capability| {
            u32::try_from(capability.worker_channel.as_raw_fd())
                .expect("test TLS trust descriptor is positive")
        })
    }

    #[doc(hidden)]
    pub fn deliver_test_tls_trust(&mut self, timeout: Duration) -> Result<(), OpenCodeSdkError> {
        validate_timeout(timeout)?;
        let Some(mut capability) = self.test_tls_trust.take() else {
            return Ok(());
        };
        capability
            .c4os_channel
            .set_write_timeout(Some(timeout))
            .map_err(OpenCodeSdkError::Io)?;
        capability
            .c4os_channel
            .write_all(&capability.descriptor)
            .and_then(|_| capability.c4os_channel.shutdown(std::net::Shutdown::Write))
            .map_err(OpenCodeSdkError::Io)?;
        capability.descriptor.fill(0);
        Ok(())
    }

    /// Adds the non-secret launch metadata and makes the broker descriptor
    /// inheritable only inside the child immediately before `exec`. Call this
    /// after any `env_clear` or environment construction on the command.
    pub fn configure_command(&self, command: &mut Command) {
        command.envs(self.environment());
        let descriptor = self.worker_channel.as_raw_fd();
        let test_tls_trust_descriptor = self
            .test_tls_trust
            .as_ref()
            .map(|capability| capability.worker_channel.as_raw_fd());
        // SAFETY: the callback performs only descriptor flag syscalls against
        // a descriptor kept alive by this binding through `Command::spawn`.
        unsafe {
            command.pre_exec(move || {
                let flags = libc::fcntl(descriptor, libc::F_GETFD);
                if flags < 0
                    || libc::fcntl(descriptor, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                {
                    return Err(std::io::Error::last_os_error());
                }
                if let Some(descriptor) = test_tls_trust_descriptor {
                    let flags = libc::fcntl(descriptor, libc::F_GETFD);
                    if flags < 0
                        || libc::fcntl(descriptor, libc::F_SETFD, flags & !libc::FD_CLOEXEC) < 0
                    {
                        return Err(std::io::Error::last_os_error());
                    }
                }
                Ok(())
            });
        }
    }
}

pub fn prepare_opencode_sdk_launch(
    sidecar_root: &Path,
    config_home: &Path,
    process_generation: u64,
    test_tls_trust_descriptor: Option<Vec<u8>>,
) -> Result<(OpenCodeSdkBroker, OpenCodeSdkLaunchBinding), OpenCodeSdkError> {
    let materialization = materialize_opencode_sdk_plugin(sidecar_root, config_home)?;
    let (broker, worker_channel) = OpenCodeSdkBroker::authenticated_pair(process_generation)?;
    let test_tls_trust = test_tls_trust_descriptor
        .map(TestTlsTrustLaunchCapability::new)
        .transpose()?;
    Ok((
        broker,
        OpenCodeSdkLaunchBinding {
            worker_channel,
            test_tls_trust,
            materialization,
            process_generation,
        },
    ))
}

impl TestTlsTrustLaunchCapability {
    fn new(descriptor: Vec<u8>) -> Result<Self, OpenCodeSdkError> {
        if descriptor.is_empty() || descriptor.len() > MAX_TEST_TLS_TRUST_DESCRIPTOR_BYTES {
            return Err(OpenCodeSdkError::InvalidConfiguration);
        }
        let (c4os_channel, worker_channel) = UnixStream::pair().map_err(OpenCodeSdkError::Io)?;
        let worker_channel = duplicate_bounded_descriptor(&worker_channel)?;
        Ok(Self {
            c4os_channel,
            worker_channel,
            descriptor,
        })
    }
}

impl Drop for TestTlsTrustLaunchCapability {
    fn drop(&mut self) {
        self.descriptor.fill(0);
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BrokerProposal {
    pub correlation_id: String,
    pub tool: String,
    pub session_id: String,
    pub message_id: String,
    pub process_generation: u64,
    pub payload: Value,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrokerEvent {
    Proposal(BrokerProposal),
    Cancelled(BrokerProposal),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BrokerDecision {
    Result(Value),
    Denied { reason_code: String },
    Cancelled,
}

#[derive(Debug)]
pub struct OpenCodeSdkBroker {
    reader: BufReader<UnixStream>,
    writer: UnixStream,
    pending: BTreeMap<String, BrokerProposal>,
    process_generation: u64,
}

impl OpenCodeSdkBroker {
    pub fn authenticated_pair(
        process_generation: u64,
    ) -> Result<(Self, UnixStream), OpenCodeSdkError> {
        if process_generation == 0 || process_generation > u32::MAX.into() {
            return Err(OpenCodeSdkError::InvalidConfiguration);
        }
        let (c4os, worker) = UnixStream::pair().map_err(OpenCodeSdkError::Io)?;
        let worker = duplicate_bounded_descriptor(&worker)?;
        let writer = c4os.try_clone().map_err(OpenCodeSdkError::Io)?;
        Ok((
            Self {
                reader: BufReader::new(c4os),
                writer,
                pending: BTreeMap::new(),
                process_generation,
            },
            worker,
        ))
    }

    pub fn receive(&mut self, timeout: Duration) -> Result<BrokerEvent, OpenCodeSdkError> {
        validate_timeout(timeout)?;
        self.reader
            .get_ref()
            .set_read_timeout(Some(timeout))
            .map_err(OpenCodeSdkError::Io)?;
        let line = read_bounded_line(&mut self.reader)?;
        let request: BrokerRequest =
            serde_json::from_slice(&line).map_err(|_| OpenCodeSdkError::InvalidFrame)?;
        match request {
            BrokerRequest::Proposal {
                schema_version,
                correlation_id,
                tool,
                session_id,
                message_id,
                process_generation,
                payload,
            } => {
                validate_envelope(
                    schema_version,
                    &correlation_id,
                    &tool,
                    &session_id,
                    &message_id,
                    process_generation,
                    self.process_generation,
                )?;
                reject_secret_surface(&payload, 0)?;
                let proposal = BrokerProposal {
                    correlation_id: correlation_id.clone(),
                    tool,
                    session_id,
                    message_id,
                    process_generation,
                    payload,
                };
                if self.pending.contains_key(&correlation_id) {
                    return Err(OpenCodeSdkError::CorrelationConflict);
                }
                self.pending.insert(correlation_id, proposal.clone());
                Ok(BrokerEvent::Proposal(proposal))
            }
            BrokerRequest::Cancel {
                schema_version,
                correlation_id,
                tool,
                session_id,
                message_id,
                process_generation,
            } => {
                validate_envelope(
                    schema_version,
                    &correlation_id,
                    &tool,
                    &session_id,
                    &message_id,
                    process_generation,
                    self.process_generation,
                )?;
                let proposal = self
                    .pending
                    .get(&correlation_id)
                    .ok_or(OpenCodeSdkError::UnknownCorrelation)?;
                if proposal.tool != tool
                    || proposal.session_id != session_id
                    || proposal.message_id != message_id
                    || proposal.process_generation != process_generation
                {
                    return Err(OpenCodeSdkError::CorrelationMismatch);
                }
                let proposal = self
                    .pending
                    .remove(&correlation_id)
                    .expect("checked pending proposal");
                self.write_result(&proposal, BrokerDecision::Cancelled, timeout)?;
                Ok(BrokerEvent::Cancelled(proposal))
            }
        }
    }

    pub fn respond(
        &mut self,
        correlation_id: &str,
        decision: BrokerDecision,
        timeout: Duration,
    ) -> Result<(), OpenCodeSdkError> {
        validate_timeout(timeout)?;
        let proposal = self
            .pending
            .remove(correlation_id)
            .ok_or(OpenCodeSdkError::UnknownCorrelation)?;
        if let Err(error) = self.write_result(&proposal, decision, timeout) {
            self.pending.insert(correlation_id.to_owned(), proposal);
            return Err(error);
        }
        Ok(())
    }

    pub fn pending_count(&self) -> usize {
        self.pending.len()
    }

    fn write_result(
        &mut self,
        proposal: &BrokerProposal,
        decision: BrokerDecision,
        timeout: Duration,
    ) -> Result<(), OpenCodeSdkError> {
        let (status, payload, reason_code) = match decision {
            BrokerDecision::Result(value) => {
                reject_secret_surface(&value, 0)?;
                ("result", Some(value), None)
            }
            BrokerDecision::Denied { reason_code } => {
                if !safe_id(&reason_code) {
                    return Err(OpenCodeSdkError::InvalidFrame);
                }
                ("denied", None, Some(reason_code))
            }
            BrokerDecision::Cancelled => ("cancelled", None, None),
        };
        let response = BrokerResponse {
            schema_version: BROKER_SCHEMA_VERSION,
            kind: "result",
            correlation_id: &proposal.correlation_id,
            tool: &proposal.tool,
            status,
            payload,
            reason_code,
        };
        let mut encoded =
            serde_json::to_vec(&response).map_err(|_| OpenCodeSdkError::InvalidFrame)?;
        encoded.push(b'\n');
        if encoded.len() > MAX_FRAME_BYTES {
            return Err(OpenCodeSdkError::FrameTooLarge);
        }
        self.writer
            .set_write_timeout(Some(timeout))
            .map_err(OpenCodeSdkError::Io)?;
        self.writer
            .write_all(&encoded)
            .map_err(OpenCodeSdkError::Io)
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
enum BrokerRequest {
    #[serde(rename = "proposal", rename_all = "camelCase")]
    Proposal {
        schema_version: u32,
        correlation_id: String,
        tool: String,
        session_id: String,
        message_id: String,
        process_generation: u64,
        payload: Value,
    },
    #[serde(rename = "cancel", rename_all = "camelCase")]
    Cancel {
        schema_version: u32,
        correlation_id: String,
        tool: String,
        session_id: String,
        message_id: String,
        process_generation: u64,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BrokerResponse<'a> {
    schema_version: u32,
    kind: &'static str,
    correlation_id: &'a str,
    tool: &'a str,
    status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason_code: Option<String>,
}

#[derive(Debug, Error)]
pub enum OpenCodeSdkError {
    #[error("OpenCode SDK integrity validation failed")]
    InvalidIntegrity,
    #[error("OpenCode SDK launch configuration is invalid")]
    InvalidConfiguration,
    #[error("OpenCode SDK broker frame is invalid")]
    InvalidFrame,
    #[error("OpenCode SDK broker frame exceeded its bound")]
    FrameTooLarge,
    #[error("OpenCode SDK broker rejected a secret-bearing value")]
    SecretRejected,
    #[error("OpenCode SDK broker correlation is unknown")]
    UnknownCorrelation,
    #[error("OpenCode SDK broker correlation was reused")]
    CorrelationConflict,
    #[error("OpenCode SDK broker correlation binding did not match")]
    CorrelationMismatch,
    #[error("OpenCode SDK broker I/O failed")]
    Io(#[source] std::io::Error),
}

fn validate_envelope(
    schema_version: u32,
    correlation_id: &str,
    tool: &str,
    session_id: &str,
    message_id: &str,
    process_generation: u64,
    expected_generation: u64,
) -> Result<(), OpenCodeSdkError> {
    if schema_version != BROKER_SCHEMA_VERSION
        || !safe_id(correlation_id)
        || !OPENCODE_SDK_TOOL_IDS.contains(&tool)
        || !safe_id(session_id)
        || !safe_id(message_id)
        || process_generation != expected_generation
    {
        return Err(OpenCodeSdkError::InvalidFrame);
    }
    Ok(())
}

fn safe_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.as_bytes()[0].is_ascii_alphanumeric()
        && value.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b':' | b'@' | b'-')
        })
}

fn reject_secret_surface(value: &Value, depth: usize) -> Result<(), OpenCodeSdkError> {
    if depth > 16 {
        return Err(OpenCodeSdkError::InvalidFrame);
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => Ok(()),
        Value::String(value) => {
            if value.len() > 64 * 1024 || secret_like_value(value) {
                Err(OpenCodeSdkError::SecretRejected)
            } else {
                Ok(())
            }
        }
        Value::Array(values) => {
            if values.len() > 1_024 {
                return Err(OpenCodeSdkError::InvalidFrame);
            }
            values
                .iter()
                .try_for_each(|value| reject_secret_surface(value, depth + 1))
        }
        Value::Object(values) => {
            if values.len() > 1_024 {
                return Err(OpenCodeSdkError::InvalidFrame);
            }
            for (key, value) in values {
                if secret_key(key) {
                    return Err(OpenCodeSdkError::SecretRejected);
                }
                reject_secret_surface(value, depth + 1)?;
            }
            Ok(())
        }
    }
}

/// Produces a bounded payload that is safe for the authenticated broker
/// result channel. MCP peers may use credential-shaped field names for
/// ordinary data, but those names cannot cross the OpenCode descriptor even
/// after their values were redacted. Replace the complete field with a stable
/// opaque marker and apply the same conservative value scan used by the final
/// writer.
pub fn sanitize_broker_result_payload(value: Value) -> Value {
    sanitize_broker_result_value(value, 0)
}

fn sanitize_broker_result_value(value: Value, depth: usize) -> Value {
    if depth > 16 {
        return Value::String("<redacted-depth-limit>".into());
    }
    match value {
        Value::Null | Value::Bool(_) | Value::Number(_) => value,
        Value::String(value) => {
            if secret_like_value(&value) {
                Value::String("<redacted>".into())
            } else if value.len() > 63 * 1024 {
                let mut end = 63 * 1024;
                while !value.is_char_boundary(end) {
                    end = end.saturating_sub(1);
                }
                Value::String(format!("{}<truncated>", &value[..end]))
            } else {
                Value::String(value)
            }
        }
        Value::Array(values) => {
            let truncated = values.len() > 1_024;
            let mut sanitized = values
                .into_iter()
                .take(if truncated { 1_023 } else { 1_024 })
                .map(|value| sanitize_broker_result_value(value, depth + 1))
                .collect::<Vec<_>>();
            if truncated {
                sanitized.push(serde_json::json!({ "truncatedItems": true }));
            }
            Value::Array(sanitized)
        }
        Value::Object(values) => {
            let truncated = values.len() > 1_024;
            let mut sanitized = serde_json::Map::new();
            for (key, value) in values
                .into_iter()
                .take(if truncated { 1_023 } else { 1_024 })
            {
                if secret_key(&key) {
                    let digest = Sha256::digest(key.as_bytes());
                    let mut hex = String::with_capacity(16);
                    for byte in &digest[..8] {
                        let _ = write!(hex, "{byte:02x}");
                    }
                    sanitized.insert(
                        format!("redacted_field_{hex}"),
                        Value::String("<redacted>".into()),
                    );
                } else {
                    sanitized.insert(key, sanitize_broker_result_value(value, depth + 1));
                }
            }
            if truncated {
                sanitized.insert("truncatedFields".into(), Value::Bool(true));
            }
            Value::Object(sanitized)
        }
    }
}

fn secret_key(value: &str) -> bool {
    matches!(
        value
            .chars()
            .filter(|character| *character != '-' && *character != '_')
            .collect::<String>()
            .to_ascii_lowercase()
            .as_str(),
        "apikey" | "authorization" | "cookie" | "credential" | "password" | "secret" | "token"
    )
}

fn secret_like_value(value: &str) -> bool {
    let lower = value.to_ascii_lowercase();
    lower
        .split_once("bearer ")
        .is_some_and(|(_, suffix)| suffix.len() >= 8 && !suffix.contains(char::is_whitespace))
        || ["sk-", "rk-", "pk-"].iter().any(|prefix| {
            lower
                .find(prefix)
                .is_some_and(|index| lower[index + 3..].len() >= 12)
        })
}

fn read_bounded_line(reader: &mut impl BufRead) -> Result<Vec<u8>, OpenCodeSdkError> {
    let mut line = Vec::with_capacity(4 * 1024);
    loop {
        let available = reader.fill_buf().map_err(OpenCodeSdkError::Io)?;
        if available.is_empty() {
            return Err(OpenCodeSdkError::InvalidFrame);
        }
        let take = available
            .iter()
            .position(|byte| *byte == b'\n')
            .map_or(available.len(), |position| position + 1);
        if line.len().saturating_add(take) > MAX_FRAME_BYTES {
            return Err(OpenCodeSdkError::FrameTooLarge);
        }
        line.extend_from_slice(&available[..take]);
        reader.consume(take);
        if line.last() == Some(&b'\n') {
            line.pop();
            return Ok(line);
        }
    }
}

fn validate_timeout(timeout: Duration) -> Result<(), OpenCodeSdkError> {
    if timeout.is_zero() || timeout > MAX_IO_TIMEOUT {
        Err(OpenCodeSdkError::InvalidConfiguration)
    } else {
        Ok(())
    }
}

fn canonical_directory(path: &Path) -> Result<PathBuf, OpenCodeSdkError> {
    let canonical = path
        .canonicalize()
        .map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    if !canonical.metadata().map_err(OpenCodeSdkError::Io)?.is_dir() {
        return Err(OpenCodeSdkError::InvalidIntegrity);
    }
    Ok(canonical)
}

fn verify_pinned_file(
    root: &Path,
    relative: &str,
    expected_sha256: &str,
    maximum_bytes: u64,
) -> Result<(), OpenCodeSdkError> {
    let file = root
        .join(relative)
        .canonicalize()
        .map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    let metadata = file
        .metadata()
        .map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    if !file.starts_with(root)
        || !metadata.is_file()
        || metadata.len() == 0
        || metadata.len() > maximum_bytes
        || sha256_file(&file).map_err(|_| OpenCodeSdkError::InvalidIntegrity)? != expected_sha256
    {
        return Err(OpenCodeSdkError::InvalidIntegrity);
    }
    Ok(())
}

fn verify_exact_lock(root: &Path) -> Result<(), OpenCodeSdkError> {
    let package: Value = serde_json::from_reader(
        File::open(root.join("package.json")).map_err(OpenCodeSdkError::Io)?,
    )
    .map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    let lock: Value = serde_json::from_reader(
        File::open(root.join("package-lock.json")).map_err(OpenCodeSdkError::Io)?,
    )
    .map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    let package_dependencies = package
        .get("dependencies")
        .and_then(Value::as_object)
        .ok_or(OpenCodeSdkError::InvalidIntegrity)?;
    if package_dependencies.len() != 2
        || package_dependencies
            .get("@opencode-ai/sdk")
            .and_then(Value::as_str)
            != Some(OPENCODE_SDK_VERSION)
        || package_dependencies
            .get("@opencode-ai/plugin")
            .and_then(Value::as_str)
            != Some(OPENCODE_SDK_PLUGIN_VERSION)
        || lock.get("lockfileVersion").and_then(Value::as_u64) != Some(3)
    {
        return Err(OpenCodeSdkError::InvalidIntegrity);
    }
    verify_lock_package(
        &lock,
        "node_modules/@opencode-ai/sdk",
        OPENCODE_SDK_VERSION,
        SDK_INTEGRITY,
    )?;
    verify_lock_package(
        &lock,
        "node_modules/@opencode-ai/plugin",
        OPENCODE_SDK_PLUGIN_VERSION,
        PLUGIN_INTEGRITY,
    )
}

fn verify_lock_package(
    lock: &Value,
    path: &str,
    version: &str,
    integrity: &str,
) -> Result<(), OpenCodeSdkError> {
    let package = lock
        .pointer(&format!(
            "/packages/{}",
            path.replace('~', "~0").replace('/', "~1")
        ))
        .and_then(Value::as_object)
        .ok_or(OpenCodeSdkError::InvalidIntegrity)?;
    if package.get("version").and_then(Value::as_str) != Some(version)
        || package.get("integrity").and_then(Value::as_str) != Some(integrity)
    {
        return Err(OpenCodeSdkError::InvalidIntegrity);
    }
    Ok(())
}

/// Deterministically hashes regular dependency files plus bounded, in-tree
/// npm links. This is public so packaging verification can reproduce the pin.
pub fn dependency_tree_sha256(node_modules: &Path) -> Result<String, OpenCodeSdkError> {
    let root = canonical_directory(node_modules)?;
    let mut entries = Vec::new();
    collect_dependency_entries(&root, &root, &mut entries)?;
    entries.sort_by(|left, right| left.0.cmp(&right.0));
    if entries.len() > MAX_TREE_FILES {
        return Err(OpenCodeSdkError::InvalidIntegrity);
    }
    let mut total_bytes = 0_u64;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 64 * 1024];
    for (relative, path, kind) in entries {
        digest.update([kind]);
        digest.update(relative.as_bytes());
        digest.update([0]);
        if kind == b'F' {
            let metadata = path.metadata().map_err(OpenCodeSdkError::Io)?;
            if metadata.len() > MAX_DEPENDENCY_FILE_BYTES {
                return Err(OpenCodeSdkError::InvalidIntegrity);
            }
            total_bytes = total_bytes
                .checked_add(metadata.len())
                .ok_or(OpenCodeSdkError::InvalidIntegrity)?;
            if total_bytes > MAX_TREE_BYTES {
                return Err(OpenCodeSdkError::InvalidIntegrity);
            }
            digest.update(metadata.len().to_be_bytes());
            let mut file = File::open(path).map_err(OpenCodeSdkError::Io)?;
            loop {
                let read = file.read(&mut buffer).map_err(OpenCodeSdkError::Io)?;
                if read == 0 {
                    break;
                }
                digest.update(&buffer[..read]);
            }
        } else {
            let target = fs::read_link(path).map_err(OpenCodeSdkError::Io)?;
            digest.update(target.as_os_str().as_bytes());
        }
        digest.update([0xff]);
    }
    let mut encoded = String::from("sha256:");
    for byte in digest.finalize() {
        write!(&mut encoded, "{byte:02x}").expect("writing a String is infallible");
    }
    Ok(encoded)
}

fn collect_dependency_entries(
    root: &Path,
    directory: &Path,
    entries: &mut Vec<(String, PathBuf, u8)>,
) -> Result<(), OpenCodeSdkError> {
    for entry in fs::read_dir(directory).map_err(OpenCodeSdkError::Io)? {
        let entry = entry.map_err(OpenCodeSdkError::Io)?;
        let path = entry.path();
        let metadata = fs::symlink_metadata(&path).map_err(OpenCodeSdkError::Io)?;
        if metadata.is_dir() {
            collect_dependency_entries(root, &path, entries)?;
        } else if metadata.is_file() {
            let relative = dependency_relative(root, &path)?;
            entries.push((relative, path, b'F'));
        } else if metadata.file_type().is_symlink() {
            let canonical = path
                .canonicalize()
                .map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
            if !canonical.starts_with(root) || !canonical.is_file() {
                return Err(OpenCodeSdkError::InvalidIntegrity);
            }
            let relative = dependency_relative(root, &path)?;
            entries.push((relative, path, b'L'));
        } else {
            return Err(OpenCodeSdkError::InvalidIntegrity);
        }
        if entries.len() > MAX_TREE_FILES {
            return Err(OpenCodeSdkError::InvalidIntegrity);
        }
    }
    Ok(())
}

fn dependency_relative(root: &Path, path: &Path) -> Result<String, OpenCodeSdkError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    let bytes = relative.as_os_str().as_bytes();
    let text = std::str::from_utf8(bytes).map_err(|_| OpenCodeSdkError::InvalidIntegrity)?;
    if text.is_empty() {
        return Err(OpenCodeSdkError::InvalidIntegrity);
    }
    Ok(text.to_owned())
}

fn atomic_private_write(path: &Path, contents: &[u8]) -> Result<(), OpenCodeSdkError> {
    let parent = path
        .parent()
        .ok_or(OpenCodeSdkError::InvalidConfiguration)?;
    let temporary = parent.join(format!(".c4os-tools-{}.tmp", Uuid::new_v4()));
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).mode(0o600);
        let mut file = options.open(&temporary).map_err(OpenCodeSdkError::Io)?;
        file.write_all(contents).map_err(OpenCodeSdkError::Io)?;
        file.sync_all().map_err(OpenCodeSdkError::Io)?;
        fs::rename(&temporary, path).map_err(OpenCodeSdkError::Io)?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn file_url(path: &Path) -> Result<String, OpenCodeSdkError> {
    let path = path
        .canonicalize()
        .map_err(|_| OpenCodeSdkError::InvalidConfiguration)?;
    let mut url = String::from("file://");
    for byte in path.as_os_str().as_bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'-' | b'.' | b'_' | b'~') {
            url.push(char::from(*byte));
        } else {
            write!(&mut url, "%{byte:02X}").expect("writing a String is infallible");
        }
    }
    Ok(url)
}

fn duplicate_bounded_descriptor(stream: &UnixStream) -> Result<UnixStream, OpenCodeSdkError> {
    // SAFETY: fcntl receives a live descriptor and returns a new owned
    // descriptor. `from_raw_fd` takes ownership exactly once on success.
    let descriptor =
        unsafe { libc::fcntl(stream.as_raw_fd(), libc::F_DUPFD_CLOEXEC, MIN_BROKER_FD) };
    if !(MIN_BROKER_FD..=MAX_BROKER_FD).contains(&descriptor) {
        if descriptor >= 0 {
            // SAFETY: descriptor was newly allocated by fcntl above.
            unsafe { libc::close(descriptor) };
        }
        return Err(OpenCodeSdkError::InvalidConfiguration);
    }
    // SAFETY: descriptor is newly owned and not represented by another owner.
    Ok(unsafe { UnixStream::from_raw_fd(descriptor) })
}

//! Hardened marketplace source resolution and immutable Git checkout storage.
//!
//! Plain absolute directories remain an explicit local-development source. Git
//! sources are acquired by the system Git client into a private staging area,
//! checked out at one full commit, stripped of repository metadata, sealed, and
//! atomically promoted. A refresh therefore publishes another immutable tree
//! instead of mutating the checkout already consumed by ExtensionService.

use std::{
    collections::BTreeSet,
    ffi::OsString,
    fs::{self, File},
    io::Read,
    path::{Component, Path, PathBuf},
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};

#[cfg(unix)]
use std::os::unix::{fs::PermissionsExt, process::CommandExt as _};

use sha2::{Digest, Sha256};
use url::Url;
use walkdir::WalkDir;

use super::ExtensionError;

const GIT_PROGRAM: &str = "/usr/bin/git";
const GIT_TIMEOUT: Duration = Duration::from_secs(60);
const GIT_TERMINATION_GRACE: Duration = Duration::from_millis(500);
const MAX_GIT_OUTPUT_BYTES: usize = 64 * 1024;
const MAX_SOURCE_BYTES: usize = 4 * 1024;
const MAX_REF_BYTES: usize = 255;
const MAX_SPARSE_PATHS: usize = 128;
const MAX_SPARSE_PATH_BYTES: usize = 1_024;
const MAX_CHECKOUT_ENTRIES: usize = 65_536;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedMarketplaceSource {
    pub checkout_root: PathBuf,
    pub display_source: String,
    pub requested_ref: Option<String>,
    /// Plain local directories are intentionally not assigned a synthetic Git
    /// identity. Every acquired Git source always returns a full object ID.
    pub resolved_commit: Option<String>,
    pub sparse_paths: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct MarketplaceSourceResolver {
    root: PathBuf,
}

impl MarketplaceSourceResolver {
    pub fn new(root: impl AsRef<Path>) -> Result<Self, ExtensionError> {
        let root = root.as_ref().to_path_buf();
        if !root.is_absolute() {
            return Err(ExtensionError::InvalidInput);
        }
        ensure_private_directory(&root)?;
        ensure_private_directory(&root.join("staging"))?;
        ensure_private_directory(&root.join("checkouts"))?;
        ensure_private_directory(&root.join("git-home"))?;
        ensure_private_directory(&root.join("git-template"))?;
        Ok(Self { root })
    }

    pub fn resolve(
        &self,
        source: &str,
        requested_ref: Option<&str>,
        sparse_paths: &[String],
    ) -> Result<ResolvedMarketplaceSource, ExtensionError> {
        validate_source_text(source)?;
        let requested_ref = requested_ref.map(validate_git_ref).transpose()?;
        let sparse_paths = normalize_sparse_paths(sparse_paths)?;
        let source = parse_source(source)?;

        if let ParsedSource::PlainDirectory { canonical } = &source {
            if requested_ref.is_none()
                && sparse_paths.is_empty()
                && !looks_like_git_repository(canonical)?
            {
                return Ok(ResolvedMarketplaceSource {
                    checkout_root: canonical.clone(),
                    display_source: canonical.display().to_string(),
                    requested_ref: None,
                    resolved_commit: None,
                    sparse_paths,
                });
            }
        }

        self.resolve_git(source, requested_ref, sparse_paths)
    }

    fn resolve_git(
        &self,
        source: ParsedSource,
        requested_ref: Option<String>,
        sparse_paths: Vec<String>,
    ) -> Result<ResolvedMarketplaceSource, ExtensionError> {
        let (git_source, display_source) = source.git_source()?;
        let staging = tempfile::Builder::new()
            .prefix("marketplace-")
            .tempdir_in(self.root.join("staging"))?;
        set_private_directory(staging.path())?;
        let checkout = staging.path().join("checkout");

        self.git(&["init".into(), "--quiet".into(), checkout.as_os_str().into()])?;
        self.git_in(
            &checkout,
            &[
                "remote".into(),
                "add".into(),
                "origin".into(),
                git_source.clone().into(),
            ],
        )?;

        let fetch_target = self.resolve_fetch_target(&checkout, requested_ref.as_deref())?;
        self.git_in(
            &checkout,
            &[
                "fetch".into(),
                "--quiet".into(),
                "--no-tags".into(),
                "--depth=1".into(),
                "origin".into(),
                fetch_target.into(),
            ],
        )?;
        let resolved_commit = self.rev_parse_commit(&checkout)?;

        if !sparse_paths.is_empty() {
            self.git_in(
                &checkout,
                &["sparse-checkout".into(), "init".into(), "--cone".into()],
            )?;
            let mut arguments = vec![
                "sparse-checkout".into(),
                "set".into(),
                "--cone".into(),
                "--".into(),
            ];
            arguments.extend(sparse_paths.iter().map(OsString::from));
            self.git_in(&checkout, &arguments)?;
        }
        self.git_in(
            &checkout,
            &[
                "checkout".into(),
                "--quiet".into(),
                "--detach".into(),
                "--force".into(),
                resolved_commit.clone().into(),
            ],
        )?;

        let git_metadata = checkout.join(".git");
        reject_symlink(&git_metadata)?;
        fs::remove_dir_all(&git_metadata)?;
        validate_checkout_tree(&checkout)?;
        let checkout_key = checkout_key(
            &display_source,
            &resolved_commit,
            requested_ref.as_deref(),
            &sparse_paths,
        );
        let target = self.root.join("checkouts").join(checkout_key);
        if target.exists() {
            validate_published_checkout(&target)?;
        } else {
            sync_tree(&checkout)?;
            seal_tree_for_promotion(&checkout)?;
            if let Err(error) = fs::rename(&checkout, &target) {
                if !target.exists() {
                    make_tree_removable(&checkout);
                    return Err(error.into());
                }
                make_tree_removable(&checkout);
                validate_published_checkout(&target)?;
            } else {
                if let Err(error) = seal_promoted_root(&target) {
                    make_tree_removable(&target);
                    let _ = fs::remove_dir_all(&target);
                    return Err(error);
                }
                sync_directory(&target)?;
                sync_directory(target.parent().ok_or(ExtensionError::InvalidState)?)?;
            }
        }

        Ok(ResolvedMarketplaceSource {
            checkout_root: target,
            display_source,
            requested_ref,
            resolved_commit: Some(resolved_commit),
            sparse_paths,
        })
    }

    fn resolve_fetch_target(
        &self,
        checkout: &Path,
        requested_ref: Option<&str>,
    ) -> Result<String, ExtensionError> {
        let Some(requested_ref) = requested_ref else {
            let output = self.git_in_output(
                checkout,
                &[
                    "ls-remote".into(),
                    "--symref".into(),
                    "origin".into(),
                    "HEAD".into(),
                ],
            )?;
            require_remote_object(&output, "HEAD")?;
            return Ok("HEAD".into());
        };
        if is_full_object_id(requested_ref) {
            return Ok(requested_ref.into());
        }
        if requested_ref.starts_with("refs/heads/") || requested_ref.starts_with("refs/tags/") {
            let output = self.git_in_output(
                checkout,
                &[
                    "ls-remote".into(),
                    "--refs".into(),
                    "origin".into(),
                    requested_ref.into(),
                ],
            )?;
            require_exact_remote_ref(&output, requested_ref)?;
            return Ok(requested_ref.into());
        }

        let branch = format!("refs/heads/{requested_ref}");
        let tag = format!("refs/tags/{requested_ref}");
        let output = self.git_in_output(
            checkout,
            &[
                "ls-remote".into(),
                "--refs".into(),
                "origin".into(),
                branch.clone().into(),
                tag.clone().into(),
            ],
        )?;
        let branch_exists = remote_ref_exists(&output, &branch)?;
        let tag_exists = remote_ref_exists(&output, &tag)?;
        match (branch_exists, tag_exists) {
            (true, false) => Ok(branch),
            (false, true) => Ok(tag),
            _ => Err(ExtensionError::InvalidInput),
        }
    }

    fn rev_parse_commit(&self, checkout: &Path) -> Result<String, ExtensionError> {
        let output = self.git_in_output(
            checkout,
            &[
                "rev-parse".into(),
                "--verify".into(),
                "FETCH_HEAD^{commit}".into(),
            ],
        )?;
        let commit = std::str::from_utf8(&output)
            .map_err(|_| ExtensionError::VerificationFailed)?
            .trim();
        if !is_full_object_id(commit) {
            return Err(ExtensionError::VerificationFailed);
        }
        Ok(commit.to_ascii_lowercase())
    }

    fn git(&self, arguments: &[OsString]) -> Result<(), ExtensionError> {
        self.run_git(&self.root, arguments).map(|_| ())
    }

    fn git_in(&self, checkout: &Path, arguments: &[OsString]) -> Result<(), ExtensionError> {
        self.git_in_output(checkout, arguments).map(|_| ())
    }

    fn git_in_output(
        &self,
        checkout: &Path,
        arguments: &[OsString],
    ) -> Result<Vec<u8>, ExtensionError> {
        if !checkout.is_absolute() {
            return Err(ExtensionError::InvalidState);
        }
        let mut scoped = vec!["-C".into(), checkout.as_os_str().into()];
        scoped.extend_from_slice(arguments);
        self.run_git(&self.root, &scoped)
    }

    fn run_git(
        &self,
        current_dir: &Path,
        arguments: &[OsString],
    ) -> Result<Vec<u8>, ExtensionError> {
        if !current_dir.is_absolute() {
            return Err(ExtensionError::InvalidState);
        }
        let mut command = Command::new(GIT_PROGRAM);
        command
            .arg("-c")
            .arg("credential.helper=")
            .arg("-c")
            .arg("core.hooksPath=/dev/null")
            .arg("-c")
            .arg("core.fsmonitor=false")
            .arg("-c")
            .arg("core.pager=cat")
            .arg("-c")
            .arg("protocol.file.allow=always")
            .arg("-c")
            .arg("submodule.recurse=false")
            .arg("-c")
            .arg("fetch.recurseSubmodules=false")
            .arg("-c")
            .arg("advice.detachedHead=false")
            .args(arguments)
            .current_dir(current_dir)
            .env_clear()
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GCM_INTERACTIVE", "never")
            .env("GIT_ASKPASS", "/usr/bin/false")
            .env("SSH_ASKPASS", "/usr/bin/false")
            .env("GIT_SSH_COMMAND", "/usr/bin/false")
            .env("GIT_PAGER", "cat")
            .env("GIT_TEMPLATE_DIR", self.root.join("git-template"))
            .env("HOME", self.root.join("git-home"))
            .env("LC_ALL", "C")
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(unix)]
        command.process_group(0);
        let mut child = command.spawn().map_err(|_| ExtensionError::Unavailable)?;
        let process_group_id = child.id();
        let stdout = child.stdout.take().ok_or(ExtensionError::Unavailable)?;
        let stderr = child.stderr.take().ok_or(ExtensionError::Unavailable)?;
        let stdout_reader = thread::spawn(move || read_bounded(stdout));
        let stderr_reader = thread::spawn(move || read_bounded(stderr));
        let started = Instant::now();
        let status = loop {
            match child.try_wait() {
                Ok(Some(status)) => break status,
                Ok(None) if started.elapsed() < GIT_TIMEOUT => {
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(None) | Err(_) => {
                    terminate_process_group(process_group_id);
                    let _ = child.wait();
                    let _ = stdout_reader.join();
                    let _ = stderr_reader.join();
                    return Err(ExtensionError::Unavailable);
                }
            }
        };
        let (stdout, stdout_exceeded) = stdout_reader
            .join()
            .map_err(|_| ExtensionError::Unavailable)??;
        let (_, stderr_exceeded) = stderr_reader
            .join()
            .map_err(|_| ExtensionError::Unavailable)??;
        if stdout_exceeded || stderr_exceeded {
            return Err(ExtensionError::BoundExceeded);
        }
        if !status.success() {
            return Err(ExtensionError::Unavailable);
        }
        Ok(stdout)
    }
}

#[derive(Clone, Debug)]
enum ParsedSource {
    PlainDirectory { canonical: PathBuf },
    FileRepository { canonical: PathBuf },
    HttpsRepository { url: String },
}

impl ParsedSource {
    fn git_source(self) -> Result<(String, String), ExtensionError> {
        match self {
            Self::PlainDirectory { canonical } => {
                let source = canonical
                    .to_str()
                    .ok_or(ExtensionError::InvalidInput)?
                    .to_owned();
                Ok((source.clone(), source))
            }
            Self::FileRepository { canonical } => {
                let source = Url::from_file_path(&canonical)
                    .map_err(|_| ExtensionError::InvalidInput)?
                    .to_string();
                Ok((source.clone(), source))
            }
            Self::HttpsRepository { url } => Ok((url.clone(), url)),
        }
    }
}

fn parse_source(source: &str) -> Result<ParsedSource, ExtensionError> {
    if let Ok(url) = Url::parse(source) {
        if !url.username().is_empty()
            || url.password().is_some()
            || url.query().is_some()
            || url.fragment().is_some()
        {
            return Err(ExtensionError::InvalidInput);
        }
        return match url.scheme() {
            "https" if url.host_str().is_some() => Ok(ParsedSource::HttpsRepository {
                url: url.to_string(),
            }),
            "file"
                if url
                    .host_str()
                    .is_none_or(|host| host.is_empty() || host == "localhost") =>
            {
                let path = url
                    .to_file_path()
                    .map_err(|_| ExtensionError::InvalidInput)?;
                Ok(ParsedSource::FileRepository {
                    canonical: canonical_repository(&path)?,
                })
            }
            _ => Err(ExtensionError::InvalidInput),
        };
    }
    let path = PathBuf::from(source);
    if !path.is_absolute() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(ParsedSource::PlainDirectory {
        canonical: canonical_repository(&path)?,
    })
}

fn validate_source_text(source: &str) -> Result<(), ExtensionError> {
    if source.is_empty()
        || source.len() > MAX_SOURCE_BYTES
        || source.trim() != source
        || source.chars().any(char::is_control)
        || source.starts_with('-')
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn validate_git_ref(value: &str) -> Result<String, ExtensionError> {
    if value.is_empty()
        || value.len() > MAX_REF_BYTES
        || value.trim() != value
        || value.starts_with('-')
        || value == "@"
        || value == "HEAD"
        || value.starts_with('/')
        || value.ends_with('/')
        || value.ends_with('.')
        || value.contains("//")
        || value.contains("..")
        || value.contains("@{")
        || value
            .chars()
            .any(|character| character.is_control() || " ~^:?*[\\".contains(character))
    {
        return Err(ExtensionError::InvalidInput);
    }
    if value.starts_with("refs/")
        && !value.starts_with("refs/heads/")
        && !value.starts_with("refs/tags/")
    {
        return Err(ExtensionError::InvalidInput);
    }
    for component in value.split('/') {
        if component.is_empty()
            || component == "."
            || component == ".."
            || component.starts_with('.')
            || component.ends_with(".lock")
        {
            return Err(ExtensionError::InvalidInput);
        }
    }
    Ok(value.into())
}

fn normalize_sparse_paths(values: &[String]) -> Result<Vec<String>, ExtensionError> {
    if values.len() > MAX_SPARSE_PATHS {
        return Err(ExtensionError::BoundExceeded);
    }
    let mut normalized = BTreeSet::new();
    for value in values {
        if value.is_empty()
            || value.len() > MAX_SPARSE_PATH_BYTES
            || value.trim() != value
            || value.contains('\\')
            || value.contains("//")
            || value
                .chars()
                .any(|character| character.is_control() || "*?[]!#".contains(character))
        {
            return Err(ExtensionError::InvalidInput);
        }
        let path = Path::new(value);
        if path.is_absolute() {
            return Err(ExtensionError::InvalidInput);
        }
        let mut segments = Vec::new();
        for component in path.components() {
            match component {
                Component::Normal(segment) => {
                    let segment = segment.to_str().ok_or(ExtensionError::InvalidInput)?;
                    if segment.is_empty()
                        || segment == "."
                        || segment == ".."
                        || segment.eq_ignore_ascii_case(".git")
                    {
                        return Err(ExtensionError::InvalidInput);
                    }
                    segments.push(segment);
                }
                _ => return Err(ExtensionError::InvalidInput),
            }
        }
        if segments.is_empty() {
            return Err(ExtensionError::InvalidInput);
        }
        normalized.insert(segments.join("/"));
    }
    Ok(normalized.into_iter().collect())
}

fn canonical_repository(path: &Path) -> Result<PathBuf, ExtensionError> {
    if !path.is_absolute() {
        return Err(ExtensionError::InvalidInput);
    }
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError::InvalidInput);
    }
    let canonical = path.canonicalize()?;
    let canonical_text = canonical.to_str().ok_or(ExtensionError::InvalidInput)?;
    if canonical_text.chars().any(char::is_control) {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(canonical)
}

fn looks_like_git_repository(path: &Path) -> Result<bool, ExtensionError> {
    let dot_git = path.join(".git");
    match fs::symlink_metadata(dot_git) {
        Ok(metadata) => {
            if metadata.file_type().is_symlink() || (!metadata.is_dir() && !metadata.is_file()) {
                return Err(ExtensionError::InvalidInput);
            }
            return Ok(true);
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let head = path.join("HEAD");
    let objects = path.join("objects");
    let head = match fs::symlink_metadata(head) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    let objects = match fs::symlink_metadata(objects) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error.into()),
    };
    if head.file_type().is_symlink() || objects.file_type().is_symlink() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(head.is_file() && objects.is_dir())
}

fn require_remote_object(output: &[u8], reference: &str) -> Result<(), ExtensionError> {
    if remote_ref_exists(output, reference)? {
        Ok(())
    } else {
        Err(ExtensionError::InvalidInput)
    }
}

fn require_exact_remote_ref(output: &[u8], reference: &str) -> Result<(), ExtensionError> {
    if remote_ref_exists(output, reference)? {
        Ok(())
    } else {
        Err(ExtensionError::InvalidInput)
    }
}

fn remote_ref_exists(output: &[u8], reference: &str) -> Result<bool, ExtensionError> {
    let output = std::str::from_utf8(output).map_err(|_| ExtensionError::VerificationFailed)?;
    let mut found = false;
    for line in output.lines() {
        let Some((object, remote_ref)) = line.split_once('\t') else {
            return Err(ExtensionError::VerificationFailed);
        };
        if object.starts_with("ref: ") {
            if remote_ref != "HEAD" {
                return Err(ExtensionError::VerificationFailed);
            }
            continue;
        }
        if !is_full_object_id(object) {
            return Err(ExtensionError::VerificationFailed);
        }
        if remote_ref == reference {
            if found {
                return Err(ExtensionError::Conflict);
            }
            found = true;
        }
    }
    Ok(found)
}

fn is_full_object_id(value: &str) -> bool {
    matches!(value.len(), 40 | 64) && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn checkout_key(
    source: &str,
    commit: &str,
    requested_ref: Option<&str>,
    sparse_paths: &[String],
) -> String {
    let mut digest = Sha256::new();
    for value in std::iter::once(source)
        .chain(std::iter::once(commit))
        .chain(requested_ref)
        .chain(sparse_paths.iter().map(String::as_str))
    {
        digest.update(value.len().to_be_bytes());
        digest.update(value.as_bytes());
    }
    super::sha256_prefixed(&digest.finalize()).replacen(':', "-", 1)
}

fn validate_checkout_tree(root: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError::InvalidState);
    }
    for (index, entry) in WalkDir::new(root)
        .follow_links(false)
        .same_file_system(true)
        .into_iter()
        .enumerate()
    {
        if index >= MAX_CHECKOUT_ENTRIES {
            return Err(ExtensionError::BoundExceeded);
        }
        let entry = entry.map_err(|_| ExtensionError::InvalidState)?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
            return Err(ExtensionError::InvalidInput);
        }
    }
    Ok(())
}

fn validate_published_checkout(root: &Path) -> Result<(), ExtensionError> {
    validate_checkout_tree(root)?;
    #[cfg(unix)]
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry.map_err(|_| ExtensionError::InvalidState)?;
        if fs::symlink_metadata(entry.path())?.permissions().mode() & 0o222 != 0 {
            return Err(ExtensionError::MutableContent);
        }
    }
    Ok(())
}

fn ensure_private_directory(path: &Path) -> Result<(), ExtensionError> {
    if path.exists() {
        let metadata = fs::symlink_metadata(path)?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(ExtensionError::InvalidState);
        }
    } else {
        fs::create_dir(path)?;
    }
    set_private_directory(path)
}

fn set_private_directory(path: &Path) -> Result<(), ExtensionError> {
    #[cfg(unix)]
    fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    Ok(())
}

fn reject_symlink(path: &Path) -> Result<(), ExtensionError> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn seal_tree_for_promotion(root: &Path) -> Result<(), ExtensionError> {
    for entry in WalkDir::new(root).contents_first(true).follow_links(false) {
        let entry = entry.map_err(|_| ExtensionError::InvalidState)?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir()) {
            return Err(ExtensionError::InvalidInput);
        }
        #[cfg(unix)]
        if entry.path() != root {
            fs::set_permissions(
                entry.path(),
                fs::Permissions::from_mode(if metadata.is_dir() { 0o500 } else { 0o400 }),
            )?;
        }
    }
    Ok(())
}

fn seal_promoted_root(root: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError::InvalidState);
    }
    #[cfg(unix)]
    fs::set_permissions(root, fs::Permissions::from_mode(0o500))?;
    Ok(())
}

fn make_tree_removable(root: &Path) {
    for entry in WalkDir::new(root).follow_links(false) {
        let Ok(entry) = entry else {
            continue;
        };
        let Ok(metadata) = fs::symlink_metadata(entry.path()) else {
            continue;
        };
        #[cfg(unix)]
        if !metadata.file_type().is_symlink() {
            let _ = fs::set_permissions(
                entry.path(),
                fs::Permissions::from_mode(if metadata.is_dir() { 0o700 } else { 0o600 }),
            );
        }
    }
}

fn sync_tree(root: &Path) -> Result<(), ExtensionError> {
    for entry in WalkDir::new(root).contents_first(true).follow_links(false) {
        let entry = entry.map_err(|_| ExtensionError::InvalidState)?;
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.is_file() {
            File::open(entry.path())?.sync_all()?;
        } else if metadata.is_dir() {
            sync_directory(entry.path())?;
        } else {
            return Err(ExtensionError::InvalidInput);
        }
    }
    Ok(())
}

fn sync_directory(path: &Path) -> Result<(), ExtensionError> {
    File::open(path)?.sync_all().map_err(Into::into)
}

fn read_bounded(mut pipe: impl Read) -> Result<(Vec<u8>, bool), ExtensionError> {
    let mut output = Vec::new();
    let mut exceeded = false;
    let mut buffer = [0_u8; 8 * 1024];
    loop {
        let read = pipe.read(&mut buffer)?;
        if read == 0 {
            return Ok((output, exceeded));
        }
        let remaining = MAX_GIT_OUTPUT_BYTES.saturating_sub(output.len());
        output.extend_from_slice(&buffer[..read.min(remaining)]);
        exceeded |= read > remaining;
    }
}

fn terminate_process_group(process_group_id: u32) {
    #[cfg(unix)]
    if let Ok(process_group_id) = i32::try_from(process_group_id) {
        if process_group_id > 0 {
            // SAFETY: Git was placed in a new process group immediately before
            // spawn. Negative IDs target only that exact in-memory group.
            unsafe {
                libc::kill(-process_group_id, libc::SIGTERM);
            }
            let deadline = Instant::now() + GIT_TERMINATION_GRACE;
            while Instant::now() < deadline {
                // SAFETY: signal zero only checks whether the process group is
                // still live and never dereferences process memory.
                if unsafe { libc::kill(-process_group_id, 0) } != 0 {
                    return;
                }
                thread::sleep(Duration::from_millis(10));
            }
            // SAFETY: this is the same process group created for the command.
            unsafe {
                libc::kill(-process_group_id, libc::SIGKILL);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_git(repository: &Path, arguments: &[&str]) {
        let status = Command::new(GIT_PROGRAM)
            .args(arguments)
            .current_dir(repository)
            .env_clear()
            .env("GIT_CONFIG_NOSYSTEM", "1")
            .env("GIT_CONFIG_GLOBAL", "/dev/null")
            .env("HOME", repository)
            .env("LC_ALL", "C")
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("fixture Git");
        assert!(status.success(), "fixture Git failed: {arguments:?}");
    }

    fn repository_fixture(root: &Path) -> PathBuf {
        let repository = root.join("repository");
        fs::create_dir(&repository).expect("repository");
        fixture_git(&repository, &["init", "--quiet"]);
        fixture_git(&repository, &["symbolic-ref", "HEAD", "refs/heads/main"]);
        fixture_git(&repository, &["config", "user.name", "C4OS Tests"]);
        fixture_git(
            &repository,
            &["config", "user.email", "c4os@example.invalid"],
        );
        fs::create_dir_all(repository.join("plugins/sample")).expect("plugin directory");
        fs::write(repository.join("c4os-marketplace.toml"), "catalog-v1").expect("catalog");
        fs::write(
            repository.join("plugins/sample/c4os-package.toml"),
            "package-v1",
        )
        .expect("package");
        fixture_git(&repository, &["add", "--all"]);
        fixture_git(&repository, &["commit", "--quiet", "-m", "initial"]);
        repository
    }

    #[test]
    fn plain_absolute_directory_without_git_selectors_remains_supported() {
        let temporary = tempfile::tempdir().expect("temporary");
        let source = temporary.path().join("catalog");
        let resolver_root = temporary.path().join("resolver");
        fs::create_dir(&source).expect("source");
        let resolver = MarketplaceSourceResolver::new(&resolver_root).expect("resolver");
        let resolved = resolver
            .resolve(source.to_str().expect("UTF-8 source"), None, &[])
            .expect("local source");
        assert_eq!(resolved.checkout_root, source.canonicalize().unwrap());
        assert_eq!(resolved.resolved_commit, None);
        assert!(resolved.sparse_paths.is_empty());
    }

    #[test]
    fn local_git_ref_is_promoted_by_commit_without_mutating_prior_checkout() {
        let temporary = tempfile::tempdir().expect("temporary");
        let repository = repository_fixture(temporary.path());
        let resolver =
            MarketplaceSourceResolver::new(temporary.path().join("resolver")).expect("resolver");
        let source = Url::from_file_path(&repository).unwrap().to_string();
        let sparse = vec!["plugins".to_owned()];

        let first = resolver
            .resolve(&source, Some("main"), &sparse)
            .expect("first checkout");
        assert!(first.checkout_root.join("c4os-marketplace.toml").is_file());
        assert!(
            first
                .checkout_root
                .join("plugins/sample/c4os-package.toml")
                .is_file()
        );
        assert!(!first.checkout_root.join(".git").exists());
        let first_commit = first.resolved_commit.clone().expect("commit");

        fs::write(repository.join("c4os-marketplace.toml"), "catalog-v2").expect("updated catalog");
        fixture_git(&repository, &["add", "--all"]);
        fixture_git(&repository, &["commit", "--quiet", "-m", "refresh"]);
        let second = resolver
            .resolve(&source, Some("main"), &sparse)
            .expect("second checkout");
        assert_ne!(
            second.resolved_commit.as_deref(),
            Some(first_commit.as_str())
        );
        assert_ne!(second.checkout_root, first.checkout_root);
        assert_eq!(
            fs::read_to_string(first.checkout_root.join("c4os-marketplace.toml")).unwrap(),
            "catalog-v1"
        );
        assert_eq!(
            fs::read_to_string(second.checkout_root.join("c4os-marketplace.toml")).unwrap(),
            "catalog-v2"
        );
    }

    #[test]
    fn absolute_git_repository_without_selectors_resolves_remote_head() {
        let temporary = tempfile::tempdir().expect("temporary");
        let repository = repository_fixture(temporary.path());
        let resolver =
            MarketplaceSourceResolver::new(temporary.path().join("resolver")).expect("resolver");
        let resolved = resolver
            .resolve(repository.to_str().expect("UTF-8 repository"), None, &[])
            .expect("Git source");
        assert!(resolved.resolved_commit.is_some());
        assert_ne!(resolved.checkout_root, repository);
        assert!(!resolved.checkout_root.join(".git").exists());
    }

    #[test]
    fn credentials_ambiguous_refs_and_unsafe_sparse_paths_are_rejected_preflight() {
        let temporary = tempfile::tempdir().expect("temporary");
        let resolver =
            MarketplaceSourceResolver::new(temporary.path().join("resolver")).expect("resolver");
        assert!(
            resolver
                .resolve("https://user:secret@example.com/catalog.git", None, &[])
                .is_err()
        );
        assert!(
            resolver
                .resolve("git@example.com:catalog.git", None, &[])
                .is_err()
        );
        assert!(
            resolver
                .resolve("https://example.com/catalog.git", Some("../main"), &[],)
                .is_err()
        );
        assert!(
            resolver
                .resolve(
                    "https://example.com/catalog.git",
                    None,
                    &["../plugins".into()],
                )
                .is_err()
        );
    }

    #[test]
    fn short_ref_is_rejected_when_branch_and_tag_names_collide() {
        let temporary = tempfile::tempdir().expect("temporary");
        let repository = repository_fixture(temporary.path());
        fixture_git(&repository, &["tag", "main"]);
        let resolver =
            MarketplaceSourceResolver::new(temporary.path().join("resolver")).expect("resolver");
        let source = Url::from_file_path(repository).unwrap().to_string();
        assert!(resolver.resolve(&source, Some("main"), &[]).is_err());
        assert!(
            resolver
                .resolve(&source, Some("refs/heads/main"), &[])
                .is_ok()
        );
    }

    #[cfg(unix)]
    #[test]
    fn git_symlink_content_is_rejected_before_publication() {
        let temporary = tempfile::tempdir().expect("temporary");
        let repository = repository_fixture(temporary.path());
        std::os::unix::fs::symlink("c4os-marketplace.toml", repository.join("catalog-link"))
            .expect("symlink");
        fixture_git(&repository, &["add", "--all"]);
        fixture_git(&repository, &["commit", "--quiet", "-m", "symlink"]);
        let resolver =
            MarketplaceSourceResolver::new(temporary.path().join("resolver")).expect("resolver");
        let source = Url::from_file_path(repository).unwrap().to_string();
        assert!(resolver.resolve(&source, Some("main"), &[]).is_err());
    }
}

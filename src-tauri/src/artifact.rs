use crate::storage::{AppStore, ArtifactRecord, NewArtifact};
use std::fs;
use std::path::{Path, PathBuf};

const PREVIEW_LIMIT_BYTES: u64 = 5 * 1024 * 1024;

#[derive(Debug)]
pub enum ArtifactError {
    MissingProvenance,
    UnsupportedType,
    StoreFailed(rusqlite::Error),
    IoFailed(std::io::Error),
    MissingArtifact,
    PreviewTooLarge,
}

#[derive(Debug, Eq, PartialEq)]
pub struct TextArtifactRequest<'a> {
    pub id: &'a str,
    pub project_id: &'a str,
    pub session_id: &'a str,
    pub tool_call_id: Option<&'a str>,
    pub artifact_type: &'a str,
    pub title: &'a str,
    pub mime_type: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Eq, PartialEq)]
pub struct RasterImageArtifactRequest<'a> {
    pub id: &'a str,
    pub project_id: &'a str,
    pub session_id: &'a str,
    pub tool_call_id: Option<&'a str>,
    pub title: &'a str,
    pub mime_type: &'a str,
    pub bytes: &'a [u8],
}

#[derive(Debug, Eq, PartialEq)]
pub struct ArtifactListItem {
    pub id: String,
    pub title: String,
    pub artifact_type: String,
    pub session_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub preview_available: bool,
    pub search_available: bool,
    pub export_available: bool,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ArtifactViewer {
    pub id: String,
    pub title: String,
    pub artifact_type: String,
    pub content: Option<String>,
    pub preview_kind: String,
    pub image_preview_path: Option<PathBuf>,
    pub mime_type: Option<String>,
    pub preview_unavailable_reason: Option<String>,
    pub active_rendering: bool,
    pub rich_preview: bool,
    pub export_available: bool,
    pub search_available: bool,
    pub include_in_model_context_by_default: bool,
    pub provenance: ArtifactProvenance,
}

#[derive(Debug, Eq, PartialEq)]
pub struct ArtifactProvenance {
    pub project_id: String,
    pub session_id: Option<String>,
    pub tool_call_id: Option<String>,
}

pub struct ArtifactService<'a> {
    app_store: &'a AppStore,
    artifacts_root: PathBuf,
}

impl<'a> ArtifactService<'a> {
    pub fn new(app_store: &'a AppStore, app_data_root: impl AsRef<Path>) -> Self {
        Self {
            app_store,
            artifacts_root: app_data_root.as_ref().join("artifacts"),
        }
    }

    pub fn capture_text_artifact(
        &self,
        request: TextArtifactRequest<'_>,
    ) -> Result<ArtifactRecord, ArtifactError> {
        if request.session_id.trim().is_empty() {
            return Err(ArtifactError::MissingProvenance);
        }

        if !is_supported_text_type(request.artifact_type) {
            return Err(ArtifactError::UnsupportedType);
        }

        let artifact_directory = self
            .artifacts_root
            .join(request.project_id)
            .join(request.id);
        fs::create_dir_all(&artifact_directory).map_err(ArtifactError::IoFailed)?;

        let redacted = redact_artifact_content(request.content);
        let original_path = artifact_directory.join("original");
        let preview_path = artifact_directory.join("preview");
        let metadata_path = artifact_directory.join("metadata.json");
        fs::write(&original_path, &redacted).map_err(ArtifactError::IoFailed)?;
        fs::write(&preview_path, &redacted).map_err(ArtifactError::IoFailed)?;
        fs::write(
            metadata_path,
            format!(
                r#"{{"projectId":"{}","sessionId":"{}","toolCallId":{},"activeRendering":false}}"#,
                request.project_id,
                request.session_id,
                request
                    .tool_call_id
                    .map(|id| format!(r#""{id}""#))
                    .unwrap_or_else(|| "null".into())
            ),
        )
        .map_err(ArtifactError::IoFailed)?;

        self.app_store
            .record_artifact(NewArtifact {
                id: request.id,
                project_id: request.project_id,
                session_id: Some(request.session_id),
                tool_call_id: request.tool_call_id,
                artifact_type: request.artifact_type,
                title: request.title,
                mime_type: Some(request.mime_type),
                file_path: Some(&original_path.to_string_lossy()),
                preview_path: Some(&preview_path.to_string_lossy()),
                metadata_json: r#"{"textLike":true,"activeRendering":false}"#,
            })
            .map_err(ArtifactError::StoreFailed)?;

        self.app_store
            .get_artifact(request.id)
            .map_err(ArtifactError::StoreFailed)?
            .ok_or(ArtifactError::MissingArtifact)
    }

    pub fn capture_raster_image_artifact(
        &self,
        request: RasterImageArtifactRequest<'_>,
    ) -> Result<ArtifactRecord, ArtifactError> {
        if request.session_id.trim().is_empty() {
            return Err(ArtifactError::MissingProvenance);
        }

        if !is_supported_raster_image(request.mime_type, request.bytes) {
            return Err(ArtifactError::UnsupportedType);
        }

        let artifact_directory = self
            .artifacts_root
            .join(request.project_id)
            .join(request.id);
        fs::create_dir_all(&artifact_directory).map_err(ArtifactError::IoFailed)?;

        let original_path = artifact_directory.join("original");
        let preview_path = artifact_directory.join("preview");
        let metadata_path = artifact_directory.join("metadata.json");
        fs::write(&original_path, request.bytes).map_err(ArtifactError::IoFailed)?;
        fs::write(&preview_path, request.bytes).map_err(ArtifactError::IoFailed)?;
        fs::write(
            metadata_path,
            format!(
                r#"{{"projectId":"{}","sessionId":"{}","toolCallId":{},"activeRendering":false,"previewKind":"raster_image"}}"#,
                request.project_id,
                request.session_id,
                request
                    .tool_call_id
                    .map(|id| format!(r#""{id}""#))
                    .unwrap_or_else(|| "null".into())
            ),
        )
        .map_err(ArtifactError::IoFailed)?;

        self.app_store
            .record_artifact(NewArtifact {
                id: request.id,
                project_id: request.project_id,
                session_id: Some(request.session_id),
                tool_call_id: request.tool_call_id,
                artifact_type: "raster_image",
                title: request.title,
                mime_type: Some(request.mime_type),
                file_path: Some(&original_path.to_string_lossy()),
                preview_path: Some(&preview_path.to_string_lossy()),
                metadata_json: r#"{"rasterImage":true,"activeRendering":false}"#,
            })
            .map_err(ArtifactError::StoreFailed)?;

        self.app_store
            .get_artifact(request.id)
            .map_err(ArtifactError::StoreFailed)?
            .ok_or(ArtifactError::MissingArtifact)
    }

    pub fn list_project_artifacts(
        &self,
        project_id: &str,
    ) -> Result<Vec<ArtifactListItem>, ArtifactError> {
        self.app_store
            .list_artifacts_for_project(project_id)
            .map_err(ArtifactError::StoreFailed)
            .map(|records| records.into_iter().map(project_list_item).collect())
    }

    pub fn open_artifact(&self, artifact_id: &str) -> Result<ArtifactViewer, ArtifactError> {
        let record = self
            .app_store
            .get_artifact(artifact_id)
            .map_err(ArtifactError::StoreFailed)?
            .ok_or(ArtifactError::MissingArtifact)?;
        let Some(preview_path) = record.preview_path.as_deref() else {
            return Ok(project_unavailable_viewer(
                record,
                "No passive text preview is available.",
            ));
        };
        let metadata = fs::metadata(preview_path).map_err(ArtifactError::IoFailed)?;

        if metadata.len() > PREVIEW_LIMIT_BYTES {
            return Err(ArtifactError::PreviewTooLarge);
        }

        if record.artifact_type == "raster_image" {
            return Ok(ArtifactViewer {
                id: record.id,
                title: record.title,
                artifact_type: record.artifact_type,
                content: None,
                preview_kind: "raster_image".into(),
                image_preview_path: Some(PathBuf::from(preview_path)),
                mime_type: record.mime_type,
                preview_unavailable_reason: None,
                active_rendering: false,
                rich_preview: true,
                export_available: false,
                search_available: false,
                include_in_model_context_by_default: false,
                provenance: ArtifactProvenance {
                    project_id: record.project_id,
                    session_id: record.session_id,
                    tool_call_id: record.tool_call_id,
                },
            });
        }

        let content = fs::read_to_string(preview_path).map_err(ArtifactError::IoFailed)?;

        Ok(ArtifactViewer {
            id: record.id,
            title: record.title,
            artifact_type: record.artifact_type,
            content: Some(content),
            preview_kind: "text".into(),
            image_preview_path: None,
            mime_type: record.mime_type,
            preview_unavailable_reason: None,
            active_rendering: false,
            rich_preview: false,
            export_available: false,
            search_available: false,
            include_in_model_context_by_default: false,
            provenance: ArtifactProvenance {
                project_id: record.project_id,
                session_id: record.session_id,
                tool_call_id: record.tool_call_id,
            },
        })
    }
}

fn project_list_item(record: ArtifactRecord) -> ArtifactListItem {
    ArtifactListItem {
        id: record.id,
        title: record.title,
        artifact_type: record.artifact_type,
        session_id: record.session_id,
        tool_call_id: record.tool_call_id,
        preview_available: record.preview_path.is_some(),
        search_available: false,
        export_available: false,
    }
}

fn project_unavailable_viewer(record: ArtifactRecord, reason: &str) -> ArtifactViewer {
    ArtifactViewer {
        id: record.id,
        title: record.title,
        artifact_type: record.artifact_type,
        content: None,
        preview_kind: "unavailable".into(),
        image_preview_path: None,
        mime_type: record.mime_type,
        preview_unavailable_reason: Some(reason.into()),
        active_rendering: false,
        rich_preview: false,
        export_available: false,
        search_available: false,
        include_in_model_context_by_default: false,
        provenance: ArtifactProvenance {
            project_id: record.project_id,
            session_id: record.session_id,
            tool_call_id: record.tool_call_id,
        },
    }
}

fn is_supported_text_type(artifact_type: &str) -> bool {
    matches!(
        artifact_type,
        "text" | "markdown" | "log" | "diff" | "source" | "config"
    )
}

fn is_supported_raster_image(mime_type: &str, bytes: &[u8]) -> bool {
    match mime_type {
        "image/png" => bytes.starts_with(&[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n']),
        "image/jpeg" => bytes.starts_with(&[0xff, 0xd8, 0xff]),
        "image/webp" => bytes.len() >= 12 && bytes.starts_with(b"RIFF") && &bytes[8..12] == b"WEBP",
        "image/gif" => bytes.starts_with(b"GIF87a") || bytes.starts_with(b"GIF89a"),
        _ => false,
    }
}

fn redact_artifact_content(content: &str) -> String {
    content
        .lines()
        .map(|line| {
            let lower = line.to_lowercase();
            if lower.contains("api_key")
                || lower.contains("apikey")
                || lower.contains("password")
                || lower.contains("secret")
                || lower.contains("token")
                || line.contains("sk-")
            {
                "[REDACTED_SECRET_LINE]".into()
            } else {
                line.into()
            }
        })
        .collect::<Vec<String>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewProject, NewSession, NewToolCall};
    use tempfile::tempdir;

    #[test]
    fn text_artifact_persists_and_reopens_after_restart() {
        let directory = tempdir().expect("tempdir");
        let database_path = directory.path().join("c4os.sqlite");

        {
            let store = AppStore::open(&database_path).expect("store opens");
            prepared_records(&store);
            let service = ArtifactService::new(&store, directory.path());

            service
                .capture_text_artifact(request("artifact-1", "text", "notes.txt", "hello"))
                .expect("artifact captured");
        }

        let reopened = AppStore::open(&database_path).expect("store reopens");
        let service = ArtifactService::new(&reopened, directory.path());
        let viewer = service.open_artifact("artifact-1").expect("artifact opens");

        assert_eq!(viewer.content, Some("hello".into()));
        assert_eq!(viewer.provenance.session_id, Some("session-1".into()));
        assert_eq!(viewer.provenance.tool_call_id, Some("tool-1".into()));
        assert!(!viewer.active_rendering);
    }

    #[test]
    fn lists_text_log_diff_source_and_config_artifacts_without_search_or_export() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        let service = ArtifactService::new(&store, directory.path());

        for artifact_type in ["text", "log", "diff", "source", "config"] {
            service
                .capture_text_artifact(request(
                    &format!("artifact-{artifact_type}"),
                    artifact_type,
                    &format!("{artifact_type}.txt"),
                    "content",
                ))
                .expect("artifact captured");
        }

        let artifacts = service
            .list_project_artifacts("project-1")
            .expect("artifacts listed");

        assert_eq!(artifacts.len(), 5);
        assert!(artifacts.iter().all(|artifact| artifact.preview_available));
        assert!(artifacts.iter().all(|artifact| !artifact.search_available));
        assert!(artifacts.iter().all(|artifact| !artifact.export_available));
    }

    #[test]
    fn active_html_is_opened_as_passive_text_without_rich_rendering() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        let service = ArtifactService::new(&store, directory.path());

        service
            .capture_text_artifact(TextArtifactRequest {
                id: "artifact-1",
                project_id: "project-1",
                session_id: "session-1",
                tool_call_id: Some("tool-1"),
                artifact_type: "text",
                title: "html.txt",
                mime_type: "text/html",
                content: "<script>alert('x')</script>",
            })
            .expect("artifact captured");

        let viewer = service.open_artifact("artifact-1").expect("artifact opens");

        assert_eq!(viewer.content, Some("<script>alert('x')</script>".into()));
        assert!(!viewer.active_rendering);
        assert!(!viewer.rich_preview);
    }

    #[test]
    fn credentials_are_redacted_before_artifact_storage() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        let service = ArtifactService::new(&store, directory.path());

        service
            .capture_text_artifact(request(
                "artifact-1",
                "log",
                "run.log",
                "OPENROUTER_API_KEY=sk-secret\nok",
            ))
            .expect("artifact captured");

        let viewer = service.open_artifact("artifact-1").expect("artifact opens");

        assert_eq!(viewer.content, Some("[REDACTED_SECRET_LINE]\nok".into()));
    }

    #[test]
    fn artifact_contents_are_not_included_in_model_context_by_default() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        let service = ArtifactService::new(&store, directory.path());

        service
            .capture_text_artifact(request("artifact-1", "markdown", "notes.md", "# Notes"))
            .expect("artifact captured");
        let viewer = service.open_artifact("artifact-1").expect("artifact opens");

        assert!(!viewer.include_in_model_context_by_default);
    }

    #[test]
    fn raster_image_artifact_opens_as_passive_local_preview() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        let service = ArtifactService::new(&store, directory.path());

        service
            .capture_raster_image_artifact(RasterImageArtifactRequest {
                id: "artifact-1",
                project_id: "project-1",
                session_id: "session-1",
                tool_call_id: Some("tool-1"),
                title: "chart.png",
                mime_type: "image/png",
                bytes: &[0x89, b'P', b'N', b'G', b'\r', b'\n', 0x1a, b'\n'],
            })
            .expect("image captured");

        let viewer = service.open_artifact("artifact-1").expect("artifact opens");

        assert_eq!(viewer.preview_kind, "raster_image");
        assert_eq!(viewer.mime_type, Some("image/png".into()));
        assert!(viewer.content.is_none());
        assert!(viewer.image_preview_path.is_some());
        assert_eq!(viewer.provenance.session_id, Some("session-1".into()));
        assert_eq!(viewer.provenance.tool_call_id, Some("tool-1".into()));
        assert!(!viewer.active_rendering);
        assert!(viewer.rich_preview);
        assert!(!viewer.search_available);
        assert!(!viewer.export_available);
        assert!(!viewer.include_in_model_context_by_default);
    }

    #[test]
    fn raster_image_artifacts_reject_svg_html_and_remote_url_inputs() {
        let directory = tempdir().expect("tempdir");
        let store = AppStore::open_in_memory().expect("store opens");
        prepared_records(&store);
        let service = ArtifactService::new(&store, directory.path());

        for (title, mime_type, bytes) in [
            ("icon.svg", "image/svg+xml", b"<svg></svg>".as_slice()),
            ("page.html", "text/html", b"<img src=x>".as_slice()),
            (
                "remote.png",
                "text/uri-list",
                b"https://example.com/image.png".as_slice(),
            ),
        ] {
            let result = service.capture_raster_image_artifact(RasterImageArtifactRequest {
                id: "artifact-1",
                project_id: "project-1",
                session_id: "session-1",
                tool_call_id: Some("tool-1"),
                title,
                mime_type,
                bytes,
            });

            assert!(matches!(result, Err(ArtifactError::UnsupportedType)));
        }
    }

    fn prepared_records(store: &AppStore) {
        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: "/tmp/example",
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .create_session(NewSession {
                id: "session-1",
                project_id: "project-1",
                title: "Run",
                status: "complete",
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");
        store
            .record_tool_call(NewToolCall {
                id: "tool-1",
                session_id: "session-1",
                message_id: None,
                tool_source: "opencode",
                tool_name: "write",
                arguments_json: "{}",
                status: "complete",
                runtime_call_ref: None,
            })
            .expect("tool inserted");
    }

    fn request<'a>(
        id: &'a str,
        artifact_type: &'a str,
        title: &'a str,
        content: &'a str,
    ) -> TextArtifactRequest<'a> {
        TextArtifactRequest {
            id,
            project_id: "project-1",
            session_id: "session-1",
            tool_call_id: Some("tool-1"),
            artifact_type,
            title,
            mime_type: "text/plain",
            content,
        }
    }
}

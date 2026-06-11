use crate::credentials::{CredentialError, CredentialStore, OPENROUTER_PROVIDER_ID};
use rusqlite::{params, Connection, OptionalExtension};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, PartialEq)]
pub struct ProjectRecord {
    pub id: String,
    pub name: String,
    pub root_path: String,
    pub default_model: Option<String>,
    pub default_agent_ref: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct SessionRecord {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub status: String,
    pub mode: String,
    pub agent_ref: Option<String>,
    pub model_id: String,
    pub runtime: String,
    pub runtime_session_ref: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct MessageRecord {
    pub id: String,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub status: String,
    pub metadata_json: String,
}

#[derive(Debug, PartialEq)]
pub struct ToolTimelineRecord {
    pub id: String,
    pub tool_name: String,
    pub status: String,
    pub output_summary: Option<String>,
    pub output_truncated: bool,
    pub redaction_applied: bool,
    pub output_summary_reason_labels: String,
    pub denial_category: Option<String>,
    pub denial_message: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct ArtifactRecord {
    pub id: String,
    pub project_id: String,
    pub session_id: Option<String>,
    pub tool_call_id: Option<String>,
    pub artifact_type: String,
    pub title: String,
    pub mime_type: Option<String>,
    pub file_path: Option<String>,
    pub preview_path: Option<String>,
    pub metadata_json: String,
}

#[derive(Debug, PartialEq)]
pub struct SessionMetadataRecord {
    pub id: String,
    pub title: String,
    pub pinned: bool,
    pub archived: bool,
}

pub struct NewProject<'a> {
    pub id: &'a str,
    pub name: &'a str,
    pub root_path: &'a str,
    pub default_model: Option<&'a str>,
    pub default_agent_ref: Option<&'a str>,
}

pub struct NewSession<'a> {
    pub id: &'a str,
    pub project_id: &'a str,
    pub title: &'a str,
    pub status: &'a str,
    pub mode: &'a str,
    pub agent_ref: Option<&'a str>,
    pub model_id: &'a str,
    pub runtime: &'a str,
    pub runtime_session_ref: Option<&'a str>,
}

pub struct NewMessage<'a> {
    pub id: &'a str,
    pub session_id: &'a str,
    pub role: &'a str,
    pub content: &'a str,
    pub status: &'a str,
    pub metadata_json: &'a str,
}

pub struct NewToolCall<'a> {
    pub id: &'a str,
    pub session_id: &'a str,
    pub message_id: Option<&'a str>,
    pub tool_source: &'a str,
    pub tool_name: &'a str,
    pub arguments_json: &'a str,
    pub status: &'a str,
    pub runtime_call_ref: Option<&'a str>,
}

pub struct NewApproval<'a> {
    pub id: &'a str,
    pub tool_call_id: &'a str,
    pub approval_source: &'a str,
    pub decision: &'a str,
    pub scope: &'a str,
    pub decided_by: &'a str,
}

pub struct NewArtifact<'a> {
    pub id: &'a str,
    pub project_id: &'a str,
    pub session_id: Option<&'a str>,
    pub tool_call_id: Option<&'a str>,
    pub artifact_type: &'a str,
    pub title: &'a str,
    pub mime_type: Option<&'a str>,
    pub file_path: Option<&'a str>,
    pub preview_path: Option<&'a str>,
    pub metadata_json: &'a str,
}

pub struct AppStore {
    connection: Connection,
}

impl AppStore {
    pub fn open(path: impl AsRef<Path>) -> rusqlite::Result<Self> {
        let connection = Connection::open(path)?;
        let store = Self { connection };

        store.initialize()?;

        Ok(store)
    }

    pub fn open_in_memory() -> rusqlite::Result<Self> {
        let connection = Connection::open_in_memory()?;
        let store = Self { connection };

        store.initialize()?;

        Ok(store)
    }

    pub fn schema_version(&self) -> rusqlite::Result<i64> {
        self.connection
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| {
                row.get::<_, Option<i64>>(0)
            })
            .map(|version| version.unwrap_or(0))
    }

    pub fn create_project(&self, project: NewProject<'_>) -> rusqlite::Result<()> {
        let now = timestamp();

        self.connection.execute(
            "
            INSERT INTO projects (
                id,
                name,
                root_path,
                default_model,
                default_agent_ref,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?6)
            ",
            params![
                project.id,
                project.name,
                project.root_path,
                project.default_model,
                project.default_agent_ref,
                now
            ],
        )?;

        Ok(())
    }

    pub fn get_project(&self, id: &str) -> rusqlite::Result<Option<ProjectRecord>> {
        self.connection
            .query_row(
                "
                SELECT id, name, root_path, default_model, default_agent_ref
                FROM projects
                WHERE id = ?1
                ",
                [id],
                |row| {
                    Ok(ProjectRecord {
                        id: row.get(0)?,
                        name: row.get(1)?,
                        root_path: row.get(2)?,
                        default_model: row.get(3)?,
                        default_agent_ref: row.get(4)?,
                    })
                },
            )
            .optional()
    }

    pub fn create_session(&self, session: NewSession<'_>) -> rusqlite::Result<()> {
        let now = timestamp();

        self.connection.execute(
            "
            INSERT INTO sessions (
                id,
                project_id,
                title,
                status,
                mode,
                agent_ref,
                model_id,
                runtime,
                runtime_session_ref,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?10)
            ",
            params![
                session.id,
                session.project_id,
                session.title,
                session.status,
                session.mode,
                session.agent_ref,
                session.model_id,
                session.runtime,
                session.runtime_session_ref,
                now
            ],
        )?;

        Ok(())
    }

    pub fn get_session(&self, id: &str) -> rusqlite::Result<Option<SessionRecord>> {
        self.connection
            .query_row(
                "
                SELECT
                    id,
                    project_id,
                    title,
                    status,
                    mode,
                    agent_ref,
                    model_id,
                    runtime,
                    runtime_session_ref
                FROM sessions
                WHERE id = ?1
                ",
                [id],
                |row| {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        title: row.get(2)?,
                        status: row.get(3)?,
                        mode: row.get(4)?,
                        agent_ref: row.get(5)?,
                        model_id: row.get(6)?,
                        runtime: row.get(7)?,
                        runtime_session_ref: row.get(8)?,
                    })
                },
            )
            .optional()
    }

    pub fn latest_session_for_project(
        &self,
        project_id: &str,
    ) -> rusqlite::Result<Option<SessionRecord>> {
        self.connection
            .query_row(
                "
                SELECT
                    id,
                    project_id,
                    title,
                    status,
                    mode,
                    agent_ref,
                    model_id,
                    runtime,
                    runtime_session_ref
                FROM sessions
                WHERE project_id = ?1
                ORDER BY updated_at DESC, created_at DESC, rowid DESC
                LIMIT 1
                ",
                [project_id],
                |row| {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        title: row.get(2)?,
                        status: row.get(3)?,
                        mode: row.get(4)?,
                        agent_ref: row.get(5)?,
                        model_id: row.get(6)?,
                        runtime: row.get(7)?,
                        runtime_session_ref: row.get(8)?,
                    })
                },
            )
            .optional()
    }

    pub fn list_sessions_for_project(
        &self,
        project_id: &str,
    ) -> rusqlite::Result<Vec<SessionRecord>> {
        let mut statement = self.connection.prepare(
            "
            SELECT
                id,
                project_id,
                title,
                status,
                mode,
                agent_ref,
                model_id,
                runtime,
                runtime_session_ref
            FROM sessions
            WHERE project_id = ?1
            ORDER BY updated_at DESC, created_at DESC, rowid DESC
            ",
        )?;

        let sessions = statement
            .query_map([project_id], |row| {
                Ok(SessionRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    title: row.get(2)?,
                    status: row.get(3)?,
                    mode: row.get(4)?,
                    agent_ref: row.get(5)?,
                    model_id: row.get(6)?,
                    runtime: row.get(7)?,
                    runtime_session_ref: row.get(8)?,
                })
            })?
            .collect();

        sessions
    }

    pub fn active_session(&self) -> rusqlite::Result<Option<SessionRecord>> {
        self.connection
            .query_row(
                "
                SELECT
                    id,
                    project_id,
                    title,
                    status,
                    mode,
                    agent_ref,
                    model_id,
                    runtime,
                    runtime_session_ref
                FROM sessions
                WHERE status IN ('running', 'waiting_for_approval')
                ORDER BY updated_at DESC, created_at DESC, rowid DESC
                LIMIT 1
                ",
                [],
                |row| {
                    Ok(SessionRecord {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        title: row.get(2)?,
                        status: row.get(3)?,
                        mode: row.get(4)?,
                        agent_ref: row.get(5)?,
                        model_id: row.get(6)?,
                        runtime: row.get(7)?,
                        runtime_session_ref: row.get(8)?,
                    })
                },
            )
            .optional()
    }

    pub fn has_active_sessions(&self) -> rusqlite::Result<bool> {
        let count: i64 = self.connection.query_row(
            "
            SELECT COUNT(*)
            FROM sessions
            WHERE status IN ('running', 'waiting_for_approval')
            ",
            [],
            |row| row.get(0),
        )?;

        Ok(count > 0)
    }

    pub fn start_openrouter_session(
        &self,
        session_id: &str,
        project_id: &str,
        title: &str,
    ) -> rusqlite::Result<()> {
        let selected_model = self
            .read_setting("provider.openrouter.selected_model")?
            .map(|value| strip_json_string(&value))
            .ok_or_else(|| {
                rusqlite::Error::InvalidParameterName("missing selected model".into())
            })?;
        let credential_reference =
            self.latest_openrouter_credential_reference()?
                .ok_or_else(|| {
                    rusqlite::Error::InvalidParameterName("missing credential reference".into())
                })?;

        self.create_session(NewSession {
            id: session_id,
            project_id,
            title,
            status: "running",
            mode: "agent",
            agent_ref: Some("default"),
            model_id: &selected_model,
            runtime: "opencode",
            runtime_session_ref: None,
        })?;
        self.upsert_adapter_ref(
            &format!("session-{session_id}-openrouter-credential"),
            Some(session_id),
            "openrouter",
            "credential_reference",
            &credential_reference,
            r#"{"capturedAtSessionStart":true}"#,
        )?;

        Ok(())
    }

    pub fn append_message(&self, message: NewMessage<'_>) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            INSERT INTO messages (
                id,
                session_id,
                role,
                content,
                status,
                metadata_json,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ",
            params![
                message.id,
                message.session_id,
                message.role,
                message.content,
                message.status,
                message.metadata_json,
                timestamp()
            ],
        )?;

        Ok(())
    }

    pub fn record_tool_call(&self, tool_call: NewToolCall<'_>) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            INSERT INTO tool_calls (
                id,
                session_id,
                message_id,
                tool_source,
                tool_name,
                arguments_json,
                status,
                started_at,
                runtime_call_ref
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            ON CONFLICT(id) DO UPDATE SET
                status = excluded.status,
                runtime_call_ref = excluded.runtime_call_ref
            ",
            params![
                tool_call.id,
                tool_call.session_id,
                tool_call.message_id,
                tool_call.tool_source,
                tool_call.tool_name,
                tool_call.arguments_json,
                tool_call.status,
                timestamp(),
                tool_call.runtime_call_ref
            ],
        )?;

        Ok(())
    }

    pub fn update_session_status(&self, session_id: &str, status: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            UPDATE sessions
            SET status = ?2,
                updated_at = ?3
            WHERE id = ?1
            ",
            params![session_id, status, timestamp()],
        )?;

        Ok(())
    }

    pub fn rename_session(&self, session_id: &str, title: &str) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            UPDATE sessions
            SET title = ?2,
                updated_at = ?3
            WHERE id = ?1
            ",
            params![session_id, title, timestamp()],
        )?;

        Ok(())
    }

    pub fn set_session_pinned(&self, session_id: &str, pinned: bool) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            UPDATE sessions
            SET pinned = ?2,
                updated_at = ?3
            WHERE id = ?1
            ",
            params![session_id, if pinned { 1 } else { 0 }, timestamp()],
        )?;

        Ok(())
    }

    pub fn set_session_archived(&self, session_id: &str, archived: bool) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            UPDATE sessions
            SET archived = ?2,
                updated_at = ?3
            WHERE id = ?1
            ",
            params![session_id, if archived { 1 } else { 0 }, timestamp()],
        )?;

        Ok(())
    }

    pub fn get_session_metadata(
        &self,
        session_id: &str,
    ) -> rusqlite::Result<Option<SessionMetadataRecord>> {
        self.connection
            .query_row(
                "
                SELECT id, title, pinned, archived
                FROM sessions
                WHERE id = ?1
                ",
                [session_id],
                |row| {
                    Ok(SessionMetadataRecord {
                        id: row.get(0)?,
                        title: row.get(1)?,
                        pinned: row.get::<_, i64>(2)? == 1,
                        archived: row.get::<_, i64>(3)? == 1,
                    })
                },
            )
            .optional()
    }

    pub fn mark_active_sessions_interrupted(&self) -> rusqlite::Result<usize> {
        self.connection.execute(
            "
            UPDATE sessions
            SET status = 'interrupted',
                updated_at = ?1
            WHERE status IN ('running', 'waiting_for_approval')
            ",
            [timestamp()],
        )
    }

    pub fn tool_call_count(&self, session_id: &str) -> rusqlite::Result<i64> {
        self.connection.query_row(
            "SELECT COUNT(*) FROM tool_calls WHERE session_id = ?1",
            [session_id],
            |row| row.get(0),
        )
    }

    pub fn list_tool_timeline(
        &self,
        session_id: &str,
    ) -> rusqlite::Result<Vec<ToolTimelineRecord>> {
        let mut statement = self.connection.prepare(
            "
            SELECT
                id,
                tool_name,
                status,
                output_summary,
                output_truncated,
                redaction_applied,
                output_summary_reason_labels,
                denial_category,
                denial_message
            FROM tool_calls
            WHERE session_id = ?1
            ORDER BY started_at ASC, id ASC
            ",
        )?;
        let rows = statement.query_map([session_id], |row| {
            Ok(ToolTimelineRecord {
                id: row.get(0)?,
                tool_name: row.get(1)?,
                status: row.get(2)?,
                output_summary: row.get(3)?,
                output_truncated: row.get::<_, i64>(4)? == 1,
                redaction_applied: row.get::<_, i64>(5)? == 1,
                output_summary_reason_labels: row.get(6)?,
                denial_category: row.get(7)?,
                denial_message: row.get(8)?,
            })
        })?;

        rows.collect()
    }

    pub fn update_tool_summary(
        &self,
        tool_call_id: &str,
        summary: &str,
        labels_json: &str,
        output_truncated: bool,
        redaction_applied: bool,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            UPDATE tool_calls
            SET output_summary = ?2,
                output_summary_reason_labels = ?3,
                output_truncated = ?4,
                redaction_applied = ?5
            WHERE id = ?1
            ",
            params![
                tool_call_id,
                summary,
                labels_json,
                if output_truncated { 1 } else { 0 },
                if redaction_applied { 1 } else { 0 }
            ],
        )?;

        Ok(())
    }

    pub fn record_approval(&self, approval: NewApproval<'_>) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            INSERT INTO approvals (
                id,
                tool_call_id,
                approval_source,
                decision,
                scope,
                decided_by,
                decided_at,
                expires_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, NULL)
            ",
            params![
                approval.id,
                approval.tool_call_id,
                approval.approval_source,
                approval.decision,
                approval.scope,
                approval.decided_by,
                timestamp()
            ],
        )?;

        Ok(())
    }

    pub fn approval_count(&self) -> rusqlite::Result<i64> {
        self.connection
            .query_row("SELECT COUNT(*) FROM approvals", [], |row| row.get(0))
    }

    pub fn record_artifact(&self, artifact: NewArtifact<'_>) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            INSERT INTO artifacts (
                id,
                project_id,
                session_id,
                tool_call_id,
                type,
                title,
                mime_type,
                file_path,
                preview_path,
                metadata_json,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
            ",
            params![
                artifact.id,
                artifact.project_id,
                artifact.session_id,
                artifact.tool_call_id,
                artifact.artifact_type,
                artifact.title,
                artifact.mime_type,
                artifact.file_path,
                artifact.preview_path,
                artifact.metadata_json,
                timestamp()
            ],
        )?;

        Ok(())
    }

    pub fn get_artifact(&self, id: &str) -> rusqlite::Result<Option<ArtifactRecord>> {
        self.connection
            .query_row(
                "
                SELECT
                    id,
                    project_id,
                    session_id,
                    tool_call_id,
                    type,
                    title,
                    mime_type,
                    file_path,
                    preview_path,
                    metadata_json
                FROM artifacts
                WHERE id = ?1
                ",
                [id],
                |row| {
                    Ok(ArtifactRecord {
                        id: row.get(0)?,
                        project_id: row.get(1)?,
                        session_id: row.get(2)?,
                        tool_call_id: row.get(3)?,
                        artifact_type: row.get(4)?,
                        title: row.get(5)?,
                        mime_type: row.get(6)?,
                        file_path: row.get(7)?,
                        preview_path: row.get(8)?,
                        metadata_json: row.get(9)?,
                    })
                },
            )
            .optional()
    }

    pub fn list_artifacts_for_project(
        &self,
        project_id: &str,
    ) -> rusqlite::Result<Vec<ArtifactRecord>> {
        let mut statement = self.connection.prepare(
            "
            SELECT
                id,
                project_id,
                session_id,
                tool_call_id,
                type,
                title,
                mime_type,
                file_path,
                preview_path,
                metadata_json
            FROM artifacts
            WHERE project_id = ?1
            ORDER BY created_at DESC, rowid DESC
            ",
        )?;

        let artifacts = statement
            .query_map([project_id], |row| {
                Ok(ArtifactRecord {
                    id: row.get(0)?,
                    project_id: row.get(1)?,
                    session_id: row.get(2)?,
                    tool_call_id: row.get(3)?,
                    artifact_type: row.get(4)?,
                    title: row.get(5)?,
                    mime_type: row.get(6)?,
                    file_path: row.get(7)?,
                    preview_path: row.get(8)?,
                    metadata_json: row.get(9)?,
                })
            })?
            .collect();

        artifacts
    }

    pub fn list_messages(&self, session_id: &str) -> rusqlite::Result<Vec<MessageRecord>> {
        let mut statement = self.connection.prepare(
            "
            SELECT id, session_id, role, content, status, metadata_json
            FROM messages
            WHERE session_id = ?1
            ORDER BY created_at ASC, rowid ASC
            ",
        )?;

        let messages = statement
            .query_map([session_id], |row| {
                Ok(MessageRecord {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    status: row.get(4)?,
                    metadata_json: row.get(5)?,
                })
            })?
            .collect();

        messages
    }

    pub fn set_setting(&self, key: &str, value_json: &str) -> rusqlite::Result<()> {
        reject_secret_setting(key)?;

        let now = timestamp();

        self.connection.execute(
            "
            INSERT INTO settings (id, key, value_json, created_at, updated_at)
            VALUES (?1, ?1, ?2, ?3, ?3)
            ON CONFLICT(key) DO UPDATE SET
                value_json = excluded.value_json,
                updated_at = excluded.updated_at
            ",
            params![key, value_json, now],
        )?;

        Ok(())
    }

    pub fn save_openrouter_credential(
        &self,
        credential_store: &impl CredentialStore,
        key: &str,
    ) -> Result<String, ProviderCredentialError> {
        let credential_reference = credential_store
            .save_openrouter_key(key)
            .map_err(ProviderCredentialError::CredentialStore)?;

        self.set_setting(
            "provider.openrouter.ready",
            r#"{"ready":true,"credentialStorage":"keychain"}"#,
        )?;
        self.upsert_adapter_ref(
            "provider-openrouter-credential",
            None,
            OPENROUTER_PROVIDER_ID,
            "credential_reference",
            &credential_reference.reference,
            r#"{"secretStoredOutsideSqlite":true}"#,
        )?;

        Ok(credential_reference.reference)
    }

    pub fn read_setting(&self, key: &str) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row(
                "SELECT value_json FROM settings WHERE key = ?1",
                [key],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn latest_openrouter_credential_reference(&self) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row(
                "
                SELECT ref_value
                FROM adapter_refs
                WHERE id = 'provider-openrouter-credential'
                ",
                [],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn session_credential_reference(
        &self,
        session_id: &str,
    ) -> rusqlite::Result<Option<String>> {
        self.connection
            .query_row(
                "
                SELECT ref_value
                FROM adapter_refs
                WHERE session_id = ?1
                    AND adapter = 'openrouter'
                    AND ref_type = 'credential_reference'
                ",
                [session_id],
                |row| row.get(0),
            )
            .optional()
    }

    pub fn record_adapter_ref(
        &self,
        id: &str,
        session_id: Option<&str>,
        adapter: &str,
        ref_type: &str,
        ref_value: &str,
        metadata_json: &str,
    ) -> rusqlite::Result<()> {
        self.connection.execute(
            "
            INSERT INTO adapter_refs (
                id,
                session_id,
                adapter,
                ref_type,
                ref_value,
                metadata_json,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            ON CONFLICT(id) DO UPDATE SET
                session_id = excluded.session_id,
                adapter = excluded.adapter,
                ref_type = excluded.ref_type,
                ref_value = excluded.ref_value,
                metadata_json = excluded.metadata_json
            ",
            params![
                id,
                session_id,
                adapter,
                ref_type,
                ref_value,
                metadata_json,
                timestamp()
            ],
        )?;

        Ok(())
    }

    fn upsert_adapter_ref(
        &self,
        id: &str,
        session_id: Option<&str>,
        adapter: &str,
        ref_type: &str,
        ref_value: &str,
        metadata_json: &str,
    ) -> rusqlite::Result<()> {
        self.record_adapter_ref(id, session_id, adapter, ref_type, ref_value, metadata_json)
    }

    fn initialize(&self) -> rusqlite::Result<()> {
        self.connection.pragma_update(None, "foreign_keys", "ON")?;
        self.connection.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                applied_at INTEGER NOT NULL
            );
            ",
        )?;

        let current_version = self.schema_version()?;

        if current_version < 1 {
            self.apply_migration_001()?;
        }

        if current_version < 2 {
            self.apply_migration_002()?;
        }

        Ok(())
    }

    fn apply_migration_001(&self) -> rusqlite::Result<()> {
        let transaction = self.connection.unchecked_transaction()?;

        transaction.execute_batch(MIGRATION_001)?;
        transaction.execute(
            "
            INSERT INTO schema_migrations (version, name, applied_at)
            VALUES (?1, ?2, ?3)
            ",
            params![1, "mvp_core_tables", timestamp()],
        )?;
        transaction.commit()?;

        Ok(())
    }

    fn apply_migration_002(&self) -> rusqlite::Result<()> {
        let transaction = self.connection.unchecked_transaction()?;

        transaction.execute_batch(MIGRATION_002)?;
        transaction.execute(
            "
            INSERT INTO schema_migrations (version, name, applied_at)
            VALUES (?1, ?2, ?3)
            ",
            params![2, "session_metadata_management", timestamp()],
        )?;
        transaction.commit()?;

        Ok(())
    }
}

#[derive(Debug)]
pub enum ProviderCredentialError {
    CredentialStore(CredentialError),
    MetadataStore(rusqlite::Error),
}

impl From<rusqlite::Error> for ProviderCredentialError {
    fn from(error: rusqlite::Error) -> Self {
        Self::MetadataStore(error)
    }
}

fn reject_secret_setting(key: &str) -> rusqlite::Result<()> {
    let lower_key = key.to_lowercase();
    let is_secret_key = ["api_key", "apikey", "token", "secret", "password"]
        .iter()
        .any(|marker| lower_key.contains(marker));

    if is_secret_key {
        return Err(rusqlite::Error::InvalidParameterName(format!(
            "secret-shaped setting key rejected: {key}"
        )));
    }

    Ok(())
}

fn strip_json_string(value: &str) -> String {
    value
        .trim()
        .trim_start_matches('"')
        .trim_end_matches('"')
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time is before unix epoch")
        .as_secs() as i64
}

const MIGRATION_001: &str = "
CREATE TABLE projects (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    root_path TEXT NOT NULL UNIQUE,
    default_model TEXT,
    default_agent_ref TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE sessions (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    title TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN (
        'created',
        'running',
        'waiting_for_approval',
        'stopped',
        'failed',
        'complete',
        'interrupted'
    )),
    mode TEXT NOT NULL,
    agent_ref TEXT,
    model_id TEXT NOT NULL,
    runtime TEXT NOT NULL,
    runtime_session_ref TEXT,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE RESTRICT,
    role TEXT NOT NULL CHECK (role IN ('user', 'assistant', 'system', 'tool')),
    content TEXT NOT NULL,
    status TEXT NOT NULL CHECK (status IN (
        'submitted',
        'streaming',
        'complete',
        'stopped',
        'failed'
    )),
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL
);

CREATE TRIGGER messages_append_only_update
BEFORE UPDATE ON messages
BEGIN
    SELECT RAISE(ABORT, 'messages are append-only');
END;

CREATE TRIGGER messages_append_only_delete
BEFORE DELETE ON messages
BEGIN
    SELECT RAISE(ABORT, 'messages are append-only');
END;

CREATE TABLE tool_calls (
    id TEXT PRIMARY KEY,
    session_id TEXT NOT NULL REFERENCES sessions(id) ON DELETE RESTRICT,
    message_id TEXT REFERENCES messages(id) ON DELETE RESTRICT,
    tool_source TEXT NOT NULL,
    tool_name TEXT NOT NULL,
    arguments_json TEXT NOT NULL DEFAULT '{}',
    status TEXT NOT NULL,
    started_at INTEGER NOT NULL,
    completed_at INTEGER,
    exit_code INTEGER,
    working_directory TEXT,
    output_summary TEXT,
    output_truncated INTEGER NOT NULL DEFAULT 0 CHECK (output_truncated IN (0, 1)),
    redaction_applied INTEGER NOT NULL DEFAULT 0 CHECK (redaction_applied IN (0, 1)),
    output_summary_reason_labels TEXT NOT NULL DEFAULT '[]',
    context_source_summary TEXT,
    result_json TEXT,
    denial_category TEXT,
    denial_message TEXT,
    risk_level TEXT,
    runtime_call_ref TEXT
);

CREATE TABLE approvals (
    id TEXT PRIMARY KEY,
    tool_call_id TEXT NOT NULL REFERENCES tool_calls(id) ON DELETE RESTRICT,
    approval_source TEXT NOT NULL,
    decision TEXT NOT NULL CHECK (decision IN ('allow_once', 'allow_session', 'deny', 'blocked')),
    scope TEXT NOT NULL,
    decided_by TEXT NOT NULL,
    decided_at INTEGER NOT NULL,
    expires_at INTEGER
);

CREATE TABLE artifacts (
    id TEXT PRIMARY KEY,
    project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE RESTRICT,
    session_id TEXT REFERENCES sessions(id) ON DELETE RESTRICT,
    tool_call_id TEXT REFERENCES tool_calls(id) ON DELETE RESTRICT,
    type TEXT NOT NULL,
    title TEXT NOT NULL,
    mime_type TEXT,
    file_path TEXT,
    preview_path TEXT,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL
);

CREATE TABLE models (
    id TEXT PRIMARY KEY,
    provider TEXT NOT NULL,
    model_slug TEXT NOT NULL,
    display_name TEXT NOT NULL,
    capabilities_json TEXT NOT NULL DEFAULT '{}',
    pricing_json TEXT NOT NULL DEFAULT '{}',
    context_window INTEGER,
    enabled INTEGER NOT NULL DEFAULT 1 CHECK (enabled IN (0, 1)),
    UNIQUE(provider, model_slug)
);

CREATE TABLE settings (
    id TEXT PRIMARY KEY,
    key TEXT NOT NULL UNIQUE CHECK (
        lower(key) NOT LIKE '%api_key%' AND
        lower(key) NOT LIKE '%apikey%' AND
        lower(key) NOT LIKE '%token%' AND
        lower(key) NOT LIKE '%secret%' AND
        lower(key) NOT LIKE '%password%'
    ),
    value_json TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);

CREATE TABLE diagnostics (
    id TEXT PRIMARY KEY,
    category TEXT NOT NULL,
    severity TEXT NOT NULL,
    message TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL
);

CREATE TABLE adapter_refs (
    id TEXT PRIMARY KEY,
    session_id TEXT REFERENCES sessions(id) ON DELETE RESTRICT,
    adapter TEXT NOT NULL,
    ref_type TEXT NOT NULL,
    ref_value TEXT NOT NULL,
    metadata_json TEXT NOT NULL DEFAULT '{}',
    created_at INTEGER NOT NULL
);

CREATE INDEX idx_sessions_project_id ON sessions(project_id);
CREATE INDEX idx_messages_session_id_created_at ON messages(session_id, created_at);
CREATE INDEX idx_tool_calls_session_id ON tool_calls(session_id);
CREATE INDEX idx_approvals_tool_call_id ON approvals(tool_call_id);
CREATE INDEX idx_artifacts_project_id ON artifacts(project_id);
CREATE INDEX idx_artifacts_session_id ON artifacts(session_id);
CREATE INDEX idx_diagnostics_created_at ON diagnostics(created_at);
CREATE INDEX idx_adapter_refs_session_id ON adapter_refs(session_id);
";

const MIGRATION_002: &str = "
ALTER TABLE sessions
ADD COLUMN pinned INTEGER NOT NULL DEFAULT 0 CHECK (pinned IN (0, 1));

ALTER TABLE sessions
ADD COLUMN archived INTEGER NOT NULL DEFAULT 0 CHECK (archived IN (0, 1));

CREATE INDEX idx_sessions_project_archived_pinned ON sessions(project_id, archived, pinned);
";

#[cfg(test)]
mod tests {
    use super::*;
    use crate::credentials::tests::FakeCredentialStore;
    use tempfile::tempdir;

    #[test]
    fn initializes_schema_from_clean_profile() {
        let store = AppStore::open_in_memory().expect("store opens");

        assert_eq!(store.schema_version().expect("schema version"), 2);
        assert!(table_exists(&store.connection, "projects"));
        assert!(table_exists(&store.connection, "adapter_refs"));
        assert!(column_exists(&store.connection, "sessions", "pinned"));
        assert!(column_exists(&store.connection, "sessions", "archived"));
    }

    #[test]
    fn persists_project_session_and_messages_after_reopen() {
        let directory = tempdir().expect("tempdir");
        let database_path = directory.path().join("c4os.sqlite");

        {
            let store = AppStore::open(&database_path).expect("store opens");

            store
                .create_project(NewProject {
                    id: "project-1",
                    name: "Example",
                    root_path: "/tmp/example",
                    default_model: Some("openrouter/model"),
                    default_agent_ref: None,
                })
                .expect("project inserted");
            store
                .create_session(NewSession {
                    id: "session-1",
                    project_id: "project-1",
                    title: "First session",
                    status: "created",
                    mode: "agent",
                    agent_ref: Some("default"),
                    model_id: "openrouter/model",
                    runtime: "opencode",
                    runtime_session_ref: Some("runtime-session-1"),
                })
                .expect("session inserted");
            store
                .append_message(NewMessage {
                    id: "message-1",
                    session_id: "session-1",
                    role: "user",
                    content: "hello",
                    status: "submitted",
                    metadata_json: "{}",
                })
                .expect("message inserted");
        }

        let reopened = AppStore::open(&database_path).expect("store reopens");
        let project = reopened
            .get_project("project-1")
            .expect("project query")
            .expect("project exists");
        let session = reopened
            .get_session("session-1")
            .expect("session query")
            .expect("session exists");
        let messages = reopened.list_messages("session-1").expect("messages query");

        assert_eq!(project.name, "Example");
        assert_eq!(
            session.runtime_session_ref,
            Some("runtime-session-1".into())
        );
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "hello");
    }

    #[test]
    fn messages_are_append_only() {
        let store = AppStore::open_in_memory().expect("store opens");

        insert_project_and_session(&store);
        store
            .append_message(NewMessage {
                id: "message-1",
                session_id: "session-1",
                role: "assistant",
                content: "partial",
                status: "stopped",
                metadata_json: "{}",
            })
            .expect("message inserted");

        let update_result = store.connection.execute(
            "UPDATE messages SET content = 'changed' WHERE id = 'message-1'",
            [],
        );
        let delete_result = store
            .connection
            .execute("DELETE FROM messages WHERE id = 'message-1'", []);

        assert!(update_result.is_err());
        assert!(delete_result.is_err());
    }

    #[test]
    fn session_metadata_updates_do_not_mutate_messages() {
        let store = AppStore::open_in_memory().expect("store opens");

        insert_project_and_session(&store);
        store
            .append_message(NewMessage {
                id: "message-1",
                session_id: "session-1",
                role: "assistant",
                content: "persist me",
                status: "complete",
                metadata_json: "{}",
            })
            .expect("message inserted");

        store
            .rename_session("session-1", "Renamed")
            .expect("session renamed");
        store
            .set_session_pinned("session-1", true)
            .expect("session pinned");
        store
            .set_session_archived("session-1", true)
            .expect("session archived");

        let metadata = store
            .get_session_metadata("session-1")
            .expect("metadata query")
            .expect("metadata exists");
        let messages = store.list_messages("session-1").expect("messages");
        let update_result = store.connection.execute(
            "UPDATE messages SET content = 'changed' WHERE id = 'message-1'",
            [],
        );

        assert_eq!(
            metadata,
            SessionMetadataRecord {
                id: "session-1".into(),
                title: "Renamed".into(),
                pinned: true,
                archived: true,
            }
        );
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].content, "persist me");
        assert!(update_result.is_err());
    }

    #[test]
    fn session_metadata_defaults_survive_reopen() {
        let directory = tempdir().expect("tempdir");
        let database_path = directory.path().join("c4os.sqlite");

        {
            let store = AppStore::open(&database_path).expect("store opens");
            insert_project_and_session(&store);
        }

        let reopened = AppStore::open(&database_path).expect("store reopens");
        let metadata = reopened
            .get_session_metadata("session-1")
            .expect("metadata query")
            .expect("metadata exists");

        assert_eq!(metadata.title, "First session");
        assert!(!metadata.pinned);
        assert!(!metadata.archived);
    }

    #[test]
    fn migrates_existing_v1_database_to_session_metadata_schema() {
        let directory = tempdir().expect("tempdir");
        let database_path = directory.path().join("c4os.sqlite");

        {
            let connection = Connection::open(&database_path).expect("connection opens");
            connection
                .execute_batch(
                    "
                    CREATE TABLE schema_migrations (
                        version INTEGER PRIMARY KEY,
                        name TEXT NOT NULL,
                        applied_at INTEGER NOT NULL
                    );
                    ",
                )
                .expect("migration table created");
            connection
                .execute_batch(MIGRATION_001)
                .expect("v1 schema applied");
            connection
                .execute(
                    "
                    INSERT INTO schema_migrations (version, name, applied_at)
                    VALUES (1, 'mvp_core_tables', 1)
                    ",
                    [],
                )
                .expect("v1 migration recorded");
        }

        let migrated = AppStore::open(&database_path).expect("store migrates");

        assert_eq!(migrated.schema_version().expect("schema version"), 2);
        assert!(column_exists(&migrated.connection, "sessions", "pinned"));
        assert!(column_exists(&migrated.connection, "sessions", "archived"));
    }

    #[test]
    fn rejects_secret_shaped_settings() {
        let store = AppStore::open_in_memory().expect("store opens");

        let result = store.set_setting("openrouter_api_key", "\"sk-test\"");

        assert!(result.is_err());
    }

    #[test]
    fn provider_key_save_persists_reference_without_raw_key() {
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

        let reference = store
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        let ready_setting = store
            .read_setting("provider.openrouter.ready")
            .expect("setting query")
            .expect("setting exists");

        assert_eq!(reference, "fake-keychain://openrouter/default");
        assert!(!ready_setting.contains("sk-or-test-secret"));
        assert!(!database_contains(&store.connection, "sk-or-test-secret"));
    }

    #[test]
    fn provider_key_save_fails_before_metadata_when_keychain_unavailable() {
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::failing();

        let result = store.save_openrouter_credential(&credential_store, "sk-or-test-secret");

        assert!(result.is_err());
        assert!(store
            .read_setting("provider.openrouter.ready")
            .expect("setting query")
            .is_none());
    }

    #[test]
    fn session_start_captures_provider_reference_and_model() {
        let store = AppStore::open_in_memory().expect("store opens");
        let credential_store = FakeCredentialStore::new();

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
            .save_openrouter_credential(&credential_store, "sk-or-test-secret")
            .expect("credential saved");
        store
            .set_setting("provider.openrouter.selected_model", "\"openai/gpt-4.1\"")
            .expect("model stored");
        store
            .start_openrouter_session("session-1", "project-1", "Run")
            .expect("session started");

        let session = store
            .get_session("session-1")
            .expect("session query")
            .expect("session exists");
        let credential_reference = store
            .session_credential_reference("session-1")
            .expect("reference query")
            .expect("reference exists");

        assert_eq!(session.model_id, "openai/gpt-4.1");
        assert_eq!(session.status, "running");
        assert_eq!(credential_reference, "fake-keychain://openrouter/default");
        assert!(store.has_active_sessions().expect("active session query"));
    }

    fn insert_project_and_session(store: &AppStore) {
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
                title: "First session",
                status: "created",
                mode: "agent",
                agent_ref: None,
                model_id: "openrouter/model",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");
    }

    fn table_exists(connection: &Connection, table_name: &str) -> bool {
        connection
            .query_row(
                "
                SELECT 1
                FROM sqlite_master
                WHERE type = 'table' AND name = ?1
                ",
                [table_name],
                |_| Ok(()),
            )
            .optional()
            .expect("table query")
            .is_some()
    }

    fn column_exists(connection: &Connection, table_name: &str, column_name: &str) -> bool {
        let mut statement = connection
            .prepare(&format!("PRAGMA table_info({table_name})"))
            .expect("pragma prepares");
        let columns = statement
            .query_map([], |row| row.get::<_, String>(1))
            .expect("columns query")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("columns collect");

        columns.iter().any(|column| column == column_name)
    }

    fn database_contains(connection: &Connection, needle: &str) -> bool {
        let mut statement = connection
            .prepare(
                "
                SELECT value_json FROM settings
                UNION ALL
                SELECT ref_value FROM adapter_refs
                UNION ALL
                SELECT metadata_json FROM adapter_refs
                ",
            )
            .expect("scan statement");
        let rows = statement
            .query_map([], |row| row.get::<_, String>(0))
            .expect("scan query");

        for row in rows {
            if row.expect("scan row").contains(needle) {
                return true;
            }
        }

        false
    }
}

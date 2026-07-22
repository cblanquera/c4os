//! App-database adapter for the MCP service's complete state and event CAS.

use std::sync::Arc;

use crate::{
    core::database::{DatabaseActor, DatabaseError, McpEventRecord, McpStateDocumentRecord},
    security::credentials::{CredentialVault, SecretSurface},
};

use super::{McpAuditEvent, McpError, McpServiceSnapshot, service::McpRepository};

pub struct DatabaseMcpRepository {
    database: Arc<DatabaseActor>,
    credential_vault: Option<CredentialVault>,
}

impl DatabaseMcpRepository {
    pub fn new(database: Arc<DatabaseActor>, credential_vault: Option<CredentialVault>) -> Self {
        Self {
            database,
            credential_vault,
        }
    }

    fn ensure_secret_free(&self, bytes: &[u8], surface: SecretSurface) -> Result<(), McpError> {
        if let Some(vault) = &self.credential_vault {
            vault
                .ensure_clean(surface, bytes)
                .map_err(|_| McpError::Credential)?;
        }
        Ok(())
    }
}

impl McpRepository for DatabaseMcpRepository {
    fn load(&self) -> Result<Option<McpServiceSnapshot>, McpError> {
        let Some(record) = self
            .database
            .mcp_state_document()
            .map_err(map_database_error)?
        else {
            return Ok(None);
        };
        self.ensure_secret_free(
            record.canonical_document.as_bytes(),
            SecretSurface::RendererState,
        )?;
        let snapshot: McpServiceSnapshot =
            serde_json::from_str(&record.canonical_document).map_err(|_| McpError::InvalidState)?;
        if snapshot.generation != record.generation {
            return Err(McpError::InvalidState);
        }
        snapshot.validate()?;
        Ok(Some(snapshot))
    }

    fn compare_and_swap(
        &self,
        expected_generation: Option<u64>,
        replacement: &McpServiceSnapshot,
        event: &McpAuditEvent,
    ) -> Result<(), McpError> {
        replacement.validate()?;
        if event.schema_version != replacement.schema_version
            || event.event_id != replacement.last_event_id
            || event.generation != replacement.generation
            || event.occurred_at_ms == 0
        {
            return Err(McpError::InvalidState);
        }
        let state_document =
            serde_json::to_string(replacement).map_err(|_| McpError::InvalidState)?;
        let event_document = serde_json::to_string(event).map_err(|_| McpError::InvalidState)?;
        self.ensure_secret_free(state_document.as_bytes(), SecretSurface::RendererState)?;
        self.ensure_secret_free(event_document.as_bytes(), SecretSurface::Log)?;
        self.database
            .save_mcp_transition(
                McpStateDocumentRecord {
                    generation: replacement.generation,
                    canonical_document: state_document,
                    updated_at_ms: event.occurred_at_ms,
                },
                McpEventRecord {
                    event_id: event.event_id,
                    generation: event.generation,
                    lifecycle_generation: event.lifecycle_generation,
                    operation_id: event.operation_id.clone(),
                    server_id: event.server_id.clone(),
                    event_kind: event.event_kind.clone(),
                    target: event.target.clone(),
                    result: event.result.clone(),
                    canonical_document: event_document,
                    occurred_at_ms: event.occurred_at_ms,
                },
                expected_generation,
            )
            .map_err(map_database_error)?;
        Ok(())
    }
}

fn map_database_error(error: DatabaseError) -> McpError {
    match error {
        DatabaseError::Conflict(_) => McpError::Conflict,
        DatabaseError::Validation(_) | DatabaseError::InvalidInput(_) => McpError::InvalidState,
        _ => McpError::Persistence("app database unavailable".into()),
    }
}

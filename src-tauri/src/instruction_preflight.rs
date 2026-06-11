use crate::storage::AppStore;
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct InstructionSource {
    pub source_type: String,
    pub relative_path: String,
    pub disclosed: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct InstructionDisclosure {
    pub session_id: String,
    pub sources: Vec<InstructionSource>,
    pub agents_resolution: AgentsResolution,
    pub redacted_config_summary: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct AgentsResolution {
    pub support_tier: String,
    pub permission_effect: String,
    pub model_context_effect: String,
    pub edit_reload_behavior: String,
    pub conflict_diagnostics: String,
    pub effective_stack: Vec<ResolvedAgentsInstruction>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ResolvedAgentsInstruction {
    pub relative_path: String,
    pub applies_to: String,
    pub precedence_rank: usize,
}

#[derive(Debug, Eq, PartialEq)]
pub enum InstructionPreflightError {
    ProjectRootUnavailable,
    SourceInventoryFailed,
    DisclosureStoreFailed,
}

pub struct InstructionPreflight;

impl InstructionPreflight {
    pub fn scan(
        project_root: impl AsRef<Path>,
        session_id: &str,
    ) -> Result<InstructionDisclosure, InstructionPreflightError> {
        let project_root = fs::canonicalize(project_root)
            .map_err(|_| InstructionPreflightError::ProjectRootUnavailable)?;
        let mut sources = Vec::new();
        let mut redacted_config_summary = Vec::new();

        scan_instruction_files(&project_root, &project_root, &mut sources)?;
        scan_config_file(
            &project_root,
            "opencode.json",
            &mut sources,
            &mut redacted_config_summary,
        )?;
        scan_config_file(
            &project_root,
            "opencode.jsonc",
            &mut sources,
            &mut redacted_config_summary,
        )?;

        Ok(InstructionDisclosure {
            session_id: session_id.into(),
            agents_resolution: resolve_agents_guidance_stack(&sources),
            sources,
            redacted_config_summary,
        })
    }

    pub fn persist(
        app_store: &AppStore,
        disclosure: &InstructionDisclosure,
    ) -> Result<(), InstructionPreflightError> {
        let metadata_json = serde_json::to_string(disclosure)
            .map_err(|_| InstructionPreflightError::DisclosureStoreFailed)?;

        app_store
            .record_adapter_ref(
                &format!("session-{}-instruction-disclosure", disclosure.session_id),
                Some(&disclosure.session_id),
                "opencode",
                "instruction_disclosure",
                "app-owned-preflight",
                &metadata_json,
            )
            .map_err(|_| InstructionPreflightError::DisclosureStoreFailed)?;

        Ok(())
    }
}

fn scan_instruction_files(
    project_root: &Path,
    current_dir: &Path,
    sources: &mut Vec<InstructionSource>,
) -> Result<(), InstructionPreflightError> {
    for entry in
        fs::read_dir(current_dir).map_err(|_| InstructionPreflightError::SourceInventoryFailed)?
    {
        let entry = entry.map_err(|_| InstructionPreflightError::SourceInventoryFailed)?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name == ".git" || file_name == "target" || file_name == "node_modules" {
            continue;
        }

        if path.is_dir() {
            scan_instruction_files(project_root, &path, sources)?;
            continue;
        }

        if file_name == "AGENTS.md" {
            sources.push(InstructionSource {
                source_type: "agents_md".into(),
                relative_path: relative_path(project_root, &path),
                disclosed: true,
            });
        }
    }

    Ok(())
}

fn scan_config_file(
    project_root: &Path,
    file_name: &str,
    sources: &mut Vec<InstructionSource>,
    redacted_config_summary: &mut Vec<String>,
) -> Result<(), InstructionPreflightError> {
    let path = project_root.join(file_name);

    if !path.is_file() {
        return Ok(());
    }

    sources.push(InstructionSource {
        source_type: "opencode_config".into(),
        relative_path: file_name.into(),
        disclosed: true,
    });

    let contents =
        fs::read_to_string(&path).map_err(|_| InstructionPreflightError::SourceInventoryFailed)?;
    redacted_config_summary.push(redact_secret_values(&contents));

    Ok(())
}

fn redact_secret_values(contents: &str) -> String {
    contents
        .lines()
        .map(|line| {
            let lower = line.to_lowercase();

            if lower.contains("key")
                || lower.contains("token")
                || lower.contains("secret")
                || lower.contains("password")
            {
                "[redacted secret-like config line]".into()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn relative_path(project_root: &Path, path: &Path) -> String {
    path.strip_prefix(project_root)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn resolve_agents_guidance_stack(sources: &[InstructionSource]) -> AgentsResolution {
    let mut agents_sources = sources
        .iter()
        .filter(|source| source.source_type == "agents_md")
        .map(|source| source.relative_path.clone())
        .collect::<Vec<_>>();

    agents_sources.sort_by(|left, right| {
        path_depth(left)
            .cmp(&path_depth(right))
            .then_with(|| left.cmp(right))
    });

    let effective_stack = agents_sources
        .into_iter()
        .enumerate()
        .map(|(index, relative_path)| ResolvedAgentsInstruction {
            applies_to: agents_applies_to(&relative_path),
            relative_path,
            precedence_rank: index + 1,
        })
        .collect();

    AgentsResolution {
        support_tier: "display_guidance_order_only".into(),
        permission_effect: "none".into(),
        model_context_effect: "none_without_explicit_runtime_read".into(),
        edit_reload_behavior: "next_preflight_only".into(),
        conflict_diagnostics: "ordered_sources_no_semantic_merge".into(),
        effective_stack,
    }
}

fn path_depth(relative_path: &str) -> usize {
    Path::new(relative_path).components().count()
}

fn agents_applies_to(relative_path: &str) -> String {
    Path::new(relative_path)
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(|parent| parent.to_string_lossy().to_string())
        .unwrap_or_else(|| ".".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::{AppStore, NewProject, NewSession};
    use tempfile::tempdir;

    #[test]
    fn inventories_root_and_nested_agents_files() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("AGENTS.md"), "root").expect("root agents");
        fs::create_dir(directory.path().join("nested")).expect("nested dir");
        fs::write(directory.path().join("nested/AGENTS.md"), "nested").expect("nested agents");

        let disclosure =
            InstructionPreflight::scan(directory.path(), "session-1").expect("preflight");

        assert_eq!(disclosure.sources.len(), 2);
        assert!(disclosure
            .sources
            .iter()
            .any(|source| source.relative_path == "AGENTS.md"));
        assert!(disclosure
            .sources
            .iter()
            .any(|source| source.relative_path == "nested/AGENTS.md"));
    }

    #[test]
    fn resolves_nested_agents_as_ordered_display_guidance() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("AGENTS.md"), "root").expect("root agents");
        fs::create_dir_all(directory.path().join("nested/deeper")).expect("nested dirs");
        fs::write(directory.path().join("nested/AGENTS.md"), "nested").expect("nested agents");
        fs::write(directory.path().join("nested/deeper/AGENTS.md"), "deeper")
            .expect("deeper agents");

        let disclosure =
            InstructionPreflight::scan(directory.path(), "session-1").expect("preflight");

        assert_eq!(
            disclosure.agents_resolution.support_tier,
            "display_guidance_order_only"
        );
        assert_eq!(disclosure.agents_resolution.permission_effect, "none");
        assert_eq!(
            disclosure.agents_resolution.model_context_effect,
            "none_without_explicit_runtime_read"
        );
        assert_eq!(
            disclosure.agents_resolution.conflict_diagnostics,
            "ordered_sources_no_semantic_merge"
        );
        assert_eq!(
            disclosure
                .agents_resolution
                .effective_stack
                .iter()
                .map(|entry| (
                    entry.relative_path.as_str(),
                    entry.applies_to.as_str(),
                    entry.precedence_rank,
                ))
                .collect::<Vec<_>>(),
            vec![
                ("AGENTS.md", ".", 1),
                ("nested/AGENTS.md", "nested", 2),
                ("nested/deeper/AGENTS.md", "nested/deeper", 3),
            ]
        );
    }

    #[test]
    fn redacts_secret_like_config_lines() {
        let directory = tempdir().expect("tempdir");
        fs::write(
            directory.path().join("opencode.json"),
            r#"{"instructions":["safe"],"api_key":"sk-or-secret"}"#,
        )
        .expect("config written");

        let disclosure =
            InstructionPreflight::scan(directory.path(), "session-1").expect("preflight");

        assert_eq!(disclosure.sources.len(), 1);
        assert!(!disclosure.redacted_config_summary[0].contains("sk-or-secret"));
        assert!(disclosure.redacted_config_summary[0].contains("redacted"));
    }

    #[test]
    fn persists_disclosure_as_app_owned_adapter_metadata() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("AGENTS.md"), "root").expect("root agents");
        let store = AppStore::open_in_memory().expect("store opens");

        store
            .create_project(NewProject {
                id: "project-1",
                name: "Example",
                root_path: directory.path().to_string_lossy().as_ref(),
                default_model: None,
                default_agent_ref: None,
            })
            .expect("project inserted");
        store
            .create_session(NewSession {
                id: "session-1",
                project_id: "project-1",
                title: "Run",
                status: "running",
                mode: "agent",
                agent_ref: None,
                model_id: "openai/gpt-4.1",
                runtime: "opencode",
                runtime_session_ref: None,
            })
            .expect("session inserted");

        let disclosure =
            InstructionPreflight::scan(directory.path(), "session-1").expect("preflight");

        InstructionPreflight::persist(&store, &disclosure).expect("persisted");
    }
}

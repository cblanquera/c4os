use crate::path_resolver::{
    FileAccessActor, FileAccessOperation, PathResolutionError, ProjectPathResolver,
};
use serde::Serialize;
use std::fs;
use std::path::{Path, PathBuf};

const SUPPORT_TIER: &str = "explicit_discovery_and_invocation_only";
const CONTEXT_EFFECT: &str = "explicit_user_selected_context";
const PERMISSION_EFFECT: &str = "none";

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ProjectSkillSummary {
    pub name: String,
    pub description: String,
    pub relative_path: String,
    pub support_tier: String,
    pub unsupported_capabilities: Vec<String>,
    pub invoked: bool,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ProjectSkillInvocation {
    pub name: String,
    pub description: String,
    pub relative_path: String,
    pub content: String,
    pub context_effect: String,
    pub permission_effect: String,
    pub unsupported_capabilities: Vec<String>,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ProjectSkillError {
    ProjectRootUnavailable,
    DiscoveryFailed,
    TargetUnavailable,
    OutsideProjectRoot,
    SecretDenied,
    NotSkillFile,
    ReadFailed,
}

pub struct ProjectSkillCatalog {
    project_root: PathBuf,
    resolver: ProjectPathResolver,
}

impl ProjectSkillCatalog {
    pub fn new(project_root: impl AsRef<Path>) -> Result<Self, ProjectSkillError> {
        let project_root = fs::canonicalize(project_root)
            .map_err(|_| ProjectSkillError::ProjectRootUnavailable)?;
        let resolver = ProjectPathResolver::new(&project_root).map_err(skill_path_error)?;

        Ok(Self {
            project_root,
            resolver,
        })
    }

    pub fn discover(&self) -> Result<Vec<ProjectSkillSummary>, ProjectSkillError> {
        let mut skills = Vec::new();

        self.scan_directory(&self.project_root, &mut skills)?;
        skills.sort_by(|left, right| left.relative_path.cmp(&right.relative_path));

        Ok(skills)
    }

    pub fn invoke(
        &self,
        relative_path: impl AsRef<Path>,
    ) -> Result<ProjectSkillInvocation, ProjectSkillError> {
        let resolved = self
            .resolver
            .resolve_existing(
                relative_path,
                FileAccessActor::Agent,
                FileAccessOperation::Read,
            )
            .map_err(skill_path_error)?;

        if !is_skill_file(&resolved.resolved_path) {
            return Err(ProjectSkillError::NotSkillFile);
        }

        let content = fs::read_to_string(&resolved.resolved_path)
            .map_err(|_| ProjectSkillError::ReadFailed)?;
        let metadata = parse_skill_metadata(&content, &resolved.relative_path);

        Ok(ProjectSkillInvocation {
            name: metadata.name,
            description: metadata.description,
            relative_path: path_string(&resolved.relative_path),
            content,
            context_effect: CONTEXT_EFFECT.into(),
            permission_effect: PERMISSION_EFFECT.into(),
            unsupported_capabilities: unsupported_capabilities(),
        })
    }

    fn scan_directory(
        &self,
        directory: &Path,
        skills: &mut Vec<ProjectSkillSummary>,
    ) -> Result<(), ProjectSkillError> {
        for entry in fs::read_dir(directory).map_err(|_| ProjectSkillError::DiscoveryFailed)? {
            let entry = entry.map_err(|_| ProjectSkillError::DiscoveryFailed)?;
            let path = entry.path();
            let file_name = entry.file_name().to_string_lossy().to_string();

            if should_skip_directory(&file_name) {
                continue;
            }

            if path.is_dir() {
                self.scan_directory(&path, skills)?;
                continue;
            }

            if !is_skill_file(&path) {
                continue;
            }

            let resolved = self
                .resolver
                .resolve_existing(
                    path.strip_prefix(&self.project_root)
                        .map_err(|_| ProjectSkillError::OutsideProjectRoot)?,
                    FileAccessActor::Agent,
                    FileAccessOperation::Read,
                )
                .map_err(skill_path_error)?;
            let content = fs::read_to_string(&resolved.resolved_path)
                .map_err(|_| ProjectSkillError::ReadFailed)?;
            let metadata = parse_skill_metadata(&content, &resolved.relative_path);

            skills.push(ProjectSkillSummary {
                name: metadata.name,
                description: metadata.description,
                relative_path: path_string(&resolved.relative_path),
                support_tier: SUPPORT_TIER.into(),
                unsupported_capabilities: unsupported_capabilities(),
                invoked: false,
            });
        }

        Ok(())
    }
}

struct SkillMetadata {
    name: String,
    description: String,
}

fn parse_skill_metadata(content: &str, relative_path: &Path) -> SkillMetadata {
    let mut name = file_stem_name(relative_path);
    let mut description = String::new();

    if let Some(frontmatter) = frontmatter(content) {
        for line in frontmatter.lines() {
            if let Some(value) = line.strip_prefix("name:") {
                name = clean_metadata_value(value);
            }

            if let Some(value) = line.strip_prefix("description:") {
                description = clean_metadata_value(value);
            }
        }
    }

    SkillMetadata { name, description }
}

fn frontmatter(content: &str) -> Option<&str> {
    let rest = content.strip_prefix("---\n")?;
    let end = rest.find("\n---")?;

    Some(&rest[..end])
}

fn clean_metadata_value(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn file_stem_name(relative_path: &Path) -> String {
    relative_path
        .parent()
        .and_then(Path::file_name)
        .or_else(|| relative_path.file_stem())
        .map(|value| value.to_string_lossy().to_string())
        .unwrap_or_else(|| "skill".into())
}

fn is_skill_file(path: &Path) -> bool {
    path.file_name()
        .map(|file_name| file_name == "SKILL.md")
        .unwrap_or(false)
}

fn should_skip_directory(file_name: &str) -> bool {
    matches!(file_name, ".git" | "node_modules" | "target")
}

fn unsupported_capabilities() -> Vec<String> {
    vec![
        "auto_invocation_unavailable".into(),
        "script_execution_unavailable".into(),
        "references_auto_load_unavailable".into(),
        "trusted_asset_rendering_unavailable".into(),
        "permission_effect_unavailable".into(),
        "global_catalog_unavailable".into(),
        "plugin_provided_skills_unavailable".into(),
        "version_conflict_resolution_unavailable".into(),
        "marketplace_unavailable".into(),
        "round_trip_compatibility_unavailable".into(),
    ]
}

fn path_string(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

fn skill_path_error(error: PathResolutionError) -> ProjectSkillError {
    match error {
        PathResolutionError::ProjectRootUnavailable => ProjectSkillError::ProjectRootUnavailable,
        PathResolutionError::TargetUnavailable => ProjectSkillError::TargetUnavailable,
        PathResolutionError::OutsideProjectRoot | PathResolutionError::TraversalEscapedRoot => {
            ProjectSkillError::OutsideProjectRoot
        }
        PathResolutionError::SecretDenied => ProjectSkillError::SecretDenied,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn discovers_project_local_skill_metadata_with_support_warnings() {
        let directory = tempdir().expect("tempdir");
        fs::create_dir_all(directory.path().join("skills/review")).expect("skill dir");
        fs::write(
            directory.path().join("skills/review/SKILL.md"),
            r#"---
name: code-review
description: Review code changes before merge.
---

# Code Review

Use this when reviewing code.
"#,
        )
        .expect("skill written");
        let catalog = ProjectSkillCatalog::new(directory.path()).expect("catalog");

        let skills = catalog.discover().expect("skills discovered");

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "code-review");
        assert_eq!(skills[0].description, "Review code changes before merge.");
        assert_eq!(skills[0].relative_path, "skills/review/SKILL.md");
        assert_eq!(
            skills[0].support_tier,
            "explicit_discovery_and_invocation_only"
        );
        assert!(skills[0]
            .unsupported_capabilities
            .contains(&"script_execution_unavailable".into()));
        assert!(!skills[0].invoked);
    }

    #[test]
    fn invocation_requires_an_explicit_project_local_skill_path() {
        let directory = tempdir().expect("tempdir");
        fs::create_dir_all(directory.path().join("skills/review")).expect("skill dir");
        fs::write(
            directory.path().join("skills/review/SKILL.md"),
            "---\nname: review\n---\n\nUse this skill.",
        )
        .expect("skill written");
        let catalog = ProjectSkillCatalog::new(directory.path()).expect("catalog");

        let invocation = catalog
            .invoke("skills/review/SKILL.md")
            .expect("skill invoked");

        assert_eq!(invocation.relative_path, "skills/review/SKILL.md");
        assert_eq!(invocation.name, "review");
        assert_eq!(invocation.context_effect, "explicit_user_selected_context");
        assert_eq!(invocation.permission_effect, "none");
        assert!(invocation.content.contains("Use this skill."));
    }

    #[test]
    fn discovery_skips_global_plugin_and_dependency_skill_locations() {
        let directory = tempdir().expect("tempdir");
        fs::create_dir_all(directory.path().join("node_modules/pkg")).expect("node modules dir");
        fs::create_dir_all(directory.path().join(".git/hooks")).expect("git dir");
        fs::create_dir_all(directory.path().join("skills/local")).expect("skill dir");
        fs::write(
            directory.path().join("node_modules/pkg/SKILL.md"),
            "---\nname: dependency\n---",
        )
        .expect("dependency skill");
        fs::write(
            directory.path().join(".git/hooks/SKILL.md"),
            "---\nname: git\n---",
        )
        .expect("git skill");
        fs::write(
            directory.path().join("skills/local/SKILL.md"),
            "---\nname: local\n---",
        )
        .expect("local skill");
        let catalog = ProjectSkillCatalog::new(directory.path()).expect("catalog");

        let skills = catalog.discover().expect("skills discovered");

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "local");
    }

    #[test]
    fn invocation_blocks_outside_root_paths() {
        let directory = tempdir().expect("tempdir");
        let outside = tempdir().expect("outside");
        fs::write(outside.path().join("SKILL.md"), "---\nname: outside\n---")
            .expect("outside skill");
        let catalog = ProjectSkillCatalog::new(directory.path()).expect("catalog");

        let result = catalog.invoke(outside.path().join("SKILL.md"));

        assert_eq!(result, Err(ProjectSkillError::OutsideProjectRoot));
    }

    #[test]
    fn invocation_blocks_secret_denied_skill_files() {
        let directory = tempdir().expect("tempdir");
        fs::write(directory.path().join("token.key"), "---\nname: secret\n---")
            .expect("secret skill");
        let catalog = ProjectSkillCatalog::new(directory.path()).expect("catalog");

        let result = catalog.invoke("token.key");

        assert_eq!(result, Err(ProjectSkillError::SecretDenied));
    }
}

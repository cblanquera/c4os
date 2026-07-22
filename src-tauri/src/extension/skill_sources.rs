//! Explicit, bounded discovery across C4OS Skill source scopes.
//!
//! Filesystem roots and their stable identities are supplied by the C4OS
//! coordinator. This module never infers a Project, Workspace, user, or
//! bundled path and never discovers Plugin-provided Skills outside the
//! verified package path.

use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    ffi::OsStr,
    fs,
    io::ErrorKind,
    path::PathBuf,
};

use super::{
    ExtensionError, ExtensionSourceKind, MAX_EXTENSION_SKILLS, SkillQualifiedIdentity,
    SkillSnapshot, sha256_prefixed,
    skill::{DiscoveredSkill, SkillCandidate, discover_skill},
    validate_identifier,
};

/// Bound on independently authorized Skill collection roots in one snapshot.
pub const MAX_SKILL_SOURCE_ROOTS: usize = 256;

/// One explicitly authorized filesystem collection containing direct
/// `<skill-id>/SKILL.md` children.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillSourceRoot {
    pub source_kind: ExtensionSourceKind,
    pub source_id: String,
    pub root: PathBuf,
    /// Project and Workspace roots are eligible only when this exact root is
    /// trusted by the coordinator. User and bundled roots are C4OS-owned and
    /// do not gain additional authority from this flag.
    pub trusted: bool,
}

/// Complete discovery input. Plugin candidates must already have been
/// discovered from a verified immutable package by the coordinator.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkillSourceContext {
    pub filesystem_roots: Vec<SkillSourceRoot>,
    pub plugin_provided: Vec<SkillCandidate>,
    /// Persisted per-identity availability. Missing entries preserve the
    /// candidate's default (`true` for filesystem sources).
    pub enabled_by_identity: BTreeMap<String, bool>,
}

/// Metadata-only discovery result. Invalid entries remain visible as truthful
/// snapshots but never enter the activatable candidate or discovered maps.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct SkillSourceDiscovery {
    pub candidates: Vec<SkillCandidate>,
    pub invalid_skills: Vec<SkillSnapshot>,
    pub discovered_by_identity: BTreeMap<String, DiscoveredSkill>,
}

/// Discovers direct children of every explicitly supplied filesystem root and
/// combines them with coordinator-verified Plugin-provided candidates.
pub fn discover_skill_sources(
    context: &SkillSourceContext,
) -> Result<SkillSourceDiscovery, ExtensionError> {
    if context.filesystem_roots.len() > MAX_SKILL_SOURCE_ROOTS
        || context.plugin_provided.len() > MAX_EXTENSION_SKILLS
        || context.enabled_by_identity.len() > MAX_EXTENSION_SKILLS
    {
        return Err(ExtensionError::BoundExceeded);
    }

    let mut roots = context.filesystem_roots.clone();
    roots.sort_by(compare_roots);
    validate_roots(&roots)?;

    let mut result = SkillSourceDiscovery::default();
    let mut stable_identities = BTreeSet::new();
    let mut scanned_entries = context.plugin_provided.len();

    for root in &roots {
        let metadata = match fs::symlink_metadata(&root.root) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == ErrorKind::NotFound => continue,
            Err(error) => return Err(error.into()),
        };
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err(ExtensionError::InvalidInput);
        }

        let mut entries = fs::read_dir(&root.root)?.collect::<Result<Vec<_>, _>>()?;
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            scanned_entries = scanned_entries
                .checked_add(1)
                .ok_or(ExtensionError::BoundExceeded)?;
            if scanned_entries > MAX_EXTENSION_SKILLS {
                return Err(ExtensionError::BoundExceeded);
            }

            let file_name = entry.file_name();
            let metadata = fs::symlink_metadata(entry.path())?;
            let file_type = metadata.file_type();
            if file_type.is_symlink() {
                push_invalid(
                    &mut result.invalid_skills,
                    root,
                    &file_name,
                    "Symbolic-link Skill entries are not followed.",
                );
                continue;
            }
            if !metadata.is_dir() {
                if !metadata.is_file() {
                    push_invalid(
                        &mut result.invalid_skills,
                        root,
                        &file_name,
                        "Special-file Skill entries are not supported.",
                    );
                }
                continue;
            }

            match discover_skill(&entry.path(), root.source_kind, &root.source_id, None) {
                Ok(skill) => {
                    let stable_id = skill.identity.stable_id();
                    let enabled = context
                        .enabled_by_identity
                        .get(&stable_id)
                        .copied()
                        .unwrap_or(true);
                    let trusted_root = root_trusted(root);
                    let candidate = SkillCandidate {
                        skill,
                        enabled,
                        eligible: trusted_root,
                        trusted_root,
                        provider_trusted_and_enabled: true,
                        revoked: false,
                    };
                    insert_candidate(&mut result, &mut stable_identities, candidate)?;
                }
                Err(error) => push_invalid(
                    &mut result.invalid_skills,
                    root,
                    &file_name,
                    discovery_diagnostic(&error),
                ),
            }
        }
    }

    for candidate in &context.plugin_provided {
        validate_plugin_candidate(candidate)?;
        let mut candidate = candidate.clone();
        let stable_id = candidate.skill.identity.stable_id();
        candidate.enabled = context
            .enabled_by_identity
            .get(&stable_id)
            .copied()
            .unwrap_or(candidate.enabled);
        candidate.eligible =
            candidate.eligible && candidate.provider_trusted_and_enabled && !candidate.revoked;
        insert_candidate(&mut result, &mut stable_identities, candidate)?;
    }

    result.candidates.sort_by(compare_candidates);
    result
        .invalid_skills
        .sort_by(|left, right| left.identity.stable_id().cmp(&right.identity.stable_id()));
    Ok(result)
}

fn validate_roots(roots: &[SkillSourceRoot]) -> Result<(), ExtensionError> {
    let mut identities = BTreeSet::new();
    for root in roots {
        if root.source_kind == ExtensionSourceKind::PluginProvided {
            return Err(ExtensionError::InvalidInput);
        }
        validate_identifier(&root.source_id)?;
        if !identities.insert((root.source_kind, root.source_id.as_str())) {
            return Err(ExtensionError::InvalidState);
        }
    }
    Ok(())
}

fn validate_plugin_candidate(candidate: &SkillCandidate) -> Result<(), ExtensionError> {
    let identity = &candidate.skill.identity;
    validate_identifier(&identity.source_id)?;
    validate_identifier(&identity.skill_id)?;
    if identity.source_kind != ExtensionSourceKind::PluginProvided
        || identity.skill_id != candidate.skill.portable.name
        || candidate.skill.package_id.as_deref() != Some(identity.source_id.as_str())
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn insert_candidate(
    result: &mut SkillSourceDiscovery,
    identities: &mut BTreeSet<String>,
    candidate: SkillCandidate,
) -> Result<(), ExtensionError> {
    let stable_id = candidate.skill.identity.stable_id();
    if !identities.insert(stable_id.clone())
        || result
            .discovered_by_identity
            .insert(stable_id, candidate.skill.clone())
            .is_some()
    {
        return Err(ExtensionError::InvalidState);
    }
    result.candidates.push(candidate);
    Ok(())
}

fn root_trusted(root: &SkillSourceRoot) -> bool {
    match root.source_kind {
        ExtensionSourceKind::ProjectLocal | ExtensionSourceKind::WorkspaceLocal => root.trusted,
        ExtensionSourceKind::UserGlobal | ExtensionSourceKind::Bundled => true,
        ExtensionSourceKind::PluginProvided => false,
    }
}

fn compare_roots(left: &SkillSourceRoot, right: &SkillSourceRoot) -> Ordering {
    left.source_kind
        .precedence_rank()
        .cmp(&right.source_kind.precedence_rank())
        .then_with(|| left.source_id.cmp(&right.source_id))
}

fn compare_candidates(left: &SkillCandidate, right: &SkillCandidate) -> Ordering {
    left.skill
        .identity
        .skill_id
        .cmp(&right.skill.identity.skill_id)
        .then_with(|| {
            left.skill
                .identity
                .source_kind
                .precedence_rank()
                .cmp(&right.skill.identity.source_kind.precedence_rank())
        })
        .then_with(|| {
            left.skill
                .identity
                .stable_id()
                .cmp(&right.skill.identity.stable_id())
        })
}

fn push_invalid(
    invalid_skills: &mut Vec<SkillSnapshot>,
    root: &SkillSourceRoot,
    file_name: &OsStr,
    diagnostic: &str,
) {
    let display_name = sanitized_display_name(file_name);
    invalid_skills.push(SkillSnapshot {
        identity: SkillQualifiedIdentity {
            source_kind: root.source_kind,
            source_id: root.source_id.clone(),
            skill_id: invalid_skill_id(root, file_name),
        },
        name: display_name,
        summary: "Skill metadata could not be validated.".into(),
        package_id: None,
        source_label: source_label(root.source_kind, &root.source_id),
        enabled: false,
        eligible: false,
        valid: false,
        active: false,
        shadowed: false,
        instructions_loaded: false,
        collision_sources: Vec::new(),
        diagnostic: Some(diagnostic.into()),
        version: "local".into(),
        is_installed: false,
        eligibility_detail: "Repair the source Skill before enabling it.".into(),
    });
}

fn invalid_skill_id(root: &SkillSourceRoot, file_name: &OsStr) -> String {
    let mut material = format!(
        "{}\0{}\0",
        root.source_kind.precedence_rank(),
        root.source_id
    )
    .into_bytes();
    append_file_name_identity(&mut material, file_name);
    let digest = sha256_prefixed(&material);
    format!("invalid-{}", &digest["sha256:".len()..])
}

#[cfg(unix)]
fn append_file_name_identity(material: &mut Vec<u8>, file_name: &OsStr) {
    use std::os::unix::ffi::OsStrExt;

    material.extend_from_slice(file_name.as_bytes());
}

#[cfg(not(unix))]
fn append_file_name_identity(material: &mut Vec<u8>, file_name: &OsStr) {
    material.extend_from_slice(file_name.to_string_lossy().as_bytes());
}

fn sanitized_display_name(file_name: &OsStr) -> String {
    let sanitized = file_name
        .to_string_lossy()
        .chars()
        .filter(|character| !character.is_control())
        .take(160)
        .collect::<String>();
    if sanitized.trim().is_empty() {
        "Invalid Skill".into()
    } else {
        sanitized
    }
}

fn source_label(source_kind: ExtensionSourceKind, source_id: &str) -> String {
    let scope = match source_kind {
        ExtensionSourceKind::ProjectLocal => "Project",
        ExtensionSourceKind::WorkspaceLocal => "Workspace",
        ExtensionSourceKind::UserGlobal => "User",
        ExtensionSourceKind::PluginProvided => "Plugin",
        ExtensionSourceKind::Bundled => "Bundled",
    };
    format!("{scope} · {source_id}")
}

fn discovery_diagnostic(error: &ExtensionError) -> &'static str {
    match error {
        ExtensionError::BoundExceeded => "Skill metadata exceeds a discovery bound.",
        ExtensionError::MutableContent => "Skill metadata changed during discovery.",
        ExtensionError::Io(_) => "Skill metadata is unavailable or has an unsupported file type.",
        _ => "Invalid SKILL.md frontmatter.",
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;
    use crate::extension::skill::{discover_skill, resolve_candidates};

    fn write_skill(collection: &Path, directory: &str, name: &str) -> PathBuf {
        let root = collection.join(directory);
        fs::create_dir_all(&root).expect("skill directory");
        fs::write(
            root.join("SKILL.md"),
            format!(
                "---\nname: {name}\ndescription: Use {name} during source discovery.\n---\nInstructions.\n"
            ),
        )
        .expect("skill metadata");
        root
    }

    fn source(
        source_kind: ExtensionSourceKind,
        source_id: &str,
        root: &Path,
        trusted: bool,
    ) -> SkillSourceRoot {
        SkillSourceRoot {
            source_kind,
            source_id: source_id.into(),
            root: root.into(),
            trusted,
        }
    }

    fn plugin_candidate(root: &Path, name: &str) -> SkillCandidate {
        let skill = discover_skill(
            root,
            ExtensionSourceKind::PluginProvided,
            "sample-plugin",
            Some("sample-plugin"),
        )
        .expect("plugin skill");
        assert_eq!(skill.identity.skill_id, name);
        SkillCandidate {
            skill,
            enabled: true,
            eligible: true,
            trusted_root: true,
            provider_trusted_and_enabled: true,
            revoked: false,
        }
    }

    #[test]
    fn discovers_all_five_scopes_in_deterministic_precedence_order() {
        let temporary = tempfile::tempdir().expect("temporary root");
        let project = temporary.path().join("project");
        let workspace = temporary.path().join("workspace");
        let user = temporary.path().join("user");
        let bundled = temporary.path().join("bundled");
        let plugin = temporary.path().join("plugin");
        for collection in [&project, &workspace, &user, &bundled] {
            fs::create_dir(collection).expect("collection");
            write_skill(collection, "deploy-directory", "deploy");
        }
        fs::create_dir(&plugin).expect("plugin root");
        fs::write(
            plugin.join("SKILL.md"),
            "---\nname: deploy\ndescription: Plugin deploy.\n---\nInstructions.\n",
        )
        .expect("plugin skill");

        let context = SkillSourceContext {
            filesystem_roots: vec![
                source(ExtensionSourceKind::Bundled, "c4os", &bundled, false),
                source(ExtensionSourceKind::UserGlobal, "user", &user, false),
                source(
                    ExtensionSourceKind::WorkspaceLocal,
                    "workspace-a",
                    &workspace,
                    true,
                ),
                source(
                    ExtensionSourceKind::ProjectLocal,
                    "project-a",
                    &project,
                    true,
                ),
            ],
            plugin_provided: vec![plugin_candidate(&plugin, "deploy")],
            enabled_by_identity: BTreeMap::new(),
        };
        let first = discover_skill_sources(&context).expect("discovery");
        let second = discover_skill_sources(&context).expect("repeat discovery");
        let identities = first
            .candidates
            .iter()
            .map(|candidate| candidate.skill.identity.stable_id())
            .collect::<Vec<_>>();
        assert_eq!(
            identities,
            vec![
                "project:project-a:deploy",
                "workspace:workspace-a:deploy",
                "user:user:deploy",
                "plugin:sample-plugin:deploy",
                "bundled:c4os:deploy",
            ]
        );
        assert_eq!(first, second);
        assert_eq!(first.discovered_by_identity.len(), 5);

        let resolution = resolve_candidates(first.candidates, None).expect("resolution");
        assert_eq!(
            resolution.active_identities[0].stable_id(),
            "project:project-a:deploy"
        );
        assert!(
            resolution
                .records
                .iter()
                .all(|record| record.collisions.len() == 4)
        );
    }

    #[test]
    fn untrusted_project_yields_to_trusted_workspace() {
        let temporary = tempfile::tempdir().expect("temporary root");
        let project = temporary.path().join("project");
        let workspace = temporary.path().join("workspace");
        fs::create_dir(&project).expect("project collection");
        fs::create_dir(&workspace).expect("workspace collection");
        write_skill(&project, "deploy", "deploy");
        write_skill(&workspace, "deploy", "deploy");

        let discovery = discover_skill_sources(&SkillSourceContext {
            filesystem_roots: vec![
                source(
                    ExtensionSourceKind::ProjectLocal,
                    "project-a",
                    &project,
                    false,
                ),
                source(
                    ExtensionSourceKind::WorkspaceLocal,
                    "workspace-a",
                    &workspace,
                    true,
                ),
            ],
            ..SkillSourceContext::default()
        })
        .expect("discovery");
        assert!(!discovery.candidates[0].eligible);
        assert!(!discovery.candidates[0].can_activate());
        assert!(discovery.candidates[1].eligible);
        assert!(discovery.candidates[1].can_activate());

        let resolution = resolve_candidates(discovery.candidates, None).expect("resolution");
        assert_eq!(
            resolution.active_identities[0].stable_id(),
            "workspace:workspace-a:deploy"
        );
    }

    #[test]
    fn project_and_workspace_require_trust_but_user_and_bundled_do_not() {
        let temporary = tempfile::tempdir().expect("temporary root");
        let project = temporary.path().join("project");
        let workspace = temporary.path().join("workspace");
        let user = temporary.path().join("user");
        let bundled = temporary.path().join("bundled");
        for (collection, name) in [
            (&project, "project-skill"),
            (&workspace, "workspace-skill"),
            (&user, "user-skill"),
            (&bundled, "bundled-skill"),
        ] {
            fs::create_dir(collection).expect("collection");
            write_skill(collection, name, name);
        }

        let discovery = discover_skill_sources(&SkillSourceContext {
            filesystem_roots: vec![
                source(
                    ExtensionSourceKind::ProjectLocal,
                    "project-a",
                    &project,
                    false,
                ),
                source(
                    ExtensionSourceKind::WorkspaceLocal,
                    "workspace-a",
                    &workspace,
                    false,
                ),
                source(ExtensionSourceKind::UserGlobal, "user", &user, false),
                source(ExtensionSourceKind::Bundled, "c4os", &bundled, false),
            ],
            ..SkillSourceContext::default()
        })
        .expect("discovery");
        let eligibility = discovery
            .candidates
            .iter()
            .map(|candidate| {
                (
                    candidate.skill.identity.source_kind,
                    (candidate.eligible, candidate.can_activate()),
                )
            })
            .collect::<BTreeMap<_, _>>();
        assert_eq!(
            eligibility[&ExtensionSourceKind::ProjectLocal],
            (false, false)
        );
        assert_eq!(
            eligibility[&ExtensionSourceKind::WorkspaceLocal],
            (false, false)
        );
        assert_eq!(eligibility[&ExtensionSourceKind::UserGlobal], (true, true));
        assert_eq!(eligibility[&ExtensionSourceKind::Bundled], (true, true));
    }

    #[test]
    fn invalid_frontmatter_is_visible_but_never_activatable() {
        let temporary = tempfile::tempdir().expect("temporary root");
        let collection = temporary.path().join("user");
        let broken = collection.join("broken-skill");
        fs::create_dir_all(&broken).expect("broken directory");
        fs::write(
            broken.join("SKILL.md"),
            "---\nname: ../invalid\ndescription: Broken\n---\nInstructions.\n",
        )
        .expect("broken skill");

        let discovery = discover_skill_sources(&SkillSourceContext {
            filesystem_roots: vec![source(
                ExtensionSourceKind::UserGlobal,
                "user",
                &collection,
                true,
            )],
            ..SkillSourceContext::default()
        })
        .expect("discovery");
        assert!(discovery.candidates.is_empty());
        assert!(discovery.discovered_by_identity.is_empty());
        assert_eq!(discovery.invalid_skills.len(), 1);
        let invalid = &discovery.invalid_skills[0];
        assert_eq!(invalid.name, "broken-skill");
        assert!(!invalid.valid);
        assert!(!invalid.enabled);
        assert!(!invalid.eligible);
        assert!(invalid.identity.skill_id.starts_with("invalid-"));
        assert_eq!(invalid.identity.skill_id.len(), 72);
    }

    #[cfg(unix)]
    #[test]
    fn symlinks_and_special_files_are_reported_without_traversal() {
        use std::{os::unix::fs::symlink, os::unix::net::UnixListener};

        let temporary = tempfile::tempdir().expect("temporary root");
        let collection = temporary.path().join("skills");
        let outside = temporary.path().join("outside");
        fs::create_dir(&collection).expect("collection");
        fs::create_dir(&outside).expect("outside");
        write_skill(&outside, "should-not-load", "outside-skill");
        symlink(
            outside.join("should-not-load"),
            collection.join("linked-skill"),
        )
        .expect("skill symlink");
        let socket_path = collection.join("socket-skill");
        let listener = match UnixListener::bind(&socket_path) {
            Ok(listener) => Some(listener),
            Err(error) if error.kind() == std::io::ErrorKind::PermissionDenied => None,
            Err(error) => panic!("socket: {error}"),
        };

        let discovery = discover_skill_sources(&SkillSourceContext {
            filesystem_roots: vec![source(
                ExtensionSourceKind::Bundled,
                "c4os",
                &collection,
                true,
            )],
            ..SkillSourceContext::default()
        })
        .expect("discovery");
        assert!(discovery.candidates.is_empty());
        assert_eq!(
            discovery.invalid_skills.len(),
            if listener.is_some() { 2 } else { 1 }
        );
        assert!(discovery.invalid_skills.iter().any(|skill| {
            skill.name == "linked-skill"
                && skill
                    .diagnostic
                    .as_deref()
                    .is_some_and(|diagnostic| diagnostic.contains("not followed"))
        }));
        if listener.is_some() {
            assert!(discovery.invalid_skills.iter().any(|skill| {
                skill.name == "socket-skill"
                    && skill
                        .diagnostic
                        .as_deref()
                        .is_some_and(|diagnostic| diagnostic.contains("Special-file"))
            }));
        }
    }

    #[cfg(unix)]
    #[test]
    fn a_symlinked_collection_root_is_rejected() {
        use std::os::unix::fs::symlink;

        let temporary = tempfile::tempdir().expect("temporary root");
        let actual = temporary.path().join("actual");
        let linked = temporary.path().join("linked");
        fs::create_dir(&actual).expect("actual collection");
        symlink(&actual, &linked).expect("collection symlink");

        let result = discover_skill_sources(&SkillSourceContext {
            filesystem_roots: vec![source(
                ExtensionSourceKind::UserGlobal,
                "user",
                &linked,
                true,
            )],
            ..SkillSourceContext::default()
        });
        assert!(matches!(result, Err(ExtensionError::InvalidInput)));
    }
}

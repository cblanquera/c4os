//! Portable `SKILL.md` discovery, progressive loading, and deterministic
//! source-qualified collision resolution.

use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{BufRead, BufReader, Read},
    path::{Component, Path, PathBuf},
};

use serde::{Deserialize, Serialize};

use super::{
    ExtensionError, ExtensionSourceKind, MAX_EXTENSION_SKILLS, SkillQualifiedIdentity,
    validate_identifier,
};

pub const SKILL_ENTRYPOINT: &str = "SKILL.md";
pub const MAX_SKILL_FRONTMATTER_BYTES: usize = 16 * 1024;
pub const MAX_SKILL_INSTRUCTION_BYTES: u64 = 256 * 1024;
pub const MAX_SKILL_RESOURCE_BYTES: u64 = 8 * 1024 * 1024;
pub const MAX_SKILL_REFERENCES: usize = 128;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SkillPortableMetadata {
    pub name: String,
    pub description: String,
    pub license: Option<String>,
    pub compatibility: Option<String>,
    pub metadata: BTreeMap<String, String>,
    pub allowed_tools: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiscoveredSkill {
    pub identity: SkillQualifiedIdentity,
    pub portable: SkillPortableMetadata,
    pub root: PathBuf,
    pub entrypoint: PathBuf,
    pub package_id: Option<String>,
    pub entrypoint_bytes: u64,
    /// Hash of only the eagerly read frontmatter. The complete entrypoint is
    /// deliberately not loaded during discovery.
    pub frontmatter_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LoadedSkill {
    pub discovered: DiscoveredSkill,
    pub instructions: String,
    pub referenced_resources: Vec<String>,
    pub entrypoint_digest: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillCandidate {
    pub skill: DiscoveredSkill,
    pub enabled: bool,
    pub eligible: bool,
    pub trusted_root: bool,
    pub provider_trusted_and_enabled: bool,
    pub revoked: bool,
}

impl SkillCandidate {
    pub fn can_activate(&self) -> bool {
        if !self.enabled || !self.eligible || self.revoked {
            return false;
        }
        match self.skill.identity.source_kind {
            ExtensionSourceKind::ProjectLocal | ExtensionSourceKind::WorkspaceLocal => {
                self.trusted_root
            }
            ExtensionSourceKind::PluginProvided => self.provider_trusted_and_enabled,
            ExtensionSourceKind::UserGlobal | ExtensionSourceKind::Bundled => true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ResolvedSkillCandidate {
    pub candidate: SkillCandidate,
    pub active: bool,
    pub shadowed: bool,
    pub collisions: Vec<SkillQualifiedIdentity>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SkillResolution {
    pub records: Vec<ResolvedSkillCandidate>,
    pub active_identities: Vec<SkillQualifiedIdentity>,
}

/// Reads only the bounded YAML frontmatter for eager discovery. Full Markdown
/// and resources remain unopened until `load_skill` is called.
pub fn discover_skill(
    root: &Path,
    source_kind: ExtensionSourceKind,
    source_id: &str,
    package_id: Option<&str>,
) -> Result<DiscoveredSkill, ExtensionError> {
    validate_identifier(source_id)?;
    if let Some(package_id) = package_id {
        validate_identifier(package_id)?;
    }
    reject_symlink_or_non_directory(&root)?;
    let root = root.canonicalize()?;
    let entrypoint = root.join(SKILL_ENTRYPOINT);
    reject_symlink_or_non_file(&entrypoint)?;
    let metadata = fs::metadata(&entrypoint)?;
    if metadata.len() > MAX_SKILL_INSTRUCTION_BYTES {
        return Err(ExtensionError::BoundExceeded);
    }
    let frontmatter = read_frontmatter_only(&entrypoint)?;
    let portable = parse_frontmatter(&frontmatter)?;
    let identity = SkillQualifiedIdentity {
        source_kind,
        source_id: source_id.to_owned(),
        skill_id: portable.name.clone(),
    };
    Ok(DiscoveredSkill {
        identity,
        portable,
        root,
        entrypoint,
        package_id: package_id.map(str::to_owned),
        entrypoint_bytes: metadata.len(),
        frontmatter_digest: sha256_prefixed(frontmatter.as_bytes()),
    })
}

/// Progressively reads and revalidates the complete entrypoint after the
/// coordinator has resolved source, trust, eligibility, and enablement.
pub fn load_skill(discovered: &DiscoveredSkill) -> Result<LoadedSkill, ExtensionError> {
    reject_symlink_or_non_file(&discovered.entrypoint)?;
    let canonical = discovered.entrypoint.canonicalize()?;
    ensure_within(&discovered.root, &canonical)?;
    let metadata = fs::metadata(&canonical)?;
    if metadata.len() > MAX_SKILL_INSTRUCTION_BYTES {
        return Err(ExtensionError::BoundExceeded);
    }
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    File::open(&canonical)?.read_to_end(&mut bytes)?;
    let text = String::from_utf8(bytes).map_err(|_| ExtensionError::InvalidInput)?;
    let (frontmatter, instructions) = split_frontmatter_and_body(&text)?;
    if sha256_prefixed(frontmatter.as_bytes()) != discovered.frontmatter_digest
        || parse_frontmatter(frontmatter)? != discovered.portable
    {
        return Err(ExtensionError::MutableContent);
    }
    let instructions = instructions.trim().to_owned();
    if instructions.is_empty()
        || instructions.len() > MAX_SKILL_INSTRUCTION_BYTES as usize
        || instructions
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\r' | '\t'))
    {
        return Err(ExtensionError::InvalidInput);
    }
    let referenced_resources = validate_markdown_references(&discovered.root, &instructions)?;
    Ok(LoadedSkill {
        discovered: discovered.clone(),
        instructions,
        referenced_resources,
        entrypoint_digest: sha256_prefixed(text.as_bytes()),
    })
}

/// Loads one previously validated relative resource on demand. C4OS callers
/// decide whether scripts may be proposed through the Action Gateway; this
/// function never executes them.
pub fn load_skill_resource(
    loaded: &LoadedSkill,
    relative_path: &str,
) -> Result<Vec<u8>, ExtensionError> {
    if !loaded
        .referenced_resources
        .iter()
        .any(|candidate| candidate == relative_path)
    {
        return Err(ExtensionError::InvalidInput);
    }
    let relative = normalize_relative_path(relative_path)?;
    let path = loaded.discovered.root.join(relative);
    reject_symlink_or_non_file(&path)?;
    let canonical = path.canonicalize()?;
    ensure_within(&loaded.discovered.root, &canonical)?;
    let metadata = fs::metadata(&canonical)?;
    if metadata.len() > MAX_SKILL_RESOURCE_BYTES {
        return Err(ExtensionError::BoundExceeded);
    }
    Ok(fs::read(canonical)?)
}

/// Preserves every fully qualified candidate while choosing at most one
/// eligible active source per portable skill name.
pub fn resolve_candidates(
    mut candidates: Vec<SkillCandidate>,
    explicitly_selected: Option<&SkillQualifiedIdentity>,
) -> Result<SkillResolution, ExtensionError> {
    if candidates.len() > MAX_EXTENSION_SKILLS {
        return Err(ExtensionError::BoundExceeded);
    }
    candidates.sort_by(|left, right| {
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
    });
    let mut identities = BTreeSet::new();
    if candidates
        .iter()
        .any(|candidate| !identities.insert(candidate.skill.identity.stable_id()))
    {
        return Err(ExtensionError::InvalidState);
    }

    let mut winners = BTreeMap::<String, String>::new();
    for candidate in &candidates {
        if !candidate.can_activate() {
            continue;
        }
        let skill_id = candidate.skill.identity.skill_id.clone();
        if explicitly_selected.is_some_and(|selected| {
            selected.skill_id == skill_id && selected == &candidate.skill.identity
        }) {
            winners.insert(skill_id, candidate.skill.identity.stable_id());
        } else {
            winners
                .entry(skill_id)
                .or_insert_with(|| candidate.skill.identity.stable_id());
        }
    }

    // A valid explicit selection must override any default winner even when it
    // appears later in deterministic order.
    if let Some(selected) = explicitly_selected {
        if let Some(candidate) = candidates
            .iter()
            .find(|candidate| &candidate.skill.identity == selected && candidate.can_activate())
        {
            winners.insert(
                selected.skill_id.clone(),
                candidate.skill.identity.stable_id(),
            );
        }
    }

    let mut records = Vec::with_capacity(candidates.len());
    let mut active_identities = Vec::new();
    for candidate in candidates {
        let stable_id = candidate.skill.identity.stable_id();
        let active = winners
            .get(&candidate.skill.identity.skill_id)
            .is_some_and(|winner| winner == &stable_id);
        let collisions = identities_for_skill(&records, &candidate, &stable_id);
        if active {
            active_identities.push(candidate.skill.identity.clone());
        }
        records.push(ResolvedSkillCandidate {
            shadowed: candidate.can_activate() && !active,
            candidate,
            active,
            collisions,
        });
    }

    // The single-pass collision construction above cannot see later records;
    // populate the complete collision set without changing resolution order.
    let all_identities: Vec<_> = records
        .iter()
        .map(|record| record.candidate.skill.identity.clone())
        .collect();
    for record in &mut records {
        record.collisions = all_identities
            .iter()
            .filter(|identity| {
                identity.skill_id == record.candidate.skill.identity.skill_id
                    && *identity != &record.candidate.skill.identity
            })
            .cloned()
            .collect();
    }

    Ok(SkillResolution {
        records,
        active_identities,
    })
}

fn identities_for_skill(
    prior: &[ResolvedSkillCandidate],
    candidate: &SkillCandidate,
    stable_id: &str,
) -> Vec<SkillQualifiedIdentity> {
    prior
        .iter()
        .filter(|record| {
            record.candidate.skill.identity.skill_id == candidate.skill.identity.skill_id
                && record.candidate.skill.identity.stable_id() != stable_id
        })
        .map(|record| record.candidate.skill.identity.clone())
        .collect()
}

fn read_frontmatter_only(path: &Path) -> Result<String, ExtensionError> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut line = String::new();
    if reader.read_line(&mut line)? == 0 || trim_line_ending(&line) != "---" {
        return Err(ExtensionError::InvalidInput);
    }
    let mut frontmatter = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Err(ExtensionError::InvalidInput);
        }
        if trim_line_ending(&line) == "---" {
            break;
        }
        frontmatter.push_str(&line);
        if frontmatter.len() > MAX_SKILL_FRONTMATTER_BYTES {
            return Err(ExtensionError::BoundExceeded);
        }
    }
    Ok(frontmatter)
}

fn split_frontmatter_and_body(input: &str) -> Result<(&str, &str), ExtensionError> {
    let input = input
        .strip_prefix("---\n")
        .or_else(|| input.strip_prefix("---\r\n"))
        .ok_or(ExtensionError::InvalidInput)?;
    if let Some(index) = input.find("\n---\n") {
        return Ok((&input[..index + 1], &input[index + 5..]));
    }
    if let Some(index) = input.find("\r\n---\r\n") {
        return Ok((&input[..index + 2], &input[index + 8..]));
    }
    Err(ExtensionError::InvalidInput)
}

fn parse_frontmatter(input: &str) -> Result<SkillPortableMetadata, ExtensionError> {
    if input.len() > MAX_SKILL_FRONTMATTER_BYTES || input.contains('\t') {
        return Err(ExtensionError::BoundExceeded);
    }
    let mut values = BTreeMap::<String, String>::new();
    let mut metadata = BTreeMap::<String, String>::new();
    let mut in_metadata = false;
    for raw_line in input.lines() {
        let line = raw_line.trim_end();
        if line.trim().is_empty() || line.trim_start().starts_with('#') {
            continue;
        }
        if let Some(indented) = line.strip_prefix("  ") {
            if !in_metadata || indented.starts_with(' ') {
                return Err(ExtensionError::InvalidInput);
            }
            let (key, value) = parse_yaml_scalar_field(indented)?;
            validate_metadata_key(&key)?;
            if metadata.insert(key, value).is_some() || metadata.len() > 64 {
                return Err(ExtensionError::InvalidState);
            }
            continue;
        }
        if line.starts_with(' ') {
            return Err(ExtensionError::InvalidInput);
        }
        let (key, value) = parse_yaml_scalar_field(line)?;
        if key == "metadata" {
            if !value.is_empty() || in_metadata {
                return Err(ExtensionError::InvalidInput);
            }
            in_metadata = true;
            continue;
        }
        in_metadata = false;
        if !matches!(
            key.as_str(),
            "name" | "description" | "license" | "compatibility" | "allowed-tools"
        ) || values.insert(key, value).is_some()
        {
            return Err(ExtensionError::InvalidInput);
        }
    }
    let name = values.remove("name").ok_or(ExtensionError::InvalidInput)?;
    validate_skill_name(&name)?;
    let description = values
        .remove("description")
        .ok_or(ExtensionError::InvalidInput)?;
    if description.is_empty()
        || description.len() > 1_024
        || description.chars().any(char::is_control)
    {
        return Err(ExtensionError::InvalidInput);
    }
    let license = values.remove("license");
    let compatibility = values.remove("compatibility");
    let allowed_tools = values.remove("allowed-tools");
    if values.len() > 0
        || [&license, &compatibility, &allowed_tools]
            .into_iter()
            .flatten()
            .any(|value| {
                value.is_empty() || value.len() > 2_048 || value.chars().any(char::is_control)
            })
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(SkillPortableMetadata {
        name,
        description,
        license,
        compatibility,
        metadata,
        allowed_tools,
    })
}

fn parse_yaml_scalar_field(line: &str) -> Result<(String, String), ExtensionError> {
    let (key, value) = line.split_once(':').ok_or(ExtensionError::InvalidInput)?;
    let key = key.trim();
    if key.is_empty()
        || key.len() > 128
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ExtensionError::InvalidInput);
    }
    let value = value.trim();
    let value = if value.len() >= 2
        && ((value.starts_with('"') && value.ends_with('"'))
            || (value.starts_with('\'') && value.ends_with('\'')))
    {
        let inner = &value[1..value.len() - 1];
        if inner.contains(value.as_bytes()[0] as char) || inner.contains('\\') {
            return Err(ExtensionError::InvalidInput);
        }
        inner
    } else {
        if value.contains(" #")
            || ['[', ']', '{', '}']
                .iter()
                .any(|character| value.contains(*character))
        {
            return Err(ExtensionError::InvalidInput);
        }
        value
    };
    Ok((key.to_owned(), value.to_owned()))
}

fn validate_skill_name(name: &str) -> Result<(), ExtensionError> {
    if name.is_empty()
        || name.len() > 64
        || name.starts_with('-')
        || name.ends_with('-')
        || name.contains("--")
        || !name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn validate_metadata_key(key: &str) -> Result<(), ExtensionError> {
    if key.is_empty()
        || key.len() > 64
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn validate_markdown_references(
    root: &Path,
    instructions: &str,
) -> Result<Vec<String>, ExtensionError> {
    let mut references = BTreeSet::new();
    let mut remainder = instructions;
    while let Some(start) = remainder.find("](") {
        remainder = &remainder[start + 2..];
        let Some(end) = remainder.find(')') else {
            return Err(ExtensionError::InvalidInput);
        };
        let target = remainder[..end].trim();
        remainder = &remainder[end + 1..];
        if target.is_empty()
            || target.starts_with('#')
            || target.starts_with("https://")
            || target.starts_with("http://")
            || target.starts_with("mailto:")
        {
            continue;
        }
        if [' ', '\t', '\n']
            .iter()
            .any(|character| target.contains(*character))
            || target.contains('#')
            || target.contains('?')
        {
            return Err(ExtensionError::InvalidInput);
        }
        let normalized = normalize_relative_path(target)?;
        let first = normalized.split('/').next().unwrap_or_default();
        if !matches!(first, "assets" | "references" | "scripts") {
            return Err(ExtensionError::InvalidInput);
        }
        let path = root.join(&normalized);
        reject_symlink_or_non_file(&path)?;
        let canonical = path.canonicalize()?;
        ensure_within(root, &canonical)?;
        if fs::metadata(&canonical)?.len() > MAX_SKILL_RESOURCE_BYTES {
            return Err(ExtensionError::BoundExceeded);
        }
        references.insert(normalized);
        if references.len() > MAX_SKILL_REFERENCES {
            return Err(ExtensionError::BoundExceeded);
        }
    }
    Ok(references.into_iter().collect())
}

fn normalize_relative_path(value: &str) -> Result<String, ExtensionError> {
    let path = Path::new(value);
    if path.is_absolute() || value.contains('\\') {
        return Err(ExtensionError::InvalidInput);
    }
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(segment) => {
                let segment = segment.to_str().ok_or(ExtensionError::InvalidInput)?;
                if segment.is_empty() || segment == "." || segment == ".." {
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
    Ok(segments.join("/"))
}

fn ensure_within(root: &Path, candidate: &Path) -> Result<(), ExtensionError> {
    let root = root.canonicalize()?;
    if !candidate.starts_with(&root) {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn reject_symlink_or_non_file(path: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn reject_symlink_or_non_directory(path: &Path) -> Result<(), ExtensionError> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(ExtensionError::InvalidInput);
    }
    Ok(())
}

fn trim_line_ending(value: &str) -> &str {
    value
        .strip_suffix("\r\n")
        .or_else(|| value.strip_suffix('\n'))
        .unwrap_or(value)
}

fn sha256_prefixed(bytes: &[u8]) -> String {
    super::sha256_prefixed(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn skill(root: &Path, name: &str, body: &str) -> DiscoveredSkill {
        fs::write(
            root.join(SKILL_ENTRYPOINT),
            format!(
                "---\nname: {name}\ndescription: Use {name} when testing source resolution.\n---\n\n{body}\n"
            ),
        )
        .expect("skill");
        discover_skill(root, ExtensionSourceKind::UserGlobal, "user", None).expect("discover")
    }

    fn candidate(
        skill: DiscoveredSkill,
        source_kind: ExtensionSourceKind,
        source_id: &str,
    ) -> SkillCandidate {
        let mut skill = skill;
        skill.identity.source_kind = source_kind;
        skill.identity.source_id = source_id.to_owned();
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
    fn discovery_is_metadata_only_and_activation_revalidates() {
        let root = tempfile::tempdir().expect("tempdir");
        let discovered = skill(root.path(), "sample-skill", "# Instructions");
        assert_eq!(discovered.portable.name, "sample-skill");
        let loaded = load_skill(&discovered).expect("load");
        assert!(loaded.instructions.contains("Instructions"));

        fs::write(
            root.path().join(SKILL_ENTRYPOINT),
            "---\nname: changed\ndescription: Use changed when testing.\n---\nbody",
        )
        .expect("mutate");
        assert!(matches!(
            load_skill(&discovered),
            Err(ExtensionError::MutableContent)
        ));
    }

    #[test]
    fn progressive_resource_load_rejects_traversal() {
        let root = tempfile::tempdir().expect("tempdir");
        fs::create_dir(root.path().join("references")).expect("references");
        fs::write(root.path().join("references/detail.md"), "detail").expect("resource");
        let discovered = skill(
            root.path(),
            "sample-skill",
            "Read [details](references/detail.md) only when required.",
        );
        let loaded = load_skill(&discovered).expect("load");
        assert_eq!(
            load_skill_resource(&loaded, "references/detail.md").expect("resource"),
            b"detail"
        );
        assert!(load_skill_resource(&loaded, "../outside").is_err());
    }

    #[test]
    fn resolution_preserves_collisions_and_honors_explicit_identity() {
        let project_root = tempfile::tempdir().expect("project");
        let plugin_root = tempfile::tempdir().expect("plugin");
        let project = candidate(
            skill(project_root.path(), "deploy", "project"),
            ExtensionSourceKind::ProjectLocal,
            "project-a",
        );
        let plugin = candidate(
            skill(plugin_root.path(), "deploy", "plugin"),
            ExtensionSourceKind::PluginProvided,
            "plugin-a",
        );
        let default =
            resolve_candidates(vec![plugin.clone(), project.clone()], None).expect("resolve");
        assert_eq!(
            default.active_identities,
            vec![project.skill.identity.clone()]
        );
        assert!(
            default
                .records
                .iter()
                .all(|record| record.collisions.len() == 1)
        );

        let selected =
            resolve_candidates(vec![project, plugin.clone()], Some(&plugin.skill.identity))
                .expect("selected");
        assert_eq!(selected.active_identities, vec![plugin.skill.identity]);
    }

    #[test]
    fn untrusted_project_and_disabled_plugin_are_ineligible() {
        let project_root = tempfile::tempdir().expect("project");
        let plugin_root = tempfile::tempdir().expect("plugin");
        let mut project = candidate(
            skill(project_root.path(), "deploy", "project"),
            ExtensionSourceKind::ProjectLocal,
            "project-a",
        );
        project.trusted_root = false;
        let mut plugin = candidate(
            skill(plugin_root.path(), "deploy", "plugin"),
            ExtensionSourceKind::PluginProvided,
            "plugin-a",
        );
        plugin.provider_trusted_and_enabled = false;
        let resolution = resolve_candidates(vec![project, plugin], None).expect("resolve");
        assert!(resolution.active_identities.is_empty());
    }
}

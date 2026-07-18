//! C4OS-owned policy classification and resolution.
//!
//! This module is deliberately free of renderer, runtime, and persistence
//! concerns. Adapters provide complete [`ActionFacts`]; the Rust core resolves
//! them against the active policy and gives the action gateway one authoritative
//! decision. Unknown or incomplete facts never resolve to `Allow`.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Policy decisions are ordered from least to most restrictive.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyDecision {
    Allow,
    Ask,
    Deny,
}

impl PolicyDecision {
    fn strictest(self, other: Self) -> Self {
        self.max(other)
    }
}

/// The four accepted user-facing presets.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ApprovalPreset {
    AskForApproval,
    ApproveSafeActions,
    ApproveForMe,
    Custom,
}

/// The seven accepted user-facing policy groups.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PolicyGroup {
    WorkspaceFiles,
    CommandsAndProcesses,
    VersionControl,
    NetworkAndSharing,
    BrowserAndDesktop,
    Credentials,
    ExtensionsAndC4os,
}

pub const POLICY_GROUPS: [PolicyGroup; 7] = [
    PolicyGroup::WorkspaceFiles,
    PolicyGroup::CommandsAndProcesses,
    PolicyGroup::VersionControl,
    PolicyGroup::NetworkAndSharing,
    PolicyGroup::BrowserAndDesktop,
    PolicyGroup::Credentials,
    PolicyGroup::ExtensionsAndC4os,
];

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionSurface {
    Terminal,
    File,
    Git,
    Browser,
    Network,
    Credential,
    Process,
    Desktop,
    C4os,
    Unknown(String),
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionEffect {
    Read,
    Capture,
    Create,
    Modify,
    Delete,
    Execute,
    Control,
    Publish,
    Reveal,
    Listen,
    Unknown,
}

impl ActionEffect {
    pub fn is_read_only(self) -> bool {
        matches!(self, Self::Read | Self::Capture)
    }

    pub fn is_mutating(self) -> bool {
        matches!(
            self,
            Self::Create
                | Self::Modify
                | Self::Delete
                | Self::Execute
                | Self::Control
                | Self::Publish
                | Self::Listen
        )
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionScope {
    Workspace,
    ExternalLocal,
    Remote,
    System,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionInitiator {
    User,
    Agent,
    Plugin,
    Runtime,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionSensitivity {
    Ordinary,
    Authenticated,
    Credential,
    Private,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionReversibility {
    Reversible,
    Destructive,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ClassificationConfidence {
    Known,
    Ambiguous,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ActionRequestOrigin {
    NaturalLanguageChat,
    DirectUserEdit,
    ArtifactReplyProposal,
    RuntimeTool,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RepositoryState {
    VersionControlled,
    NotVersionControlled,
    NotApplicable,
    Unknown,
}

/// Canonical, composable facts for one complete proposed action.
///
/// Strings in this structure are stable C4OS identifiers or canonicalized
/// values, never renderer labels. An unresolved target is represented by
/// `target_resolved = false`, not by inventing a canonical path or host.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionFacts {
    pub action_kind: String,
    pub native_tool: String,
    pub surface: ActionSurface,
    pub effects: BTreeSet<ActionEffect>,
    pub scope: ActionScope,
    pub initiator: ActionInitiator,
    pub sensitivity: ActionSensitivity,
    pub reversibility: ActionReversibility,
    pub confidence: ClassificationConfidence,
    pub request_origin: ActionRequestOrigin,
    pub repository_state: RepositoryState,
    pub inside_active_project: bool,
    pub canonical_target: String,
    pub workspace_id: String,
    pub session_id: String,
    pub runtime_id: String,
    pub environment_id: String,
    pub plugin_or_mcp_id: Option<String>,
    pub target_resolved: bool,
    pub authenticated: bool,
    pub trusted_root: bool,
    pub explicit_scope_grant: bool,
    pub sandbox_allows: bool,
    pub declaration_exceeded: bool,
}

impl ActionFacts {
    /// Returns every user-facing group implicated by the complete action.
    pub fn policy_groups(&self) -> BTreeSet<PolicyGroup> {
        self.policy_components()
            .into_iter()
            .map(|(group, _)| group)
            .collect()
    }

    fn policy_components(&self) -> BTreeSet<(PolicyGroup, ActionEffect)> {
        let primary = match self.surface {
            ActionSurface::File => PolicyGroup::WorkspaceFiles,
            ActionSurface::Terminal | ActionSurface::Process => PolicyGroup::CommandsAndProcesses,
            ActionSurface::Git => PolicyGroup::VersionControl,
            ActionSurface::Network => PolicyGroup::NetworkAndSharing,
            ActionSurface::Browser | ActionSurface::Desktop => PolicyGroup::BrowserAndDesktop,
            ActionSurface::Credential => PolicyGroup::Credentials,
            ActionSurface::C4os | ActionSurface::Unknown(_) => PolicyGroup::ExtensionsAndC4os,
        };
        let mut components = BTreeSet::new();
        for effect in self.effects.iter().copied() {
            components.insert((primary, effect));

            if matches!(effect, ActionEffect::Publish | ActionEffect::Listen) {
                components.insert((PolicyGroup::NetworkAndSharing, effect));
            }
            if self.sensitivity == ActionSensitivity::Credential || effect == ActionEffect::Reveal {
                components.insert((PolicyGroup::Credentials, effect));
            }
            if self.plugin_or_mcp_id.is_some() {
                components.insert((PolicyGroup::ExtensionsAndC4os, effect));
            }
        }

        components
    }

    fn is_complete(&self) -> bool {
        !self.action_kind.trim().is_empty()
            && !self.native_tool.trim().is_empty()
            && !self.effects.is_empty()
            && !self.workspace_id.trim().is_empty()
            && !self.session_id.trim().is_empty()
            && !self.runtime_id.trim().is_empty()
            && !self.environment_id.trim().is_empty()
            && self.target_resolved
            && !self.canonical_target.trim().is_empty()
    }

    fn is_unknown(&self) -> bool {
        !self.is_complete()
            || matches!(self.surface, ActionSurface::Unknown(_))
            || self.effects.contains(&ActionEffect::Unknown)
            || self.scope == ActionScope::Unknown
            || self.initiator == ActionInitiator::Unknown
            || self.sensitivity == ActionSensitivity::Unknown
            || self.reversibility == ActionReversibility::Unknown
            || self.confidence == ClassificationConfidence::Ambiguous
            || self.request_origin == ActionRequestOrigin::Unknown
            || self.repository_state == RepositoryState::Unknown
    }
}

/// A composable matcher for category and ceiling rules.
///
/// Every populated field must match. `effects` is an all-of match so a rule can
/// target a multi-effect action without ignoring its other effects.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMatcher {
    pub surface: Option<ActionSurface>,
    pub effects: BTreeSet<ActionEffect>,
    pub scope: Option<ActionScope>,
    pub initiator: Option<ActionInitiator>,
    pub sensitivity: Option<ActionSensitivity>,
    pub reversibility: Option<ActionReversibility>,
    pub confidence: Option<ClassificationConfidence>,
    pub request_origin: Option<ActionRequestOrigin>,
    pub repository_state: Option<RepositoryState>,
    pub inside_active_project: Option<bool>,
    pub action_kind: Option<String>,
    pub native_tool: Option<String>,
    pub canonical_target: Option<String>,
    pub workspace_id: Option<String>,
    pub session_id: Option<String>,
    pub runtime_id: Option<String>,
    pub environment_id: Option<String>,
    pub plugin_or_mcp_id: Option<String>,
}

impl RuleMatcher {
    pub fn matches(&self, facts: &ActionFacts) -> bool {
        optional_eq(self.surface.as_ref(), &facts.surface)
            && self.effects.is_subset(&facts.effects)
            && optional_eq(self.scope.as_ref(), &facts.scope)
            && optional_eq(self.initiator.as_ref(), &facts.initiator)
            && optional_eq(self.sensitivity.as_ref(), &facts.sensitivity)
            && optional_eq(self.reversibility.as_ref(), &facts.reversibility)
            && optional_eq(self.confidence.as_ref(), &facts.confidence)
            && optional_eq(self.request_origin.as_ref(), &facts.request_origin)
            && optional_eq(self.repository_state.as_ref(), &facts.repository_state)
            && optional_eq(
                self.inside_active_project.as_ref(),
                &facts.inside_active_project,
            )
            && optional_str_eq(self.action_kind.as_deref(), &facts.action_kind)
            && optional_str_eq(self.native_tool.as_deref(), &facts.native_tool)
            && optional_str_eq(self.canonical_target.as_deref(), &facts.canonical_target)
            && optional_str_eq(self.workspace_id.as_deref(), &facts.workspace_id)
            && optional_str_eq(self.session_id.as_deref(), &facts.session_id)
            && optional_str_eq(self.runtime_id.as_deref(), &facts.runtime_id)
            && optional_str_eq(self.environment_id.as_deref(), &facts.environment_id)
            && match self.plugin_or_mcp_id.as_deref() {
                Some(expected) => facts.plugin_or_mcp_id.as_deref() == Some(expected),
                None => true,
            }
    }
}

fn optional_eq<T: PartialEq>(expected: Option<&T>, actual: &T) -> bool {
    expected.is_none_or(|expected| expected == actual)
}

fn optional_str_eq(expected: Option<&str>, actual: &str) -> bool {
    expected.is_none_or(|expected| expected == actual)
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryRule {
    pub id: String,
    pub group: PolicyGroup,
    pub decision: PolicyDecision,
    pub matcher: RuleMatcher,
}

impl CategoryRule {
    fn matches(&self, facts: &ActionFacts) -> bool {
        facts.policy_groups().contains(&self.group) && self.matcher.matches(facts)
    }
}

/// The duration and scope of a prompt-created concrete exception.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum ExceptionDuration {
    Session { session_id: String },
    Until { expires_at_ms: u64 },
    Persistent,
}

/// A narrow prompt-created exception.
///
/// Unlike a category rule, an exception always binds operation, complete fact
/// dimensions, canonical target, workspace, runtime, and environment. The
/// optional plugin/MCP binding is also exact: `None` matches only an action that
/// did not originate from a plugin or MCP server.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConcreteException {
    pub id: String,
    pub decision: PolicyDecision,
    pub action_kind: String,
    pub native_tool: String,
    pub surface: ActionSurface,
    pub effects: BTreeSet<ActionEffect>,
    pub scope: ActionScope,
    pub initiator: ActionInitiator,
    pub sensitivity: ActionSensitivity,
    pub request_origin: ActionRequestOrigin,
    pub repository_state: RepositoryState,
    pub inside_active_project: bool,
    pub canonical_target: String,
    pub workspace_id: String,
    pub runtime_id: String,
    pub environment_id: String,
    pub plugin_or_mcp_id: Option<String>,
    pub duration: ExceptionDuration,
}

impl ConcreteException {
    pub fn from_action(
        id: impl Into<String>,
        decision: PolicyDecision,
        facts: &ActionFacts,
        duration: ExceptionDuration,
    ) -> Self {
        Self {
            id: id.into(),
            decision,
            action_kind: facts.action_kind.clone(),
            native_tool: facts.native_tool.clone(),
            surface: facts.surface.clone(),
            effects: facts.effects.clone(),
            scope: facts.scope,
            initiator: facts.initiator,
            sensitivity: facts.sensitivity,
            request_origin: facts.request_origin,
            repository_state: facts.repository_state,
            inside_active_project: facts.inside_active_project,
            canonical_target: facts.canonical_target.clone(),
            workspace_id: facts.workspace_id.clone(),
            runtime_id: facts.runtime_id.clone(),
            environment_id: facts.environment_id.clone(),
            plugin_or_mcp_id: facts.plugin_or_mcp_id.clone(),
            duration,
        }
    }

    pub fn matches(&self, facts: &ActionFacts, now_ms: u64) -> bool {
        let duration_matches = match &self.duration {
            ExceptionDuration::Session { session_id } => session_id == &facts.session_id,
            ExceptionDuration::Until { expires_at_ms } => now_ms < *expires_at_ms,
            ExceptionDuration::Persistent => true,
        };
        duration_matches
            && self.action_kind == facts.action_kind
            && self.native_tool == facts.native_tool
            && self.surface == facts.surface
            && self.effects == facts.effects
            && self.scope == facts.scope
            && self.initiator == facts.initiator
            && self.sensitivity == facts.sensitivity
            && self.request_origin == facts.request_origin
            && self.repository_state == facts.repository_state
            && self.inside_active_project == facts.inside_active_project
            && self.canonical_target == facts.canonical_target
            && self.workspace_id == facts.workspace_id
            && self.runtime_id == facts.runtime_id
            && self.environment_id == facts.environment_id
            && self.plugin_or_mcp_id == facts.plugin_or_mcp_id
    }
}

/// A maximum-authority or managed-policy ceiling. Ceiling rules may only
/// restrict, never grant authority.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CeilingRule {
    pub id: String,
    pub decision: PolicyDecision,
    pub matcher: RuleMatcher,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CeilingRuleError {
    AllowCannotBeACeiling,
}

impl CeilingRule {
    pub fn new(
        id: impl Into<String>,
        decision: PolicyDecision,
        matcher: RuleMatcher,
    ) -> Result<Self, CeilingRuleError> {
        if decision == PolicyDecision::Allow {
            return Err(CeilingRuleError::AllowCannotBeACeiling);
        }
        Ok(Self {
            id: id.into(),
            decision,
            matcher,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyConfiguration {
    pub preset: ApprovalPreset,
    pub category_rules: Vec<CategoryRule>,
    pub exceptions: Vec<ConcreteException>,
    pub maximum_authority: Vec<CeilingRule>,
    pub managed_requirements: Vec<CeilingRule>,
}

impl Default for PolicyConfiguration {
    fn default() -> Self {
        Self {
            preset: ApprovalPreset::AskForApproval,
            category_rules: Vec::new(),
            exceptions: Vec::new(),
            maximum_authority: Vec::new(),
            managed_requirements: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "id")]
pub enum DecisionSource {
    Preset,
    CategoryRule(String),
    Exception(String),
    MaximumAuthority(String),
    ManagedRequirement(String),
    UnknownOrIncomplete,
    Sandbox,
    UntrustedWorkspaceTarget,
    CredentialReveal,
    UnresolvedAuthenticatedPublish,
    DestructiveSystemAction,
    UngrantedExternalMutation,
    UndeclaredAuthority,
    NonVersionControlledChatWrite,
    OutOfProjectChatWrite,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DecisionContribution {
    pub decision: PolicyDecision,
    pub source: DecisionSource,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyResolution {
    pub decision: PolicyDecision,
    pub controlling_sources: Vec<DecisionSource>,
    pub contributions: Vec<DecisionContribution>,
}

/// Resolves one complete action using the most-restrictive-wins contract.
pub fn resolve_policy(
    facts: &ActionFacts,
    configuration: &PolicyConfiguration,
    now_ms: u64,
) -> PolicyResolution {
    let mut contributions = safety_ceilings(facts);

    for rule in configuration
        .maximum_authority
        .iter()
        .filter(|rule| rule.matcher.matches(facts))
    {
        contributions.push(DecisionContribution {
            decision: rule.decision,
            source: DecisionSource::MaximumAuthority(rule.id.clone()),
        });
    }
    for rule in configuration
        .managed_requirements
        .iter()
        .filter(|rule| rule.matcher.matches(facts))
    {
        contributions.push(DecisionContribution {
            decision: rule.decision,
            source: DecisionSource::ManagedRequirement(rule.id.clone()),
        });
    }

    let matched_category: Vec<_> = configuration
        .category_rules
        .iter()
        .filter(|rule| rule.matches(facts))
        .collect();
    let matched_exceptions: Vec<_> = configuration
        .exceptions
        .iter()
        .filter(|exception| exception.matches(facts, now_ms))
        .collect();

    let mut specific = Vec::new();
    for rule in &matched_category {
        specific.push(DecisionContribution {
            decision: rule.decision,
            source: DecisionSource::CategoryRule(rule.id.clone()),
        });
    }
    for exception in &matched_exceptions {
        specific.push(DecisionContribution {
            decision: exception.decision,
            source: DecisionSource::Exception(exception.id.clone()),
        });
    }

    let category_covers_complete_action =
        facts.policy_components().iter().all(|(group, effect)| {
            matched_category.iter().any(|rule| {
                rule.group == *group
                    && (rule.matcher.effects.is_empty() || rule.matcher.effects.contains(effect))
            })
        });
    if matched_exceptions.is_empty() && !category_covers_complete_action {
        contributions.push(DecisionContribution {
            decision: preset_decision(facts, configuration.preset),
            source: DecisionSource::Preset,
        });
    }
    contributions.extend(specific);

    let decision = contributions
        .iter()
        .fold(PolicyDecision::Allow, |current, item| {
            current.strictest(item.decision)
        });
    let controlling_sources = contributions
        .iter()
        .filter(|item| item.decision == decision)
        .map(|item| item.source.clone())
        .collect();

    PolicyResolution {
        decision,
        controlling_sources,
        contributions,
    }
}

fn safety_ceilings(facts: &ActionFacts) -> Vec<DecisionContribution> {
    let mut contributions = Vec::new();
    let mut add = |decision, source| {
        contributions.push(DecisionContribution { decision, source });
    };

    if facts.is_unknown() {
        add(PolicyDecision::Ask, DecisionSource::UnknownOrIncomplete);
    }
    if !facts.sandbox_allows {
        add(PolicyDecision::Deny, DecisionSource::Sandbox);
    }
    if facts.scope == ActionScope::Workspace
        && !facts.trusted_root
        && facts.effects.iter().any(|effect| effect.is_mutating())
    {
        add(
            PolicyDecision::Ask,
            DecisionSource::UntrustedWorkspaceTarget,
        );
    }
    if facts.sensitivity == ActionSensitivity::Credential
        && facts.effects.contains(&ActionEffect::Reveal)
    {
        add(PolicyDecision::Deny, DecisionSource::CredentialReveal);
    }
    if facts.scope == ActionScope::Remote
        && facts.effects.contains(&ActionEffect::Publish)
        && (facts.authenticated || facts.sensitivity == ActionSensitivity::Authenticated)
        && !facts.target_resolved
    {
        add(
            PolicyDecision::Ask,
            DecisionSource::UnresolvedAuthenticatedPublish,
        );
    }
    if facts.scope == ActionScope::System && facts.reversibility == ActionReversibility::Destructive
    {
        add(
            PolicyDecision::Deny,
            DecisionSource::DestructiveSystemAction,
        );
    }
    if facts.scope == ActionScope::ExternalLocal
        && facts.effects.iter().any(|effect| effect.is_mutating())
        && !facts.explicit_scope_grant
    {
        add(
            PolicyDecision::Ask,
            DecisionSource::UngrantedExternalMutation,
        );
    }
    if facts.declaration_exceeded {
        add(PolicyDecision::Deny, DecisionSource::UndeclaredAuthority);
    }
    if facts.surface == ActionSurface::File
        && facts.request_origin == ActionRequestOrigin::NaturalLanguageChat
        && facts.effects.iter().any(|effect| effect.is_mutating())
    {
        if !facts.inside_active_project {
            add(PolicyDecision::Ask, DecisionSource::OutOfProjectChatWrite);
        } else if facts.repository_state != RepositoryState::VersionControlled {
            add(
                PolicyDecision::Ask,
                DecisionSource::NonVersionControlledChatWrite,
            );
        }
    }
    contributions
}

fn preset_decision(facts: &ActionFacts, preset: ApprovalPreset) -> PolicyDecision {
    let read_only =
        !facts.effects.is_empty() && facts.effects.iter().all(|effect| effect.is_read_only());
    let trusted_read = read_only
        && facts.confidence == ClassificationConfidence::Known
        && facts.scope == ActionScope::Workspace
        && facts.sensitivity == ActionSensitivity::Ordinary
        && facts.trusted_root;
    let safe_workspace = facts.confidence == ClassificationConfidence::Known
        && facts.scope == ActionScope::Workspace
        && facts.sensitivity == ActionSensitivity::Ordinary
        && facts.reversibility == ActionReversibility::Reversible
        && facts.trusted_root
        && !facts.effects.is_empty()
        && facts.effects.iter().all(|effect| {
            matches!(
                effect,
                ActionEffect::Read
                    | ActionEffect::Capture
                    | ActionEffect::Create
                    | ActionEffect::Modify
            )
        });

    match preset {
        ApprovalPreset::AskForApproval => {
            if trusted_read {
                PolicyDecision::Allow
            } else {
                PolicyDecision::Ask
            }
        }
        ApprovalPreset::ApproveSafeActions => {
            if safe_workspace {
                PolicyDecision::Allow
            } else {
                PolicyDecision::Ask
            }
        }
        ApprovalPreset::ApproveForMe => PolicyDecision::Allow,
        ApprovalPreset::Custom => PolicyDecision::Ask,
    }
}

use std::collections::{BTreeMap, BTreeSet};

use c4os_lib::runtime::capability::{
    AttachmentMediaType, AttachmentRequirement, CAPABILITY_SCHEMA_VERSION, CapabilityDescriptor,
    CapabilityEvidence, CapabilityKey, CapabilityLayer, CapabilityState, DraftRequirements,
    InstalledResourcePreflight, LimitConfidence, ModelLifecycle, NumericCapabilityEvidence,
    NumericCapabilityKey, PolicyPreflight, PreflightOutcome, PreflightResolution, RouteIdentity,
    effective_intersection, preflight,
};

const NOW: u64 = 1_721_300_000_000;

fn digest(character: char) -> String {
    format!("sha256:{}", character.to_string().repeat(64))
}

fn resources() -> InstalledResourcePreflight {
    InstalledResourcePreflight {
        snapshot_id: "installed-resources-1".into(),
        snapshot_sha256: digest('c'),
        tool_ids: ["c4os-propose-action".into()].into_iter().collect(),
        attachment_converters: BTreeSet::new(),
    }
}

fn policy() -> PolicyPreflight {
    PolicyPreflight {
        snapshot_id: "policy-1".into(),
        version: 1,
        tool_use_allowed: true,
        attachment_conversion_allowed: true,
    }
}

fn route() -> RouteIdentity {
    RouteIdentity {
        provider_id: "provider-openrouter".into(),
        endpoint_id: "openrouter-chat-completions".into(),
        provider_model_id: "anthropic/claude-sonnet-4.5:thinking".into(),
        model_revision: "2026-07-18+stable".into(),
        adapter_kind: "opencode".into(),
        adapter_version: "1.0.0".into(),
        runtime_kind: "opencode".into(),
        native_runtime_version: "1.18.3".into(),
        session_configuration_sha256: digest('a'),
    }
}

fn evidence(
    layer: CapabilityLayer,
    state: CapabilityState,
    allowed_values: &[&str],
) -> CapabilityEvidence {
    CapabilityEvidence {
        state,
        layer,
        source: format!("fixture.{layer:?}").to_lowercase(),
        checked_at_ms: NOW - 1_000,
        expires_at_ms: Some(NOW + 60_000),
        constraints: Vec::new(),
        allowed_values: allowed_values.iter().map(|value| (*value).into()).collect(),
        reason: (state != CapabilityState::Supported)
            .then(|| format!("fixture reports {state:?}").to_lowercase()),
    }
}

fn descriptor(layer: CapabilityLayer) -> CapabilityDescriptor {
    let mut features = BTreeMap::new();
    for key in [
        CapabilityKey::InputImage,
        CapabilityKey::Reasoning,
        CapabilityKey::ToolCalling,
        CapabilityKey::StructuredJsonSchema,
        CapabilityKey::Streaming,
    ] {
        let allowed = if key == CapabilityKey::Reasoning {
            &["off", "medium", "high"][..]
        } else {
            &[][..]
        };
        features.insert(key, evidence(layer, CapabilityState::Supported, allowed));
    }
    let mut numeric_limits = BTreeMap::new();
    for (key, maximum) in [
        (NumericCapabilityKey::ContextTokens, 128_000),
        (NumericCapabilityKey::OutputTokens, 8_192),
        (NumericCapabilityKey::AttachmentBytes, 10_000),
        (NumericCapabilityKey::AttachmentCount, 2),
    ] {
        numeric_limits.insert(
            key,
            NumericCapabilityEvidence {
                evidence: evidence(layer, CapabilityState::Supported, &[]),
                maximum: Some(maximum),
                confidence: LimitConfidence::Confirmed,
            },
        );
    }
    CapabilityDescriptor {
        schema_version: CAPABILITY_SCHEMA_VERSION,
        layer,
        route: route(),
        lifecycle: ModelLifecycle::Active,
        features,
        numeric_limits,
        raw_evidence_sha256: digest('b'),
    }
}

#[test]
fn effective_descriptor_uses_the_most_restrictive_state_and_narrowest_limit() {
    let declared = descriptor(CapabilityLayer::Declared);
    let mut normalized = descriptor(CapabilityLayer::AdapterNormalized);
    normalized.features.insert(
        CapabilityKey::Streaming,
        evidence(
            CapabilityLayer::AdapterNormalized,
            CapabilityState::Degraded,
            &[],
        ),
    );
    normalized
        .numeric_limits
        .get_mut(&NumericCapabilityKey::ContextTokens)
        .unwrap()
        .maximum = Some(32_000);
    let mut observed = descriptor(CapabilityLayer::Observed);
    observed.features.insert(
        CapabilityKey::ToolCalling,
        evidence(CapabilityLayer::Observed, CapabilityState::Unsupported, &[]),
    );

    let effective = effective_intersection(&[declared, normalized, observed], NOW).unwrap();

    assert_eq!(
        effective.feature_state(CapabilityKey::ToolCalling),
        CapabilityState::Unsupported
    );
    assert_eq!(
        effective.feature_state(CapabilityKey::Streaming),
        CapabilityState::Degraded
    );
    assert_eq!(
        effective.numeric_maximum(NumericCapabilityKey::ContextTokens),
        Some(32_000)
    );
    assert!(
        effective
            .features
            .values()
            .all(|value| value.layer == CapabilityLayer::Effective)
    );
    assert!(effective.raw_evidence_sha256.starts_with("sha256:"));
}

#[test]
fn a_missing_layer_field_stays_unknown_instead_of_upgrading() {
    let declared = descriptor(CapabilityLayer::Declared);
    let normalized = descriptor(CapabilityLayer::AdapterNormalized);
    let mut observed = descriptor(CapabilityLayer::Observed);
    observed.features.remove(&CapabilityKey::InputImage);

    let effective = effective_intersection(&[declared, normalized, observed], NOW).unwrap();

    assert_eq!(
        effective.feature_state(CapabilityKey::InputImage),
        CapabilityState::Unknown
    );
}

#[test]
fn expired_or_route_mismatched_evidence_fails_closed() {
    let declared = descriptor(CapabilityLayer::Declared);
    let normalized = descriptor(CapabilityLayer::AdapterNormalized);
    let mut expired = descriptor(CapabilityLayer::Observed);
    expired
        .features
        .get_mut(&CapabilityKey::Streaming)
        .unwrap()
        .expires_at_ms = Some(NOW);
    assert!(effective_intersection(&[declared.clone(), normalized.clone(), expired], NOW).is_err());

    let mut mismatch = descriptor(CapabilityLayer::Observed);
    mismatch.route.native_runtime_version = "1.18.4".into();
    assert!(effective_intersection(&[declared, normalized, mismatch], NOW).is_err());
}

#[test]
fn effective_capabilities_require_one_truthful_descriptor_per_source_layer() {
    let declared = descriptor(CapabilityLayer::Declared);
    let normalized = descriptor(CapabilityLayer::AdapterNormalized);
    let observed = descriptor(CapabilityLayer::Observed);

    assert!(matches!(
        effective_intersection(&[declared.clone(), normalized.clone()], NOW),
        Err(c4os_lib::runtime::capability::CapabilityError::MissingLayers)
    ));
    assert!(matches!(
        effective_intersection(&[declared.clone(), declared.clone(), observed.clone()], NOW),
        Err(c4os_lib::runtime::capability::CapabilityError::MissingLayers)
    ));

    let mut mislabeled = normalized;
    mislabeled
        .features
        .get_mut(&CapabilityKey::Streaming)
        .unwrap()
        .layer = CapabilityLayer::Declared;
    assert!(effective_intersection(&[declared, mislabeled, observed], NOW).is_err());
}

#[test]
fn preflight_preserves_incompatible_input_and_offers_explicit_resolutions() {
    let mut descriptor = descriptor(CapabilityLayer::Effective);
    descriptor.features.insert(
        CapabilityKey::InputImage,
        evidence(
            CapabilityLayer::Effective,
            CapabilityState::Unsupported,
            &[],
        ),
    );
    let draft = DraftRequirements {
        attachments: vec![AttachmentRequirement {
            attachment_id: "attachment-1".into(),
            media_type: AttachmentMediaType::Image,
            mime_type: "image/png".into(),
            bytes: 1_024,
        }],
        reasoning_mode: Some("medium".into()),
        requires_tools: false,
        requires_json_schema: false,
        prefers_streaming: true,
        estimated_input_tokens: 2_000,
        requested_output_tokens: 1_000,
        installed_resources: InstalledResourcePreflight {
            attachment_converters: [AttachmentMediaType::Image].into_iter().collect(),
            ..resources()
        },
        policy: policy(),
    };

    let PreflightOutcome::Blocked { issues } = preflight(&descriptor, &draft).unwrap() else {
        panic!("unsupported attachment should block dispatch");
    };
    let issue = issues
        .iter()
        .find(|issue| issue.code == "attachment-incompatible")
        .unwrap();
    assert_eq!(issue.subject_id.as_deref(), Some("attachment-1"));
    assert!(issue.resolutions.contains(&PreflightResolution::Convert));
    assert!(issue.resolutions.contains(&PreflightResolution::Remove));
    assert_eq!(
        draft.attachments.len(),
        1,
        "preflight must not silently drop input"
    );
}

#[test]
fn preflight_blocks_route_limits_and_unavailable_models_but_degrades_streaming_visibly() {
    let mut descriptor = descriptor(CapabilityLayer::Effective);
    descriptor.lifecycle = ModelLifecycle::Unavailable;
    descriptor.features.insert(
        CapabilityKey::Streaming,
        evidence(CapabilityLayer::Effective, CapabilityState::Degraded, &[]),
    );
    let draft = DraftRequirements {
        attachments: vec![
            AttachmentRequirement {
                attachment_id: "attachment-1".into(),
                media_type: AttachmentMediaType::Image,
                mime_type: "image/png".into(),
                bytes: 6_000,
            },
            AttachmentRequirement {
                attachment_id: "attachment-2".into(),
                media_type: AttachmentMediaType::Image,
                mime_type: "image/png".into(),
                bytes: 6_000,
            },
        ],
        reasoning_mode: None,
        requires_tools: false,
        requires_json_schema: false,
        prefers_streaming: true,
        estimated_input_tokens: 129_000,
        requested_output_tokens: 9_000,
        installed_resources: resources(),
        policy: policy(),
    };

    let PreflightOutcome::Blocked { issues } = preflight(&descriptor, &draft).unwrap() else {
        panic!("unavailable, over-limit route should block");
    };
    assert!(issues.iter().any(|issue| issue.code == "model-unavailable"));
    assert!(issues.iter().any(|issue| issue.code == "attachment-limit"));
    assert!(issues.iter().any(|issue| issue.code == "token-limit"));
}

#[test]
fn reasoning_choices_are_intersected_and_revalidated_per_route() {
    let declared = descriptor(CapabilityLayer::Declared);
    let mut normalized = descriptor(CapabilityLayer::AdapterNormalized);
    normalized.features.insert(
        CapabilityKey::Reasoning,
        evidence(
            CapabilityLayer::AdapterNormalized,
            CapabilityState::Supported,
            &["off", "medium"],
        ),
    );
    let observed = descriptor(CapabilityLayer::Observed);
    let effective = effective_intersection(&[declared, normalized, observed], NOW).unwrap();
    assert_eq!(
        effective.features[&CapabilityKey::Reasoning].allowed_values,
        vec!["medium", "off"]
    );

    let draft = DraftRequirements {
        attachments: vec![],
        reasoning_mode: Some("high".into()),
        requires_tools: false,
        requires_json_schema: false,
        prefers_streaming: false,
        estimated_input_tokens: 10,
        requested_output_tokens: 10,
        installed_resources: resources(),
        policy: policy(),
    };
    let PreflightOutcome::Blocked { issues } = preflight(&effective, &draft).unwrap() else {
        panic!("stale reasoning selection should block");
    };
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == "reasoning-incompatible")
    );
}

#[test]
fn preflight_requires_concrete_installed_resources_and_policy_permission() {
    let descriptor = descriptor(CapabilityLayer::Effective);
    let mut installed_resources = resources();
    installed_resources.tool_ids.clear();
    let draft = DraftRequirements {
        attachments: vec![],
        reasoning_mode: None,
        requires_tools: true,
        requires_json_schema: false,
        prefers_streaming: false,
        estimated_input_tokens: 10,
        requested_output_tokens: 10,
        installed_resources,
        policy: PolicyPreflight {
            tool_use_allowed: false,
            ..policy()
        },
    };

    let PreflightOutcome::Blocked { issues } = preflight(&descriptor, &draft).unwrap() else {
        panic!("missing resources and denied policy must block dispatch");
    };
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == "tools-not-installed")
    );
    assert!(
        issues
            .iter()
            .any(|issue| issue.code == "tools-policy-blocked")
    );
}

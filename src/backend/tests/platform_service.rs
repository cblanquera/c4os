#[allow(dead_code, unused_imports)]
#[path = "../src/platform/mod.rs"]
mod platform;
#[allow(dead_code)]
#[path = "../src/protocol.rs"]
mod protocol;

use platform::*;
use protocol::{PickerGrantId, RequestId};
use serde_json::{Value, json};
use tempfile::TempDir;

fn service() -> PlatformService {
    PlatformService::qualify(PlatformTarget::new("macos", "aarch64")).unwrap()
}

fn request_id(value: &str) -> RequestId {
    RequestId::new(value).unwrap()
}

fn grant_id(value: &str) -> PickerGrantId {
    PickerGrantId::new(value).unwrap()
}

#[test]
fn native_picker_identity_rejects_same_path_replacement_before_consumption() {
    let temporary = TempDir::new().expect("temporary picker root");
    let path = temporary.path().join("selected.txt");
    std::fs::write(&path, "selected object").expect("selected file");
    let selection =
        NativePickerSelection::new(&path, PickerObjectKind::File).expect("native picker selection");
    selection
        .verify_current_identity()
        .expect("unchanged selection");

    std::fs::remove_file(&path).expect("remove selected object");
    std::fs::write(&path, "replacement object").expect("replacement file");
    assert!(matches!(
        selection.verify_current_identity(),
        Err(PlatformError::InvalidPickerSelection(
            "native picker selection changed before consumption"
        ))
    ));
}

#[test]
fn target_qualification_is_exact_and_fails_closed() {
    let service = service();
    assert_eq!(service.target().os, "macos");
    assert_eq!(service.target().architecture, "aarch64");

    for target in [
        PlatformTarget::new("macos", "x86_64"),
        PlatformTarget::new("windows", "aarch64"),
        PlatformTarget::new("linux", "aarch64"),
        PlatformTarget::new("", ""),
    ] {
        assert!(matches!(
            PlatformService::qualify(target),
            Err(PlatformError::UnsupportedTarget { .. })
        ));
    }
}

#[test]
fn pre_reveal_snapshot_is_source_qualified_and_serializable() {
    let snapshot = service().initial_snapshot(InitialThemeSnapshot::new(
        ColorScheme::Dark,
        InitialThemeSource::MacosAppearance,
    ));
    let value = serde_json::to_value(snapshot).unwrap();

    assert_eq!(value["contractVersion"], PLATFORM_CONTRACT_VERSION);
    assert_eq!(value["platform"], "macos");
    assert_eq!(value["architecture"], "aarch64");
    assert_eq!(value["initialTheme"]["scheme"], "dark");
    assert_eq!(value["initialTheme"]["source"], "macosAppearance");
    assert_eq!(value["liveThemeSource"], "webviewPrefersColorScheme");
    assert_eq!(value["window"]["decorations"], "standard");
    assert_eq!(value["window"]["titlebarTransparent"], false);
    assert_eq!(value["window"]["titlebarOverlay"], false);
    assert_eq!(value["window"]["initiallyVisible"], false);
    assert_eq!(value["window"]["revealFallbackTimeoutMs"], 5_000);
    assert_eq!(value["vocabulary"]["revealAction"], "Reveal in Finder");
    assert_eq!(value["vocabulary"]["primaryModifierSymbol"], "⌘");
    assert!(value.get("themeOverride").is_none());
    assert!(!serde_json::to_string(&value).unwrap().contains("manual"));
}

#[test]
fn settings_menu_contract_has_stable_native_identifiers() {
    let snapshot = service().initial_snapshot(InitialThemeSnapshot::new(
        ColorScheme::Light,
        InitialThemeSource::SemanticFallback,
    ));

    assert_eq!(NativeMenuItemId::Settings.as_str(), SETTINGS_MENU_ITEM_ID);
    assert_eq!(
        NativeCommandId::OpenSettings.as_str(),
        OPEN_SETTINGS_COMMAND_ID
    );
    assert_eq!(snapshot.settings_menu.route, SETTINGS_ROUTE);
    assert_eq!(snapshot.settings_menu.accelerator, "CmdOrCtrl+,");
    assert_eq!(snapshot.settings_menu.keyboard_label, "⌘,");
    assert_eq!(snapshot.capabilities, PlatformCapabilities::default());
}

#[test]
fn native_capabilities_are_reported_only_after_explicit_composition() {
    let capabilities = PlatformCapabilities {
        native_application_menu: true,
        native_settings_shortcut: true,
        native_file_picker: true,
        native_folder_picker: true,
        native_workspace_picker: true,
        standard_window_decorations: true,
    };
    let platform =
        PlatformService::qualify_installed(PlatformTarget::new("macos", "aarch64"), capabilities)
            .unwrap();
    let snapshot = platform.initial_snapshot(InitialThemeSnapshot::new(
        ColorScheme::Light,
        InitialThemeSource::WebviewPreferredColorScheme,
    ));
    assert_eq!(snapshot.capabilities, capabilities);

    let invalid = PlatformCapabilities {
        native_settings_shortcut: true,
        ..PlatformCapabilities::default()
    };
    assert!(matches!(
        PlatformService::qualify_installed(PlatformTarget::new("macos", "aarch64"), invalid),
        Err(PlatformError::InvalidCapabilities(_))
    ));
}

#[test]
fn picker_purpose_owns_its_selection_policy() {
    let platform = service();
    let request =
        platform.picker_request(request_id("picker-1"), PickerPurpose::OpenWorkspaceArchive);
    request.validate().unwrap();
    assert_eq!(request.selection.object_kind, PickerObjectKind::File);
    assert!(!request.selection.allows_multiple);
    assert_eq!(request.selection.allowed_extensions, vec!["zip".to_owned()]);

    let mut hostile = serde_json::to_value(&request).unwrap();
    hostile["selection"]["allowsMultiple"] = json!(true);
    let hostile: NativePickerRequest = serde_json::from_value(hostile).unwrap();
    assert!(matches!(
        hostile.validate(),
        Err(PlatformError::InvalidPickerRequest(_))
    ));

    let mut future = request;
    future.contract_version = PICKER_CONTRACT_VERSION + 1;
    assert!(matches!(
        future.validate(),
        Err(PlatformError::UnsupportedContractVersion { .. })
    ));

    let request = platform.picker_request(
        request_id("picker-wrong-extension"),
        PickerPurpose::OpenWorkspaceArchive,
    );
    let non_archive =
        NativePickerSelection::new("/private/tmp/workspace.txt", PickerObjectKind::File).unwrap();
    assert!(matches!(
        platform.selected_picker(&request, &[non_archive], Vec::new()),
        Err(PlatformError::InvalidPickerSelection(_))
    ));
}

#[test]
fn cancellation_is_correlated_and_contains_no_grant() {
    let platform = service();
    let request = platform.picker_request(request_id("picker-cancel"), PickerPurpose::OpenFile);
    let result = platform.cancelled_picker(&request).unwrap();
    result.validate_for(&request).unwrap();

    assert_eq!(
        serde_json::to_value(result).unwrap(),
        json!({
            "type": "cancelled",
            "contractVersion": PICKER_CONTRACT_VERSION,
            "requestId": "picker-cancel"
        })
    );
}

#[test]
fn artifact_folder_picker_is_single_folder_only() {
    let request = service().picker_request(
        request_id("picker-artifact-folder"),
        PickerPurpose::OpenFolder,
    );

    assert_eq!(request.selection.object_kind, PickerObjectKind::Folder);
    assert!(!request.selection.allows_multiple);
    assert!(request.selection.allowed_extensions.is_empty());
}

#[test]
fn selected_result_exposes_only_opaque_correlated_grants() {
    let platform = service();
    let request = platform.picker_request(
        request_id("picker-attachments"),
        PickerPurpose::AttachChatFiles,
    );
    let selections = [
        NativePickerSelection::new("/private/tmp/one.txt", PickerObjectKind::File).unwrap(),
        NativePickerSelection::new("/private/tmp/two.png", PickerObjectKind::File).unwrap(),
    ];
    let grants = vec![
        PickerGrantSnapshot::new(grant_id("grant-1"), PickerObjectKind::File, "one.txt").unwrap(),
        PickerGrantSnapshot::new(grant_id("grant-2"), PickerObjectKind::File, "two.png").unwrap(),
    ];

    let result = platform
        .selected_picker(&request, &selections, grants)
        .unwrap();
    result.validate_for(&request).unwrap();
    let json = serde_json::to_string(&result).unwrap();

    assert!(json.contains("picker-attachments"));
    assert!(json.contains("grant-1"));
    assert!(json.contains("one.txt"));
    assert!(!json.contains("/private/tmp"));
    assert_eq!(selections[0].path().to_str(), Some("/private/tmp/one.txt"));
}

#[test]
fn picker_result_rejects_kind_cardinality_duplicates_and_mismatch() {
    let platform = service();
    let folder_request = platform.picker_request(
        request_id("picker-folder"),
        PickerPurpose::OpenProjectFolder,
    );
    let file = NativePickerSelection::new("/private/tmp/project", PickerObjectKind::File).unwrap();
    assert!(matches!(
        platform.selected_picker(&folder_request, &[file], Vec::new()),
        Err(PlatformError::InvalidPickerSelection(_))
    ));

    let folder =
        NativePickerSelection::new("/private/tmp/project", PickerObjectKind::Folder).unwrap();
    let duplicate_folders = [folder.clone(), folder];
    assert!(matches!(
        platform.selected_picker(&folder_request, &duplicate_folders, Vec::new()),
        Err(PlatformError::InvalidPickerSelection(_))
    ));

    let other_request =
        platform.picker_request(request_id("picker-other"), PickerPurpose::OpenProjectFolder);
    let result = platform.cancelled_picker(&folder_request).unwrap();
    assert_eq!(
        result.validate_for(&other_request),
        Err(PlatformError::PickerCorrelationMismatch)
    );

    assert!(matches!(
        NativePickerSelection::new("relative/path", PickerObjectKind::Folder),
        Err(PlatformError::InvalidPickerSelection(_))
    ));
}

#[test]
fn picker_contract_rejects_unknown_fields_and_invalid_grant_names() {
    let request = service().picker_request(request_id("picker-1"), PickerPurpose::OpenFile);
    let mut value = serde_json::to_value(request).unwrap();
    value
        .as_object_mut()
        .unwrap()
        .insert("ambientFilesystemAccess".into(), Value::Bool(true));
    assert!(serde_json::from_value::<NativePickerRequest>(value).is_err());

    assert!(matches!(
        PickerGrantSnapshot::new(grant_id("grant-1"), PickerObjectKind::File, "../secret.txt"),
        Err(PlatformError::InvalidPickerGrant(_))
    ));
}

#[test]
fn picker_registry_mints_opaque_grants_atomically_and_consumes_once() {
    let platform = service();
    let request = platform.picker_request(
        request_id("picker-registry"),
        PickerPurpose::OpenProjectFolder,
    );
    let selection =
        NativePickerSelection::new("/private/tmp/project", PickerObjectKind::Folder).unwrap();
    let mut registry = PickerGrantRegistry::default();

    let snapshots = registry
        .register_batch(
            &request,
            std::slice::from_ref(&selection),
            vec![grant_id("grant-registry")],
            1_721_300_000_000,
        )
        .unwrap();
    let serialized = serde_json::to_string(&snapshots).unwrap();

    assert_eq!(registry.len(), 1);
    assert!(serialized.contains("project"));
    assert!(!serialized.contains("/private/tmp"));

    let grant = registry.take(&grant_id("grant-registry")).unwrap();
    assert_eq!(grant.request_id(), &request.request_id);
    assert_eq!(grant.purpose(), PickerPurpose::OpenProjectFolder);
    assert_eq!(grant.path().to_str(), Some("/private/tmp/project"));
    assert_eq!(grant.object_kind(), PickerObjectKind::Folder);
    assert_eq!(grant.issued_at_ms(), 1_721_300_000_000);
    assert!(registry.take(&grant_id("grant-registry")).is_none());
    assert!(registry.is_empty());
}

#[test]
fn picker_registry_rejects_conflicts_without_partial_registration() {
    let platform = service();
    let request = platform.picker_request(
        request_id("picker-conflict"),
        PickerPurpose::AttachChatFiles,
    );
    let selections = [
        NativePickerSelection::new("/private/tmp/one.txt", PickerObjectKind::File).unwrap(),
        NativePickerSelection::new("/private/tmp/two.txt", PickerObjectKind::File).unwrap(),
    ];
    let mut registry = PickerGrantRegistry::default();

    assert_eq!(
        registry.register_batch(
            &request,
            &selections,
            vec![grant_id("duplicate"), grant_id("duplicate")],
            1,
        ),
        Err(PlatformError::PickerGrantConflict)
    );
    assert!(registry.is_empty());
}

#[test]
fn picker_registry_batch_take_is_all_or_nothing_and_preserves_order() {
    let platform = service();
    let request =
        platform.picker_request(request_id("picker-batch"), PickerPurpose::AttachChatFiles);
    let selections = [
        NativePickerSelection::new("/private/tmp/one.txt", PickerObjectKind::File).unwrap(),
        NativePickerSelection::new("/private/tmp/two.txt", PickerObjectKind::File).unwrap(),
    ];
    let mut registry = PickerGrantRegistry::default();
    registry
        .register_batch(
            &request,
            &selections,
            vec![grant_id("grant-one"), grant_id("grant-two")],
            1,
        )
        .unwrap();
    assert!(
        registry
            .take_batch(&[grant_id("grant-one"), grant_id("missing")])
            .is_err()
    );
    assert_eq!(registry.len(), 2);
    let grants = registry
        .take_batch(&[grant_id("grant-two"), grant_id("grant-one")])
        .unwrap();
    assert_eq!(grants[0].path().to_str(), Some("/private/tmp/two.txt"));
    assert_eq!(grants[1].path().to_str(), Some("/private/tmp/one.txt"));
    assert!(registry.is_empty());
}

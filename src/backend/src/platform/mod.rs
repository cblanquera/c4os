//! Target-qualified native platform contracts owned by the Rust core.
//!
//! The Tauri composition root supplies the actual AppKit menu, window, theme,
//! and picker adapters. This module keeps their renderer-visible state and
//! picker correlation rules independent from those adapters.

mod service;

pub use service::{
    ColorScheme, INITIAL_REVEAL_FALLBACK_MS, InitialThemeSnapshot, InitialThemeSource,
    LiveThemeSource, MAX_PICKER_SELECTIONS, NativeCommandId, NativeMenuItemId, NativePickerGrant,
    NativePickerRequest, NativePickerSelection, OPEN_SETTINGS_COMMAND_ID, PICKER_CONTRACT_VERSION,
    PLATFORM_CONTRACT_VERSION, PickerGrantRegistry, PickerGrantSnapshot, PickerObjectKind,
    PickerOutcome, PickerPurpose, PickerSelectionPolicy, PlatformCapabilities, PlatformError,
    PlatformFamily, PlatformService, PlatformSnapshot, PlatformTarget, PlatformVocabulary,
    SETTINGS_ACCELERATOR, SETTINGS_MENU_ITEM_ID, SETTINGS_ROUTE, SettingsMenuContract,
    WindowChromeSnapshot, WindowDecorations,
};

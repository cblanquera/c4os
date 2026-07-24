//! Rust-owned native Browser facility boundaries.
//!
//! Website-controlled state remains inside public WebKit containers. This
//! module exposes only C4OS-owned profile metadata and native-controller
//! intents/events; it never exposes a page-to-application bridge.

pub mod native;
pub mod profile;

//! Screenshot tools facade.
//!
//! This module re-exports the screenshot operations defined in [`crate::ops::screenshot_tools`],
//! providing a top-level `screenshot` module gated behind the `screenshot-tools` feature.

pub use crate::ops::screenshot_tools::*;

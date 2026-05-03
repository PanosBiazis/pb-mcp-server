//! OCR tools facade.
//!
//! This module re-exports the OCR operations defined in [`crate::ops::ocr_tools`],
//! providing a top-level `ocr` module gated behind the `ocr-tools` feature.

pub use crate::ops::ocr_tools::*;

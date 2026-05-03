//! Machine-learning tools facade.
//!
//! This module re-exports the neural/ML operations defined in [`crate::ops::neural`],
//! providing a top-level `ml` module gated behind the `ml-tools` feature.

pub use crate::ops::neural::*;

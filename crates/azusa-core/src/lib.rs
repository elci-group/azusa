// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Azusa comparison engine.
//!
//! Compares two [`azusa_model::RepositoryState`]s layer by layer
//! (identity, ancestry, filesystem, structure, dependencies, configuration,
//! interfaces, behaviour, semantics, operations), producing an
//! evidence-first [`azusa_model::DriftReport`] with deterministic scores,
//! drift direction, risk classification and recommendations.
//!
//! The first pass is deterministic: given the same states, configuration and
//! tool version, the structural result never varies. Probabilistic
//! interpretation is never mixed into deterministic scoring silently.

pub mod behaviour;
pub mod config;
pub mod engine;
pub mod error;
pub mod intent;
pub mod plan;
pub mod resolve;
pub mod scoring;
pub mod structural;

pub use config::{AzusaConfig, CONFIG_FILE_NAME, DriftWeights, ExitThresholds};
pub use engine::compare;
pub use error::CoreError;
pub use resolve::StateResolver;

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-core";

/// Smoke test.
#[test]
fn core_smoke() {
    assert_eq!(CRATE_NAME, "azusa-core");
}

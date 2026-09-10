//! Azusa comparison engine.
//!
//! Compares two [`azusa_model::RepositoryState`]s layer by layer
//! (identity, ancestry, filesystem, structure, dependencies, configuration,
//! interfaces, behaviour, semantics, operations), producing an
//! evidence-first [`azusa_model::DriftReport`] with deterministic scores,
//! drift direction, risk classification, intent estimates and a
//! reconciliation plan.
//!
//! The first pass is deterministic: given the same states, configuration and
//! tool version, the structural result never varies. Probabilistic
//! interpretation is never mixed into deterministic scoring silently.

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-core";

/// Smoke test.
#[test]
fn core_smoke() {
    assert_eq!(CRATE_NAME, "azusa-core");
}

//! Azusa policy engine.
//!
//! Declares configurable drift limits (maximum score, dependency rules,
//! structure rules, configuration rules) and checks a
//! [`azusa_model::DriftReport`] against them, producing violations suitable
//! for CI gating (exit code 3).

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-policy";

/// Smoke test.
#[test]
fn policy_smoke() {
    assert_eq!(CRATE_NAME, "azusa-policy");
}

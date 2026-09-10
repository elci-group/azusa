//! Azusa local evidence store.
//!
//! Maintains a compact, append-only, auditable record of drift reports and
//! state identities under `.azusa/` — JSONL history plus content-addressed
//! state digests. Never duplicates entire repositories.

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-store";

/// Smoke test.
#[test]
fn store_smoke() {
    assert_eq!(CRATE_NAME, "azusa-store");
}

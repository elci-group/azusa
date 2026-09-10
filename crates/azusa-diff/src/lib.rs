//! Azusa comparison primitives.
//!
//! Deterministic fileset comparison over two
//! [`azusa_model::FilesystemState`] maps: additions, deletions,
//! modifications, executable-bit changes, rename detection via content and
//! path similarity, line deltas and logical change clustering.

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-diff";

/// Smoke test.
#[test]
fn diff_smoke() {
    assert_eq!(CRATE_NAME, "azusa-diff");
}

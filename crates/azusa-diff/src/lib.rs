//! Azusa comparison primitives.
//!
//! Deterministic fileset comparison over two
//! [`azusa_model::FilesystemState`] maps: additions, deletions,
//! modifications, executable-bit changes, rename detection via content and
//! path similarity, and line deltas. Logical change clustering lives here
//! too (see `cluster`), so "147 changed lines" can become "five logical
//! change clusters".

pub mod cluster;
pub mod fileset;
pub mod similarity;

pub use cluster::{classify, cluster};
pub use fileset::{FilesetDiff, ModifiedFile, Rename, RenameOptions, diff};
pub use similarity::{basename_rename_gate, jaro, jaro_winkler, path_rename_similarity};

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-diff";

/// Smoke test.
#[test]
fn diff_smoke() {
    assert_eq!(CRATE_NAME, "azusa-diff");
}

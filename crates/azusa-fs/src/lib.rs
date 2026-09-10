//! Azusa filesystem state provider.
//!
//! Snapshots plain directories (generated repositories, extracted archives,
//! deployment directories, non-Git projects) into
//! [`azusa_model::RepositoryState`], using ignore-rule walking, parallel
//! content hashing and an incremental stat-based hash cache.

/// Name under which this provider identifies itself.
pub const PROVIDER_NAME: &str = "fs";

/// Smoke test.
#[test]
fn fs_smoke() {
    assert_eq!(PROVIDER_NAME, "fs");
}

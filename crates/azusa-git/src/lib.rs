//! Azusa Git state provider.
//!
//! Resolves state selectors (`worktree`, `index`, `head`, `upstream`,
//! `origin/main`, arbitrary revisions) against a Git repository and snapshots
//! them into [`azusa_model::RepositoryState`]. Git is one state provider
//! among several — it is not Azusa's fundamental abstraction.

/// Name under which this provider identifies itself.
pub const PROVIDER_NAME: &str = "git";

/// Smoke test.
#[test]
fn git_smoke() {
    assert_eq!(PROVIDER_NAME, "git");
}

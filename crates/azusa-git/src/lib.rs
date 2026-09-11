//! Azusa Git state provider.
//!
//! Resolves state selectors (`worktree`, `index`, `head`, `upstream`,
//! `origin/main`, arbitrary revisions) against a Git repository and snapshots
//! them into [`azusa_model::RepositoryState`]. Git is one state provider
//! among several — it is not Azusa's fundamental abstraction.

pub mod error;
pub mod provider;
pub mod selector;

#[cfg(any(test, feature = "fixtures"))]
pub mod testutil;

pub use error::GitError;
pub use provider::{GitProvider, PROVIDER_NAME};
pub use selector::GitSelector;

/// Smoke test.
#[test]
fn git_smoke() {
    assert_eq!(PROVIDER_NAME, "git");
}

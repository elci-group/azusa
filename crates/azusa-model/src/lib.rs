//! Azusa domain model.
//!
//! Pure data structures describing repository states, drift observations,
//! scores, risks, recommendations and reports. This crate intentionally has no
//! I/O and no provider logic: every azusa crate speaks these types, and no UI
//! concept leaks into them.
//!
//! States are modelled, not Git trees: [`RepositoryState`] aggregates
//! identity, ancestry, filesystem, structure, dependency, configuration and
//! behaviour facets, whichever state provider produced them.

/// The state provider that produced a [`crate::RepositoryState`].
///
/// Git is one provider among several: filesystem snapshots, archives and
/// future providers share the same state model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProviderKind {
    /// A state captured from a Git repository (worktree, index, commit, remote branch).
    Git,
    /// A state captured from a plain filesystem tree.
    Filesystem,
}

impl ProviderKind {
    /// Canonical short name used in reports and JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Filesystem => "fs",
        }
    }
}

/// Smoke test: the crate builds and its types are nameable.
#[test]
fn model_smoke() {
    assert_eq!(ProviderKind::Git.as_str(), "git");
    assert_eq!(ProviderKind::Filesystem.as_str(), "fs");
}

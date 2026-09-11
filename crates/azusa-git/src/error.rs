//! Errors for the Git state provider.

/// Errors raised while capturing Git states.
#[derive(Debug, thiserror::Error)]
pub enum GitError {
    /// A libgit2 failure.
    #[error("git error: {0}")]
    Git(#[from] git2::Error),
    /// The directory is not a Git repository.
    #[error("not a git repository: {0}")]
    NotARepository(String),
    /// A selector could not be resolved.
    #[error("cannot resolve git selector `{selector}`: {reason}")]
    UnresolvedSelector {
        /// The selector as requested.
        selector: String,
        /// Why it failed.
        reason: String,
    },
}

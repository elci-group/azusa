//! Errors for the comparison engine.

use std::path::PathBuf;

/// Errors raised while resolving states or comparing them.
#[derive(Debug, thiserror::Error)]
pub enum CoreError {
    /// A state selector could not be resolved.
    #[error("cannot resolve state `{spec}`: {reason}")]
    UnresolvedState {
        /// The requested selector.
        spec: String,
        /// Why resolution failed.
        reason: String,
    },
    /// The working directory is not a usable repository for any provider.
    #[error("no state provider applies at {0:?} (not a git repository, not a directory)")]
    NoProvider(PathBuf),
    /// A provider failed while capturing a state.
    #[error("provider failure: {0}")]
    Provider(String),
    /// A configuration file could not be parsed.
    #[error("configuration error: {0}")]
    Config(String),
}

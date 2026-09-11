//! Errors for the filesystem provider.

use std::path::PathBuf;

/// Errors raised while capturing filesystem states.
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    /// An I/O failure.
    #[error("filesystem error: {0}")]
    Io(#[from] std::io::Error),
    /// The requested root does not exist or is not a directory.
    #[error("not a directory: {0}")]
    NotADirectory(PathBuf),
    /// The persisted hash cache could not be parsed.
    #[error("hash cache parse error: {0}")]
    Cache(String),
}

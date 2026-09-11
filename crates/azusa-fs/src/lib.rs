//! Azusa filesystem state provider.
//!
//! Snapshots plain directories (generated repositories, extracted archives,
//! deployment directories, non-Git projects) into
//! [`azusa_model::RepositoryState`], using ignore-rule walking, parallel
//! content hashing and an incremental stat-based hash cache.

pub mod cache;
pub mod error;
pub mod walk;

pub use cache::{CachedStat, HashCache};
pub use error::FsError;
pub use walk::{STREAM_THRESHOLD, SnapshotOptions, SnapshotOutcome, snapshot, stat_digest};

use std::path::Path;

use azusa_model::{
    ConfigurationState, DependencyState, ProviderKind, RepositoryState, StateIdentity,
    StructureState,
};

/// Name under which this provider identifies itself.
pub const PROVIDER_NAME: &str = "fs";

/// Default runtime directory (relative to a repository root) for provider
/// caches. Always excluded from snapshots.
pub const RUNTIME_DIR: &str = ".azusa";

/// The filesystem state provider.
#[derive(Debug)]
pub struct FsProvider {
    root: std::path::PathBuf,
    options: SnapshotOptions,
    cache: HashCache,
    cache_path: Option<std::path::PathBuf>,
}

impl FsProvider {
    /// Create a provider over `root`.
    pub fn new(root: impl AsRef<Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            options: SnapshotOptions::default(),
            cache: HashCache::default(),
            cache_path: None,
        }
    }

    /// Enable persistent hash caching under `<root>/.azusa/cache/fs-hashes.json`.
    #[must_use]
    pub fn with_persistent_cache(mut self) -> Self {
        self.cache_path = Some(
            self.root
                .join(RUNTIME_DIR)
                .join("cache")
                .join("fs-hashes.json"),
        );
        self.cache = self
            .cache_path
            .as_ref()
            .map(|p| HashCache::load(p))
            .unwrap_or_default();
        self
    }

    /// Override snapshot ignore options.
    #[must_use]
    pub fn with_options(mut self, options: SnapshotOptions) -> Self {
        self.options = options;
        self
    }

    /// The provider root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Snapshot the filesystem into a full [`RepositoryState`].
    ///
    /// Structure, dependency, configuration and behaviour facets are filled
    /// by `azusa-graph` enrichment; this provider captures identity and
    /// filesystem facets.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::NotFound`] when the root is not a
    /// directory; other I/O failures propagate from the walker.
    pub fn snapshot(&mut self) -> std::io::Result<RepositoryState> {
        let outcome = walk::snapshot(&self.root, &self.options, &mut self.cache)?;
        if let Some(path) = &self.cache_path {
            // Best-effort cache persistence; failures never block a snapshot.
            self.cache.save(path).ok();
        }
        let filesystem = outcome.filesystem;
        let captured_text = outcome.captured_text;
        let identity = StateIdentity {
            provider: ProviderKind::Filesystem,
            label: self.root.display().to_string(),
            root: Some(self.root.display().to_string()),
            digest: filesystem.digest.clone(),
            file_count: filesystem.files.len() as u64,
            total_bytes: filesystem.total_bytes,
            vcs: None,
        };
        Ok(RepositoryState {
            identity,
            ancestry: azusa_model::Ancestry::default(),
            filesystem,
            structure: StructureState::default(),
            dependencies: DependencyState::default(),
            configuration: ConfigurationState::default(),
            behaviour: None,
            captured_text,
        })
    }

    /// A cheap stat-only digest, for watch-mode escalation before any full
    /// analysis.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::ErrorKind::NotFound`] when the root is not a
    /// directory.
    pub fn quick_digest(&self) -> std::io::Result<String> {
        walk::stat_digest(&self.root, &self.options)
    }
}

/// Smoke test.
#[test]
fn fs_smoke() {
    assert_eq!(PROVIDER_NAME, "fs");
}

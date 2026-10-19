// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Incremental hash cache.
//!
//! Re-hashing an unchanged repository is the dominant cost of repeated drift
//! checks. The cache maps `(path, size, mtime)` to a previously computed
//! content hash, so unchanged files are never re-read. It persists as JSON
//! under the runtime directory (`.azusa/cache/fs-hashes.json`).

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use azusa_model::ContentHash;

/// Cached stat fingerprint for one path.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedStat {
    /// Content hash computed when the stat matched.
    pub hash: ContentHash,
    /// File size at hash time.
    pub size: u64,
    /// Modification time (seconds since epoch) at hash time.
    pub mtime_secs: u64,
    /// Modification time (nanosecond component) at hash time.
    pub mtime_nanos: u32,
    /// Line count at hash time, when known.
    pub lines: Option<u64>,
}

/// A persistent stat-keyed hash cache.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct HashCache {
    entries: BTreeMap<String, CachedStat>,
    #[serde(skip)]
    hits: u64,
    #[serde(skip)]
    misses: u64,
}

impl HashCache {
    /// Look up a hash for a path whose stat matches a cached fingerprint.
    ///
    /// A hit requires identical size and mtime; anything else is a miss
    /// (conservative: better to re-hash than to trust a stale hash).
    #[must_use]
    pub fn lookup(
        &mut self,
        path: &str,
        size: u64,
        mtime_secs: u64,
        mtime_nanos: u32,
    ) -> Option<ContentHash> {
        match self.entries.get(path) {
            Some(stat)
                if stat.size == size
                    && stat.mtime_secs == mtime_secs
                    && stat.mtime_nanos == mtime_nanos =>
            {
                self.hits += 1;
                Some(stat.hash.clone())
            }
            _ => {
                self.misses += 1;
                None
            }
        }
    }

    /// Look up stat + hash together, so callers avoid re-reading for lines.
    #[must_use]
    pub fn lookup_stat(
        &mut self,
        path: &str,
        size: u64,
        mtime_secs: u64,
        mtime_nanos: u32,
    ) -> Option<CachedStat> {
        match self.entries.get(path) {
            Some(stat)
                if stat.size == size
                    && stat.mtime_secs == mtime_secs
                    && stat.mtime_nanos == mtime_nanos =>
            {
                self.hits += 1;
                Some(stat.clone())
            }
            _ => {
                self.misses += 1;
                None
            }
        }
    }

    /// Record a hash for a path with its stat fingerprint.
    pub fn record(&mut self, path: &str, stat: CachedStat) {
        self.entries.insert(path.to_owned(), stat);
    }

    /// Number of cache hits since the last [`Self::reset_counters`].
    #[must_use]
    pub fn hits(&self) -> u64 {
        self.hits
    }

    /// Number of cache misses since the last [`Self::reset_counters`].
    #[must_use]
    pub fn misses(&self) -> u64 {
        self.misses
    }

    /// Reset hit/miss counters.
    pub fn reset_counters(&mut self) {
        self.hits = 0;
        self.misses = 0;
    }

    /// Drop entries for paths that no longer exist, keeping the cache compact.
    pub fn retain_existing(&mut self, live_paths: &std::collections::BTreeSet<String>) {
        self.entries.retain(|path, _| live_paths.contains(path));
    }

    /// Serialise the cache to JSON bytes.
    ///
    /// Deterministic key order makes the file byte-stable for identical data.
    ///
    /// # Errors
    ///
    /// Propagates serde serialisation failures (practically impossible for
    /// this plain-data type).
    pub fn to_json(&self) -> serde_json::Result<String> {
        serde_json::to_string(self)
    }

    /// Deserialise the cache from JSON bytes.
    ///
    /// # Errors
    ///
    /// Returns [`crate::FsError::Cache`] when the bytes are not valid cache
    /// JSON.
    pub fn from_json(bytes: &[u8]) -> Result<Self, crate::FsError> {
        serde_json::from_slice(bytes).map_err(|e| crate::FsError::Cache(e.to_string()))
    }

    /// Load the cache from a JSON file, returning an empty cache if absent.
    #[must_use]
    pub fn load(path: &Path) -> Self {
        match std::fs::read(path) {
            Ok(bytes) => Self::from_json(&bytes).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Persist the cache to a JSON file, creating parent directories.
    ///
    /// # Errors
    ///
    /// Propagates I/O failures; serialisation failures are reported as
    /// [`std::io::ErrorKind::InvalidData`] via [`std::io::Error::other`].
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let json = self.to_json().map_err(std::io::Error::other)?;
        // Atomic-ish write: temp file + rename, so a crash never truncates
        // the cache.
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, json)?;
        std::fs::rename(&tmp, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_and_lookup() {
        let mut cache = HashCache::default();
        cache.record(
            "a.txt",
            CachedStat {
                hash: "h1".into(),
                size: 10,
                mtime_secs: 5,
                mtime_nanos: 0,
                lines: Some(2),
            },
        );
        let json = cache.to_json().unwrap();
        let mut loaded = HashCache::from_json(json.as_bytes()).unwrap();
        assert_eq!(loaded.lookup("a.txt", 10, 5, 0), Some("h1".to_string()));
        assert_eq!(loaded.lookup("a.txt", 11, 5, 0), None);
        assert_eq!(loaded.lookup("missing", 1, 1, 0), None);
        assert_eq!(loaded.hits(), 1);
        assert_eq!(loaded.misses(), 2);
    }

    #[test]
    fn save_and_load_file() {
        let dir = tempfile_dir();
        let mut cache = HashCache::default();
        cache.record(
            "x",
            CachedStat {
                hash: "h".into(),
                size: 1,
                mtime_secs: 1,
                mtime_nanos: 1,
                lines: None,
            },
        );
        let path = dir.path().join("nested").join("cache.json");
        cache.save(&path).unwrap();
        let mut loaded = HashCache::load(&path);
        assert_eq!(loaded.lookup("x", 1, 1, 1), Some("h".to_string()));
        std::fs::remove_dir_all(dir.path()).ok();
    }

    fn tempfile_dir() -> tempfile::TempDir {
        tempfile::tempdir().expect("tempdir")
    }
}

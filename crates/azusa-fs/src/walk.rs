//! Ignore-aware directory walking and snapshotting.
//!
//! The walker respects `.gitignore` files (via the `ignore` crate), always
//! skips `.git` directories, and hashes files in parallel with rayon, reusing
//! the stat-keyed [`HashCache`] so unchanged files are never re-read.

use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use rayon::prelude::*;

use azusa_model::{
    FileEntry, FileKind, FilesystemState, content_hash, content_hash_reader, count_lines,
    fileset_digest, looks_like_text,
};

use crate::cache::{CachedStat, HashCache};

/// Files larger than this are streamed rather than read into memory
/// (hashes are identical; the threshold only bounds peak memory).
pub const STREAM_THRESHOLD: u64 = 4 * 1024 * 1024;

/// Options controlling a filesystem snapshot.
#[derive(Debug, Clone)]
pub struct SnapshotOptions {
    /// Extra path prefixes to skip, in addition to `.git` and ignore rules.
    /// Supports `*/name` patterns to skip a directory at any depth.
    pub extra_ignores: Vec<String>,
}

impl Default for SnapshotOptions {
    fn default() -> Self {
        Self {
            extra_ignores: vec![".azusa".to_owned(), "target".to_owned()],
        }
    }
}

/// One walked path with its stat fingerprint, produced before hashing.
#[derive(Debug)]
struct WalkedFile {
    /// Repository-relative path, forward slashes.
    rel: String,
    /// Absolute path on disk.
    abs: PathBuf,
    /// File size in bytes.
    size: u64,
    /// mtime seconds component.
    mtime_secs: u64,
    /// mtime nanoseconds component.
    mtime_nanos: u32,
    /// Executable bit.
    executable: bool,
    /// Whether the entry is a symbolic link.
    is_symlink: bool,
}

/// Collect all files under `root` that survive ignore rules.
fn walk_paths(root: &Path, options: &SnapshotOptions) -> Vec<WalkedFile> {
    let mut walker = ignore::WalkBuilder::new(root);
    walker
        .hidden(false)
        .git_ignore(true)
        .git_global(true)
        .git_exclude(true)
        .parents(true);

    let root = root.to_path_buf();
    let extra_ignores = options.extra_ignores.clone();
    let mut out = Vec::new();
    for entry in walker.build().flatten() {
        let Some(file_type) = entry.file_type() else {
            continue;
        };
        let path = entry.path();

        // Skip our own runtime directory and VCS internals everywhere.
        if is_skipped_path(path, &root, &extra_ignores) {
            continue;
        }

        if file_type.is_dir() {
            continue;
        }
        if file_type.is_symlink() {
            out.push(WalkedFile {
                rel: relpath(path, &root),
                abs: path.to_path_buf(),
                size: 0,
                mtime_secs: 0,
                mtime_nanos: 0,
                executable: false,
                is_symlink: true,
            });
            continue;
        }
        if !file_type.is_file() {
            continue;
        }
        let Ok(meta) = entry.metadata() else { continue };
        #[cfg(unix)]
        let executable = {
            use std::os::unix::fs::PermissionsExt;
            meta.permissions().mode() & 0o111 != 0
        };
        #[cfg(not(unix))]
        let executable = false;
        let mtime = meta
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok());
        let (secs, nanos) = mtime.map_or((0, 0), |d| (d.as_secs(), d.subsec_nanos()));
        out.push(WalkedFile {
            rel: relpath(path, &root),
            abs: path.to_path_buf(),
            size: meta.len(),
            mtime_secs: secs,
            mtime_nanos: nanos,
            executable,
            is_symlink: false,
        });
    }
    out
}

fn is_skipped_path(path: &Path, root: &Path, extra: &[String]) -> bool {
    let Ok(rel) = path.strip_prefix(root) else {
        return false;
    };
    if rel.components().any(|c| c.as_os_str() == ".git") {
        return true;
    }
    let rel_str = rel.to_string_lossy().replace('\\', "/");
    extra.iter().any(|prefix| {
        if let Some(stripped) = prefix.strip_prefix("*/") {
            // `*/name` matches a directory component at any depth.
            rel_str.split('/').any(|component| component == stripped)
        } else {
            rel_str == *prefix || rel_str.starts_with(&format!("{prefix}/"))
        }
    })
}

fn relpath(path: &Path, root: &Path) -> String {
    path.strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Result of a snapshot: the filesystem facet, retained analysis text and
/// cache statistics.
#[derive(Debug, Default)]
pub struct SnapshotOutcome {
    /// The captured filesystem facet.
    pub filesystem: FilesystemState,
    /// UTF-8 text of analysis-interest files (size-capped), keyed by path.
    pub captured_text: std::collections::BTreeMap<String, String>,
    /// Hash-cache hits during this snapshot.
    pub cache_hits: u64,
    /// Hash-cache misses during this snapshot.
    pub cache_misses: u64,
}

/// Hash one walked file, streaming large files and skipping vanished ones.
/// When the file is of analysis interest and small, retain its UTF-8 text.
fn hash_walked(
    file: &WalkedFile,
    captured_text: &std::sync::Mutex<std::collections::BTreeMap<String, String>>,
) -> Option<(String, Option<u64>)> {
    if file.is_symlink {
        let target = std::fs::read_link(&file.abs).ok()?;
        let hash = content_hash(target.to_string_lossy().as_bytes());
        return Some((hash, None));
    }
    if file.size > STREAM_THRESHOLD {
        let hash = File::open(&file.abs)
            .ok()
            .and_then(|f| content_hash_reader(f).ok())?;
        return Some((hash, None));
    }
    let bytes = std::fs::read(&file.abs).ok()?;
    if azusa_model::is_analysis_file(&file.rel)
        && bytes.len() <= azusa_model::CAPTURED_TEXT_MAX_BYTES
    {
        if let Ok(text) = std::str::from_utf8(&bytes) {
            captured_text
                .lock()
                .expect("captured-text mutex")
                .insert(file.rel.clone(), text.to_owned());
        }
    }
    let lines = looks_like_text(&bytes).then(|| count_lines(&bytes));
    Some((content_hash(&bytes), lines))
}

/// Capture a filesystem snapshot of `root`.
///
/// Deterministic: the resulting [`FilesystemState`] depends only on file
/// content, paths and modes — never on mtimes (which feed only the cache).
///
/// # Errors
///
/// Returns [`std::io::ErrorKind::NotFound`] when `root` is not a directory;
/// I/O failures during walking are skipped (vanishing files) rather than
/// failing the snapshot.
///
/// # Panics
///
/// Panics only if the internal hash-collection mutex is poisoned by a panic
/// in a hashing thread.
pub fn snapshot(
    root: &Path,
    options: &SnapshotOptions,
    cache: &mut HashCache,
) -> std::io::Result<SnapshotOutcome> {
    if !root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("not a directory: {}", root.display()),
        ));
    }
    let walked = walk_paths(root, options);
    cache.reset_counters();

    // Cache lookups are cheap; do them serially, then hash misses in parallel.
    let mut pending: Vec<WalkedFile> = Vec::new();
    let mut files: BTreeMap<String, FileEntry> = BTreeMap::new();
    for file in walked {
        let kind = if file.is_symlink {
            FileKind::Symlink
        } else {
            FileKind::File
        };
        if let Some(stat) =
            cache.lookup_stat(&file.rel, file.size, file.mtime_secs, file.mtime_nanos)
        {
            files.insert(
                file.rel,
                FileEntry {
                    hash: stat.hash,
                    size: file.size,
                    executable: file.executable,
                    kind,
                    lines: stat.lines,
                },
            );
        } else {
            pending.push(file);
        }
    }

    let hashed: Mutex<Vec<(String, FileEntry, CachedStat)>> = Mutex::new(Vec::new());
    let captured_text: Mutex<BTreeMap<String, String>> = Mutex::new(BTreeMap::new());
    pending.into_par_iter().for_each(|file| {
        let Some((hash, lines)) = hash_walked(&file, &captured_text) else {
            return;
        };
        let entry = FileEntry {
            hash,
            size: file.size,
            executable: file.executable,
            kind: if file.is_symlink {
                FileKind::Symlink
            } else {
                FileKind::File
            },
            lines,
        };
        let stat = CachedStat {
            hash: entry.hash.clone(),
            size: file.size,
            mtime_secs: file.mtime_secs,
            mtime_nanos: file.mtime_nanos,
            lines: entry.lines,
        };
        hashed
            .lock()
            .expect("hash mutex")
            .push((file.rel, entry, stat));
    });

    for (rel, entry, stat) in hashed.into_inner().expect("hash mutex") {
        cache.record(&rel, stat);
        files.insert(rel, entry);
    }

    let mut captured_text = captured_text.into_inner().expect("captured-text mutex");
    // Cache-hit interest files were never re-read; capture their text with a
    // cheap serial pass (few, small files by definition).
    for path in files.keys() {
        if !azusa_model::is_analysis_file(path) || captured_text.contains_key(path) {
            continue;
        }
        let abs = root.join(path);
        if let Ok(bytes) = std::fs::read(&abs) {
            if bytes.len() <= azusa_model::CAPTURED_TEXT_MAX_BYTES {
                if let Ok(text) = std::str::from_utf8(&bytes) {
                    captured_text.insert(path.clone(), text.to_owned());
                }
            }
        }
    }
    let total_bytes = files.values().map(|e| e.size).sum();
    let digest = fileset_digest(files.iter().map(|(path, entry)| {
        (
            entry.kind.mode_string(entry.executable).to_owned(),
            entry.hash.clone(),
            path.clone(),
        )
    }));
    let hits = cache.hits();
    let misses = cache.misses();
    Ok(SnapshotOutcome {
        filesystem: FilesystemState {
            files,
            total_bytes,
            digest,
        },
        captured_text,
        cache_hits: hits,
        cache_misses: misses,
    })
}

/// A cheap stat-only digest for watch-mode escalation: hashes only
/// `(path, size, mtime)` triples, never file content.
///
/// # Errors
///
/// Returns [`std::io::ErrorKind::NotFound`] when `root` is not a directory.
pub fn stat_digest(root: &Path, options: &SnapshotOptions) -> std::io::Result<String> {
    if !root.is_dir() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("not a directory: {}", root.display()),
        ));
    }
    let mut hasher = blake3::Hasher::new();
    let mut triples: Vec<(String, u64, u64, u32)> = walk_paths(root, options)
        .into_iter()
        .map(|f| (f.rel, f.size, f.mtime_secs, f.mtime_nanos))
        .collect();
    triples.sort_unstable();
    for (rel, size, secs, nanos) in triples {
        hasher.update(rel.as_bytes());
        hasher.update(&size.to_le_bytes());
        hasher.update(&secs.to_le_bytes());
        hasher.update(&nanos.to_le_bytes());
    }
    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    fn write(path: &Path, content: &str) {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }

    #[test]
    fn snapshot_respects_gitignore_and_skips_runtime_dirs() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(&root.join("src/main.rs"), "fn main() {}\n");
        write(&root.join("ignored.log"), "noise");
        write(&root.join(".git/config"), "[core]");
        write(&root.join(".azusa/cache/fs-hashes.json"), "{}");
        write(&root.join("target/debug/x.o"), "junk");
        std::fs::write(root.join(".gitignore"), "ignored.log\n").unwrap();

        let mut cache = HashCache::default();
        let outcome = snapshot(root, &SnapshotOptions::default(), &mut cache).unwrap();
        let paths: BTreeSet<String> = outcome.filesystem.files.keys().cloned().collect();
        assert_eq!(
            paths,
            BTreeSet::from([".gitignore".to_owned(), "src/main.rs".to_owned()])
        );
    }

    #[test]
    fn snapshot_is_deterministic_and_cache_reuses_hashes() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(&root.join("a.txt"), "alpha\n");
        write(&root.join("b.bin"), "\u{0}\u{1}binary");

        let mut cache = HashCache::default();
        let first = snapshot(root, &SnapshotOptions::default(), &mut cache).unwrap();
        assert_eq!(first.cache_misses, 2);
        let second = snapshot(root, &SnapshotOptions::default(), &mut cache).unwrap();
        assert_eq!(second.cache_hits, 2);
        assert_eq!(first.filesystem.digest, second.filesystem.digest);
        // Text files get line counts; binary-looking files do not.
        assert_eq!(first.filesystem.files["a.txt"].lines, Some(1));
        assert_eq!(first.filesystem.files["b.bin"].lines, None);
    }

    #[test]
    fn snapshot_tracks_executable_bit() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(&root.join("run.sh"), "#!/bin/sh\n");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(root.join("run.sh"), std::fs::Permissions::from_mode(0o755))
                .unwrap();
        }
        let mut cache = HashCache::default();
        let outcome = snapshot(root, &SnapshotOptions::default(), &mut cache).unwrap();
        #[cfg(unix)]
        assert!(outcome.filesystem.files["run.sh"].executable);
    }

    #[test]
    fn stat_digest_detects_new_files() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        write(&root.join("x.txt"), "one\n");
        let options = SnapshotOptions::default();
        let before = stat_digest(root, &options).unwrap();
        write(&root.join("y.txt"), "two\n");
        let after = stat_digest(root, &options).unwrap();
        assert_ne!(before, after);
    }
}

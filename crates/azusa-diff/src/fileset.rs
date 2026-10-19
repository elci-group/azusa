// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Fileset comparison.
//!
//! Deterministic diff of two [`FilesystemState`]s: additions, deletions,
//! modifications, executable-mode changes and rename detection. Renames pair
//! a deleted path with an added path when content hashes match exactly (a
//! pure move) or when path similarity and size proximity are both high.

use std::collections::BTreeMap;

use azusa_model::{FileEntry, FilesystemState};

use crate::similarity::path_rename_similarity;

/// Options for rename detection.
#[derive(Debug, Clone)]
pub struct RenameOptions {
    /// Minimum path similarity (0.0–1.0) to pair a deletion with an addition.
    pub path_threshold: f64,
    /// Minimum size ratio (smaller/larger) for a similarity-based pairing.
    pub size_ratio_threshold: f64,
    /// Files smaller than this never pair by path similarity alone — tiny
    /// files with similar names are usually distinct files, not moves.
    /// Exact-content pairings are exempt (their hashes match).
    pub min_similarity_size: u64,
}

impl Default for RenameOptions {
    fn default() -> Self {
        Self {
            path_threshold: 0.85,
            size_ratio_threshold: 0.5,
            min_similarity_size: 16,
        }
    }
}

/// One modified path.
#[derive(Debug, Clone, PartialEq)]
pub struct ModifiedFile {
    /// The path.
    pub path: String,
    /// Entry in state A.
    pub old: FileEntry,
    /// Entry in state B.
    pub new: FileEntry,
    /// Change in line count (new − old), when both sides know line counts.
    pub line_delta: Option<i64>,
    /// Whether only the executable bit changed (content identical).
    pub mode_only: bool,
}

/// One detected rename or move.
#[derive(Debug, Clone, PartialEq)]
pub struct Rename {
    /// Path in state A.
    pub from: String,
    /// Path in state B.
    pub to: String,
    /// Similarity score 0.0–1.0 (1.0 = identical content).
    pub similarity: f64,
    /// Whether content also changed during the move.
    pub content_changed: bool,
}

/// The full diff between two filesets.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct FilesetDiff {
    /// Paths present only in B (additions, from A's perspective).
    pub added: BTreeMap<String, FileEntry>,
    /// Paths present only in A (deletions, from A's perspective).
    pub deleted: BTreeMap<String, FileEntry>,
    /// Paths present in both with different content or mode.
    pub modified: Vec<ModifiedFile>,
    /// Detected renames/moves.
    pub renamed: Vec<Rename>,
    /// Paths present and identical in both states.
    pub unchanged: usize,
}

impl FilesetDiff {
    /// Number of changed paths (added + deleted + modified + renamed).
    #[must_use]
    pub fn change_count(&self) -> usize {
        self.added.len() + self.deleted.len() + self.modified.len() + self.renamed.len()
    }

    /// Whether the two filesets are identical.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.change_count() == 0
    }

    /// Total bytes added by B relative to A (added + grown modifications).
    #[must_use]
    pub fn bytes_added(&self) -> u64 {
        self.added.values().map(|e| e.size).sum::<u64>()
            + self
                .modified
                .iter()
                .map(|m| m.new.size.saturating_sub(m.old.size))
                .sum::<u64>()
    }

    /// Total bytes removed (deleted + shrunk modifications).
    #[must_use]
    pub fn bytes_removed(&self) -> u64 {
        self.deleted.values().map(|e| e.size).sum::<u64>()
            + self
                .modified
                .iter()
                .map(|m| m.old.size.saturating_sub(m.new.size))
                .sum::<u64>()
    }
}

/// Compute the fileset diff from A to B.
#[must_use]
pub fn diff(a: &FilesystemState, b: &FilesystemState, options: &RenameOptions) -> FilesetDiff {
    let mut result = FilesetDiff {
        added: BTreeMap::new(),
        deleted: BTreeMap::new(),
        modified: Vec::new(),
        renamed: Vec::new(),
        unchanged: 0,
    };

    for (path, old) in &a.files {
        match b.files.get(path) {
            Some(new) if new.hash == old.hash && new.executable == old.executable => {
                result.unchanged += 1;
            }
            Some(new) => {
                let line_delta = match (old.lines, new.lines) {
                    (Some(o), Some(n)) => Some(n as i64 - o as i64),
                    _ => None,
                };
                result.modified.push(ModifiedFile {
                    path: path.clone(),
                    old: old.clone(),
                    new: new.clone(),
                    line_delta,
                    mode_only: new.hash == old.hash,
                });
            }
            None => {
                result.deleted.insert(path.clone(), old.clone());
            }
        }
    }
    for (path, entry) in &b.files {
        if !a.files.contains_key(path) {
            result.added.insert(path.clone(), entry.clone());
        }
    }

    detect_renames(&mut result, options);
    result
}

/// Pair deletions with additions to detect renames/moves.
fn detect_renames(diff: &mut FilesetDiff, options: &RenameOptions) {
    // Exact-content moves first: highest confidence, greedy by best path
    // similarity among hash-equal candidates.
    let mut by_hash: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for (path, entry) in &diff.added {
        by_hash
            .entry(entry.hash.clone())
            .or_default()
            .push(path.clone());
    }
    for from in diff.deleted.keys().cloned().collect::<Vec<_>>() {
        let Some(candidates) = by_hash.get_mut(&diff.deleted[&from].hash) else {
            continue;
        };
        if candidates.is_empty() {
            continue;
        }
        let best = candidates
            .iter()
            .max_by(|x, y| {
                path_rename_similarity(&from, x)
                    .partial_cmp(&path_rename_similarity(&from, y))
                    .expect("no NaN in similarity")
            })
            .cloned()
            .expect("candidate list is non-empty");
        candidates.retain(|p| *p != best);
        diff.renamed.push(Rename {
            from: from.clone(),
            to: best.clone(),
            similarity: 1.0,
            content_changed: false,
        });
        diff.deleted.remove(&from);
        diff.added.remove(&best);
    }

    // Similarity-based moves: same size class and high path similarity.
    let added_paths: Vec<String> = diff.added.keys().cloned().collect();
    let mut used: Vec<String> = Vec::new();
    for from in diff.deleted.keys().cloned().collect::<Vec<_>>() {
        let old_size = diff.deleted[&from].size;
        let best = added_paths
            .iter()
            .filter(|to| !used.contains(to))
            .map(|to| {
                let entry = &diff.added[to.as_str()];
                (to.clone(), path_rename_similarity(&from, to), entry.size)
            })
            .filter(|(to, sim, size)| {
                *sim >= options.path_threshold
                    && crate::similarity::basename_rename_gate(&from, to)
                    && old_size >= options.min_similarity_size
                    && size_ratio_u64(old_size, *size) >= options.size_ratio_threshold
            })
            .max_by(|x, y| x.1.partial_cmp(&y.1).expect("no NaN in similarity"));
        if let Some((to, sim, _)) = best {
            used.push(to.clone());
            diff.renamed.push(Rename {
                from: from.clone(),
                to: to.clone(),
                similarity: sim,
                content_changed: true,
            });
            diff.added.remove(&to);
            diff.deleted.remove(&from);
        }
    }
}

fn size_ratio_u64(a: u64, b: u64) -> f64 {
    let smaller = a.min(b) as f64;
    let larger = a.max(b) as f64;
    if larger == 0.0 { 1.0 } else { smaller / larger }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_model::content_hash;

    fn entry(content: &str) -> FileEntry {
        FileEntry {
            hash: content_hash(content.as_bytes()),
            size: content.len() as u64,
            executable: false,
            kind: azusa_model::FileKind::File,
            lines: Some(content.lines().count() as u64),
        }
    }

    fn state(files: &[(&str, FileEntry)]) -> FilesystemState {
        FilesystemState {
            files: files
                .iter()
                .map(|(p, e)| ((*p).to_owned(), e.clone()))
                .collect(),
            total_bytes: files.iter().map(|(_, e)| e.size).sum(),
            digest: String::new(),
        }
    }

    #[test]
    fn classifies_add_delete_modify_unchanged() {
        let a = state(&[
            ("keep.txt", entry("same")),
            ("mod.txt", entry("old")),
            ("gone.txt", entry("bye")),
        ]);
        let b = state(&[
            ("keep.txt", entry("same")),
            ("mod.txt", entry("new")),
            ("new.txt", entry("hello")),
        ]);
        let d = diff(&a, &b, &RenameOptions::default());
        assert_eq!(d.unchanged, 1);
        assert_eq!(d.added.len(), 1);
        assert_eq!(d.deleted.len(), 1);
        assert_eq!(d.modified.len(), 1);
        assert_eq!(d.change_count(), 3);
        assert_eq!(d.modified[0].line_delta, Some(0));
    }

    #[test]
    fn detects_exact_content_move() {
        let a = state(&[("old/place.rs", entry("fn x() {}"))]);
        let b = state(&[("new/place.rs", entry("fn x() {}"))]);
        let d = diff(&a, &b, &RenameOptions::default());
        assert_eq!(d.renamed.len(), 1);
        assert_eq!(d.renamed[0].similarity, 1.0);
        assert!(!d.renamed[0].content_changed);
        assert!(d.added.is_empty() && d.deleted.is_empty());
    }

    #[test]
    fn detects_similar_path_move_with_content_change() {
        let long = "pub fn login() { /* long enough to look like a real source file */ }\n";
        let longer = "pub fn login() { /* long enough to look like a real source file, v2 */ }\n";
        let a = state(&[("src/auth.rs", entry(long))]);
        let b = state(&[("src/session/auth.rs", entry(longer))]);
        let d = diff(&a, &b, &RenameOptions::default());
        assert_eq!(d.renamed.len(), 1, "renames: {:?}", d.renamed);
        assert!(d.renamed[0].content_changed);
    }

    #[test]
    fn unrelated_paths_are_not_renamed() {
        let a = state(&[("src/auth.rs", entry("fn a() {}"))]);
        let b = state(&[("docs/billing.md", entry("# billing notes\nmore\nmore\n"))]);
        let d = diff(&a, &b, &RenameOptions::default());
        assert!(d.renamed.is_empty(), "no rename should be detected");
        assert_eq!(d.added.len(), 1);
        assert_eq!(d.deleted.len(), 1);
    }

    #[test]
    fn tiny_similarly_named_files_are_not_renamed() {
        let a = state(&[("only-a.txt", entry("a\n"))]);
        let b = state(&[("only-b.txt", entry("b\n"))]);
        let d = diff(&a, &b, &RenameOptions::default());
        assert!(d.renamed.is_empty(), "renames: {:?}", d.renamed);
        assert_eq!(d.added.len(), 1);
        assert_eq!(d.deleted.len(), 1);
    }

    #[test]
    fn mode_only_change_is_flagged() {
        let mut e1 = entry("echo one\n");
        e1.executable = false;
        let mut e2 = entry("echo one\n");
        e2.executable = true;
        let a = state(&[("run.sh", e1)]);
        let b = state(&[("run.sh", e2)]);
        let d = diff(&a, &b, &RenameOptions::default());
        assert_eq!(d.modified.len(), 1);
        assert!(d.modified[0].mode_only);
    }
}

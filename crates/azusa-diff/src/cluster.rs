// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Change clustering.
//!
//! Groups a [`FilesetDiff`] into logical clusters so a wall of changed paths
//! reads as "authentication, migrations, CLI, tests, formatting". Purely
//! path-based, deterministic, provider-independent.

use std::collections::BTreeMap;

use azusa_model::{ChangeCluster, ClusterKind};

use crate::fileset::FilesetDiff;

/// Maximum paths listed per cluster.
const MAX_LISTED_PATHS: usize = 6;

/// Cluster every changed path in the diff.
#[must_use]
pub fn cluster(diff: &FilesetDiff) -> Vec<ChangeCluster> {
    struct Tally {
        kind: ClusterKind,
        files: u64,
        added: u64,
        deleted: u64,
        modified: u64,
        renamed: u64,
        paths: Vec<String>,
    }

    let mut clusters: BTreeMap<String, Tally> = BTreeMap::new();
    let mut record = |label: String, kind: ClusterKind, path: String, change: Change| {
        let tally = clusters.entry(label).or_insert(Tally {
            kind,
            files: 0,
            added: 0,
            deleted: 0,
            modified: 0,
            renamed: 0,
            paths: Vec::new(),
        });
        tally.files += 1;
        match change {
            Change::Added => tally.added += 1,
            Change::Deleted => tally.deleted += 1,
            Change::Modified => tally.modified += 1,
            Change::Renamed => tally.renamed += 1,
        }
        if tally.paths.len() < MAX_LISTED_PATHS + 1 {
            tally.paths.push(path);
        }
    };

    for path in diff.added.keys() {
        let (label, kind) = classify(path);
        record(label, kind, path.clone(), Change::Added);
    }
    for path in diff.deleted.keys() {
        let (label, kind) = classify(path);
        record(label, kind, path.clone(), Change::Deleted);
    }
    for modified in &diff.modified {
        let (label, kind) = classify(&modified.path);
        record(label, kind, modified.path.clone(), Change::Modified);
    }
    for rename in &diff.renamed {
        let (label, kind) = classify(&rename.to);
        record(
            label,
            kind,
            format!("{} → {}", rename.from, rename.to),
            Change::Renamed,
        );
    }

    clusters
        .into_iter()
        .map(|(label, tally)| ChangeCluster {
            label,
            kind: tally.kind,
            files: tally.files,
            added: tally.added,
            deleted: tally.deleted,
            modified: tally.modified,
            renamed: tally.renamed,
            paths: cap_paths(tally.paths),
        })
        .collect()
}

enum Change {
    Added,
    Deleted,
    Modified,
    Renamed,
}

fn cap_paths(mut paths: Vec<String>) -> Vec<String> {
    if paths.len() > MAX_LISTED_PATHS {
        paths.truncate(MAX_LISTED_PATHS);
        paths.push("…".to_owned());
    }
    paths
}

/// Classify one path into (cluster label, cluster kind).
#[must_use]
pub fn classify(path: &str) -> (String, ClusterKind) {
    let name = path.rsplit('/').next().unwrap_or(path);

    let is_lock = name == "Cargo.lock"
        || std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lock"));
    // Lockfiles first: cross-cutting and almost always their own cluster.
    if is_lock {
        return ("lockfiles".to_owned(), ClusterKind::Lockfile);
    }
    if name == "Cargo.toml" {
        return ("manifests".to_owned(), ClusterKind::Manifest);
    }
    if is_test_path(path) {
        return ("tests".to_owned(), ClusterKind::Tests);
    }
    let extension = std::path::Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase);
    if extension.as_deref() == Some("md")
        || path.starts_with("docs/")
        || name == "LICENSE"
        || name == "README"
    {
        return ("docs".to_owned(), ClusterKind::Documentation);
    }
    if is_operations_path(path, name) {
        return ("operations".to_owned(), ClusterKind::Operations);
    }
    if is_configuration_path(name) {
        return ("configuration".to_owned(), ClusterKind::Configuration);
    }

    // Module clusters: crates/<name>/… labels by the crate; other trees by
    // their first path component.
    if let Some(rest) = path.strip_prefix("crates/") {
        let crate_name = rest.split('/').next().unwrap_or(rest);
        return (format!("crates/{crate_name}"), ClusterKind::Module);
    }
    match path.split_once('/') {
        Some((top, _)) => (top.to_owned(), ClusterKind::Module),
        None => ("root".to_owned(), ClusterKind::Other),
    }
}

fn is_test_path(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.starts_with("tests/")
        || lower.contains("/tests/")
        || lower.starts_with("benches/")
        || lower.contains("/benches/")
        || lower.ends_with("_test.rs")
        || lower.ends_with(".test.js")
        || lower.ends_with(".spec.ts")
}

fn is_operations_path(path: &str, name: &str) -> bool {
    path.starts_with(".github/")
        || path.starts_with("deploy/")
        || path.starts_with("k8s/")
        || path.contains("/k8s/")
        || path.contains("/deploy/")
        || name.starts_with("Dockerfile")
        || name.starts_with("docker-compose")
        || name.starts_with("compose.")
        || name.ends_with(".service")
}

fn is_configuration_path(name: &str) -> bool {
    matches!(
        name,
        "rustfmt.toml" | ".rustfmt.toml" | "clippy.toml" | "deny.toml" | ".editorconfig"
    ) || name.starts_with("rust-toolchain")
        || name.starts_with(".env")
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_model::{FileEntry, FileKind, FilesystemState, content_hash};

    fn entry(content: &str) -> FileEntry {
        FileEntry {
            hash: content_hash(content.as_bytes()),
            size: content.len() as u64,
            executable: false,
            kind: FileKind::File,
            lines: Some(1),
        }
    }

    fn state(files: &[(&str, &str)]) -> FilesystemState {
        FilesystemState {
            files: files
                .iter()
                .map(|(p, c)| ((*p).to_owned(), entry(c)))
                .collect(),
            total_bytes: 0,
            digest: String::new(),
        }
    }

    #[test]
    fn clusters_group_by_logical_area() {
        let a = state(&[
            ("crates/core/src/lib.rs", "one"),
            ("crates/core/src/auth.rs", "one"),
            ("tests/integration.rs", "one"),
            ("Cargo.lock", "one"),
            ("README.md", "one"),
        ]);
        let b = state(&[
            ("crates/core/src/lib.rs", "two"),
            ("crates/core/src/auth.rs", "two"),
            ("tests/integration.rs", "two"),
            ("Cargo.lock", "two"),
            ("README.md", "two"),
            ("crates/cli/src/main.rs", "new"),
            (".github/workflows/ci.yml", "new"),
        ]);
        let diff = crate::diff(&a, &b, &crate::RenameOptions::default());
        let clusters = cluster(&diff);

        let labels: Vec<(String, ClusterKind, u64)> = clusters
            .iter()
            .map(|c| (c.label.clone(), c.kind, c.files))
            .collect();
        assert!(labels.contains(&("crates/core".to_owned(), ClusterKind::Module, 2)));
        assert!(labels.contains(&("crates/cli".to_owned(), ClusterKind::Module, 1)));
        assert!(labels.contains(&("tests".to_owned(), ClusterKind::Tests, 1)));
        assert!(labels.contains(&("lockfiles".to_owned(), ClusterKind::Lockfile, 1)));
        assert!(labels.contains(&("docs".to_owned(), ClusterKind::Documentation, 1)));
        assert!(labels.contains(&("operations".to_owned(), ClusterKind::Operations, 1)));
        // Deterministic ordering by label.
        assert!(labels.windows(2).all(|w| w[0].0 < w[1].0));
    }

    #[test]
    fn cluster_path_listing_is_capped() {
        let mut files_a = Vec::new();
        let mut files_b = Vec::new();
        for i in 0..10 {
            files_a.push((format!("crates/big/src/file{i}.rs"), "one"));
            files_b.push((format!("crates/big/src/file{i}.rs"), "two"));
        }
        let a = state(
            &files_a
                .iter()
                .map(|(p, c)| (p.as_str(), *c))
                .collect::<Vec<_>>(),
        );
        let b = state(
            &files_b
                .iter()
                .map(|(p, c)| (p.as_str(), *c))
                .collect::<Vec<_>>(),
        );
        let diff = crate::diff(&a, &b, &crate::RenameOptions::default());
        let clusters = cluster(&diff);
        assert_eq!(clusters.len(), 1);
        assert_eq!(clusters[0].files, 10);
        assert!(clusters[0].paths.len() <= 7); // 6 listed + ellipsis
        assert!(clusters[0].paths.last().is_some_and(|p| p == "…"));
    }
}

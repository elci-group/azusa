// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Behavioural facet derivation (layers L6–L7 inputs).
//!
//! Classifies the behavioural surface of a captured state from its
//! filesystem listing and retained analysis text: test files, benches,
//! executable entry points, build scripts, test declarations and the
//! public item count per Rust source file. A pure function of the state —
//! no I/O, no parsing beyond line inspection — so enrichment stays
//! deterministic.

use azusa_model::{BehaviourState, RepositoryState};

/// Derive the behaviour facet of a captured state.
#[must_use]
pub fn behaviour_of(state: &RepositoryState) -> BehaviourState {
    let mut behaviour = BehaviourState::default();

    for path in state.filesystem.files.keys() {
        if let Some(role) = classify_path(path) {
            match role {
                Role::Test => behaviour.test_files.push(path.clone()),
                Role::Bench => behaviour.bench_files.push(path.clone()),
                Role::EntryPoint => behaviour.entry_points.push(path.clone()),
                Role::BuildScript => behaviour.build_scripts.push(path.clone()),
            }
        }
    }

    for (path, text) in &state.captured_text {
        if is_rust_source(path) {
            if is_test_text(text) && !behaviour.test_files.contains(path) {
                behaviour.test_files.push(path.clone());
            }
            behaviour.test_declarations += count_lines_with(text, "#[test]");
            let public_items = count_public_items(text);
            if public_items > 0 {
                behaviour
                    .public_items_per_file
                    .insert(path.clone(), public_items);
            }
        }
    }

    behaviour
}

enum Role {
    Test,
    Bench,
    EntryPoint,
    BuildScript,
}

/// Path-based behavioural role of a file, if any.
fn classify_path(path: &str) -> Option<Role> {
    let segments: Vec<&str> = path.split('/').collect();
    let is_rust = is_rust_source(path);
    match segments.as_slice() {
        [.., "src", "main.rs"] => Some(Role::EntryPoint),
        [.., "src", "bin", _] if is_rust => Some(Role::EntryPoint),
        [.., "tests", _] if is_rust => Some(Role::Test),
        [.., "benches", _] if is_rust => Some(Role::Bench),
        [.., "build.rs"] => Some(Role::BuildScript),
        _ => None,
    }
}

/// Whether retained Rust text carries a unit-test module.
fn is_test_text(text: &str) -> bool {
    text.lines().any(|line| {
        let trimmed = line.trim_start();
        trimmed.starts_with("#[cfg(test)]")
    })
}

/// Whether a captured path is Rust source text.
fn is_rust_source(path: &str) -> bool {
    std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"))
}

/// Count lines containing a marker at declaration position.
fn count_lines_with(text: &str, marker: &str) -> u64 {
    text.lines().filter(|line| line.contains(marker)).count() as u64
}

/// Count heuristic public API items in Rust text: `pub fn`, `pub struct`,
/// `pub enum`, `pub trait`, `pub mod`, `pub const`, `pub static`,
/// `pub type`, `pub use`.
fn count_public_items(text: &str) -> u64 {
    const KEYWORDS: [&str; 9] = [
        "pub fn", "pub struct", "pub enum", "pub trait", "pub mod", "pub const", "pub static",
        "pub type", "pub use",
    ];
    text.lines()
        .filter(|line| {
            let trimmed = line.trim_start();
            KEYWORDS.iter().any(|kw| trimmed.starts_with(kw))
        })
        .count() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_model::{FileEntry, FileKind, ProviderKind, content_hash};

    fn entry(content: &str) -> FileEntry {
        FileEntry {
            hash: content_hash(content.as_bytes()),
            size: content.len() as u64,
            executable: false,
            kind: FileKind::File,
            lines: Some(content.lines().count() as u64),
        }
    }

    fn state(files: &[(&str, &str)]) -> RepositoryState {
        let mut state = RepositoryState::empty(ProviderKind::Git, "fixture");
        for (path, text) in files {
            state.filesystem.files.insert((*path).to_owned(), entry(text));
            state.captured_text.insert((*path).to_owned(), (*text).to_owned());
        }
        state
    }

    #[test]
    fn classifies_tests_benches_entry_points_and_build_scripts() {
        let behaviour = behaviour_of(&state(&[
            ("src/main.rs", "fn main() {}\n"),
            ("crates/app/src/lib.rs", "pub fn a() {}\n"),
            ("crates/app/src/bin/tool.rs", "fn main() {}\n"),
            ("tests/cli.rs", "#[test]\nfn t() {}\n"),
            ("benches/wide.rs", "fn bench() {}\n"),
            ("crates/app/build.rs", "fn main() {}\n"),
        ]));
        assert!(behaviour.entry_points.contains(&"src/main.rs".to_owned()));
        assert!(
            behaviour
                .entry_points
                .contains(&"crates/app/src/bin/tool.rs".to_owned())
        );
        assert!(behaviour.test_files.contains(&"tests/cli.rs".to_owned()));
        assert!(behaviour.bench_files.contains(&"benches/wide.rs".to_owned()));
        assert!(
            behaviour
                .build_scripts
                .contains(&"crates/app/build.rs".to_owned())
        );
    }

    #[test]
    fn counts_test_declarations_and_public_items_from_text() {
        let lib = "pub fn one() {}\npub struct Two;\nfn hidden() {}\n#[cfg(test)]\nmod tests {\n    #[test]\n    fn t() {}\n    #[test]\n    fn u() {}\n}\n";
        let behaviour = behaviour_of(&state(&[("crates/app/src/lib.rs", lib)]));
        assert_eq!(behaviour.test_declarations, 2);
        assert_eq!(
            behaviour.public_items_per_file.get("crates/app/src/lib.rs"),
            Some(&2)
        );
        // The lib carries a #[cfg(test)] module, so it registers as test-bearing.
        assert!(
            behaviour
                .test_files
                .contains(&"crates/app/src/lib.rs".to_owned())
        );
    }

    #[test]
    fn empty_state_yields_empty_facet() {
        let behaviour = behaviour_of(&state(&[("docs/readme.md", "text\n")]));
        assert!(behaviour.test_files.is_empty(), "no tests expected");
        assert!(
            behaviour.public_items_per_file.is_empty(),
            "no public items expected"
        );
        assert_eq!(behaviour.test_declarations, 0);
    }
}

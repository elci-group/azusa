// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Repository state model.
//!
//! A [`RepositoryState`] is a provider-independent snapshot of one observable
//! state of a repository: its identity, ancestry, filesystem, structure,
//! dependencies, configuration and behaviour. States are compared by the
//! azusa-core engine; they never contain UI concepts.

use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};

use crate::ProviderKind;
use crate::hash::ContentHash;

/// The kind of a tracked file entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FileKind {
    /// A regular file blob.
    File,
    /// A symbolic link.
    Symlink,
    /// A submodule / nested repository pointer.
    Submodule,
}

impl FileKind {
    /// Canonical name used in digests and JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::File => "file",
            Self::Symlink => "symlink",
            Self::Submodule => "submodule",
        }
    }

    /// Compact mode string in the style of Git tree modes (digest input).
    #[must_use]
    pub fn mode_string(self, executable: bool) -> &'static str {
        match self {
            Self::File if executable => "100755",
            Self::File => "100644",
            Self::Symlink => "120000",
            Self::Submodule => "160000",
        }
    }
}

/// One tracked path inside a [`FilesystemState`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// BLAKE3 content hash (hex).
    pub hash: ContentHash,
    /// Size in bytes (0 for symlinks/submodules).
    pub size: u64,
    /// Whether the file is executable.
    pub executable: bool,
    /// Kind of entry.
    pub kind: FileKind,
    /// Line count for text files, when known (used for cheap line deltas).
    pub lines: Option<u64>,
}

/// The filesystem facet of a state: an ordered map of path → [`FileEntry`].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct FilesystemState {
    /// All tracked paths, forward-slash separated, repository-relative.
    pub files: BTreeMap<String, FileEntry>,
    /// Total content size in bytes.
    pub total_bytes: u64,
    /// Digest over the canonical listing; equal digests mean equal trees.
    pub digest: ContentHash,
}

impl FilesystemState {
    /// Number of file entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Whether the state tracks no files at all.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

/// Version-control identity facets (only present for Git states).
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct VcsIdentity {
    /// HEAD commit id, when a commit exists.
    pub head_oid: Option<String>,
    /// Current branch short name.
    pub branch: Option<String>,
    /// Whether HEAD is detached.
    pub detached: bool,
    /// Upstream tracking ref (e.g. `origin/main`), if configured.
    pub upstream: Option<String>,
    /// Remote URLs, if any.
    pub remote_urls: Vec<String>,
    /// Whether the repository is shallow-cloned.
    pub shallow: bool,
}

/// Summary of uncommitted divergence in a worktree state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct UncommittedSummary {
    /// Files staged but not committed.
    pub staged_files: u64,
    /// Files modified but not staged.
    pub unstaged_files: u64,
    /// Untracked files.
    pub untracked_files: u64,
}

impl UncommittedSummary {
    /// Total number of uncommitted paths.
    #[must_use]
    pub fn total(&self) -> u64 {
        self.staged_files + self.unstaged_files + self.untracked_files
    }

    /// Whether anything at all is uncommitted.
    #[must_use]
    pub fn is_clean(&self) -> bool {
        self.total() == 0
    }
}

/// How two states relate through history.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Relation {
    /// States share a common ancestor.
    Related,
    /// States share no ancestry or content overlap.
    Unrelated,
    /// Relationship cannot be established with the available evidence.
    #[default]
    Unknown,
}

/// The ancestry facet of a state, relative to its upstream when one exists.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ancestry {
    /// Relation between this state and its upstream/base.
    pub relation: Relation,
    /// Merge-base commit id with the upstream, if computable.
    pub merge_base: Option<String>,
    /// Commits ahead of upstream, if computable.
    pub ahead: Option<u64>,
    /// Commits behind upstream, if computable.
    pub behind: Option<u64>,
    /// Uncommitted summary (worktree states only).
    pub uncommitted: Option<UncommittedSummary>,
}

/// Identity of one captured state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateIdentity {
    /// Provider that captured the state.
    pub provider: ProviderKind,
    /// Selector that produced the state (e.g. `worktree`, `origin/main`).
    pub label: String,
    /// Repository root path, when the state lives on disk.
    pub root: Option<String>,
    /// Fileset digest — the primary identity of state content.
    pub digest: ContentHash,
    /// Number of tracked paths.
    pub file_count: u64,
    /// Total content bytes.
    pub total_bytes: u64,
    /// VCS identity, for Git states.
    pub vcs: Option<VcsIdentity>,
}

/// A discovered module inside a repository (crate, package, binary target).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleInfo {
    /// Module name (crate/package name).
    pub name: String,
    /// Repository-relative path.
    pub path: String,
    /// What kind of module this is.
    pub kind: ModuleKind,
}

/// Classification of discovered modules.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModuleKind {
    /// A Rust library crate.
    RustLibrary,
    /// A Rust binary crate.
    RustBinary,
    /// A Cargo workspace root.
    RustWorkspace,
    /// A module not recognised by a specific ecosystem.
    Other,
}

impl ModuleKind {
    /// Canonical name used in JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::RustLibrary => "rust-library",
            Self::RustBinary => "rust-binary",
            Self::RustWorkspace => "rust-workspace",
            Self::Other => "other",
        }
    }
}

/// The structure facet: module inventory and directory topology.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StructureState {
    /// Discovered modules.
    pub modules: Vec<ModuleInfo>,
    /// All directories that contain at least one file.
    pub directories: BTreeSet<String>,
}

/// Classification of dependency entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyKind {
    /// A normal (runtime) dependency.
    Normal,
    /// A development dependency.
    Dev,
    /// A build dependency.
    Build,
}

impl DependencyKind {
    /// Canonical name used in JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Normal => "normal",
            Self::Dev => "dev",
            Self::Build => "build",
        }
    }
}

/// One dependency declared in a manifest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyEntry {
    /// Manifest file declaring the dependency.
    pub manifest: String,
    /// Dependency name.
    pub name: String,
    /// Requirement as written (e.g. `1`, `^0.20`, git URL).
    pub requirement: String,
    /// Dependency kind.
    pub kind: DependencyKind,
    /// Resolved version, when a lockfile pins one.
    pub resolved: Option<String>,
}

/// A lockfile observed in the state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockfileInfo {
    /// Lockfile path.
    pub path: String,
    /// Content digest.
    pub digest: ContentHash,
    /// Number of packages recorded.
    pub package_count: u64,
}

/// The dependency facet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyState {
    /// Declared dependencies across all manifests.
    pub entries: Vec<DependencyEntry>,
    /// Observed lockfiles.
    pub lockfiles: Vec<LockfileInfo>,
}

/// Classification of configuration files of interest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConfigurationKind {
    /// `Cargo.toml` (workspace or package manifest).
    CargoManifest,
    /// CI pipeline definition.
    CiWorkflow,
    /// Container build file.
    Dockerfile,
    /// Container composition file.
    Compose,
    /// Kubernetes-style manifest.
    Kubernetes,
    /// systemd service unit.
    SystemdUnit,
    /// Formatter configuration.
    Formatter,
    /// Linter configuration.
    Linter,
    /// Environment/feature template.
    Environment,
    /// Recognised as configuration but not classified further.
    Other,
}

impl ConfigurationKind {
    /// Canonical name used in JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CargoManifest => "cargo-manifest",
            Self::CiWorkflow => "ci-workflow",
            Self::Dockerfile => "dockerfile",
            Self::Compose => "compose",
            Self::Kubernetes => "kubernetes",
            Self::SystemdUnit => "systemd-unit",
            Self::Formatter => "formatter",
            Self::Linter => "linter",
            Self::Environment => "environment",
            Self::Other => "other",
        }
    }

    /// Whether this kind describes deployment- or service-facing
    /// infrastructure (the L9 operations surface).
    #[must_use]
    pub fn is_operational(self) -> bool {
        matches!(
            self,
            Self::CiWorkflow
                | Self::Dockerfile
                | Self::Kubernetes
                | Self::SystemdUnit
                | Self::Compose
        )
    }
}

/// One configuration file observed in the state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationFile {
    /// Repository-relative path.
    pub path: String,
    /// Classification.
    pub kind: ConfigurationKind,
    /// Content digest.
    pub digest: ContentHash,
}

/// The configuration facet.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConfigurationState {
    /// Configuration files of interest.
    pub files: Vec<ConfigurationFile>,
}

/// One operational unit: a deployment- or service-facing artefact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalUnit {
    /// Repository-relative path.
    pub path: String,
    /// Operational classification (CI workflow, container, service unit…).
    pub kind: ConfigurationKind,
    /// Content digest.
    pub digest: ContentHash,
}

/// The operations facet: the deployment/service surface of the state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct OperationalState {
    /// Operational units of interest.
    pub units: Vec<OperationalUnit>,
}

/// The behaviour facet: tests, entry points, build scripts, public surface.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BehaviourState {
    /// Files recognised as tests.
    pub test_files: Vec<String>,
    /// Individual test function declarations.
    pub test_declarations: u64,
    /// Files recognised as benchmarks.
    pub bench_files: Vec<String>,
    /// Executable entry points.
    pub entry_points: Vec<String>,
    /// Build scripts (`build.rs` and equivalents).
    pub build_scripts: Vec<String>,
    /// Public API items per Rust source file (heuristic count).
    pub public_items_per_file: BTreeMap<String, u64>,
}

/// A complete repository state: everything azusa knows about one observable
/// state of a repository, provider-independent.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RepositoryState {
    /// Identity of the state.
    pub identity: StateIdentity,
    /// Ancestry relative to upstream/base.
    pub ancestry: Ancestry,
    /// Filesystem facet.
    pub filesystem: FilesystemState,
    /// Structure facet.
    pub structure: StructureState,
    /// Dependency facet.
    pub dependencies: DependencyState,
    /// Configuration facet.
    pub configuration: ConfigurationState,
    /// Operations facet: the deployment/service surface.
    pub operations: OperationalState,
    /// Behaviour facet, when behaviour analysis ran.
    pub behaviour: Option<BehaviourState>,
    /// Raw UTF-8 text of analysis-interest files (manifests, lockfiles,
    /// configuration), captured by the provider while hashing. Small by
    /// construction (size-capped); lets structural analysis stay a pure
    /// function of the state.
    #[serde(default)]
    pub captured_text: BTreeMap<String, String>,
}

/// Maximum retained text size per analysis-interest file (128 KiB). Files
/// larger than this are compared by hash only.
pub const CAPTURED_TEXT_MAX_BYTES: usize = 128 * 1024;

/// Whether a repository path is of structural-analysis interest, so
/// providers retain its text in [`RepositoryState::captured_text`].
///
/// Interest = Cargo manifests and lockfiles, toolchain and formatter/linter
/// configuration, CI workflows, container and service definitions (feeding
/// layers L3–L5), plus the behavioural surface: build scripts, library and
/// binary roots, and integration tests/benches (feeding layers L6–L7).
#[must_use]
pub fn is_analysis_file(path: &str) -> bool {
    let name = path.rsplit('/').next().unwrap_or(path);
    let matches_name = |candidates: &[&str]| candidates.contains(&name);
    if matches_name(&[
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain",
        "rust-toolchain.toml",
        "rustfmt.toml",
        ".rustfmt.toml",
        "clippy.toml",
        "deny.toml",
        ".editorconfig",
        ".env.example",
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ]) {
        return true;
    }
    if name.starts_with("Dockerfile") {
        return true;
    }
    if path.starts_with(".github/workflows/")
        && std::path::Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("yml") || ext.eq_ignore_ascii_case("yaml"))
    {
        return true;
    }
    if name.ends_with(".service") {
        return true;
    }
    is_behaviour_file(path)
}

/// Whether a path is part of the behavioural surface (layers L6–L7): build
/// scripts, library/binary roots, and Rust integration tests or benches.
#[must_use]
pub fn is_behaviour_file(path: &str) -> bool {
    let is_rust = std::path::Path::new(path)
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("rs"));
    let name = path.rsplit('/').next().unwrap_or(path);
    if name == "build.rs" {
        return true;
    }
    let segments: Vec<&str> = path.split('/').collect();
    segments.windows(2).any(|window| match window {
        ["src", "lib.rs" | "main.rs"] => true,
        ["src", "bin"] | ["tests" | "benches", _] => is_rust,
        _ => false,
    })
}

impl RepositoryState {
    /// An empty state under the given label (used for missing states).
    #[must_use]
    pub fn empty(provider: ProviderKind, label: impl Into<String>) -> Self {
        Self {
            identity: StateIdentity {
                provider,
                label: label.into(),
                root: None,
                digest: String::new(),
                file_count: 0,
                total_bytes: 0,
                vcs: None,
            },
            ancestry: Ancestry::default(),
            filesystem: FilesystemState::default(),
            structure: StructureState::default(),
            dependencies: DependencyState::default(),
            configuration: ConfigurationState::default(),
            operations: OperationalState::default(),
            behaviour: None,
            captured_text: BTreeMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_state_is_consistent() {
        let state = RepositoryState::empty(ProviderKind::Filesystem, "nothing");
        assert_eq!(state.identity.file_count, 0);
        assert!(state.filesystem.is_empty(), "filesystem should be empty");
        assert!(state.identity.digest.is_empty(), "digest should be empty");
    }

    #[test]
    fn mode_strings_match_git_conventions() {
        assert_eq!(FileKind::File.mode_string(false), "100644");
        assert_eq!(FileKind::File.mode_string(true), "100755");
        assert_eq!(FileKind::Symlink.mode_string(false), "120000");
    }

    #[test]
    fn analysis_interest_predicate() {
        assert!(is_analysis_file("Cargo.toml"));
        assert!(is_analysis_file("crates/foo/Cargo.toml"));
        assert!(is_analysis_file("Cargo.lock"));
        assert!(is_analysis_file(".github/workflows/ci.yml"));
        assert!(is_analysis_file("deploy/Dockerfile"));
        assert!(is_analysis_file("deploy/api.service"));
        assert!(!is_analysis_file("docs/ci.md"));
    }

    #[test]
    fn behaviour_interest_predicate() {
        assert!(is_analysis_file("src/lib.rs"));
        assert!(is_analysis_file("src/main.rs"));
        assert!(is_analysis_file("crates/app/src/lib.rs"));
        assert!(is_analysis_file("crates/app/src/bin/run.rs"));
        assert!(is_analysis_file("tests/cli.rs"));
        assert!(is_analysis_file("crates/app/tests/integration.rs"));
        assert!(is_analysis_file("benches/wide.rs"));
        assert!(is_analysis_file("build.rs"));
        assert!(is_analysis_file("crates/app/build.rs"));
        assert!(!is_analysis_file("tests/fixtures/data.json"));
        assert!(!is_analysis_file("docs/notes.rs"));
        assert!(!is_analysis_file("src/helpers/util.rs"));
        assert!(is_behaviour_file("src/lib.rs"));
        assert!(!is_behaviour_file("Cargo.toml"));
    }

    #[test]
    fn uncommitted_summary_totals() {
        let summary = UncommittedSummary {
            staged_files: 2,
            unstaged_files: 3,
            untracked_files: 4,
        };
        assert_eq!(summary.total(), 9);
        assert!(!summary.is_clean());
    }
}

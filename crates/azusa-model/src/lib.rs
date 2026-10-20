// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Azusa domain model.
//!
//! Pure data structures describing repository states, drift observations,
//! scores, risks, recommendations and reports. This crate intentionally has no
//! I/O and no provider logic: every azusa crate speaks these types, and no UI
//! concept leaks into them.
//!
//! States are modelled, not Git trees: [`RepositoryState`] aggregates
//! identity, ancestry, filesystem, structure, dependencies, configuration and
//! behaviour facets, whichever state provider produced them.

pub mod drift;
pub mod hash;
pub mod plan;
pub mod state;
pub mod text;

pub use drift::{
    AnalysisMode, ChangeCluster, ClusterKind, DRIFT_REPORT_SCHEMA_VERSION, DriftDirection,
    DriftObservation, DriftReport, DriftRisk, DriftScore, DriftSummary, Evidence, EvidenceKind,
    EvidenceStrength, IntentAssessment, IntentClass, IntentSignal, Layer, Priority, Provenance,
    Recommendation, RepositoryInfo, RiskLevel, Severity, Side, StateRef,
};
pub use hash::{ContentHash, content_hash, content_hash_reader, fileset_digest};
pub use plan::{PLAN_SCHEMA_VERSION, PlanStep, PlanStepKind, ReconciliationPlan, SafetyGate};
pub use state::{
    Ancestry, BehaviourState, CAPTURED_TEXT_MAX_BYTES, ConfigurationFile, ConfigurationKind,
    ConfigurationState, DependencyEntry, DependencyKind, DependencyState, FileEntry, FileKind,
    FilesystemState, LockfileInfo, ModuleInfo, ModuleKind, OperationalState, OperationalUnit,
    PublicSymbol, Relation, RepositoryState, StateIdentity, StructureState, UncommittedSummary,
    VcsIdentity, is_analysis_file,
};
pub use text::{count_lines, looks_like_text};

/// The state provider that produced a [`RepositoryState`].
///
/// Git is one provider among several: filesystem snapshots, archives and
/// future providers share the same state model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum ProviderKind {
    /// A state captured from a Git repository (worktree, index, commit, remote branch).
    Git,
    /// A state captured from a plain filesystem tree.
    Filesystem,
}

impl ProviderKind {
    /// Canonical short name used in reports and JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Git => "git",
            Self::Filesystem => "fs",
        }
    }
}

/// Version of the azusa toolchain producing states and reports.
///
/// Embedded from the kaptaind-managed root `VERSION` file — the single
/// version authority. Theosis reconciles an installed azusa against the
/// remote `VERSION` and verifies via the binary's self-reported version, so
/// the binary must speak the same number kaptaind publishes.
#[must_use]
pub fn tool_version() -> &'static str {
    include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../VERSION")).trim()
}

/// Smoke test: the crate builds and its types are nameable.
#[test]
fn model_smoke() {
    assert_eq!(ProviderKind::Git.as_str(), "git");
    assert_eq!(ProviderKind::Filesystem.as_str(), "fs");
    // The embedded version is the kaptaind VERSION: strict numeric A.B.C,
    // no trailing whitespace.
    let version = tool_version();
    let mut parts = version.split('.');
    assert_eq!(parts.clone().count(), 3, "version must be A.B.C");
    assert!(
        parts.all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit())),
        "version must be numeric: {version}"
    );
    assert!(!version.ends_with('\n'));
}

/// One version authority: the embedded root `VERSION` and the workspace
/// Cargo package version must never disagree. When kaptaind bumps VERSION,
/// run `scripts/sync-version.sh` (CI enforces the equality) so crate
/// metadata, SBOMs and external scanners see the same release number.
#[test]
fn version_authority_is_unified() {
    assert_eq!(
        tool_version(),
        env!("CARGO_PKG_VERSION"),
        "root VERSION and workspace Cargo version diverged; run scripts/sync-version.sh"
    );
}

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
    IntentAssessment, IntentClass, IntentSignal, Layer, Priority, Provenance, Recommendation,
    RepositoryInfo, RiskLevel, Severity, Side, StateRef,
};
pub use hash::{ContentHash, content_hash, content_hash_reader, fileset_digest};
pub use plan::{
    PLAN_SCHEMA_VERSION, PlanStep, PlanStepKind, ReconciliationPlan, SafetyGate,
};
pub use state::{
    Ancestry, BehaviourState, CAPTURED_TEXT_MAX_BYTES, ConfigurationFile, ConfigurationKind,
    ConfigurationState, DependencyEntry, DependencyKind, DependencyState, FileEntry, FileKind,
    FilesystemState, LockfileInfo, ModuleInfo, ModuleKind, OperationalState, OperationalUnit,
    Relation, RepositoryState, StateIdentity, StructureState, UncommittedSummary, VcsIdentity,
    is_analysis_file,
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
pub const TOOL_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Smoke test: the crate builds and its types are nameable.
#[test]
fn model_smoke() {
    assert_eq!(ProviderKind::Git.as_str(), "git");
    assert_eq!(ProviderKind::Filesystem.as_str(), "fs");
    assert_eq!(TOOL_VERSION, "0.1.0");
}

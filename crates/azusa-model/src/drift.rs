// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Drift model.
//!
//! Comparing two [`crate::RepositoryState`]s produces a [`DriftReport`]:
//! a direction, a score, evidence-first observations, risks, an intent
//! estimate and recommendations. Every material score must be explainable by
//! the observations that produced it — the score never replaces evidence.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

/// Canonical schema version of the drift report artefact.
pub const DRIFT_REPORT_SCHEMA_VERSION: u32 = 2;

/// The comparison layers, L0–L9. Layer names double as drift categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Layer {
    /// L0: is this the same repository at all?
    Identity,
    /// L1: ancestry, ahead/behind, committed vs uncommitted drift.
    Ancestry,
    /// L2: files added, deleted, modified, renamed.
    Filesystem,
    /// L3: modules, crates, directory topology.
    Structure,
    /// L4: manifest and lockfile divergence.
    Dependency,
    /// L5: project configuration files.
    Configuration,
    /// L6: public API surface.
    Interface,
    /// L7: tests, build scripts, entry points.
    Behaviour,
    /// L8: security/behaviour-critical regions (heuristic, evidence-first).
    Semantic,
    /// L9: deployment and service configuration.
    Operational,
}

impl Layer {
    /// Canonical short name used in reports and JSON output.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Ancestry => "ancestry",
            Self::Filesystem => "filesystem",
            Self::Structure => "structure",
            Self::Dependency => "dependency",
            Self::Configuration => "configuration",
            Self::Interface => "interface",
            Self::Behaviour => "behaviour",
            Self::Semantic => "semantic",
            Self::Operational => "operational",
        }
    }

    /// All layers in comparison order (L0 first).
    #[must_use]
    pub fn all() -> [Layer; 10] {
        [
            Self::Identity,
            Self::Ancestry,
            Self::Filesystem,
            Self::Structure,
            Self::Dependency,
            Self::Configuration,
            Self::Interface,
            Self::Behaviour,
            Self::Semantic,
            Self::Operational,
        ]
    }
}

/// Severity of a single observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Severity {
    /// Noteworthy but harmless.
    Info,
    /// Minor drift.
    Low,
    /// Meaningful drift.
    Medium,
    /// Significant drift.
    High,
    /// Drift that likely breaks correctness, security or operations.
    Critical,
}

impl Severity {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// Which side of the comparison an observation belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Side {
    /// Only present in state A (the local side by convention).
    A,
    /// Only present in state B (the remote side by convention).
    B,
    /// Involves both states.
    Both,
}

impl Side {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::A => "a",
            Self::B => "b",
            Self::Both => "both",
        }
    }
}

/// Direction of drift between two states.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DriftDirection {
    /// States are materially identical.
    InSync,
    /// State A is strictly ahead of state B.
    AToB,
    /// State B is strictly ahead of state A.
    BToA,
    /// Both states carry changes the other lacks.
    Bidirectional,
    /// States share neither ancestry nor meaningful content overlap.
    Unrelated,
    /// Direction cannot be determined from available evidence.
    Unknown,
}

impl DriftDirection {
    /// Canonical name used in reports.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InSync => "in-sync",
            Self::AToB => "a-to-b",
            Self::BToA => "b-to-a",
            Self::Bidirectional => "bidirectional",
            Self::Unrelated => "unrelated",
            Self::Unknown => "unknown",
        }
    }
}

/// How a piece of evidence was established.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceKind {
    /// Commit ancestry facts.
    Commit,
    /// Branch or ref facts.
    Branch,
    /// File presence or content hash.
    File,
    /// Manifest parsing.
    Manifest,
    /// Lockfile parsing.
    Lockfile,
    /// Configuration classification.
    Config,
    /// Test inventory.
    Test,
    /// Detected rename.
    Rename,
    /// Path pattern classification.
    PathPattern,
    /// Generated-file or policy marker.
    Marker,
}

impl EvidenceKind {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Commit => "commit",
            Self::Branch => "branch",
            Self::File => "file",
            Self::Manifest => "manifest",
            Self::Lockfile => "lockfile",
            Self::Config => "config",
            Self::Test => "test",
            Self::Rename => "rename",
            Self::PathPattern => "path-pattern",
            Self::Marker => "marker",
        }
    }
}

/// One concrete, checkable piece of evidence behind an observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    /// How this evidence was established.
    pub kind: EvidenceKind,
    /// Human-readable statement of the evidence.
    pub description: String,
    /// Machine-checkable references (paths, commit ids, names).
    pub references: Vec<String>,
}

/// A single drift observation: one explained fact about the divergence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftObservation {
    /// Stable observation id (e.g. `DRIFT-0007`), assigned deterministically.
    pub id: String,
    /// Layer / category of the observation.
    pub category: Layer,
    /// Severity of the observation.
    pub severity: Severity,
    /// Confidence in the observation, 0.0–1.0.
    pub confidence: f32,
    /// Short title.
    pub title: String,
    /// Longer explanation of what was observed.
    pub detail: String,
    /// Which side the observation belongs to.
    pub side: Side,
    /// Concrete evidence backing the observation.
    pub evidence: Vec<Evidence>,
    /// Epistemic strength of the observation: how the evidence was
    /// obtained, independent of its severity. Autonomous consumers gate on
    /// this — act on `fact`/`derived`, review `heuristic`.
    #[serde(default)]
    pub strength: EvidenceStrength,
}

/// How an observation's evidence was obtained.
///
/// Deliberately orthogonal to severity and confidence: `severity` says how
/// much the change matters, `strength` says how much the *claim* can be
/// trusted.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceStrength {
    /// Verified directly from content: digests, merge-base, exact identity.
    #[default]
    Fact,
    /// Derived deterministically from parsed declarations or structured
    /// inventories (symbol tables, manifests).
    Derived,
    /// A heuristic inference (lexical proxies, path similarity).
    Heuristic,
    /// Speculative: pattern-based judgement with no direct evidence
    /// (reserved for semantic layers).
    Speculative,
}

impl EvidenceStrength {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Fact => "fact",
            Self::Derived => "derived",
            Self::Heuristic => "heuristic",
            Self::Speculative => "speculative",
        }
    }
}

/// Composite drift index: per-layer scores plus the weighted overall.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DriftScore {
    /// Weighted overall score, 0.0–10.0, rounded to two decimals.
    pub overall: f32,
    /// Per-layer scores, 0.0–10.0 each.
    pub layers: BTreeMap<Layer, f32>,
}

/// Risk classification of the whole comparison.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RiskLevel {
    /// Reconciliation is routine.
    Low,
    /// Reconciliation needs attention.
    Medium,
    /// Reconciliation is risky.
    High,
    /// Reconciliation is likely unsafe or needs human arbitration.
    Critical,
}

impl RiskLevel {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }
}

/// The risk assessment attached to a report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftRisk {
    /// Classified level.
    pub level: RiskLevel,
    /// Why this level was chosen.
    pub rationale: String,
    /// Contributing factors, each traceable to observations.
    pub factors: Vec<String>,
}

/// Priority of a recommendation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Priority {
    /// Do first.
    High,
    /// Do when convenient.
    Medium,
    /// Optional.
    Low,
}

impl Priority {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// One recommended action derived from observations.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Recommendation {
    /// The action to take.
    pub action: String,
    /// Why the action is recommended.
    pub rationale: String,
    /// Priority ordering.
    pub priority: Priority,
}

/// Intent classification for a set of changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum IntentClass {
    /// Clearly deliberate change.
    Intentional,
    /// Evidence leans deliberate.
    ProbablyIntentional,
    /// Evidence leans incidental.
    ProbablyAccidental,
    /// Clearly incidental change.
    Accidental,
    /// Not enough evidence to say.
    Indeterminate,
}

impl IntentClass {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Intentional => "intentional",
            Self::ProbablyIntentional => "probably-intentional",
            Self::ProbablyAccidental => "probably-accidental",
            Self::Accidental => "accidental",
            Self::Indeterminate => "indeterminate",
        }
    }
}

/// One signal feeding an intent estimate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentSignal {
    /// What the signal is.
    pub description: String,
    /// Signed weight: positive supports intent, negative supports accident.
    pub weight: f32,
}

/// An intent estimate over a set of changes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IntentAssessment {
    /// Classification.
    pub classification: IntentClass,
    /// Confidence in the classification, 0.0–1.0.
    pub confidence: f32,
    /// Signals considered.
    pub signals: Vec<IntentSignal>,
}

/// Classification of a logical change cluster.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClusterKind {
    /// Dependency lockfiles.
    Lockfile,
    /// Package manifests.
    Manifest,
    /// Tests and test fixtures.
    Tests,
    /// Documentation.
    Documentation,
    /// Project configuration (formatting, linting).
    Configuration,
    /// Deployment, CI and service configuration.
    Operations,
    /// A source module or crate.
    Module,
    /// Anything else.
    Other,
}

impl ClusterKind {
    /// Canonical kebab-case name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Lockfile => "lockfile",
            Self::Manifest => "manifest",
            Self::Tests => "tests",
            Self::Documentation => "documentation",
            Self::Configuration => "configuration",
            Self::Operations => "operations",
            Self::Module => "module",
            Self::Other => "other",
        }
    }
}

/// One logical cluster of changed paths: "147 changed lines" becomes
/// "authentication, migrations, CLI, tests, formatting".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChangeCluster {
    /// Human label (e.g. `crates/azusa-core`, `tests`, `ci`).
    pub label: String,
    /// Cluster classification.
    pub kind: ClusterKind,
    /// Changed paths in this cluster (capped listing).
    pub paths: Vec<String>,
    /// Total changed paths in this cluster.
    pub files: u64,
    /// Added paths.
    pub added: u64,
    /// Deleted paths.
    pub deleted: u64,
    /// Modified paths.
    pub modified: u64,
    /// Renamed paths.
    pub renamed: u64,
}

/// Headline numbers of a comparison, for quick human and machine scanning.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DriftSummary {
    /// Files added (present only in state B, absent in A).
    pub files_added: u64,
    /// Files deleted (present only in state A, absent in B).
    pub files_deleted: u64,
    /// Files whose content changed.
    pub files_modified: u64,
    /// Files renamed/moved between states.
    pub files_renamed: u64,
    /// Commits state A is ahead of state B by (Git states).
    pub commits_ahead: Option<u64>,
    /// Commits state A is behind state B by (Git states).
    pub commits_behind: Option<u64>,
    /// Whether state A carries uncommitted work.
    pub uncommitted_a: bool,
    /// Whether state B carries uncommitted work.
    pub uncommitted_b: bool,
}

/// How the deterministic analysis was produced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AnalysisMode {
    /// Pure deterministic rules over captured states.
    Deterministic,
    /// Deterministic core augmented by (recorded, confidence-scored) model
    /// interpretation.
    Augmented,
}

impl AnalysisMode {
    /// Canonical lowercase name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Deterministic => "deterministic",
            Self::Augmented => "augmented",
        }
    }
}

/// Provenance of a drift report.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// azusa version that produced the report.
    pub tool_version: String,
    /// Analysis mode — deterministic results never mix silent probabilistic
    /// interpretation.
    pub analysis_mode: AnalysisMode,
    /// Wall-clock capture time (display only; never part of scoring).
    pub captured_at: Option<String>,
    /// Digest of the effective configuration, for reproducibility.
    pub config_digest: Option<String>,
}

/// Which state a report compared, in compact form.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRef {
    /// The selector the user (or defaults) requested.
    pub spec: String,
    /// Provider that resolved the selector.
    pub provider: crate::ProviderKind,
    /// Human label (e.g. `worktree`, `origin/main`, a path).
    pub label: String,
    /// Fileset digest of the resolved state.
    pub digest: String,
}

/// Identity block of the repository under comparison.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RepositoryInfo {
    /// Repository name (directory or remote-derived).
    pub name: String,
    /// Root path, when known.
    pub root: Option<String>,
}

/// The complete drift report — azusa's primary artefact.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DriftReport {
    /// Schema version of this artefact.
    pub schema_version: u32,
    /// Repository identity.
    pub repository: RepositoryInfo,
    /// State A (conventionally the local side).
    pub state_a: StateRef,
    /// State B (conventionally the remote side).
    pub state_b: StateRef,
    /// Direction of drift.
    pub direction: DriftDirection,
    /// Confidence in the direction determination, 0.0–1.0.
    pub direction_confidence: f32,
    /// Composite drift index.
    pub score: DriftScore,
    /// Risk assessment.
    pub risk: DriftRisk,
    /// Headline numbers.
    pub summary: DriftSummary,
    /// Evidence-first observations.
    pub observations: Vec<DriftObservation>,
    /// Intent estimate, when intent analysis ran.
    pub intent: Option<IntentAssessment>,
    /// Recommended actions.
    pub recommendations: Vec<Recommendation>,
    /// Logical change clusters (phase 2).
    #[serde(default)]
    pub clusters: Vec<ChangeCluster>,
    /// Report provenance.
    pub provenance: Provenance,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_order_is_l0_to_l9() {
        let all = Layer::all();
        assert_eq!(all[0], Layer::Identity);
        assert_eq!(all[9], Layer::Operational);
        assert!(all.windows(2).all(|w| w[0] < w[1]));
    }

    #[test]
    fn severity_orders_monotonically() {
        assert!(Severity::Info < Severity::Low);
        assert!(Severity::Low < Severity::Medium);
        assert!(Severity::Medium < Severity::High);
        assert!(Severity::High < Severity::Critical);
    }

    #[test]
    fn direction_names_are_stable() {
        assert_eq!(DriftDirection::Bidirectional.as_str(), "bidirectional");
        assert_eq!(DriftDirection::InSync.as_str(), "in-sync");
    }
}

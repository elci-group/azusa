// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Reconciliation plans.
//!
//! A plan is an ordered, evidence-linked sequence of steps that would
//! reconcile two drifted states, plus the safety gates that must hold
//! before those steps run. Azusa plans; Kaptaind executes — a plan never
//! mutates anything by itself, and every step that rewrites history or
//! publishes work carries `requires_approval`.

use serde::{Deserialize, Serialize};

use crate::drift::RiskLevel;

/// Schema version of the plan artefact.
pub const PLAN_SCHEMA_VERSION: u32 = 1;

/// What a plan step does, at execution granularity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PlanStepKind {
    /// Nothing to do; states agree.
    Noop,
    /// Commit or stash local work.
    Commit,
    /// Fetch the other side.
    Fetch,
    /// Fast-forward to the other side.
    FastForward,
    /// Rebase local work onto the other side (rewrites local history).
    Rebase,
    /// Merge the two sides (creates a merge commit).
    Merge,
    /// Push local work to the remote.
    Push,
    /// Review a change cluster by hand.
    Review,
    /// Run the test suite.
    RunTests,
    /// Human inspection required; no automated action exists.
    Manual,
}

impl PlanStepKind {
    /// Canonical kebab-case name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Noop => "noop",
            Self::Commit => "commit",
            Self::Fetch => "fetch",
            Self::FastForward => "fast-forward",
            Self::Rebase => "rebase",
            Self::Merge => "merge",
            Self::Push => "push",
            Self::Review => "review",
            Self::RunTests => "run-tests",
            Self::Manual => "manual",
        }
    }
}

/// One ordered step of a reconciliation plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlanStep {
    /// Stable id (`PLAN-0001`…).
    pub id: String,
    /// What kind of action this is.
    pub kind: PlanStepKind,
    /// What to do.
    pub description: String,
    /// Why this step exists (evidence-linked).
    pub rationale: String,
    /// Risk of executing this step.
    pub risk: RiskLevel,
    /// Whether a human must approve before execution.
    pub requires_approval: bool,
    /// Who executes: `kaptaind` or `human`.
    pub executed_by: String,
}

/// A condition that must hold before (parts of) a plan may run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SafetyGate {
    /// Stable id (`GATE-0001`…).
    pub id: String,
    /// What must hold.
    pub condition: String,
    /// Why the gate exists.
    pub reason: String,
    /// Whether the plan is blocked until the gate passes.
    pub blocking: bool,
}

/// A deterministic reconciliation plan for one drift report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReconciliationPlan {
    /// Schema version of the plan artefact.
    pub schema_version: u32,
    /// Repository name.
    pub repository: String,
    /// State A selector.
    pub state_a: String,
    /// State B selector.
    pub state_b: String,
    /// Drift direction the plan reconciles.
    pub direction: String,
    /// Overall drift score the plan was derived from.
    pub drift_score: f32,
    /// Risk level of the underlying report.
    pub risk: String,
    /// Intent classification of the underlying report, when assessed.
    pub intent: Option<String>,
    /// Ordered steps.
    pub steps: Vec<PlanStep>,
    /// Safety gates that must pass before execution.
    pub gates: Vec<SafetyGate>,
    /// One-line summary.
    pub summary: String,
}

impl ReconciliationPlan {
    /// Whether every gate passes and no blocking condition remains.
    #[must_use]
    pub fn is_executable(&self) -> bool {
        self.gates.iter().all(|gate| !gate.blocking)
    }

    /// Steps that require human approval before execution.
    pub fn approval_steps(&self) -> impl Iterator<Item = &PlanStep> {
        self.steps.iter().filter(|step| step.requires_approval)
    }
}

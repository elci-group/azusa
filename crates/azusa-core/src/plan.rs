// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Reconciliation planning (phase 6).
//!
//! [`generate`] turns a finished [`DriftReport`] into an ordered
//! [`ReconciliationPlan`]: the steps that would reconcile the states, the
//! risks each step carries, and the safety gates that must pass first.
//! The plan is a pure function of the report and the policy — it never
//! re-analyses states and never executes anything. Destructive or
//! publishing steps (`rebase`, `merge`, `push`) always require human
//! approval; Kaptaind executes, humans decide.

use azusa_model::{
    DriftDirection, DriftReport, Layer, PlanStep, PlanStepKind, ReconciliationPlan, RiskLevel,
    SafetyGate,
};

/// A step under construction before id assignment.
struct StepDraft {
    kind: PlanStepKind,
    description: String,
    rationale: String,
    risk: RiskLevel,
    requires_approval: bool,
    executed_by: &'static str,
}

/// Derive a reconciliation plan from a drift report and the policy verdict.
///
/// `policy_compliant` gates the whole plan: a report that violates the
/// configured policy cannot be auto-reconciled until the violations are
/// resolved.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn generate(report: &DriftReport, policy_compliant: bool) -> ReconciliationPlan {
    let mut steps: Vec<StepDraft> = Vec::new();
    let mut gates: Vec<(String, String, bool)> = Vec::new();

    let dirty_a = report.summary.uncommitted_a;
    let deletions = report.summary.files_deleted;
    let clusters = report.clusters.len();
    let behaviour_touched = report.score.layers.contains_key(&Layer::Behaviour)
        || report.score.layers.contains_key(&Layer::Interface);

    match report.direction {
        DriftDirection::InSync => {
            steps.push(StepDraft {
                kind: PlanStepKind::Noop,
                description: "nothing to reconcile: states are content-identical".to_owned(),
                rationale: format!(
                    "{} and {} agree on every path",
                    report.state_a.spec, report.state_b.spec
                ),
                risk: RiskLevel::Low,
                requires_approval: false,
                executed_by: "kaptaind",
            });
        }
        DriftDirection::AToB => {
            if dirty_a {
                steps.push(StepDraft {
                    kind: PlanStepKind::Commit,
                    description: "commit or stash uncommitted local work".to_owned(),
                    rationale: "uncommitted paths would be lost or tangled by a push".to_owned(),
                    risk: RiskLevel::Low,
                    requires_approval: false,
                    executed_by: "kaptaind",
                });
            }
            if clusters > 1 {
                steps.push(StepDraft {
                    kind: PlanStepKind::Review,
                    description: format!("review {clusters} change cluster(s) before publishing"),
                    rationale:
                        "the changeset spans several logical areas; review each cluster's intent"
                            .to_owned(),
                    risk: RiskLevel::Low,
                    requires_approval: false,
                    executed_by: "human",
                });
            }
            if behaviour_touched {
                steps.push(StepDraft {
                    kind: PlanStepKind::RunTests,
                    description: "run the test suite before publishing".to_owned(),
                    rationale: "tests or the public surface changed in this drift".to_owned(),
                    risk: RiskLevel::Low,
                    requires_approval: false,
                    executed_by: "kaptaind",
                });
            }
            let ahead = report.summary.commits_ahead.unwrap_or(0);
            let push_description = if ahead > 0 {
                format!("push {ahead} commit(s) to {}", report.state_b.spec)
            } else {
                format!(
                    "publish the newly committed work to {}",
                    report.state_b.spec
                )
            };
            steps.push(StepDraft {
                kind: PlanStepKind::Push,
                description: push_description,
                rationale:
                    "local state is strictly ahead; a clean fast-forward on the remote is expected"
                        .to_owned(),
                risk: push_risk(report),
                requires_approval: true,
                executed_by: "kaptaind",
            });
        }
        DriftDirection::BToA => {
            steps.push(StepDraft {
                kind: PlanStepKind::Fetch,
                description: format!("fetch {}", report.state_b.spec),
                rationale: "the remote carries work the local state lacks".to_owned(),
                risk: RiskLevel::Low,
                requires_approval: false,
                executed_by: "kaptaind",
            });
            steps.push(StepDraft {
                kind: PlanStepKind::FastForward,
                description: format!(
                    "fast-forward to {} ({} commit(s), {deletions} file(s) deleted on that side)",
                    report.state_b.spec,
                    report.summary.commits_behind.unwrap_or(0)
                ),
                rationale:
                    "remote state is strictly ahead; a fast-forward keeps local history intact"
                        .to_owned(),
                risk: if deletions > 0 {
                    RiskLevel::Medium
                } else {
                    RiskLevel::Low
                },
                requires_approval: deletions > 0,
                executed_by: "kaptaind",
            });
        }
        DriftDirection::Bidirectional => {
            steps.push(StepDraft {
                kind: PlanStepKind::Fetch,
                description: format!("fetch {}", report.state_b.spec),
                rationale: "both sides evolved; start from a complete picture".to_owned(),
                risk: RiskLevel::Low,
                requires_approval: false,
                executed_by: "kaptaind",
            });
            for cluster in &report.clusters {
                steps.push(StepDraft {
                    kind: PlanStepKind::Review,
                    description: format!(
                        "review cluster `{}` ({} file(s))",
                        cluster.label, cluster.files
                    ),
                    rationale:
                        "bidirectional drift requires a human merge decision per logical area"
                            .to_owned(),
                    risk: RiskLevel::Medium,
                    requires_approval: false,
                    executed_by: "human",
                });
            }
            if behaviour_touched {
                steps.push(StepDraft {
                    kind: PlanStepKind::RunTests,
                    description: "run the test suite after integrating".to_owned(),
                    rationale: "tests or the public surface changed on at least one side"
                        .to_owned(),
                    risk: RiskLevel::Low,
                    requires_approval: false,
                    executed_by: "kaptaind",
                });
            }
            steps.push(StepDraft {
                kind: PlanStepKind::Merge,
                description: format!("merge {} into {}", report.state_b.spec, report.state_a.spec),
                rationale: "both sides carry unique work; a merge commit preserves both histories"
                    .to_owned(),
                risk: RiskLevel::High,
                requires_approval: true,
                executed_by: "kaptaind",
            });
            steps.push(StepDraft {
                kind: PlanStepKind::Push,
                description: format!("push the reconciliation to {}", report.state_b.spec),
                rationale: "the merge result must be published to close the drift".to_owned(),
                risk: RiskLevel::High,
                requires_approval: true,
                executed_by: "kaptaind",
            });
        }
        DriftDirection::Unrelated | DriftDirection::Unknown => {
            steps.push(StepDraft {
                kind: PlanStepKind::Manual,
                description: format!(
                    "inspect {} and {} by hand before any reconciliation",
                    report.state_a.spec, report.state_b.spec
                ),
                rationale: "the drift direction could not be established with confidence"
                    .to_owned(),
                risk: RiskLevel::High,
                requires_approval: true,
                executed_by: "human",
            });
        }
    }

    // Safety gates.
    if !policy_compliant {
        gates.push((
            "the configured drift policy is satisfied".to_owned(),
            "`azusa policy check` reports violations; resolve them before reconciling".to_owned(),
            true,
        ));
    }
    if report.risk.level >= RiskLevel::High || report.direction == DriftDirection::Bidirectional {
        gates.push((
            "the report risk is accepted by a human".to_owned(),
            format!(
                "risk level is {} ({})",
                report.risk.level.as_str(),
                report.risk.factors.join("; ")
            ),
            true,
        ));
    }
    if let Some(intent) = &report.intent {
        if matches!(
            intent.classification,
            azusa_model::IntentClass::Accidental | azusa_model::IntentClass::ProbablyAccidental
        ) {
            gates.push((
                "accidental drift is distinguished from intended work".to_owned(),
                format!(
                    "intent assessed {} ({:.2}); confirm the accidental side is safe to discard",
                    intent.classification.as_str(),
                    intent.confidence
                ),
                false,
            ));
        }
    }
    if report.direction == DriftDirection::Unrelated {
        gates.push((
            "both states are verified to describe the same repository".to_owned(),
            "the states share almost no content; a wrong remote or path is likely".to_owned(),
            true,
        ));
    }

    let plan_steps = assign_step_ids(steps);
    let gate_list = assign_gate_ids(gates);
    let summary = summarize(report, plan_steps.len(), gate_list.len());
    ReconciliationPlan {
        schema_version: azusa_model::PLAN_SCHEMA_VERSION,
        repository: report.repository.name.clone(),
        state_a: report.state_a.spec.clone(),
        state_b: report.state_b.spec.clone(),
        direction: report.direction.as_str().to_owned(),
        drift_score: report.score.overall,
        risk: report.risk.level.as_str().to_owned(),
        intent: report
            .intent
            .as_ref()
            .map(|intent| intent.classification.as_str().to_owned()),
        steps: plan_steps,
        gates: gate_list,
        summary,
    }
}

/// Push risk scales with how much unseen content would be published.
fn push_risk(report: &DriftReport) -> RiskLevel {
    if report.risk.level >= RiskLevel::High {
        RiskLevel::High
    } else if report.summary.files_deleted > 0 {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    }
}

fn assign_step_ids(steps: Vec<StepDraft>) -> Vec<PlanStep> {
    steps
        .into_iter()
        .enumerate()
        .map(|(index, draft)| PlanStep {
            id: format!("PLAN-{:04}", index + 1),
            kind: draft.kind,
            description: draft.description,
            rationale: draft.rationale,
            risk: draft.risk,
            requires_approval: draft.requires_approval,
            executed_by: draft.executed_by.to_owned(),
        })
        .collect()
}

fn assign_gate_ids(gates: Vec<(String, String, bool)>) -> Vec<SafetyGate> {
    gates
        .into_iter()
        .enumerate()
        .map(|(index, (condition, reason, blocking))| SafetyGate {
            id: format!("GATE-{:04}", index + 1),
            condition,
            reason,
            blocking,
        })
        .collect()
}

fn summarize(report: &DriftReport, steps: usize, gates: usize) -> String {
    format!(
        "{} step(s), {} gate(s) to reconcile {} → {} (drift {:.1}, risk {})",
        steps,
        gates,
        report.state_a.spec,
        report.state_b.spec,
        report.score.overall,
        report.risk.level.as_str()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AzusaConfig;
    use crate::engine::compare;
    use crate::resolve::StateResolver;
    use azusa_git::testutil::FixtureRepo;
    use azusa_model::{DriftDirection, PlanStepKind};

    fn report_for(fixture_root: &std::path::Path) -> DriftReport {
        let resolver = StateResolver::new(fixture_root);
        let a = resolver.resolve("worktree").unwrap();
        let b = resolver.resolve("upstream").unwrap();
        compare("worktree", &a, "upstream", &b, &AzusaConfig::default(), &[])
    }

    #[test]
    fn in_sync_yields_noop_plan_without_gates() {
        let fixture = FixtureRepo::with_remote();
        let plan = generate(&report_for(&fixture.path), true);
        assert_eq!(plan.direction, "in-sync");
        assert_eq!(plan.steps.len(), 1);
        assert_eq!(plan.steps[0].kind, PlanStepKind::Noop);
        assert!(plan.is_executable());
    }

    #[test]
    fn local_ahead_plans_commit_push_with_approval() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("feature.rs", "pub fn feature() {}\n", "feat: feature");
        let plan = generate(&report_for(&fixture.path), true);
        assert_eq!(plan.direction, "a-to-b");
        assert!(
            plan.steps
                .iter()
                .any(|step| step.kind == PlanStepKind::Push && step.requires_approval)
        );
        assert!(plan.is_executable());
    }

    #[test]
    fn uncommitted_work_inserts_commit_step() {
        let fixture = FixtureRepo::with_remote();
        fixture.write("dirty.txt", "uncommitted\n");
        let plan = generate(&report_for(&fixture.path), true);
        assert_eq!(plan.direction, "a-to-b");
        assert!(
            plan.steps
                .iter()
                .any(|step| step.kind == PlanStepKind::Commit)
        );
    }

    #[test]
    fn bidirectional_requires_merge_and_push_approval() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("local.rs", "pub fn local() {}\n", "feat: local");
        fixture.advance_remote("remote.rs", "pub fn remote() {}\n", "feat: remote");
        let report = report_for(&fixture.path);
        assert_eq!(report.direction, DriftDirection::Bidirectional);
        let plan = generate(&report, true);
        assert!(
            plan.steps
                .iter()
                .any(|s| s.kind == PlanStepKind::Merge && s.requires_approval)
        );
        assert!(
            plan.steps
                .iter()
                .any(|s| s.kind == PlanStepKind::Push && s.requires_approval)
        );
        assert!(plan.steps.iter().any(|s| s.kind == PlanStepKind::Review));
        // High risk report → blocking acceptance gate.
        assert!(!plan.is_executable());
        assert!(plan.gates.iter().any(|gate| gate.blocking));
    }

    #[test]
    fn policy_violation_blocks_the_plan() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("feature.rs", "pub fn feature() {}\n", "feat: feature");
        let plan = generate(&report_for(&fixture.path), false);
        assert!(!plan.is_executable());
        assert!(
            plan.gates
                .iter()
                .any(|gate| gate.blocking && gate.condition.contains("policy"))
        );
    }

    #[test]
    fn plans_are_deterministic() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("one.rs", "pub fn one() {}\n", "feat: one");
        let report = report_for(&fixture.path);
        let first = generate(&report, true);
        let second = generate(&report, true);
        assert_eq!(
            serde_json::to_string(&first).unwrap(),
            serde_json::to_string(&second).unwrap()
        );
    }
}

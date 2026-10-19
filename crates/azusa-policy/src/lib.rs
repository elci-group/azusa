// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Azusa policy engine.
//!
//! Declares configurable drift limits and checks a
//! [`azusa_model::DriftReport`] against them, producing violations suitable
//! for CI gating (`azusa policy check`, exit code 3). Policies are evidence
//! checks over the report — they never re-analyse states.

use serde::{Deserialize, Serialize};

use azusa_model::{DriftObservation, DriftReport, Layer, Severity};

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-policy";

/// How production-facing configuration changes are treated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ProductionChanges {
    /// Production configuration changes are refused outright.
    Forbid,
    /// Allowed but flagged for review (default).
    #[default]
    RequireReview,
    /// Allowed silently.
    Allow,
}

/// Dependency policy knobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DependencyPolicy {
    /// Whether major-version bumps are permitted at all.
    #[serde(default = "default_false")]
    pub allow_major: bool,
    /// Whether adding new dependencies is permitted.
    #[serde(default = "default_true")]
    pub allow_new: bool,
}

fn default_false() -> bool {
    false
}

fn default_true() -> bool {
    true
}

impl Default for DependencyPolicy {
    fn default() -> Self {
        Self {
            allow_major: false,
            allow_new: true,
        }
    }
}

/// Structure policy knobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructurePolicy {
    /// Whether new crates/modules may be introduced.
    #[serde(default = "default_true")]
    pub allow_new_crates: bool,
}

impl Default for StructurePolicy {
    fn default() -> Self {
        Self {
            allow_new_crates: true,
        }
    }
}

/// Configuration policy knobs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigurationPolicy {
    /// Treatment of production-facing configuration changes.
    #[serde(default)]
    pub production_changes: ProductionChanges,
}

impl Default for ConfigurationPolicy {
    fn default() -> Self {
        Self {
            production_changes: ProductionChanges::RequireReview,
        }
    }
}

/// Top-level policy block (`[policy]` in `azusa.toml`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    /// Maximum tolerated overall drift score (0–10).
    #[serde(default = "default_max_drift")]
    pub max_drift: f32,
    /// Dependency rules.
    #[serde(default)]
    pub dependencies: DependencyPolicy,
    /// Structure rules.
    #[serde(default)]
    pub structure: StructurePolicy,
    /// Configuration rules.
    #[serde(default)]
    pub configuration: ConfigurationPolicy,
}

fn default_max_drift() -> f32 {
    5.0
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            max_drift: default_max_drift(),
            dependencies: DependencyPolicy::default(),
            structure: StructurePolicy::default(),
            configuration: ConfigurationPolicy::default(),
        }
    }
}

/// One policy violation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyViolation {
    /// Stable id (`POLICY-0001`…).
    pub id: String,
    /// Which policy fired.
    pub rule: String,
    /// What violated it.
    pub description: String,
    /// Severity of the violation.
    pub severity: Severity,
    /// References to the offending evidence.
    pub references: Vec<String>,
}

/// The result of checking a report against a policy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PolicyResult {
    /// All violations found (empty when the report complies).
    pub violations: Vec<PolicyViolation>,
}

impl PolicyResult {
    /// Whether the report complies with the policy.
    #[must_use]
    pub fn is_compliant(&self) -> bool {
        self.violations.is_empty()
    }

    /// Highest severity among violations (Info when compliant).
    #[must_use]
    pub fn max_severity(&self) -> Severity {
        self.violations
            .iter()
            .map(|v| v.severity)
            .max()
            .unwrap_or(Severity::Info)
    }
}

/// Check a drift report against a policy.
#[must_use]
pub fn check(report: &DriftReport, policy: &Policy) -> PolicyResult {
    let mut violations = Vec::new();

    if report.score.overall > policy.max_drift {
        violations.push(PolicyViolation {
            id: String::new(),
            rule: "policy.max_drift".into(),
            description: format!(
                "overall drift {:.2} exceeds maximum {:.2}",
                report.score.overall, policy.max_drift
            ),
            severity: Severity::High,
            references: report
                .score
                .layers
                .iter()
                .map(|(layer, score)| format!("{}: {:.2}", layer.as_str(), score))
                .collect(),
        });
    }

    for observation in &report.observations {
        match observation.category {
            Layer::Dependency => {
                check_dependencies(observation, &policy.dependencies, &mut violations);
            }
            Layer::Structure => {
                check_structure(observation, &policy.structure, &mut violations);
            }
            Layer::Configuration => {
                check_configuration(observation, &policy.configuration, &mut violations);
            }
            _ => {}
        }
    }

    // Deterministic ids after stable ordering.
    violations.sort_by(|a, b| (&b.severity, &a.rule).cmp(&(&a.severity, &b.rule)));
    for (index, violation) in violations.iter_mut().enumerate() {
        violation.id = format!("POLICY-{:04}", index + 1);
    }
    PolicyResult { violations }
}

fn check_dependencies(
    observation: &DriftObservation,
    policy: &DependencyPolicy,
    violations: &mut Vec<PolicyViolation>,
) {
    let majors: Vec<String> = observation
        .evidence
        .iter()
        .flat_map(|e| &e.references)
        .filter(|r| r.ends_with("(major)"))
        .cloned()
        .collect();
    if !policy.allow_major && !majors.is_empty() {
        violations.push(PolicyViolation {
            id: String::new(),
            rule: "policy.dependencies.allow_major".into(),
            description: format!("{} major dependency bump(s) not permitted", majors.len()),
            severity: Severity::High,
            references: majors,
        });
    }
    if !policy.allow_new && observation.title.contains("declared only in") {
        violations.push(PolicyViolation {
            id: String::new(),
            rule: "policy.dependencies.allow_new".into(),
            description: observation.title.clone(),
            severity: Severity::Medium,
            references: observation
                .evidence
                .iter()
                .flat_map(|e| &e.references)
                .cloned()
                .collect(),
        });
    }
}

fn check_structure(
    observation: &DriftObservation,
    policy: &StructurePolicy,
    violations: &mut Vec<PolicyViolation>,
) {
    if !policy.allow_new_crates && observation.title.contains("module(s) present only in") {
        violations.push(PolicyViolation {
            id: String::new(),
            rule: "policy.structure.allow_new_crates".into(),
            description: observation.title.clone(),
            severity: Severity::Medium,
            references: observation
                .evidence
                .iter()
                .flat_map(|e| &e.references)
                .cloned()
                .collect(),
        });
    }
}

const PRODUCTION_KINDS: [&str; 4] = ["dockerfile", "kubernetes", "systemd-unit", "compose"];

fn check_configuration(
    observation: &DriftObservation,
    policy: &ConfigurationPolicy,
    violations: &mut Vec<PolicyViolation>,
) {
    let is_production = observation.title.contains("configuration changed")
        && (PRODUCTION_KINDS
            .iter()
            .any(|kind| observation.title.contains(kind))
            || observation
                .evidence
                .iter()
                .flat_map(|e| &e.references)
                .any(|r| r.contains("prod")));
    if !is_production {
        return;
    }
    let (severity, rule_suffix) = match policy.production_changes {
        ProductionChanges::Forbid => (Severity::Critical, "production_changes=forbid"),
        ProductionChanges::RequireReview => (Severity::Medium, "production_changes=require-review"),
        ProductionChanges::Allow => return,
    };
    violations.push(PolicyViolation {
        id: String::new(),
        rule: format!("policy.configuration.{rule_suffix}"),
        description: format!(
            "production-facing configuration changed: {}",
            observation.title
        ),
        severity,
        references: observation
            .evidence
            .iter()
            .flat_map(|e| &e.references)
            .cloned()
            .collect(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use azusa_model::{DriftScore, DriftSummary, Provenance, RepositoryInfo, RiskLevel, StateRef};

    fn report_with(overall: f32, observations: Vec<DriftObservation>) -> DriftReport {
        DriftReport {
            schema_version: 1,
            repository: RepositoryInfo {
                name: "fixture".into(),
                root: None,
            },
            state_a: StateRef {
                spec: "worktree".into(),
                provider: azusa_model::ProviderKind::Git,
                label: "worktree".into(),
                digest: String::new(),
            },
            state_b: StateRef {
                spec: "upstream".into(),
                provider: azusa_model::ProviderKind::Git,
                label: "upstream".into(),
                digest: String::new(),
            },
            direction: azusa_model::DriftDirection::Bidirectional,
            direction_confidence: 0.9,
            score: DriftScore {
                overall,
                layers: BTreeMap::default(),
            },
            risk: azusa_model::DriftRisk {
                level: RiskLevel::High,
                rationale: String::new(),
                factors: Vec::new(),
            },
            summary: DriftSummary::default(),
            observations,
            intent: None,
            clusters: Vec::new(),
            recommendations: Vec::new(),
            provenance: Provenance {
                tool_version: "test".into(),
                analysis_mode: azusa_model::AnalysisMode::Deterministic,
                captured_at: None,
                config_digest: None,
            },
        }
    }

    fn observation(category: Layer, title: &str, references: Vec<String>) -> DriftObservation {
        DriftObservation {
            id: "DRIFT-0001".into(),
            category,
            severity: Severity::Low,
            confidence: 1.0,
            title: title.into(),
            detail: String::new(),
            side: azusa_model::Side::Both,
            evidence: vec![azusa_model::Evidence {
                kind: azusa_model::EvidenceKind::Manifest,
                description: String::new(),
                references,
            }],
        }
    }

    #[test]
    fn max_drift_violation() {
        let report = report_with(6.1, Vec::new());
        let result = check(&report, &Policy::default());
        assert!(!result.is_compliant());
        assert_eq!(result.violations[0].rule, "policy.max_drift");
        assert_eq!(result.max_severity(), Severity::High);
    }

    #[test]
    fn compliant_under_limit() {
        let report = report_with(4.9, Vec::new());
        assert!(check(&report, &Policy::default()).is_compliant());
    }

    #[test]
    fn major_bump_violates_default_dependency_policy() {
        let observation = observation(
            Layer::Dependency,
            "1 dependency version(s) changed",
            vec!["serde: 1.0.1 → 2.0.0 (major)".into()],
        );
        let report = report_with(3.0, vec![observation]);
        let result = check(&report, &Policy::default());
        assert!(!result.is_compliant());
        assert!(
            result
                .violations
                .iter()
                .any(|v| v.rule == "policy.dependencies.allow_major")
        );
    }

    #[test]
    fn minor_bump_does_not_violate() {
        let observation = observation(
            Layer::Dependency,
            "1 dependency version(s) changed",
            vec!["serde: 1.0.1 → 1.1.0 (minor)".into()],
        );
        let report = report_with(3.0, vec![observation]);
        assert!(check(&report, &Policy::default()).is_compliant());
    }

    #[test]
    fn new_crates_can_be_forbidden() {
        let observation = observation(
            Layer::Structure,
            "1 module(s) present only in worktree",
            vec!["extra".into()],
        );
        let mut policy = Policy::default();
        policy.structure.allow_new_crates = false;
        let report = report_with(3.0, vec![observation]);
        let result = check(&report, &policy);
        assert!(
            result
                .violations
                .iter()
                .any(|v| v.rule == "policy.structure.allow_new_crates")
        );
    }

    #[test]
    fn production_config_require_review() {
        let observation = observation(
            Layer::Configuration,
            "1 configuration changed: kubernetes",
            vec!["~k8s/prod/deploy.yml".into()],
        );
        let report = report_with(3.0, vec![observation]);
        let result = check(&report, &Policy::default());
        let violation = result
            .violations
            .iter()
            .find(|v| v.rule.contains("production_changes"))
            .expect("violation");
        assert_eq!(violation.severity, Severity::Medium);

        let mut strict = Policy::default();
        strict.configuration.production_changes = ProductionChanges::Forbid;
        let result = check(&report, &strict);
        assert_eq!(result.max_severity(), Severity::Critical);
    }

    #[test]
    fn formatter_changes_are_not_production() {
        let observation = observation(
            Layer::Configuration,
            "1 configuration changed: formatter",
            vec!["~rustfmt.toml".into()],
        );
        let report = report_with(3.0, vec![observation]);
        assert!(check(&report, &Policy::default()).is_compliant());
    }

    #[test]
    fn violation_ids_are_stable() {
        let observations = vec![
            observation(
                Layer::Dependency,
                "1 dependency version(s) changed",
                vec!["a: 1.0.1 → 2.0.0 (major)".into()],
            ),
            observation(
                Layer::Structure,
                "1 module(s) present only in worktree",
                vec!["x".into()],
            ),
        ];
        let report = report_with(7.5, observations);
        let mut policy = Policy::default();
        policy.structure.allow_new_crates = false;
        let first = check(&report, &policy);
        let second = check(&report, &policy);
        assert_eq!(first, second);
        assert_eq!(first.violations[0].id, "POLICY-0001");
    }

    #[test]
    fn parses_policy_toml() {
        // Inside azusa.toml the [policy] table hosts these fields directly.
        let policy: Policy =
            toml::from_str("max_drift = 3.0\n\n[dependencies]\nallow_major = true\n").unwrap();
        assert_eq!(policy.max_drift, 3.0);
        assert!(policy.dependencies.allow_major);
        assert!(policy.structure.allow_new_crates);
    }
}

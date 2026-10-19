// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Intent detection.
//!
//! Estimates whether drift looks deliberate or incidental from deterministic
//! signals: cluster composition, path patterns, generated-file markers,
//! branch naming and uncommitted hygiene. Azusa never pretends intent is
//! certain: every classification carries a confidence, and "indeterminate"
//! is a first-class answer.

use azusa_model::{
    ChangeCluster, ClusterKind, DriftDirection, IntentAssessment, IntentClass, IntentSignal,
    RepositoryState,
};

/// Weights: positive supports intent, negative supports accident.
const GENERATED_MARKER_PENALTY: f32 = -1.0;
const LOCKFILE_SIGNAL: f32 = -0.5;
const TESTS_SIGNAL: f32 = 0.5;
const DOCS_SIGNAL: f32 = 0.3;
const FEATURE_BRANCH_BONUS: f32 = 0.8;
const FIX_BRANCH_BONUS: f32 = 0.6;
const MODULE_SIGNAL: f32 = 0.5;
const OPERATIONS_SIGNAL: f32 = 0.4;

/// Assess intent for the drift between two states.
#[must_use]
#[allow(clippy::too_many_lines)]
pub fn assess(
    a: &RepositoryState,
    b: &RepositoryState,
    direction: DriftDirection,
    clusters: &[ChangeCluster],
) -> Option<IntentAssessment> {
    if clusters.is_empty() || direction == DriftDirection::InSync {
        return None;
    }
    let mut signals = Vec::new();

    // Cluster composition.
    for cluster in clusters {
        match cluster.kind {
            ClusterKind::Lockfile => signals.push(IntentSignal {
                description: format!(
                    "lockfile-only changes ({} file(s)) are usually regeneration artifacts",
                    cluster.files
                ),
                weight: LOCKFILE_SIGNAL,
            }),
            ClusterKind::Tests => signals.push(IntentSignal {
                description: format!(
                    "test changes ({} file(s)) usually accompany deliberate behaviour change",
                    cluster.files
                ),
                weight: TESTS_SIGNAL,
            }),
            ClusterKind::Documentation => signals.push(IntentSignal {
                description: format!("documentation changes ({} file(s))", cluster.files),
                weight: DOCS_SIGNAL,
            }),
            ClusterKind::Module if cluster.added > 0 || cluster.deleted > 0 => signals.push(IntentSignal {
                description: format!(
                    "new or removed source modules in {} (structural change is rarely accidental)",
                    cluster.label
                ),
                weight: MODULE_SIGNAL,
            }),
            ClusterKind::Operations => signals.push(IntentSignal {
                description: format!(
                    "deployment/CI changes in {} (usually deliberate)",
                    cluster.label
                ),
                weight: OPERATIONS_SIGNAL,
            }),
            _ => {}
        }
    }

    // Generated-file markers in retained text.
    let generated = generated_marker_count(b);
    if generated > 0 {
        signals.push(IntentSignal {
            description: format!(
                "{generated} changed file(s) carry generated-content markers (@generated / DO NOT EDIT)"
            ),
            weight: GENERATED_MARKER_PENALTY,
        });
    }

    // Branch naming (side A's VCS identity).
    if let Some(branch) = a.identity.vcs.as_ref().and_then(|v| v.branch.as_deref()) {
        let lower = branch.to_lowercase();
        if lower.starts_with("feature/") || lower.starts_with("feat/") {
            signals.push(IntentSignal {
                description: format!("branch `{branch}` follows the feature-branch convention"),
                weight: FEATURE_BRANCH_BONUS,
            });
        } else if lower.starts_with("fix/") || lower.starts_with("hotfix/") {
            signals.push(IntentSignal {
                description: format!("branch `{branch}` follows the fix-branch convention"),
                weight: FIX_BRANCH_BONUS,
            });
        } else if lower.starts_with("wip-") || lower.contains("scratch") {
            signals.push(IntentSignal {
                description: format!("branch `{branch}` looks experimental"),
                weight: -0.4,
            });
        }
    }

    // Uncommitted sloppiness leans accidental for trivial changes.
    if let Some(uncommitted) = a.ancestry.uncommitted.as_ref() {
        if uncommitted.untracked_files > 0
            && clusters.len() == 1
            && clusters[0].kind == ClusterKind::Lockfile
        {
            signals.push(IntentSignal {
                description: "untracked files present while only lockfiles changed".to_owned(),
                weight: -0.3,
            });
        }
    }

    let total: f32 = signals.iter().map(|s| s.weight).sum();
    let strength = signals.iter().map(|s| s.weight.abs()).sum::<f32>().max(1.0);
    let classification = if signals.is_empty() {
        IntentClass::Indeterminate
    } else {
        let score = total / strength;
        if score >= 0.6 {
            IntentClass::Intentional
        } else if score >= 0.2 {
            IntentClass::ProbablyIntentional
        } else if score <= -0.6 {
            IntentClass::Accidental
        } else if score <= -0.2 {
            IntentClass::ProbablyAccidental
        } else {
            IntentClass::Indeterminate
        }
    };
    let confidence = (signals.len().min(6) as f32 / 6.0 * 0.6 + 0.3).clamp(0.0, 0.95);

    Some(IntentAssessment {
        classification,
        confidence: (confidence * 100.0).round() / 100.0,
        signals,
    })
}

/// Count retained analysis files in state B that carry generated markers.
fn generated_marker_count(state: &RepositoryState) -> usize {
    state
        .captured_text
        .values()
        .filter(|text| {
            text.lines()
                .take(5)
                .any(|line| line.contains("@generated") || line.contains("DO NOT EDIT"))
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cluster(label: &str, kind: ClusterKind, files: u64) -> ChangeCluster {
        ChangeCluster {
            label: label.to_owned(),
            kind,
            paths: Vec::new(),
            files,
            added: files,
            deleted: 0,
            modified: 0,
            renamed: 0,
        }
    }

    fn state_with_branch(branch: Option<&str>) -> RepositoryState {
        let mut state = RepositoryState::empty(azusa_model::ProviderKind::Git, "worktree");
        state.identity.vcs = Some(azusa_model::VcsIdentity {
            head_oid: Some("abc".into()),
            branch: branch.map(std::borrow::ToOwned::to_owned),
            detached: false,
            upstream: Some("origin/main".into()),
            remote_urls: Vec::new(),
            shallow: false,
        });
        state
    }

    #[test]
    fn lockfile_only_drift_leans_accidental() {
        let a = state_with_branch(Some("main"));
        let assessment = assess(
            &a,
            &a,
            DriftDirection::AToB,
            &[cluster("lockfiles", ClusterKind::Lockfile, 1)],
        )
        .expect("assessment");
        assert!(
            assessment.classification == IntentClass::ProbablyAccidental
                || assessment.classification == IntentClass::Accidental,
            "got {:?}",
            assessment.classification
        );
        assert!(
            !assessment.signals.is_empty(),
            "lockfile drift should produce signals"
        );
    }

    #[test]
    fn feature_branch_with_modules_leans_intentional() {
        let a = state_with_branch(Some("feature/auth-v2"));
        let b = a.clone();
        let clusters = vec![
            cluster("crates/azusa-core", ClusterKind::Module, 6),
            cluster("tests", ClusterKind::Tests, 3),
        ];
        let assessment = assess(&a, &b, DriftDirection::AToB, &clusters).expect("assessment");
        assert!(
            assessment.classification == IntentClass::Intentional
                || assessment.classification == IntentClass::ProbablyIntentional,
            "got {:?}",
            assessment.classification
        );
    }

    #[test]
    fn in_sync_has_no_intent() {
        let a = state_with_branch(None);
        assert!(assess(&a, &a, DriftDirection::InSync, &[]).is_none());
    }

    #[test]
    fn no_clusters_no_intent() {
        let a = state_with_branch(None);
        assert!(assess(&a, &a, DriftDirection::AToB, &[]).is_none());
    }

    #[test]
    fn generated_markers_push_toward_accidental() {
        let mut a = state_with_branch(None);
        a.captured_text.insert(
            "generated/api.rs".into(),
            "// @generated — DO NOT EDIT\n".into(),
        );
        let clusters = vec![cluster("generated", ClusterKind::Module, 2)];
        let assessment = assess(&a, &a, DriftDirection::AToB, &clusters).expect("assessment");
        assert!(
            assessment
                .signals
                .iter()
                .any(|s| s.description.contains("@generated"))
        );
    }
}

//! The comparison engine.
//!
//! [`compare`] takes two captured [`RepositoryState`]s and produces an
//! evidence-first [`DriftReport`]. The engine is a pure function of its
//! inputs plus [`AzusaConfig`]: identical states, configuration and tool
//! version always produce identical reports (L0–L2 in this milestone;
//! structural layers arrive with phase 2).

use std::collections::BTreeMap;

use azusa_diff::FilesetDiff;
use azusa_model::{
    AnalysisMode, DRIFT_REPORT_SCHEMA_VERSION, DriftDirection, DriftObservation, DriftReport,
    DriftRisk, DriftScore, DriftSummary, Evidence, EvidenceKind, Layer, Priority, Recommendation,
    RepositoryInfo, RiskLevel, Severity, Side, StateRef,
};

use crate::config::AzusaConfig;
use crate::scoring::drift_score;

/// Cap on evidence reference paths listed per observation.
const MAX_EVIDENCE_PATHS: usize = 8;

/// An observation under construction (ids assigned deterministically after
/// sorting).
struct Draft {
    category: Layer,
    severity: Severity,
    confidence: f32,
    title: String,
    detail: String,
    side: Side,
    evidence: Vec<Evidence>,
}

/// Compare two states into a full drift report.
///
/// The body is a linear pipeline (identity → ancestry → filesystem → scoring
/// → risk → recommendations); splitting it would hide the layer order.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn compare(
    a_name: &str,
    a: &azusa_model::RepositoryState,
    b_name: &str,
    b: &azusa_model::RepositoryState,
    config: &AzusaConfig,
    notes: &[String],
) -> DriftReport {
    let mut drafts: Vec<Draft> = Vec::new();
    let repository = RepositoryInfo {
        name: repository_name(a, b),
        root: a.identity.root.clone().or_else(|| b.identity.root.clone()),
    };

    // L0 identity: are these comparable states at all?
    let overlap = path_overlap(&a.filesystem, &b.filesystem);
    let digests_equal = a.identity.digest == b.identity.digest;
    let unrelated = !digests_equal && overlap < config.relatedness.min_overlap;

    let empty_diff = FilesetDiff::default();
    let computed_diff = if digests_equal {
        None
    } else {
        Some(azusa_diff::diff(
            &a.filesystem,
            &b.filesystem,
            &config.rename_options(),
        ))
    };
    let diff = computed_diff.as_ref().unwrap_or(&empty_diff);

    if digests_equal {
        drafts.push(Draft {
            category: Layer::Identity,
            severity: Severity::Info,
            confidence: 1.0,
            title: "states are content-identical".into(),
            detail: format!(
                "both states hash to the same fileset digest ({})",
                truncate_digest(&a.identity.digest)
            ),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: format!(
                    "{} and {} agree on all {} paths",
                    a.identity.label, b.identity.label, a.identity.file_count
                ),
                references: vec![truncate_digest(&a.identity.digest)],
            }],
        });
    } else if unrelated {
        drafts.push(Draft {
            category: Layer::Identity,
            severity: Severity::High,
            confidence: 0.9,
            title: "states appear unrelated".into(),
            detail: format!(
                "common-path overlap is {:.0}% (minimum {:.0}%): these states likely do not describe the same repository",
                overlap * 100.0,
                config.relatedness.min_overlap * 100.0
            ),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::PathPattern,
                description: format!(
                    "{} paths in `{}`, {} in `{}`, {} in common",
                    a.identity.file_count,
                    a.identity.label,
                    b.identity.file_count,
                    b.identity.label,
                    common_path_count(&a.filesystem, &b.filesystem)
                ),
                references: Vec::new(),
            }],
        });
    }

    // L1 ancestry: committed drift direction, uncommitted drift presence.
    let (direction, direction_confidence) =
        determine_direction(a, b, diff, digests_equal, unrelated);
    push_ancestry_observations(a, b_name, &mut drafts);

    // L2 filesystem: additions, deletions, modifications, renames.
    push_filesystem_observations(a_name, b_name, diff, &mut drafts);

    // Carry resolver notes (e.g. upstream fallback) as evidence.
    for note in notes {
        drafts.push(Draft {
            category: Layer::Identity,
            severity: Severity::Info,
            confidence: 1.0,
            title: "resolution note".into(),
            detail: note.clone(),
            side: Side::Both,
            evidence: Vec::new(),
        });
    }

    // Structural layers (L3–L9) are filled by phase-2+ enrichment; layers
    // with no analysis are excluded from scoring rather than counted as 0.
    let structural: BTreeMap<Layer, f32> = BTreeMap::new();
    let effective_ancestry = effective_ancestry_for_scoring(a, b);
    let score: DriftScore = drift_score(&effective_ancestry, diff, &structural, &config.drift);

    let risk = assess_risk(&score, direction, &drafts);
    let recommendations = recommend(direction, diff, &effective_ancestry);
    let summary = DriftSummary {
        files_added: diff.added.len() as u64,
        files_deleted: diff.deleted.len() as u64,
        files_modified: diff.modified.len() as u64,
        files_renamed: diff.renamed.len() as u64,
        commits_ahead: effective_ancestry.ahead,
        commits_behind: effective_ancestry.behind,
        uncommitted_a: a
            .ancestry
            .uncommitted
            .as_ref()
            .is_some_and(|u| !u.is_clean()),
        uncommitted_b: b
            .ancestry
            .uncommitted
            .as_ref()
            .is_some_and(|u| !u.is_clean()),
    };

    let observations = assign_ids(drafts);
    DriftReport {
        schema_version: DRIFT_REPORT_SCHEMA_VERSION,
        repository,
        state_a: StateRef {
            spec: a_name.to_owned(),
            provider: a.identity.provider,
            label: a.identity.label.clone(),
            digest: a.identity.digest.clone(),
        },
        state_b: StateRef {
            spec: b_name.to_owned(),
            provider: b.identity.provider,
            label: b.identity.label.clone(),
            digest: b.identity.digest.clone(),
        },
        direction,
        direction_confidence,
        score,
        risk,
        summary,
        observations,
        intent: None,
        recommendations,
        provenance: azusa_model::Provenance {
            tool_version: azusa_model::TOOL_VERSION.to_owned(),
            analysis_mode: AnalysisMode::Deterministic,
            captured_at: None,
            config_digest: Some(config.digest()),
        },
    }
}

/// The ancestry facet that describes A relative to B, when A is a local Git
/// state whose upstream is B. Otherwise a neutral facet.
fn effective_ancestry_for_scoring(
    a: &azusa_model::RepositoryState,
    _b: &azusa_model::RepositoryState,
) -> azusa_model::Ancestry {
    a.ancestry.clone()
}

fn determine_direction(
    a: &azusa_model::RepositoryState,
    b: &azusa_model::RepositoryState,
    diff: &FilesetDiff,
    digests_equal: bool,
    unrelated: bool,
) -> (DriftDirection, f32) {
    if digests_equal {
        return (DriftDirection::InSync, 1.0);
    }
    if unrelated {
        return (DriftDirection::Unrelated, 0.9);
    }
    // Git ancestry is authoritative when present.
    let ahead = a.ancestry.ahead;
    let behind = a.ancestry.behind;
    if let (Some(ahead), Some(behind)) = (ahead, behind) {
        return match (ahead, behind) {
            (0, 0) => {
                // Aligned commits but content differs: uncommitted drift.
                let a_dirty = a
                    .ancestry
                    .uncommitted
                    .as_ref()
                    .is_some_and(|u| !u.is_clean());
                let b_dirty = b
                    .ancestry
                    .uncommitted
                    .as_ref()
                    .is_some_and(|u| !u.is_clean());
                if a_dirty && !b_dirty {
                    (DriftDirection::AToB, 0.9)
                } else if b_dirty && !a_dirty {
                    (DriftDirection::BToA, 0.9)
                } else {
                    (DriftDirection::Bidirectional, 0.5)
                }
            }
            (0, n) if n > 0 => (DriftDirection::BToA, 0.95),
            (n, 0) if n > 0 => (DriftDirection::AToB, 0.95),
            (_, _) => (DriftDirection::Bidirectional, 0.95),
        };
    }
    // Filesystem heuristic: unique content on both sides suggests divergence
    // on both sides; unique content on one side suggests one-way evolution.
    let a_unique = diff.deleted.len();
    let b_unique = diff.added.len();
    if a_unique > 0 && b_unique > 0 {
        (DriftDirection::Bidirectional, 0.5)
    } else if b_unique > 0 && a_unique == 0 && diff.modified.is_empty() {
        (DriftDirection::BToA, 0.6)
    } else if a_unique > 0 && b_unique == 0 && diff.modified.is_empty() {
        (DriftDirection::AToB, 0.6)
    } else {
        (DriftDirection::Unknown, 0.3)
    }
}

fn push_ancestry_observations(
    a: &azusa_model::RepositoryState,
    b_name: &str,
    drafts: &mut Vec<Draft>,
) {
    let ancestry = &a.ancestry;
    match (ancestry.ahead, ancestry.behind) {
        (Some(ahead), Some(behind)) if ahead > 0 || behind > 0 => {
            let (title, severity) = match (ahead, behind) {
                (a, b) if a > 0 && b > 0 => (
                    format!(
                        "diverged: {a} local commit(s) not in {b_name}, {b} remote commit(s) not local"
                    ),
                    Severity::High,
                ),
                (a, 0) => (
                    format!("{a} local commit(s) not yet in {b_name}"),
                    Severity::Medium,
                ),
                (0, b) => (
                    format!("{b} remote commit(s) not present locally"),
                    Severity::Medium,
                ),
                _ => unreachable!("guarded by outer match"),
            };
            drafts.push(Draft {
                category: Layer::Ancestry,
                severity,
                confidence: 1.0,
                title,
                detail: "committed history has diverged between the states".into(),
                side: Side::Both,
                evidence: ancestry_evidence(a),
            });
        }
        _ => {}
    }
    if let Some(uncommitted) = &ancestry.uncommitted {
        if !uncommitted.is_clean() {
            drafts.push(Draft {
                category: Layer::Ancestry,
                severity: Severity::Medium,
                confidence: 1.0,
                title: "uncommitted drift present".into(),
                detail: format!(
                    "the worktree carries uncommitted work: {} staged, {} unstaged, {} untracked",
                    uncommitted.staged_files,
                    uncommitted.unstaged_files,
                    uncommitted.untracked_files
                ),
                side: Side::A,
                evidence: vec![Evidence {
                    kind: EvidenceKind::File,
                    description: "uncommitted paths detected by status classification".into(),
                    references: Vec::new(),
                }],
            });
        }
    }
}

fn ancestry_evidence(a: &azusa_model::RepositoryState) -> Vec<Evidence> {
    let mut evidence = Vec::new();
    if let Some(vcs) = &a.identity.vcs {
        if let Some(head) = &vcs.head_oid {
            evidence.push(Evidence {
                kind: EvidenceKind::Commit,
                description: format!("local head is {head}"),
                references: vec![truncate_digest(head)],
            });
        }
        if let Some(upstream) = &vcs.upstream {
            evidence.push(Evidence {
                kind: EvidenceKind::Branch,
                description: format!("tracking upstream {upstream}"),
                references: vec![upstream.clone()],
            });
        }
    }
    if let Some(base) = &a.ancestry.merge_base {
        evidence.push(Evidence {
            kind: EvidenceKind::Commit,
            description: format!("merge base is {}", truncate_digest(base)),
            references: vec![truncate_digest(base)],
        });
    }
    evidence
}

fn push_filesystem_observations(
    a_name: &str,
    b_name: &str,
    diff: &FilesetDiff,
    drafts: &mut Vec<Draft>,
) {
    let added = diff.added.keys().cloned().collect::<Vec<_>>();
    let deleted = diff.deleted.keys().cloned().collect::<Vec<_>>();
    let modified = diff
        .modified
        .iter()
        .map(|m| m.path.clone())
        .collect::<Vec<_>>();
    let renamed = diff
        .renamed
        .iter()
        .map(|r| format!("{} → {}", r.from, r.to))
        .collect::<Vec<_>>();

    if !added.is_empty() {
        let severity = severity_for_count(added.len(), 20, 100);
        drafts.push(Draft {
            category: Layer::Filesystem,
            severity,
            confidence: 1.0,
            title: format!("{} file(s) added in {b_name}", added.len()),
            detail: "paths present only in the second state".into(),
            side: Side::B,
            evidence: vec![path_evidence("added paths", &added)],
        });
    }
    if !deleted.is_empty() {
        let severity = severity_for_count(deleted.len(), 10, 50).max(Severity::Medium);
        drafts.push(Draft {
            category: Layer::Filesystem,
            severity,
            confidence: 1.0,
            title: format!(
                "{} file(s) deleted in {b_name} (relative to {a_name})",
                deleted.len()
            ),
            detail: "paths present only in the first state; deletions weigh double in scoring"
                .into(),
            side: Side::A,
            evidence: vec![path_evidence("deleted paths", &deleted)],
        });
    }
    if !modified.is_empty() {
        let severity = severity_for_count(modified.len(), 20, 100);
        let lines_delta: i64 = diff.modified.iter().filter_map(|m| m.line_delta).sum();
        let mut detail = "paths whose content changed between the states".to_owned();
        if lines_delta != 0 {
            detail.push_str(&format!(" (net {lines_delta} lines)"));
        }
        drafts.push(Draft {
            category: Layer::Filesystem,
            severity,
            confidence: 1.0,
            title: format!("{} file(s) modified", modified.len()),
            detail,
            side: Side::Both,
            evidence: vec![path_evidence("modified paths", &modified)],
        });
    }
    if !renamed.is_empty() {
        drafts.push(Draft {
            category: Layer::Filesystem,
            severity: Severity::Low,
            confidence: 0.9,
            title: format!("{} file(s) renamed/moved", renamed.len()),
            detail:
                "detected by content identity or path similarity; renames weigh half in scoring"
                    .into(),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::Rename,
                description: "detected moves".into(),
                references: cap(&renamed),
            }],
        });
    }
    let mode_only = diff.modified.iter().filter(|m| m.mode_only).count();
    if mode_only > 0 {
        drafts.push(Draft {
            category: Layer::Filesystem,
            severity: Severity::Low,
            confidence: 1.0,
            title: format!("{mode_only} file(s) changed executable mode only"),
            detail: "content identical; only the executable bit differs".into(),
            side: Side::Both,
            evidence: Vec::new(),
        });
    }
}

fn severity_for_count(count: usize, medium_at: usize, high_at: usize) -> Severity {
    if count >= high_at {
        Severity::High
    } else if count >= medium_at {
        Severity::Medium
    } else {
        Severity::Low
    }
}

fn path_evidence(description: &str, paths: &[String]) -> Evidence {
    Evidence {
        kind: EvidenceKind::File,
        description: format!(
            "{description} ({} shown of {})",
            paths.len().min(MAX_EVIDENCE_PATHS),
            paths.len()
        ),
        references: cap(paths),
    }
}

fn cap(paths: &[String]) -> Vec<String> {
    let mut capped: Vec<String> = paths.iter().take(MAX_EVIDENCE_PATHS).cloned().collect();
    if paths.len() > MAX_EVIDENCE_PATHS {
        capped.push(format!("… and {} more", paths.len() - MAX_EVIDENCE_PATHS));
    }
    capped
}

fn path_overlap(a: &azusa_model::FilesystemState, b: &azusa_model::FilesystemState) -> f64 {
    let common = common_path_count(a, b);
    let smaller = a.files.len().min(b.files.len());
    if smaller == 0 {
        return if a.files.len() == b.files.len() {
            1.0
        } else {
            0.0
        };
    }
    common as f64 / smaller as f64
}

fn common_path_count(a: &azusa_model::FilesystemState, b: &azusa_model::FilesystemState) -> usize {
    a.files
        .keys()
        .filter(|path| b.files.contains_key(*path))
        .count()
}

fn assess_risk(score: &DriftScore, direction: DriftDirection, drafts: &[Draft]) -> DriftRisk {
    let mut factors = Vec::new();
    let has_critical = drafts.iter().any(|d| d.severity >= Severity::Critical);
    let has_high = drafts.iter().any(|d| d.severity >= Severity::High);
    let level = if has_critical {
        factors.push("a critical-severity observation is present".into());
        RiskLevel::Critical
    } else if score.overall >= 7.0
        || (direction == DriftDirection::Bidirectional && score.overall >= 5.0)
        || has_high && score.overall >= 4.0
    {
        if score.overall >= 7.0 {
            factors.push(format!("overall drift {:.1} ≥ 7.0", score.overall));
        }
        if direction == DriftDirection::Bidirectional {
            factors.push("both sides carry changes the other lacks".into());
        }
        if has_high {
            factors.push("high-severity observations present".into());
        }
        RiskLevel::High
    } else if score.overall >= 4.0 || direction == DriftDirection::Bidirectional {
        if score.overall >= 4.0 {
            factors.push(format!("overall drift {:.1} ≥ 4.0", score.overall));
        }
        if direction == DriftDirection::Bidirectional {
            // No fast-forward path exists; reconciliation requires a merge
            // decision even when the numeric drift is small.
            factors.push("bidirectional drift requires a merge decision".into());
        }
        RiskLevel::Medium
    } else {
        factors.push(format!(
            "overall drift {:.1} below all thresholds",
            score.overall
        ));
        RiskLevel::Low
    };
    DriftRisk {
        level,
        rationale: format!(
            "risk classified from overall score {:.2}, direction {} and the highest observation severity",
            score.overall,
            direction.as_str()
        ),
        factors,
    }
}

fn recommend(
    direction: DriftDirection,
    diff: &FilesetDiff,
    ancestry: &azusa_model::Ancestry,
) -> Vec<Recommendation> {
    let mut out = Vec::new();
    match direction {
        DriftDirection::InSync => out.push(Recommendation {
            action: "nothing to reconcile".into(),
            rationale: "states are content-identical".into(),
            priority: Priority::Low,
        }),
        DriftDirection::AToB => out.push(Recommendation {
            action: "publish local work (azusa plan → kaptaind push)".into(),
            rationale: "local state is strictly ahead; no conflicting remote evolution detected"
                .into(),
            priority: Priority::Medium,
        }),
        DriftDirection::BToA => out.push(Recommendation {
            action: "integrate remote work (azusa plan → kaptaind pull)".into(),
            rationale: "remote state is strictly ahead; a clean fast-forward path is likely".into(),
            priority: Priority::Medium,
        }),
        DriftDirection::Bidirectional => {
            out.push(Recommendation {
                action: "reconcile carefully: review clusters, then rebase or merge".into(),
                rationale: "both sides evolved independently; conflict likelihood is non-trivial"
                    .into(),
                priority: Priority::High,
            });
            if diff.change_count() > 30 {
                out.push(Recommendation {
                    action: "decompose reconciliation into logical clusters before merging".into(),
                    rationale: format!(
                        "{} changed paths exceed the single-changeset comfort zone",
                        diff.change_count()
                    ),
                    priority: Priority::High,
                });
            }
        }
        DriftDirection::Unrelated => out.push(Recommendation {
            action: "verify you are comparing the right repositories".into(),
            rationale: "states share almost no content; likely a path or remote mistake".into(),
            priority: Priority::High,
        }),
        DriftDirection::Unknown => out.push(Recommendation {
            action: "inspect both states manually before acting".into(),
            rationale: "insufficient evidence to determine drift direction".into(),
            priority: Priority::Medium,
        }),
    }
    if let Some(uncommitted) = &ancestry.uncommitted {
        if !uncommitted.is_clean() {
            out.push(Recommendation {
                action: "commit or stash uncommitted work before reconciling".into(),
                rationale: format!(
                    "{} uncommitted path(s) would complicate a merge or rebase",
                    uncommitted.total()
                ),
                priority: Priority::High,
            });
        }
    }
    out
}

/// Sort drafts deterministically and assign stable ids (`DRIFT-0001`…).
fn assign_ids(drafts: Vec<Draft>) -> Vec<DriftObservation> {
    let mut sorted = drafts;
    sorted.sort_by(|x, y| {
        (x.category, y.severity, &x.title).cmp(&(y.category, x.severity, &y.title))
    });
    sorted
        .into_iter()
        .enumerate()
        .map(|(index, draft)| DriftObservation {
            id: format!("DRIFT-{:04}", index + 1),
            category: draft.category,
            severity: draft.severity,
            confidence: (draft.confidence * 100.0).round() / 100.0,
            title: draft.title,
            detail: draft.detail,
            side: draft.side,
            evidence: draft.evidence,
        })
        .collect()
}

fn repository_name(a: &azusa_model::RepositoryState, b: &azusa_model::RepositoryState) -> String {
    let from_vcs = a
        .identity
        .vcs
        .as_ref()
        .and_then(|v| v.remote_urls.first())
        .and_then(|url| url.rsplit('/').next().map(std::borrow::ToOwned::to_owned))
        .or_else(|| {
            b.identity
                .vcs
                .as_ref()
                .and_then(|v| v.remote_urls.first())
                .and_then(|url| url.rsplit('/').next().map(std::borrow::ToOwned::to_owned))
        });
    let from_path = a
        .identity
        .root
        .as_ref()
        .or(b.identity.root.as_ref())
        .and_then(|root| {
            std::path::Path::new(root)
                .file_name()
                .and_then(|n| n.to_str())
                .map(std::borrow::ToOwned::to_owned)
        });
    from_vcs
        .map(|name| name.trim_end_matches(".git").to_owned())
        .or(from_path)
        .unwrap_or_else(|| "unknown".to_owned())
}

fn truncate_digest(digest: &str) -> String {
    digest.chars().take(12).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resolve::StateResolver;
    use azusa_git::testutil::FixtureRepo;

    fn report_for(fixture_root: &std::path::Path, a: &str, b: &str) -> DriftReport {
        let resolver = StateResolver::new(fixture_root);
        let state_a = resolver.resolve(a).unwrap();
        let state_b = resolver.resolve(b).unwrap();
        compare(a, &state_a, b, &state_b, &AzusaConfig::default(), &[])
    }

    #[test]
    fn clean_repository_is_in_sync_with_low_risk() {
        let fixture = FixtureRepo::with_remote();
        let report = report_for(&fixture.path, "worktree", "upstream");
        assert_eq!(report.direction, DriftDirection::InSync);
        assert_eq!(report.risk.level, RiskLevel::Low);
        assert_eq!(report.score.overall, 0.0);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.title.contains("content-identical"))
        );
    }

    #[test]
    fn uncommitted_only_drift_is_atob_with_uncommitted_observation() {
        let fixture = FixtureRepo::with_remote();
        fixture.write("dirty.txt", "uncommitted\n");
        let report = report_for(&fixture.path, "worktree", "upstream");
        assert_eq!(report.direction, DriftDirection::AToB);
        assert!(report.summary.uncommitted_a);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.title == "uncommitted drift present")
        );
        assert!(report.score.layers[&Layer::Filesystem] > 0.0);
        assert!(report.score.layers[&Layer::Ancestry] > 0.0);
    }

    #[test]
    fn local_ahead_by_commits_is_atob() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("feature.rs", "pub fn feature() {}\n", "feat: feature");
        let report = report_for(&fixture.path, "worktree", "upstream");
        assert_eq!(report.direction, DriftDirection::AToB);
        assert_eq!(report.summary.commits_ahead, Some(1));
        assert_eq!(report.summary.commits_behind, Some(0));
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.title.contains("1 local commit"))
        );
    }

    #[test]
    fn remote_ahead_is_btoa() {
        let fixture = FixtureRepo::with_remote();
        fixture.advance_remote(
            "remote-change.rs",
            "pub fn remote() {}\n",
            "feat: remote work",
        );
        let report = report_for(&fixture.path, "worktree", "upstream");
        assert_eq!(report.direction, DriftDirection::BToA);
        assert_eq!(report.summary.commits_behind, Some(1));
    }

    #[test]
    fn divergence_is_bidirectional_and_risky() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("local-change.rs", "pub fn local() {}\n", "feat: local work");
        fixture.advance_remote(
            "remote-change.rs",
            "pub fn remote() {}\n",
            "feat: remote work",
        );
        let report = report_for(&fixture.path, "worktree", "upstream");
        assert_eq!(report.direction, DriftDirection::Bidirectional);
        assert!(report.risk.level >= RiskLevel::Medium);
        assert!(
            report
                .recommendations
                .iter()
                .any(|r| r.action.contains("reconcile carefully"))
        );
    }

    #[test]
    fn unrelated_states_are_flagged() {
        let fixture = FixtureRepo::new();
        let resolver = StateResolver::new(&fixture.path);
        let a = resolver.resolve("head").unwrap();

        // A directory with entirely different content is an unrelated state.
        let other = tempfile::tempdir().unwrap();
        std::fs::write(other.path().join("alpha.txt"), "one\n").unwrap();
        std::fs::write(other.path().join("beta.txt"), "two\n").unwrap();
        let other_resolver = StateResolver::new(other.path());
        let b = other_resolver
            .resolve(other.path().to_str().unwrap())
            .unwrap();

        let report = compare("head", &a, "other", &b, &AzusaConfig::default(), &[]);
        assert_eq!(report.direction, DriftDirection::Unrelated);
        assert!(
            report
                .observations
                .iter()
                .any(|o| o.title.contains("unrelated"))
        );
    }

    #[test]
    fn reports_are_deterministic() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("one.rs", "pub fn one() {}\n", "feat: one");
        let first = report_for(&fixture.path, "worktree", "upstream");
        let second = report_for(&fixture.path, "worktree", "upstream");
        let first_json = serde_json::to_string(&first).unwrap();
        let second_json = serde_json::to_string(&second).unwrap();
        assert_eq!(first_json, second_json);
    }

    #[test]
    fn observation_ids_are_stable_and_sorted() {
        let fixture = FixtureRepo::with_remote();
        fixture.commit_file("one.rs", "pub fn one() {}\n", "feat: one");
        fixture.write("dirty.txt", "x\n");
        let report = report_for(&fixture.path, "worktree", "upstream");
        let ids: Vec<&str> = report.observations.iter().map(|o| o.id.as_str()).collect();
        assert_eq!(ids.first(), Some(&"DRIFT-0001"));
        assert!(ids.windows(2).all(|w| w[0] < w[1]));
    }
}

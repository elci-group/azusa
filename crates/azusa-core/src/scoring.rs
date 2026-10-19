// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Deterministic layer scoring.
//!
//! Every layer score comes from a fixed saturating curve
//! `10 · c / (c + k)` over a weighted change count `c`, so scores are
//! reproducible, monotonic in change volume, and never depend on wall-clock
//! time or environment. Constants are documented per layer and tested.

use azusa_diff::FilesetDiff;
use azusa_model::{Ancestry, DriftDirection, DriftScore, Layer, Severity};

/// Saturating score: 0 for no change, approaching 10 as `count` grows.
#[must_use]
pub fn saturating(count: f32, k: f32) -> f32 {
    if count <= 0.0 {
        0.0
    } else {
        10.0 * count / (count + k)
    }
}

/// Severity for facet items present only on one side (L6 interfaces, L9
/// operations).
///
/// The *breaking* case is items on the side that reconciliation would
/// overwrite (the side behind the drift source): those items vanish from
/// the surface. Items on the drift source's side are growth. When both
/// sides evolved (bidirectional), either could be the casualty, so both
/// rate Medium.
#[must_use]
pub fn side_severity(side_a: bool, direction: DriftDirection, count: usize) -> Severity {
    let breaking = match direction {
        DriftDirection::AToB => !side_a,
        DriftDirection::BToA => side_a,
        DriftDirection::Bidirectional => true,
        _ => false,
    };
    if breaking {
        if count >= 5 {
            Severity::High
        } else {
            Severity::Medium
        }
    } else {
        Severity::Low
    }
}

/// Round to two decimal places (display and JSON stability).
#[must_use]
pub fn round2(value: f32) -> f32 {
    (value * 100.0).round() / 100.0
}

/// L1 ancestry score: commits ahead/behind plus uncommitted paths.
///
/// k = 12: 3 novel commits score ≈ 1.9, 12 commits ≈ 5.0, 36 commits ≈ 7.5.
#[must_use]
pub fn ancestry_score(ancestry: &Ancestry) -> f32 {
    let commits = ancestry.ahead.unwrap_or(0) + ancestry.behind.unwrap_or(0);
    let uncommitted = ancestry.uncommitted.map_or(0, |u| u.total());
    saturating(commits as f32 + uncommitted as f32 * 0.5, 12.0)
}

/// L2 filesystem score: deletions weigh double (removals carry more risk
/// than additions), renames weigh half (moves are cheap to reconcile).
///
/// k = 40: 10 changed files ≈ 2.0, 40 ≈ 5.0, 120 ≈ 7.5.
#[must_use]
pub fn filesystem_score(diff: &FilesetDiff) -> f32 {
    let count = diff.added.len() as f32
        + diff.modified.len() as f32
        + 2.0 * diff.deleted.len() as f32
        + 0.5 * diff.renamed.len() as f32;
    saturating(count, 40.0)
}

/// Overall score: weighted mean over analyzed layers only. A layer missing
/// from `scores` was not analyzed (empty facets on both sides) and is
/// excluded from both numerator and denominator — never silently counted as
/// zero drift.
#[must_use]
pub fn overall_score(
    scores: &std::collections::BTreeMap<Layer, f32>,
    weights: &crate::config::DriftWeights,
) -> f32 {
    let mut total = 0.0_f32;
    let mut weight_sum = 0.0_f32;
    for layer in Layer::all() {
        let weight = weights.get(layer);
        if weight <= 0.0 {
            continue;
        }
        let Some(score) = scores.get(&layer) else {
            continue;
        };
        total += weight * score;
        weight_sum += weight;
    }
    if weight_sum == 0.0 {
        0.0
    } else {
        round2(total / weight_sum)
    }
}

/// Assemble the composite drift index from per-layer scores.
#[must_use]
pub fn drift_score(
    ancestry: &Ancestry,
    diff: &FilesetDiff,
    structural: &std::collections::BTreeMap<Layer, f32>,
    weights: &crate::config::DriftWeights,
) -> DriftScore {
    let mut layers = structural.clone();
    layers.insert(Layer::Ancestry, round2(ancestry_score(ancestry)));
    layers.insert(Layer::Filesystem, round2(filesystem_score(diff)));
    let overall = overall_score(&layers, weights);
    DriftScore { overall, layers }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_model::UncommittedSummary;

    #[test]
    fn saturating_is_monotonic_and_bounded() {
        assert_eq!(saturating(0.0, 10.0), 0.0);
        let s1 = saturating(5.0, 10.0);
        let s2 = saturating(10.0, 10.0);
        let s3 = saturating(10_000.0, 10.0);
        assert!(s1 < s2 && s2 < s3 && s3 <= 10.0);
    }

    #[test]
    fn ancestry_counts_uncommitted_at_half_weight() {
        let ancestry = Ancestry {
            ahead: Some(2),
            uncommitted: Some(UncommittedSummary {
                staged_files: 2,
                unstaged_files: 2,
                untracked_files: 0,
            }),
            ..Ancestry::default()
        };
        // c = 2 + 4*0.5 = 4 → 10*4/16 = 2.5
        assert!((ancestry_score(&ancestry) - 2.5).abs() < 1e-4);
    }

    #[test]
    fn filesystem_double_weights_deletions() {
        let empty = azusa_model::FilesystemState::default();
        let mut with_one = empty.clone();
        use azusa_model::{FileEntry, FileKind, content_hash};
        with_one.files.insert(
            "x".into(),
            FileEntry {
                hash: content_hash(b"x"),
                size: 1,
                executable: false,
                kind: FileKind::File,
                lines: Some(1),
            },
        );
        let options = azusa_diff::RenameOptions::default();
        // A deleted one file vs empty: c = 2 → 10*2/42
        let diff = azusa_diff::diff(&with_one, &empty, &options);
        let score_deleted = filesystem_score(&diff);
        // A added one file vs empty: c = 1 → 10/41
        let diff = azusa_diff::diff(&empty, &with_one, &options);
        let score_added = filesystem_score(&diff);
        assert!(score_deleted > score_added);
    }

    #[test]
    fn overall_respects_weights_and_skips_unanalyzed() {
        let mut scores = std::collections::BTreeMap::new();
        scores.insert(Layer::Semantic, 10.0);
        scores.insert(Layer::Filesystem, 0.0);
        let weights = crate::config::DriftWeights {
            semantic: 2.0,
            filesystem: 1.0,
            ..crate::config::DriftWeights::default()
        };
        let overall = overall_score(&scores, &weights);
        // Only the two analyzed layers participate: (2*10 + 1*0) / 3.
        assert!((overall - 20.0 / 3.0).abs() < 0.02);
    }
}

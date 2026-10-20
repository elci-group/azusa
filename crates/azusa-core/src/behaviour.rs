// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Behavioural comparison layers (L6–L7).
//!
//! Compares the behavioural facets of two states: the public API surface
//! (L6 interfaces) and the test/entry-point/build-script surface (L7
//! behaviour). Each layer produces evidence-first observations plus a
//! deterministic layer score; empty facets on both sides mean "not
//! analyzed" and contribute nothing.

use azusa_model::{
    BehaviourState, DriftDirection, Evidence, EvidenceKind, EvidenceStrength, Layer, Severity, Side,
};

use crate::engine::Draft;
use crate::scoring::{round2, saturating, side_severity};

/// Cap on evidence reference paths listed per observation.
const MAX_EVIDENCE_PATHS: usize = 8;

fn facet(state: &azusa_model::RepositoryState) -> BehaviourState {
    state.behaviour.clone().unwrap_or_default()
}

/// L6: public API surface, compared at symbol identity level.
///
/// Each state's facet carries extracted public symbols (fqn, kind,
/// normalized signature) per file. The diff reports symbols that exist on
/// only one side and symbols whose signature changed with the same
/// identity — so a parameter-type change registers as interface drift even
/// when the symbol count is unchanged.
#[must_use]
#[allow(clippy::too_many_lines)]
pub(crate) fn compare_interfaces(
    a: &azusa_model::RepositoryState,
    b: &azusa_model::RepositoryState,
    direction: DriftDirection,
) -> crate::structural::LayerOutcome {
    let mut drafts = Vec::new();
    let (fa, fb) = (facet(a), facet(b));
    if fa.public_symbols.is_empty() && fb.public_symbols.is_empty() {
        return crate::structural::LayerOutcome { drafts, score: 0.0 };
    }

    // Flatten to one identity map: key = file::fqn.
    let symbol_map =
        |facet: &BehaviourState| -> std::collections::BTreeMap<String, (String, String)> {
            facet
                .public_symbols
                .iter()
                .flat_map(|(path, symbols)| {
                    symbols.iter().map(move |symbol| {
                        (
                            format!("{path}::{}", symbol.fqn),
                            (symbol.kind.clone(), symbol.signature.clone()),
                        )
                    })
                })
                .collect()
        };
    let (symbols_a, symbols_b) = (symbol_map(&fa), symbol_map(&fb));

    let mut removed: Vec<String> = Vec::new();
    let mut added: Vec<String> = Vec::new();
    let mut changed: Vec<String> = Vec::new();
    for (key, (kind_a, signature_a)) in &symbols_a {
        match symbols_b.get(key) {
            None => removed.push(key.clone()),
            Some((kind_b, signature_b)) => {
                if kind_a != kind_b || signature_a != signature_b {
                    changed.push(key.clone());
                }
            }
        }
    }
    for key in symbols_b.keys() {
        if !symbols_a.contains_key(key) {
            added.push(key.clone());
        }
    }

    // Score: a changed signature (same identity, different declaration) is
    // the strongest breaking signal; removals next; additions cheapest.
    let count = 2.0 * removed.len() as f32 + 3.0 * changed.len() as f32 + added.len() as f32;

    if !removed.is_empty() {
        drafts.push(Draft {
            category: Layer::Interface,
            strength: EvidenceStrength::Derived,
            severity: side_severity(true, direction, removed.len()),
            confidence: 1.0,
            title: format!(
                "{} public symbol(s) present only in {}",
                removed.len(),
                a.identity.label
            ),
            detail: "public symbols exist on this side that the other state lacks; reconciling toward the other state removes them from the API surface"
                .to_owned(),
            side: Side::A,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "public symbols absent from the second state".to_owned(),
                references: removed.iter().take(MAX_EVIDENCE_PATHS).cloned().collect(),
            }],
        });
    }
    if !added.is_empty() {
        drafts.push(Draft {
            category: Layer::Interface,
            strength: EvidenceStrength::Derived,
            severity: side_severity(false, direction, added.len()),
            confidence: 1.0,
            title: format!(
                "{} public symbol(s) present only in {}",
                added.len(),
                b.identity.label
            ),
            detail: "public symbols exist on this side that the other state lacks; reconciling toward the other state removes them from the API surface"
                .to_owned(),
            side: Side::B,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "public symbols absent from the first state".to_owned(),
                references: added.iter().take(MAX_EVIDENCE_PATHS).cloned().collect(),
            }],
        });
    }
    if !changed.is_empty() {
        drafts.push(Draft {
            category: Layer::Interface,
            strength: EvidenceStrength::Derived,
            severity: if changed.len() >= 5 {
                Severity::High
            } else {
                Severity::Medium
            },
            confidence: 1.0,
            title: format!("{} public signature(s) changed", changed.len()),
            detail: "same symbol identity, different declaration: potential breaking API change that a pure symbol count would miss"
                .to_owned(),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "symbols whose declaration signature changed".to_owned(),
                references: changed.iter().take(MAX_EVIDENCE_PATHS).cloned().collect(),
            }],
        });
    }

    crate::structural::LayerOutcome {
        drafts,
        score: round2(saturating(count, 6.0)),
    }
}

/// L7: tests, benches, entry points and build scripts.
#[must_use]
#[allow(clippy::too_many_lines)]
pub(crate) fn compare_behaviour(
    a: &azusa_model::RepositoryState,
    b: &azusa_model::RepositoryState,
) -> crate::structural::LayerOutcome {
    let mut drafts = Vec::new();
    let (fa, fb) = (facet(a), facet(b));
    if fa == BehaviourState::default() && fb == BehaviourState::default() {
        return crate::structural::LayerOutcome { drafts, score: 0.0 };
    }

    let removed = |a_side: &[String], b_side: &[String]| -> Vec<String> {
        a_side
            .iter()
            .filter(|path| !b_side.contains(path))
            .cloned()
            .collect()
    };
    let added = |a_side: &[String], b_side: &[String]| -> Vec<String> {
        b_side
            .iter()
            .filter(|path| !a_side.contains(path))
            .cloned()
            .collect()
    };

    let mut score_count = 0.0_f32;

    // Test files: removals shrink behavioural coverage, additions extend it.
    let tests_removed = removed(&fa.test_files, &fb.test_files);
    let tests_added = added(&fa.test_files, &fb.test_files);
    if !tests_removed.is_empty() {
        score_count += 1.5 * tests_removed.len() as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: if tests_removed.len() >= 3 {
                Severity::High
            } else {
                Severity::Medium
            },
            confidence: 1.0,
            title: format!("{} test file(s) removed", tests_removed.len()),
            detail: "tests present in the first state are absent from the second; coverage may have regressed"
                .to_owned(),
            side: Side::A,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "removed test files".to_owned(),
                references: tests_removed.iter().take(MAX_EVIDENCE_PATHS).cloned().collect(),
            }],
        });
    }
    if !tests_added.is_empty() {
        score_count += tests_added.len() as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: Severity::Low,
            confidence: 1.0,
            title: format!("{} test file(s) added", tests_added.len()),
            detail: "new tests accompany (or should accompany) behaviour change".to_owned(),
            side: Side::B,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "added test files".to_owned(),
                references: tests_added
                    .iter()
                    .take(MAX_EVIDENCE_PATHS)
                    .cloned()
                    .collect(),
            }],
        });
    }

    // Test declarations: aggregate count movement across retained sources.
    let decl_delta = fb.test_declarations as i64 - fa.test_declarations as i64;
    if decl_delta < 0 {
        score_count += 0.2 * (-decl_delta) as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: Severity::Medium,
            confidence: 0.9,
            title: format!("{0} test declaration(s) removed", -decl_delta),
            detail: "fewer #[test] declarations exist in the second state".to_owned(),
            side: Side::A,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: format!(
                    "test declarations {} → {}",
                    fa.test_declarations, fb.test_declarations
                ),
                references: Vec::new(),
            }],
        });
    } else if decl_delta > 0 {
        score_count += 0.1 * decl_delta as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: Severity::Low,
            confidence: 0.9,
            title: format!("{decl_delta} test declaration(s) added"),
            detail: "more #[test] declarations exist in the second state".to_owned(),
            side: Side::B,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: format!(
                    "test declarations {} → {}",
                    fa.test_declarations, fb.test_declarations
                ),
                references: Vec::new(),
            }],
        });
    }

    // Entry points: executables appearing or disappearing.
    let entries_removed = removed(&fa.entry_points, &fb.entry_points);
    let entries_added = added(&fa.entry_points, &fb.entry_points);
    if !entries_removed.is_empty() {
        score_count += 2.0 * entries_removed.len() as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: Severity::Medium,
            confidence: 1.0,
            title: format!("{} entry point(s) removed", entries_removed.len()),
            detail: "executables present in the first state are absent from the second".to_owned(),
            side: Side::A,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "removed entry points".to_owned(),
                references: entries_removed
                    .iter()
                    .take(MAX_EVIDENCE_PATHS)
                    .cloned()
                    .collect(),
            }],
        });
    }
    if !entries_added.is_empty() {
        score_count += 1.0 * entries_added.len() as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: Severity::Low,
            confidence: 1.0,
            title: format!("{} entry point(s) added", entries_added.len()),
            detail: "new executables extend the command surface".to_owned(),
            side: Side::B,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "added entry points".to_owned(),
                references: entries_added
                    .iter()
                    .take(MAX_EVIDENCE_PATHS)
                    .cloned()
                    .collect(),
            }],
        });
    }

    // Build scripts: added, removed or content-changed build.rs files.
    let mut build_changed = removed(&fa.build_scripts, &fb.build_scripts);
    build_changed.extend(added(&fa.build_scripts, &fb.build_scripts));
    for path in &fa.build_scripts {
        if fb.build_scripts.contains(path) {
            let hash = |state: &azusa_model::RepositoryState| {
                state
                    .filesystem
                    .files
                    .get(path)
                    .map(|entry| entry.hash.clone())
            };
            if hash(a) != hash(b) {
                build_changed.push(path.clone());
            }
        }
    }
    if !build_changed.is_empty() {
        score_count += 2.0 * build_changed.len() as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: Severity::Medium,
            confidence: 1.0,
            title: format!("{} build script(s) changed", build_changed.len()),
            detail: "build scripts execute at compile time; changes alter how the tree builds"
                .to_owned(),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::File,
                description: "changed build scripts".to_owned(),
                references: build_changed
                    .iter()
                    .take(MAX_EVIDENCE_PATHS)
                    .cloned()
                    .collect(),
            }],
        });
    }

    // Benches: informational, weighted lightly.
    let benches_changed = removed(&fa.bench_files, &fb.bench_files).len()
        + added(&fa.bench_files, &fb.bench_files).len();
    if benches_changed > 0 {
        score_count += 0.5 * benches_changed as f32;
        drafts.push(Draft {
            category: Layer::Behaviour,
            strength: EvidenceStrength::Heuristic,
            severity: Severity::Low,
            confidence: 1.0,
            title: format!("{benches_changed} benchmark file(s) changed"),
            detail: "benchmark surface moved between the states".to_owned(),
            side: Side::Both,
            evidence: Vec::new(),
        });
    }

    crate::structural::LayerOutcome {
        drafts,
        score: round2(saturating(score_count, 8.0)),
    }
}

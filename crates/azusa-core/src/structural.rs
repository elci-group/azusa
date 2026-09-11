//! Structural comparison layers (L3–L5).
//!
//! Compares enriched facets of two states: module topology (L3),
//! dependency declarations and lockfiles (L4), and configuration files
//! (L5). Each layer produces evidence-first observations plus a
//! deterministic layer score; empty facets on both sides mean "not
//! analyzed" and contribute nothing.

use std::collections::BTreeMap;

use azusa_model::{
    ConfigurationKind, DependencyEntry, Evidence, EvidenceKind, Layer, Severity, Side,
};

use crate::engine::Draft;
use crate::scoring::{round2, saturating};

/// A dependency version change classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VersionBump {
    /// Major segment changed (or requirement crossed a major boundary).
    Major,
    /// Minor segment changed.
    Minor,
    /// Patch segment changed.
    Patch,
    /// Changed but not parseable as semver.
    Unclassified,
}

impl VersionBump {
    fn as_str(self) -> &'static str {
        match self {
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Patch => "patch",
            Self::Unclassified => "unclassified",
        }
    }
}

/// Classify an old→new version pair.
#[must_use]
pub fn version_bump(old: &str, new: &str) -> VersionBump {
    let parse = |v: &str| -> Option<Vec<u64>> {
        v.trim_start_matches(['^', '~', '=', '>', '<'])
            .split('.')
            .map(|part| {
                part.split(['-', '+'])
                    .next()
                    .unwrap_or(part)
                    .parse::<u64>()
                    .ok()
            })
            .collect::<Option<Vec<_>>>()
    };
    match (parse(old), parse(new)) {
        (Some(old_parts), Some(new_parts)) => {
            let len = old_parts.len().max(new_parts.len());
            for index in 0..len {
                let o = old_parts.get(index).copied().unwrap_or(0);
                let n = new_parts.get(index).copied().unwrap_or(0);
                if o != n {
                    return match index {
                        0 => VersionBump::Major,
                        1 => VersionBump::Minor,
                        _ => VersionBump::Patch,
                    };
                }
            }
            VersionBump::Patch
        }
        _ => VersionBump::Unclassified,
    }
}

/// Outcome of one structural layer: observations plus its layer score.
pub(crate) struct LayerOutcome {
    /// Observation drafts for the layer.
    pub drafts: Vec<Draft>,
    /// Layer score, 0.0–10.0.
    pub score: f32,
}

/// L3: module topology.
///
/// Linear evidence assembly per outcome; length is breadth, not complexity.
#[allow(clippy::too_many_lines)]
#[must_use]
pub(crate) fn compare_structure(
    a_name: &str,
    a: &azusa_model::RepositoryState,
    b_name: &str,
    b: &azusa_model::RepositoryState,
) -> LayerOutcome {
    let mut drafts = Vec::new();
    if a.structure.modules.is_empty() && b.structure.modules.is_empty() {
        return LayerOutcome { drafts, score: 0.0 };
    }

    let modules_a: BTreeMap<&str, &azusa_model::ModuleInfo> = a
        .structure
        .modules
        .iter()
        .map(|m| (m.name.as_str(), m))
        .collect();
    let modules_b: BTreeMap<&str, &azusa_model::ModuleInfo> = b
        .structure
        .modules
        .iter()
        .map(|m| (m.name.as_str(), m))
        .collect();

    let added: Vec<&str> = modules_b
        .keys()
        .copied()
        .filter(|name| !modules_a.contains_key(name))
        .collect();
    let removed: Vec<&str> = modules_a
        .keys()
        .copied()
        .filter(|name| !modules_b.contains_key(name))
        .collect();
    let kind_changed: Vec<String> = modules_a
        .iter()
        .filter_map(|(name, module)| {
            modules_b
                .get(name)
                .filter(|other| other.kind != module.kind)
                .map(|_| (*name).to_owned())
        })
        .collect();

    if !added.is_empty() {
        drafts.push(Draft {
            category: Layer::Structure,
            severity: if added.len() >= 5 {
                Severity::High
            } else {
                Severity::Medium
            },
            confidence: 1.0,
            title: format!("{} module(s) present only in {b_name}", added.len()),
            detail: "crates or packages appear in the second state but not the first".to_owned(),
            side: Side::B,
            evidence: vec![Evidence {
                kind: EvidenceKind::Manifest,
                description: "added modules".to_owned(),
                references: added
                    .iter()
                    .map(|name| (*name).to_owned())
                    .take(8)
                    .collect(),
            }],
        });
    }
    if !removed.is_empty() {
        drafts.push(Draft {
            category: Layer::Structure,
            severity: Severity::High,
            confidence: 1.0,
            title: format!("{} module(s) present only in {a_name}", removed.len()),
            detail: "crates or packages present in the first state are absent from the second"
                .to_owned(),
            side: Side::A,
            evidence: vec![Evidence {
                kind: EvidenceKind::Manifest,
                description: "removed modules".to_owned(),
                references: removed
                    .iter()
                    .map(|name| (*name).to_owned())
                    .take(8)
                    .collect(),
            }],
        });
    }
    if !kind_changed.is_empty() {
        drafts.push(Draft {
            category: Layer::Structure,
            severity: Severity::Low,
            confidence: 1.0,
            title: format!("{} module(s) changed shape", kind_changed.len()),
            detail: "module kind changed (library ↔ binary ↔ workspace)".to_owned(),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::Manifest,
                description: "reshaped modules".to_owned(),
                references: kind_changed,
            }],
        });
    }

    let dirs_added = b
        .structure
        .directories
        .difference(&a.structure.directories)
        .count();
    let dirs_removed = a
        .structure
        .directories
        .difference(&b.structure.directories)
        .count();
    if dirs_added + dirs_removed > 0 {
        drafts.push(Draft {
            category: Layer::Structure,
            severity: Severity::Low,
            confidence: 0.9,
            title: format!(
                "directory topology changed ({dirs_added} added, {dirs_removed} removed)"
            ),
            detail: "source trees reorganised between the states".to_owned(),
            side: Side::Both,
            evidence: Vec::new(),
        });
    }

    // Score: modules weigh 1.5 each (removals double), directories 0.2.
    let count =
        added.len() as f32 + 2.0 * removed.len() as f32 + 0.2 * (dirs_added + dirs_removed) as f32;
    LayerOutcome {
        drafts,
        score: round2(saturating(count, 10.0)),
    }
}

/// L4: dependency declarations and lockfiles.
///
/// Linear evidence assembly per outcome; length is breadth, not complexity.
#[allow(clippy::too_many_lines)]
#[must_use]
pub(crate) fn compare_dependencies(
    a_name: &str,
    a: &azusa_model::RepositoryState,
    b_name: &str,
    b: &azusa_model::RepositoryState,
) -> LayerOutcome {
    let mut drafts = Vec::new();
    if a.dependencies.entries.is_empty() && b.dependencies.entries.is_empty() {
        return LayerOutcome { drafts, score: 0.0 };
    }

    let key = |entry: &DependencyEntry| (entry.manifest.clone(), entry.name.clone());
    let deps_a: BTreeMap<(String, String), &DependencyEntry> =
        a.dependencies.entries.iter().map(|e| (key(e), e)).collect();
    let deps_b: BTreeMap<(String, String), &DependencyEntry> =
        b.dependencies.entries.iter().map(|e| (key(e), e)).collect();

    let mut added = Vec::new();
    let mut removed = Vec::new();
    let mut bumps: Vec<(String, VersionBump, String, String)> = Vec::new();

    for (dep_key, entry) in &deps_b {
        match deps_a.get(dep_key) {
            None => added.push(format!("{} → {}", entry.manifest, entry.name)),
            Some(old) => {
                let old_version = old
                    .resolved
                    .clone()
                    .unwrap_or_else(|| old.requirement.clone());
                let new_version = entry
                    .resolved
                    .clone()
                    .unwrap_or_else(|| entry.requirement.clone());
                if old_version != new_version && !(old_version.is_empty() && new_version.is_empty())
                {
                    let bump = version_bump(&old_version, &new_version);
                    bumps.push((entry.name.clone(), bump, old_version, new_version));
                }
            }
        }
    }
    for (dep_key, entry) in &deps_a {
        if !deps_b.contains_key(dep_key) {
            removed.push(format!("{} → {}", entry.manifest, entry.name));
        }
    }

    if !added.is_empty() {
        drafts.push(Draft {
            category: Layer::Dependency,
            severity: Severity::Medium,
            confidence: 1.0,
            title: format!("{} dependency(ies) declared only in {b_name}", added.len()),
            detail: "new third-party code enters the build on that side".to_owned(),
            side: Side::B,
            evidence: vec![Evidence {
                kind: EvidenceKind::Manifest,
                description: "added dependencies".to_owned(),
                references: added.into_iter().take(8).collect(),
            }],
        });
    }
    if !removed.is_empty() {
        drafts.push(Draft {
            category: Layer::Dependency,
            severity: Severity::Medium,
            confidence: 1.0,
            title: format!(
                "{} dependency(ies) declared only in {a_name}",
                removed.len()
            ),
            detail: "third-party code present on this side is absent from the other".to_owned(),
            side: Side::A,
            evidence: vec![Evidence {
                kind: EvidenceKind::Manifest,
                description: "removed dependencies".to_owned(),
                references: removed.into_iter().take(8).collect(),
            }],
        });
    }
    if !bumps.is_empty() {
        let majors = bumps
            .iter()
            .filter(|(_, bump, _, _)| *bump == VersionBump::Major)
            .count();
        let severity = if majors > 0 {
            Severity::High
        } else if bumps.len() > 3 {
            Severity::Medium
        } else {
            Severity::Low
        };
        drafts.push(Draft {
            category: Layer::Dependency,
            severity,
            confidence: 1.0,
            title: format!("{} dependency version(s) changed", bumps.len()),
            detail: format!("{majors} major bump(s); resolved or required versions moved"),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::Lockfile,
                description: "version transitions (old → new)".to_owned(),
                references: bumps
                    .iter()
                    .map(|(name, bump, old, new)| {
                        format!("{name}: {old} → {new} ({})", bump.as_str())
                    })
                    .take(8)
                    .collect(),
            }],
        });
    }

    // Lockfile divergence, independent of manifest-level changes.
    let lock_a = a.dependencies.lockfiles.first();
    let lock_b = b.dependencies.lockfiles.first();
    let lock_diverged = match (lock_a, lock_b) {
        (Some(x), Some(y)) => x.digest != y.digest,
        (None, Some(_)) | (Some(_), None) => true,
        (None, None) => false,
    };
    let mut lock_weight = 0.0_f32;
    if lock_diverged {
        lock_weight = 1.0;
        drafts.push(Draft {
            category: Layer::Dependency,
            severity: Severity::Low,
            confidence: 1.0,
            title: "lockfile diverged".into(),
            detail: "the dependency resolution differs between the states; usually regenerated rather than hand-merged"
                .to_owned(),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::Lockfile,
                description: "Cargo.lock digest changed".to_owned(),
                references: Vec::new(),
            }],
        });
    }

    // Score: additions/removals 1.5, majors 3, minors 2, patches 1, lockfile 1.
    let mut count = 0.0;
    for (_, bump, _, _) in &bumps {
        count += match bump {
            VersionBump::Major => 3.0,
            VersionBump::Minor => 2.0,
            VersionBump::Patch | VersionBump::Unclassified => 1.0,
        };
    }
    count += 1.5 * (added_count(&deps_a, &deps_b) + removed_count(&deps_a, &deps_b)) as f32;
    count += lock_weight;
    LayerOutcome {
        drafts,
        score: round2(saturating(count, 8.0)),
    }
}

fn added_count(
    a: &BTreeMap<(String, String), &DependencyEntry>,
    b: &BTreeMap<(String, String), &DependencyEntry>,
) -> usize {
    b.keys().filter(|k| !a.contains_key(*k)).count()
}

fn removed_count(
    a: &BTreeMap<(String, String), &DependencyEntry>,
    b: &BTreeMap<(String, String), &DependencyEntry>,
) -> usize {
    a.keys().filter(|k| !b.contains_key(*k)).count()
}

/// L5: configuration files.
///
/// Linear evidence assembly per outcome; length is breadth, not complexity.
#[allow(clippy::too_many_lines)]
#[must_use]
pub(crate) fn compare_configuration(
    a: &azusa_model::RepositoryState,
    b: &azusa_model::RepositoryState,
) -> LayerOutcome {
    let mut drafts = Vec::new();
    if a.configuration.files.is_empty() && b.configuration.files.is_empty() {
        return LayerOutcome { drafts, score: 0.0 };
    }

    let configs_a: BTreeMap<&str, &azusa_model::ConfigurationFile> = a
        .configuration
        .files
        .iter()
        .map(|f| (f.path.as_str(), f))
        .collect();
    let configs_b: BTreeMap<&str, &azusa_model::ConfigurationFile> = b
        .configuration
        .files
        .iter()
        .map(|f| (f.path.as_str(), f))
        .collect();

    // Group changed configuration files by kind.
    let mut by_kind: BTreeMap<ConfigurationKind, Vec<String>> = BTreeMap::new();
    for (path, config_b) in &configs_b {
        match configs_a.get(path) {
            None => by_kind
                .entry(config_b.kind)
                .or_default()
                .push(format!("+{path}")),
            Some(previous)
                if previous.digest != config_b.digest || previous.kind != config_b.kind =>
            {
                by_kind
                    .entry(config_b.kind)
                    .or_default()
                    .push(format!("~{path}"));
            }
            _ => {}
        }
    }
    for (path, config_a) in &configs_a {
        if !configs_b.contains_key(path) {
            by_kind
                .entry(config_a.kind)
                .or_default()
                .push(format!("-{path}"));
        }
    }

    let weight_of = |kind: ConfigurationKind| -> f32 {
        match kind {
            ConfigurationKind::CiWorkflow
            | ConfigurationKind::Dockerfile
            | ConfigurationKind::Kubernetes
            | ConfigurationKind::SystemdUnit => 2.5,
            ConfigurationKind::Compose | ConfigurationKind::Environment => 1.5,
            ConfigurationKind::CargoManifest => 1.0,
            ConfigurationKind::Formatter | ConfigurationKind::Linter | ConfigurationKind::Other => {
                0.5
            }
        }
    };

    let mut score_count = 0.0;
    for (kind, paths) in by_kind {
        let weight = weight_of(kind);
        score_count += weight * paths.len() as f32;
        let severity = if weight >= 2.5 && paths.len() >= 2 {
            Severity::High
        } else if weight >= 2.5 {
            Severity::Medium
        } else {
            Severity::Low
        };
        drafts.push(Draft {
            category: Layer::Configuration,
            severity,
            confidence: 1.0,
            title: format!("{} configuration changed: {}", paths.len(), kind.as_str()),
            detail: format!(
                "{} files in the {} configuration area changed between the states",
                paths.len(),
                kind.as_str()
            ),
            side: Side::Both,
            evidence: vec![Evidence {
                kind: EvidenceKind::Config,
                description: "changed configuration paths (+ added, ~ modified, − removed)"
                    .to_owned(),
                references: paths.into_iter().take(8).collect(),
            }],
        });
    }

    LayerOutcome {
        drafts,
        score: round2(saturating(score_count, 6.0)),
    }
}

/// Assemble per-layer structural scores for the drift index.
#[must_use]
pub(crate) fn structural_scores(outcomes: &[(Layer, LayerOutcome)]) -> BTreeMap<Layer, f32> {
    outcomes
        .iter()
        .filter(|(_, outcome)| !outcome.drafts.is_empty() || outcome.score > 0.0)
        .map(|(layer, outcome)| (*layer, outcome.score))
        .collect()
}

/// Flatten layer outcomes into drafts (deterministic order by layer).
#[must_use]
pub(crate) fn structural_drafts(outcomes: &[(Layer, LayerOutcome)]) -> Vec<Draft> {
    outcomes
        .iter()
        .flat_map(|(_, outcome)| outcome.drafts.iter().cloned())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_bump_classification() {
        assert_eq!(version_bump("1.2.3", "2.0.0"), VersionBump::Major);
        assert_eq!(version_bump("1.2.3", "1.3.0"), VersionBump::Minor);
        assert_eq!(version_bump("1.2.3", "1.2.4"), VersionBump::Patch);
        assert_eq!(version_bump("^1.2", "1.4"), VersionBump::Minor);
        assert_eq!(version_bump("branch", "main"), VersionBump::Unclassified);
        assert_eq!(version_bump("1.2.3", "1.2.3"), VersionBump::Patch);
    }
}

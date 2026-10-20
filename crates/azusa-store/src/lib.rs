// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Azusa local evidence store.
//!
//! Maintains a compact, append-only, auditable record of drift reports under
//! `.azusa/`: a JSONL report history plus the event log. Never duplicates
//! repository content — reports reference state digests, not file trees.

use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use azusa_model::DriftReport;

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-store";

/// Default runtime directory (relative to a repository root).
pub const RUNTIME_DIR: &str = ".azusa";

/// Schema version of store records.
pub const STORE_SCHEMA_VERSION: u32 = 1;

/// Watch-mode escalation state, persisted under `.azusa/watch.json`.
///
/// Stage 0 = idle (quick check found no change); stage 1 = change detected
/// and analysed; stage 2 = escalated (risk grew or is high/critical; a
/// reconciliation plan was generated). The quick digest is the stat-only
/// worktree fingerprint at the last full analysis — mtime feeds the change
/// detector only, never the deterministic analysis path.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WatchState {
    /// Schema version of this record.
    pub schema_version: u32,
    /// Stat digest at the last completed full analysis.
    pub last_full_quick_digest: String,
    /// Current escalation stage (0–2).
    pub stage: u32,
    /// Consecutive cycles with drift at stage 2 or above.
    pub streak: u32,
    /// Overall score of the last full analysis.
    pub last_score: Option<f32>,
    /// Risk level of the last full analysis.
    pub last_risk: String,
}

impl Default for WatchState {
    fn default() -> Self {
        Self {
            schema_version: STORE_SCHEMA_VERSION,
            last_full_quick_digest: String::new(),
            stage: 0,
            streak: 0,
            last_score: None,
            last_risk: String::new(),
        }
    }
}

/// One history record: the report, trimmed to what history needs.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HistoryRecord {
    /// Store schema version.
    pub schema_version: u32,
    /// RFC 3339 timestamp of when the report was produced.
    pub timestamp: String,
    /// Repository name.
    pub repository: String,
    /// State A spec.
    pub state_a: String,
    /// State B spec.
    pub state_b: String,
    /// Direction.
    pub direction: String,
    /// Overall drift score.
    pub score: f32,
    /// Risk level.
    pub risk: String,
    /// Intent classification, when assessed.
    pub intent: Option<String>,
    /// Number of observations.
    pub observations: u64,
    /// Number of change clusters.
    pub clusters: u64,
    /// Digest of state A at comparison time.
    pub digest_a: String,
    /// Digest of state B at comparison time.
    pub digest_b: String,
}

impl HistoryRecord {
    /// Build a record from a report.
    #[must_use]
    pub fn from_report(report: &DriftReport, timestamp: &str) -> Self {
        Self {
            schema_version: STORE_SCHEMA_VERSION,
            timestamp: timestamp.to_owned(),
            repository: report.repository.name.clone(),
            state_a: report.state_a.spec.clone(),
            state_b: report.state_b.spec.clone(),
            direction: report.direction.as_str().to_owned(),
            score: report.score.overall,
            risk: report.risk.level.as_str().to_owned(),
            intent: report
                .intent
                .as_ref()
                .map(|intent| intent.classification.as_str().to_owned()),
            observations: report.observations.len() as u64,
            clusters: report.clusters.len() as u64,
            digest_a: report.state_a.digest.clone(),
            digest_b: report.state_b.digest.clone(),
        }
    }
}

/// The local evidence store under `.azusa/`.
pub struct Store {
    root: PathBuf,
}

impl Store {
    /// Open (or create lazily) the store under `<repo_root>/.azusa`.
    #[must_use]
    pub fn open(repo_root: &Path) -> Self {
        Self {
            root: repo_root.join(RUNTIME_DIR),
        }
    }

    /// The store root directory.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Path of the report history file.
    #[must_use]
    pub fn history_path(&self) -> PathBuf {
        self.root.join("history.jsonl")
    }

    /// Path of the ELCI event log.
    #[must_use]
    pub fn events_path(&self) -> PathBuf {
        self.root.join("events.jsonl")
    }

    /// Path of the watch-mode state file.
    #[must_use]
    pub fn watch_state_path(&self) -> PathBuf {
        self.root.join("watch.json")
    }

    /// Load the watch-mode state, or a fresh default when absent or
    /// unreadable. Best-effort: a corrupt state resets the watcher rather
    /// than failing it.
    #[must_use]
    pub fn load_watch_state(&self) -> WatchState {
        std::fs::read(self.watch_state_path())
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default()
    }

    /// Persist the watch-mode state. Best-effort: failures never fail the
    /// watcher.
    pub fn save_watch_state(&self, state: &WatchState) {
        if let Some(parent) = self.watch_state_path().parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(json) = serde_json::to_string_pretty(state) {
            std::fs::write(self.watch_state_path(), json).ok();
        }
    }

    /// Directory for export artefacts.
    #[must_use]
    pub fn exports_dir(&self) -> PathBuf {
        self.root.join("exports")
    }

    /// Append a report to the history.
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] when the history file cannot be written.
    pub fn append_history(&self, report: &DriftReport, timestamp: &str) -> std::io::Result<()> {
        let record = HistoryRecord::from_report(report, timestamp);
        Self::append_line(&self.history_path(), &record)
    }

    /// Read the history (oldest first).
    ///
    /// # Errors
    ///
    /// Returns [`std::io::Error`] when the history file cannot be read.
    pub fn read_history(&self) -> std::io::Result<Vec<HistoryRecord>> {
        let bytes = std::fs::read(self.history_path()).unwrap_or_default();
        let mut records = Vec::new();
        for line in String::from_utf8_lossy(&bytes).lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(record) = serde_json::from_str::<HistoryRecord>(line) {
                records.push(record);
            }
        }
        Ok(records)
    }

    /// Append a raw NDJSON line to a store file.
    fn append_line<T: Serialize>(path: &Path, value: &T) -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)?;
        writeln!(file, "{}", serde_json::to_string(value).unwrap_or_default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_model::{
        AnalysisMode, DriftDirection, DriftRisk, DriftScore, DriftSummary, Provenance,
        RepositoryInfo, RiskLevel, StateRef,
    };
    use std::collections::BTreeMap;

    fn report() -> DriftReport {
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
                digest: "aaa".into(),
            },
            state_b: StateRef {
                spec: "upstream".into(),
                provider: azusa_model::ProviderKind::Git,
                label: "upstream".into(),
                digest: "bbb".into(),
            },
            direction: DriftDirection::Bidirectional,
            direction_confidence: 0.9,
            score: DriftScore {
                overall: 5.5,
                layers: BTreeMap::default(),
            },
            risk: DriftRisk {
                level: RiskLevel::High,
                rationale: String::new(),
                factors: Vec::new(),
            },
            summary: DriftSummary::default(),
            observations: Vec::new(),
            intent: None,
            clusters: vec![azusa_model::ChangeCluster {
                label: "tests".into(),
                kind: azusa_model::ClusterKind::Tests,
                paths: Vec::new(),
                files: 2,
                added: 1,
                deleted: 0,
                modified: 1,
                renamed: 0,
            }],
            recommendations: Vec::new(),
            provenance: Provenance {
                tool_version: "test".into(),
                analysis_mode: AnalysisMode::Deterministic,
                captured_at: None,
                config_digest: None,
            },
        }
    }

    #[test]
    fn history_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(dir.path());
        let report = report();
        store
            .append_history(&report, "2026-09-11T00:00:00Z")
            .expect("append");
        store
            .append_history(&report, "2026-09-11T01:00:00Z")
            .expect("append");
        let history = store.read_history().expect("read");
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].repository, "fixture");
        assert_eq!(history[0].risk, "high");
        assert_eq!(history[0].clusters, 1);
        assert_eq!(history[1].timestamp, "2026-09-11T01:00:00Z");
    }

    #[test]
    fn history_survives_missing_file() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(dir.path());
        assert!(
            store.read_history().expect("read").is_empty(),
            "fresh store should have no history"
        );
    }

    #[test]
    fn watch_state_roundtrips_and_defaults() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(dir.path());
        assert_eq!(
            store.load_watch_state(),
            WatchState::default(),
            "fresh store starts idle"
        );

        let state = WatchState {
            last_full_quick_digest: "abc123".to_owned(),
            stage: 2,
            streak: 3,
            last_score: Some(6.5),
            last_risk: "high".to_owned(),
            ..WatchState::default()
        };
        store.save_watch_state(&state);
        assert_eq!(store.load_watch_state(), state);

        assert!(store.watch_state_path().is_file());
    }

    #[test]
    fn corrupt_watch_state_resets_to_default() {
        let dir = tempfile::tempdir().expect("tempdir");
        let store = Store::open(dir.path());
        std::fs::create_dir_all(dir.path().join(".azusa")).expect("mkdir");
        std::fs::write(store.watch_state_path(), "not json at all").expect("write");
        assert_eq!(store.load_watch_state(), WatchState::default());
    }
}

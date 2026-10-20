// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! Azusa ELCI integration.
//!
//! Emits the ELCI universal event contract as NDJSON (for Vamos-style
//! transports) and writes versioned export artefacts — canonical JSON drift
//! reports with provenance — that other ELCI tools (Amber, Fract, Traci,
//! Isopod, Bart, Scrawny, Padagonia) can consume as a shared data product.

use std::io::Write;
use std::path::Path;

use serde::{Deserialize, Serialize};

use azusa_model::{DriftReport, RiskLevel};

/// Name of this crate, used in provenance records.
pub const CRATE_NAME: &str = "azusa-elci";

/// Schema version of the event contract.
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Canonical event names of the ELCI contract.
pub mod events {
    /// Emitted when any drift is detected.
    pub const DRIFT_DETECTED: &str = "azusa.drift.detected";
    /// Emitted when drift exceeds configured thresholds.
    pub const DRIFT_THRESHOLD_EXCEEDED: &str = "azusa.drift.threshold_exceeded";
    /// Emitted when a policy check fails.
    pub const POLICY_FAILED: &str = "azusa.policy.failed";
    /// Emitted when a reconciliation plan is generated.
    pub const PLAN_GENERATED: &str = "azusa.plan.generated";
    /// Emitted when an analysis run starts.
    pub const INSPECT_STARTED: &str = "azusa.inspect.started";
    /// Emitted when an analysis run completes.
    pub const INSPECT_COMPLETED: &str = "azusa.inspect.completed";
}

/// One ELCI event record.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Event {
    /// Schema version of the event contract.
    pub schema_version: u32,
    /// Event name (see [`events`]).
    pub event: String,
    /// RFC 3339 / UTC timestamp.
    pub timestamp: String,
    /// Repository name.
    pub repository: String,
    /// State A selector.
    pub state_a: String,
    /// State B selector.
    pub state_b: String,
    /// Risk level at emission time.
    pub severity: String,
    /// Overall drift score at emission time.
    pub score: f32,
    /// Number of observations backing the event.
    pub evidence: u64,
    /// azusa tool version.
    pub tool_version: String,
    /// Analysis mode of the underlying report.
    pub analysis_mode: String,
}

impl Event {
    /// Build an event from a drift report.
    #[must_use]
    pub fn from_report(event: &str, report: &DriftReport, timestamp: &str) -> Self {
        Self {
            schema_version: EVENT_SCHEMA_VERSION,
            event: event.to_owned(),
            timestamp: timestamp.to_owned(),
            repository: report.repository.name.clone(),
            state_a: report.state_a.spec.clone(),
            state_b: report.state_b.spec.clone(),
            severity: report.risk.level.as_str().to_owned(),
            score: report.score.overall,
            evidence: report.observations.len() as u64,
            tool_version: report.provenance.tool_version.clone(),
            analysis_mode: report.provenance.analysis_mode.as_str().to_owned(),
        }
    }

    /// Serialise to one NDJSON line.
    #[must_use]
    pub fn to_ndjson(&self) -> String {
        serde_json::to_string(self).unwrap_or_default()
    }
}

/// Current UTC time as an RFC 3339 string (second precision).
#[must_use]
pub fn utc_now_rfc3339() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    format_unix_as_rfc3339(now.as_secs())
}

/// Format unix seconds as an RFC 3339 UTC timestamp.
///
/// Deterministic calendar math (no external time database).
#[must_use]
pub fn format_unix_as_rfc3339(seconds: u64) -> String {
    let days = seconds / 86_400;
    let time_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days as i64);
    let hour = time_of_day / 3600;
    let minute = (time_of_day % 3600) / 60;
    let second = time_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

/// Days-since-epoch to civil date (Howard Hinnant's algorithm).
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = if m <= 2 { y + 1 } else { y };
    (year, m as u32, d as u32)
}

/// An NDJSON event emitter writing to a file (and optionally stdout).
pub struct EventEmitter {
    path: std::path::PathBuf,
    also_stdout: bool,
}

impl EventEmitter {
    /// Create an emitter writing to `path` (created, appended).
    #[must_use]
    pub fn to_file(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
            also_stdout: false,
        }
    }

    /// Also mirror events to stdout.
    #[must_use]
    pub fn with_stdout(mut self, enabled: bool) -> Self {
        self.also_stdout = enabled;
        self
    }

    /// Emit one event. Best-effort: failures to write never fail analysis.
    pub fn emit(&self, event: &Event) {
        let line = event.to_ndjson();
        if self.also_stdout {
            println!("{line}");
        }
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = writeln!(file, "{line}");
        }
    }

    /// Emit the events implied by a report (drift detected; threshold
    /// exceeded at high risk or above).
    pub fn emit_for_report(&self, report: &DriftReport) {
        let timestamp = utc_now_rfc3339();
        if report.direction != azusa_model::DriftDirection::InSync
            && report.direction != azusa_model::DriftDirection::Unrelated
        {
            self.emit(&Event::from_report(
                events::DRIFT_DETECTED,
                report,
                &timestamp,
            ));
            if report.risk.level >= RiskLevel::High {
                self.emit(&Event::from_report(
                    events::DRIFT_THRESHOLD_EXCEEDED,
                    report,
                    &timestamp,
                ));
            }
        }
    }
}

/// Write a canonical export artefact for a report.
///
/// The artefact embeds the full report plus an export envelope (schema
/// version, artefact kind, timestamp). Returns the written path.
///
/// # Errors
///
/// Returns [`std::io::Error`] when the export directory cannot be created
/// or the file cannot be written.
pub fn export_report(
    report: &DriftReport,
    directory: &Path,
    timestamp: &str,
) -> std::io::Result<std::path::PathBuf> {
    std::fs::create_dir_all(directory)?;
    let artefact = serde_json::json!({
        "schema_version": EVENT_SCHEMA_VERSION,
        "kind": "azusa.drift-report",
        "timestamp": timestamp,
        "report": report,
    });
    let stamp = timestamp.replace([':', '-'], "");
    let path = directory.join(format!(
        "drift-{}-{}.json",
        stamp,
        report.state_a.digest.chars().take(8).collect::<String>()
    ));
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&artefact).unwrap_or_default(),
    )?;
    Ok(path)
}

/// Write a canonical export artefact for a reconciliation plan.
///
/// The artefact embeds the plan plus an export envelope. Returns the
/// written path.
///
/// # Errors
///
/// Returns [`std::io::Error`] when the export directory cannot be created
/// or the file cannot be written.
pub fn export_plan(
    plan: &azusa_model::ReconciliationPlan,
    directory: &Path,
    timestamp: &str,
) -> std::io::Result<std::path::PathBuf> {
    std::fs::create_dir_all(directory)?;
    let artefact = serde_json::json!({
        "schema_version": EVENT_SCHEMA_VERSION,
        "kind": "azusa.reconciliation-plan",
        "timestamp": timestamp,
        "plan": plan,
    });
    let stamp = timestamp.replace([':', '-'], "");
    let slug = format!("{}-{}", plan.repository, plan.direction);
    let path = directory.join(format!("plan-{stamp}-{slug}.json"));
    std::fs::write(
        &path,
        serde_json::to_string_pretty(&artefact).unwrap_or_default(),
    )?;
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use azusa_model::{DriftScore, DriftSummary};

    #[test]
    fn rfc3339_formatting_is_correct() {
        assert_eq!(format_unix_as_rfc3339(0), "1970-01-01T00:00:00Z");
        assert_eq!(format_unix_as_rfc3339(1), "1970-01-01T00:00:01Z");
        // 2026-09-11T00:00:00Z
        assert_eq!(
            format_unix_as_rfc3339(1_789_084_800),
            "2026-09-11T00:00:00Z"
        );
        // Leap-year day: 2024-02-29T12:00:00Z
        assert_eq!(
            format_unix_as_rfc3339(1_709_208_000),
            "2024-02-29T12:00:00Z"
        );
    }

    #[test]
    fn event_serialises_to_one_line() {
        let event = Event {
            schema_version: 1,
            event: events::DRIFT_DETECTED.to_owned(),
            timestamp: "2026-09-11T00:00:00Z".into(),
            repository: "azusa".into(),
            state_a: "worktree".into(),
            state_b: "upstream".into(),
            severity: "medium".into(),
            score: 4.2,
            evidence: 3,
            tool_version: "0.1.0".into(),
            analysis_mode: "deterministic".into(),
        };
        let line = event.to_ndjson();
        assert!(!line.contains('\n'));
        let parsed: Event = serde_json::from_str(&line).expect("roundtrip");
        assert_eq!(parsed, event);
    }

    #[test]
    fn emitter_writes_ndjson_lines() {
        let dir = tempfile::tempdir().expect("tempdir");
        let emitter = EventEmitter::to_file(dir.path().join("events.jsonl"));
        let event = Event {
            schema_version: 1,
            event: events::POLICY_FAILED.to_owned(),
            timestamp: "2026-09-11T00:00:00Z".into(),
            repository: "azusa".into(),
            state_a: "worktree".into(),
            state_b: "upstream".into(),
            severity: "critical".into(),
            score: 8.0,
            evidence: 1,
            tool_version: "0.1.0".into(),
            analysis_mode: "deterministic".into(),
        };
        emitter.emit(&event);
        emitter.emit(&event);
        let content = std::fs::read_to_string(dir.path().join("events.jsonl")).expect("read");
        assert_eq!(content.lines().count(), 2);
        assert!(content.contains("azusa.policy.failed"));
    }

    #[test]
    fn export_writes_canonical_artefact() {
        let dir = tempfile::tempdir().expect("tempdir");
        let report = minimal_report();
        let path = export_report(&report, &dir.path().join("exports"), "2026-09-11T01:02:03Z")
            .expect("export");
        let content = std::fs::read_to_string(&path).expect("read");
        let parsed: serde_json::Value = serde_json::from_str(&content).expect("json");
        assert_eq!(parsed["kind"], "azusa.drift-report");
        assert_eq!(parsed["report"]["repository"]["name"], "fixture");
        assert!(
            path.file_name()
                .and_then(|n| n.to_str())
                .is_some_and(|n| n.starts_with("drift-20260911T"))
        );
    }

    fn minimal_report() -> DriftReport {
        DriftReport {
            schema_version: 1,
            repository: azusa_model::RepositoryInfo {
                name: "fixture".into(),
                root: None,
            },
            state_a: azusa_model::StateRef {
                spec: "worktree".into(),
                provider: azusa_model::ProviderKind::Git,
                label: "worktree".into(),
                digest: "deadbeef".into(),
            },
            state_b: azusa_model::StateRef {
                spec: "upstream".into(),
                provider: azusa_model::ProviderKind::Git,
                label: "upstream".into(),
                digest: "cafebabe".into(),
            },
            direction: azusa_model::DriftDirection::AToB,
            direction_confidence: 0.9,
            score: DriftScore::default(),
            risk: azusa_model::DriftRisk {
                level: RiskLevel::Low,
                rationale: String::new(),
                factors: Vec::new(),
            },
            summary: DriftSummary::default(),
            observations: Vec::new(),
            intent: None,
            clusters: Vec::new(),
            recommendations: Vec::new(),
            provenance: azusa_model::Provenance {
                tool_version: "test".into(),
                analysis_mode: azusa_model::AnalysisMode::Deterministic,
                captured_at: None,
                config_digest: None,
            },
        }
    }
}

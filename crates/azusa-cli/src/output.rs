//! Output rendering: human, JSON, NDJSON, quiet.
//!
//! JSON is the machine contract; human output is presentation only. Every
//! renderer consumes the same [`DriftReport`] / [`RepositoryState`] data, so
//! the two can never disagree about facts.

use azusa_model::{DriftReport, Layer, RepositoryState, RiskLevel};

/// Output format options shared by every command.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, clap::Args)]
pub struct OutputOpts {
    /// Emit machine-readable JSON instead of human output.
    #[arg(long, global = true, conflicts_with = "ndjson")]
    pub json: bool,

    /// Emit newline-delimited JSON (one record per line).
    #[arg(long, global = true)]
    pub ndjson: bool,

    /// Suppress output; communicate through exit codes only.
    #[arg(long, global = true, conflicts_with_all = ["json", "ndjson"])]
    pub quiet: bool,
}

impl OutputOpts {
    /// Whether any machine format was requested.
    #[must_use]
    pub fn machine(&self) -> bool {
        self.json || self.ndjson
    }
}

/// Render a drift report in the requested format.
pub fn render_report(report: &DriftReport, opts: &OutputOpts) -> String {
    if opts.ndjson {
        return report_to_ndjson(report);
    }
    if opts.json {
        return serde_json::to_string_pretty(report).unwrap_or_default();
    }
    report_to_human(report)
}

/// Exit code for a report: 0 only when the states are in sync; otherwise
/// drift detected maps to the configured medium code (default 1) and
/// high/critical risk to the high code (default 2).
#[must_use]
pub fn exit_code(report: &DriftReport, thresholds: &azusa_core::ExitThresholds) -> i32 {
    match report.risk.level {
        RiskLevel::Low => {
            if report.direction == azusa_model::DriftDirection::InSync {
                0
            } else {
                thresholds.medium
            }
        }
        RiskLevel::Medium => thresholds.medium,
        RiskLevel::High => thresholds.high,
        RiskLevel::Critical => thresholds.critical,
    }
}

/// NDJSON form: one line per observation, then one summary line.
#[must_use]
pub fn report_to_ndjson(report: &DriftReport) -> String {
    let mut lines = Vec::new();
    for observation in &report.observations {
        let record = serde_json::json!({
            "record": "observation",
            "id": observation.id,
            "category": observation.category.as_str(),
            "severity": observation.severity.as_str(),
            "confidence": observation.confidence,
            "title": observation.title,
            "detail": observation.detail,
            "side": observation.side.as_str(),
            "evidence": observation.evidence,
            "repository": report.repository.name,
            "state_a": report.state_a.spec,
            "state_b": report.state_b.spec,
        });
        lines.push(record.to_string());
    }
    let summary = serde_json::json!({
        "record": "summary",
        "repository": report.repository.name,
        "state_a": report.state_a.spec,
        "state_b": report.state_b.spec,
        "direction": report.direction.as_str(),
        "direction_confidence": report.direction_confidence,
        "score": report.score,
        "risk": report.risk.level.as_str(),
        "summary": report.summary,
        "schema_version": report.schema_version,
    });
    lines.push(summary.to_string());
    lines.join("\n")
}

/// Human-readable drift report.
///
/// Linear section-by-section rendering; length is presentation breadth,
/// not complexity.
#[allow(clippy::too_many_lines)]
#[must_use]
pub fn report_to_human(report: &DriftReport) -> String {
    let mut out = String::new();
    let rule = "─".repeat(60);
    out.push_str(&format!("AZUSA — {}\n{rule}\n", report.repository.name));
    out.push_str(&format!(
        "A: {:<12} {} ({})\n",
        report.state_a.spec,
        truncate(&report.state_a.digest),
        report.state_a.label
    ));
    out.push_str(&format!(
        "B: {:<12} {} ({})\n",
        report.state_b.spec,
        truncate(&report.state_b.digest),
        report.state_b.label
    ));
    out.push('\n');
    out.push_str(&format!("DRIFT        {:.1} / 10\n", report.score.overall));
    out.push_str(&format!(
        "DIRECTION    {} (confidence {:.2})\n",
        report.direction.as_str().to_uppercase(),
        report.direction_confidence
    ));
    out.push_str(&format!(
        "RISK         {}\n",
        report.risk.level.as_str().to_uppercase()
    ));
    out.push('\n');
    out.push_str(&format!(
        "FILES        +{} added  -{} deleted  ~{} modified  <>{} renamed\n",
        report.summary.files_added,
        report.summary.files_deleted,
        report.summary.files_modified,
        report.summary.files_renamed
    ));
    if report.summary.commits_ahead.is_some() || report.summary.commits_behind.is_some() {
        out.push_str(&format!(
            "COMMITS      +{} ahead  -{} behind\n",
            report.summary.commits_ahead.unwrap_or(0),
            report.summary.commits_behind.unwrap_or(0)
        ));
    }
    if report.summary.uncommitted_a || report.summary.uncommitted_b {
        out.push_str(&format!(
            "UNCOMMITTED  A: {}  B: {}\n",
            yes_no(report.summary.uncommitted_a),
            yes_no(report.summary.uncommitted_b)
        ));
    }

    if !report.score.layers.is_empty() {
        out.push_str("\nLayers\n");
        for layer in Layer::all() {
            if let Some(score) = report.score.layers.get(&layer) {
                out.push_str(&format!("  {:<14} {:>5.2}\n", layer.as_str(), score));
            }
        }
    }

    if !report.clusters.is_empty() {
        out.push_str("\nChange clusters\n");
        for cluster in report.clusters.iter().take(10) {
            out.push_str(&format!(
                "  {:<24} {:<14} {} file(s)\n",
                cluster.label,
                cluster.kind.as_str(),
                cluster.files
            ));
            for path in cluster.paths.iter().take(3) {
                out.push_str(&format!("      {path}\n"));
            }
        }
        if report.clusters.len() > 10 {
            out.push_str(&format!(
                "      … and {} more clusters\n",
                report.clusters.len() - 10
            ));
        }
    }

    if !report.observations.is_empty() {
        out.push_str("\nObservations\n");
        for observation in &report.observations {
            out.push_str(&format!(
                "  {} [{:<8}] {}: {}\n",
                observation.id,
                observation.severity.as_str(),
                observation.category.as_str(),
                observation.title
            ));
            for evidence in &observation.evidence {
                out.push_str(&format!(
                    "      evidence: {} ({})\n",
                    evidence.description,
                    evidence.kind.as_str()
                ));
                for reference in evidence.references.iter().take(4) {
                    out.push_str(&format!("        {reference}\n"));
                }
            }
        }
    }

    if !report.recommendations.is_empty() {
        out.push_str("\nRecommended\n");
        for (index, recommendation) in report.recommendations.iter().enumerate() {
            out.push_str(&format!(
                "  {}. {} ({})\n",
                index + 1,
                recommendation.action,
                recommendation.priority.as_str()
            ));
        }
    }
    out.push_str(&format!(
        "\nanalysis_mode: {}\n",
        report.provenance.analysis_mode.as_str()
    ));
    out
}

/// Render a repository state summary (for `azusa status`).
#[must_use]
pub fn render_state(state: &RepositoryState, opts: &OutputOpts) -> String {
    if opts.json {
        return serde_json::to_string_pretty(state).unwrap_or_default();
    }
    let mut out = String::new();
    out.push_str(&format!(
        "state:       {} ({})\n",
        state.identity.label,
        state.identity.provider.as_str()
    ));
    if let Some(root) = &state.identity.root {
        out.push_str(&format!("root:        {root}\n"));
    }
    if let Some(vcs) = &state.identity.vcs {
        out.push_str(&format!(
            "branch:      {}{}\n",
            vcs.branch.as_deref().unwrap_or("(detached)"),
            if vcs.detached { " (detached)" } else { "" }
        ));
        if let Some(head) = &vcs.head_oid {
            out.push_str(&format!("head:        {}\n", truncate(head)));
        }
        if let Some(upstream) = &vcs.upstream {
            out.push_str(&format!("upstream:    {upstream}\n"));
        }
        out.push_str(&format!("remotes:     {}\n", vcs.remote_urls.len()));
    }
    out.push_str(&format!("files:       {}\n", state.identity.file_count));
    out.push_str(&format!("bytes:       {}\n", state.identity.total_bytes));
    out.push_str(&format!(
        "digest:      {}\n",
        truncate(&state.identity.digest)
    ));
    if let Some(uncommitted) = &state.ancestry.uncommitted {
        if uncommitted.is_clean() {
            out.push_str("uncommitted: clean\n");
        } else {
            out.push_str(&format!(
                "uncommitted: {} staged, {} unstaged, {} untracked\n",
                uncommitted.staged_files, uncommitted.unstaged_files, uncommitted.untracked_files
            ));
        }
    }
    if let Some(base) = &state.ancestry.merge_base {
        out.push_str(&format!("merge-base:  {}\n", truncate(base)));
    }
    out
}

fn yes_no(value: bool) -> &'static str {
    if value { "yes" } else { "no" }
}

fn truncate(value: &str) -> String {
    value.chars().take(12).collect()
}

// Copyright (c) 2026 The azusa Authors
// SPDX-License-Identifier: MIT

//! End-to-end CLI tests.
//!
//! Fixtures are real Git repositories built with the `git` binary (test code
//! only — the azusa binary itself never shells out to git).

use std::process::Command;

use assert_cmd::prelude::*;
use predicates::str::contains;
use tempfile::TempDir;

/// A throwaway git repository with an upstream-tracking branch.
struct Fixture {
    _guard: TempDir,
    path: std::path::PathBuf,
}

impl Fixture {
    fn new() -> Self {
        let guard = tempfile::tempdir().expect("tempdir");
        let path = guard.path().join("repo");
        std::fs::create_dir_all(&path).expect("mkdir");
        let fixture = Self {
            _guard: guard,
            path,
        };
        fixture.git(&["init", "-b", "main"]);
        fixture.git(&["config", "user.email", "test@azusa.local"]);
        fixture.git(&["config", "user.name", "Azusa Test"]);
        std::fs::write(fixture.path.join("README.md"), "# fixture\n").expect("write");
        std::fs::create_dir_all(fixture.path.join("src")).expect("mkdir");
        std::fs::write(fixture.path.join("src/main.rs"), "fn main() {}\n").expect("write");
        fixture.git(&["add", "."]);
        fixture.git(&["commit", "-m", "chore: initial state"]);
        fixture.git(&["init", "--bare", "../origin.git"]);
        fixture.git(&["remote", "add", "origin", "../origin.git"]);
        fixture.git(&["push", "-u", "origin", "main"]);
        fixture
    }

    fn git(&self, args: &[&str]) -> String {
        let output = Command::new("git")
            .args(args)
            .current_dir(&self.path)
            .output()
            .expect("spawn git");
        assert!(
            output.status.success(),
            "git {:?} failed: {}{}",
            args,
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8_lossy(&output.stdout).into_owned()
    }
}

fn azusa() -> Command {
    Command::cargo_bin("azusa").expect("locate azusa binary")
}

#[test]
fn version_prints() {
    azusa()
        .arg("--version")
        .assert()
        .success()
        .stdout(contains("azusa"));
}

#[test]
fn clean_repository_reports_in_sync_with_exit_zero() {
    let fixture = Fixture::new();
    azusa()
        .arg("drift")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .stdout(contains("\"direction\": \"in-sync\""));
}

#[test]
fn dirty_worktree_reports_drift_with_exit_one() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path.join("notes.txt"), "uncommitted\n").expect("write");
    azusa()
        .arg("drift")
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .stdout(contains("A-TO-B"))
        .stdout(contains("uncommitted drift present"));
}

#[test]
fn fast_path_equals_drift() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path.join("notes.txt"), "uncommitted\n").expect("write");
    azusa()
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .stdout(contains("DRIFT"));
}

#[test]
fn local_commit_is_atob_and_scores() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path.join("feature.rs"), "pub fn feature() {}\n").expect("write");
    fixture.git(&["add", "."]);
    fixture.git(&["commit", "-m", "feat: feature"]);
    azusa()
        .arg("score")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .stdout(contains("\"overall\""));
}

#[test]
fn remote_ahead_is_btoa() {
    let fixture = Fixture::new();
    // Simulate fetched remote work by advancing the tracking ref.
    std::fs::write(fixture.path.join("remote-work.rs"), "pub fn remote() {}\n").expect("write");
    fixture.git(&["add", "."]);
    fixture.git(&["commit", "-m", "feat: remote work"]);
    let head = fixture.git(&["rev-parse", "HEAD"]).trim().to_owned();
    fixture.git(&["reset", "--keep", "HEAD~1"]);
    fixture.git(&["update-ref", "refs/remotes/origin/main", &head]);
    std::fs::remove_file(fixture.path.join("remote-work.rs")).ok();

    azusa()
        .arg("drift")
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .stdout(contains("B-TO-A"))
        .stdout(contains("1 remote commit"));
}

#[test]
fn unknown_state_is_analysis_failure() {
    let fixture = Fixture::new();
    azusa()
        .arg("drift")
        .arg("no-such-ref")
        .current_dir(&fixture.path)
        .assert()
        .code(4);
}

#[test]
fn unknown_flag_is_invalid_invocation() {
    let fixture = Fixture::new();
    azusa()
        .arg("--bogus-flag")
        .current_dir(&fixture.path)
        .assert()
        .code(5);
}

#[test]
fn status_and_inspect_emit_valid_json() {
    let fixture = Fixture::new();
    azusa()
        .arg("status")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0);

    let output = azusa()
        .arg("inspect")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("inspect JSON");
    assert!(parsed["identity"]["vcs"]["branch"].is_string());
    assert!(parsed["filesystem"]["files"].is_object());
}

#[test]
fn ndjson_emits_records_and_summary() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path.join("notes.txt"), "uncommitted\n").expect("write");
    let output = azusa()
        .arg("drift")
        .arg("--ndjson")
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let text = String::from_utf8(output).expect("utf8");
    let lines: Vec<&str> = text.lines().filter(|l| !l.is_empty()).collect();
    assert!(
        lines.len() >= 2,
        "expected observations + summary, got {lines:?}"
    );
    let last: serde_json::Value =
        serde_json::from_str(lines[lines.len() - 1]).expect("summary line");
    assert_eq!(last["record"], "summary");
    assert_eq!(last["direction"], "a-to-b");
}

#[test]
fn fs_provider_compares_two_directories() {
    let dir = tempfile::tempdir().expect("tempdir");
    let a = dir.path().join("a");
    let b = dir.path().join("b");
    std::fs::create_dir_all(&a).expect("mkdir");
    std::fs::create_dir_all(&b).expect("mkdir");
    std::fs::write(a.join("shared.txt"), "same\n").expect("write");
    std::fs::write(b.join("shared.txt"), "same\n").expect("write");
    std::fs::write(a.join("only-a.txt"), "a\n").expect("write");
    std::fs::write(b.join("only-b.txt"), "b\n").expect("write");

    let a_str = a.to_string_lossy().to_string();
    let b_str = b.to_string_lossy().to_string();
    azusa()
        .arg("drift")
        .arg(&a_str)
        .arg(&b_str)
        .arg("--json")
        .current_dir(dir.path())
        .assert()
        .code(1)
        .stdout(contains("\"direction\": \"bidirectional\""));
}

#[test]
fn quiet_mode_prints_nothing() {
    let fixture = Fixture::new();
    let output = azusa()
        .arg("drift")
        .arg("--quiet")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    assert!(output.is_empty(), "expected no stdout, got {output:?}");
}

#[test]
fn help_exits_zero() {
    azusa().arg("--help").assert().code(0);
}

#[test]
fn policy_check_exits_three_on_violation() {
    let fixture = Fixture::new();
    // Commit local work, then set a policy the drift violates.
    std::fs::write(fixture.path.join("feature.rs"), "pub fn feature() {}\n").expect("write");
    fixture.git(&["add", "."]);
    fixture.git(&["commit", "-m", "feat: feature"]);
    std::fs::write(
        fixture.path.join("azusa.toml"),
        "[policy]\nmax_drift = 0.1\n",
    )
    .expect("write");

    azusa()
        .arg("policy")
        .arg("check")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(3)
        .stdout(contains("\"rule\": \"policy.max_drift\""));

    let events =
        std::fs::read_to_string(fixture.path.join(".azusa/events.jsonl")).expect("events log");
    assert!(
        events.contains("\"azusa.policy.failed\""),
        "policy failure should be emitted: {events}"
    );
}

#[test]
fn policy_check_passes_with_generous_limits() {
    let fixture = Fixture::new();
    azusa()
        .arg("policy")
        .arg("check")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .stdout(contains("compliant"));
}

#[test]
fn export_writes_artefact_and_events() {
    let fixture = Fixture::new();
    let output = azusa()
        .arg("export")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    let export_path = std::path::PathBuf::from(parsed["export"].as_str().expect("export path"));
    assert!(export_path.is_file(), "artefact missing: {export_path:?}");
    let events_path = std::path::PathBuf::from(parsed["events"].as_str().expect("events path"));
    assert!(events_path.is_file());
}

#[test]
fn drift_records_history() {
    let fixture = Fixture::new();
    azusa()
        .arg("drift")
        .arg("--quiet")
        .current_dir(&fixture.path)
        .assert()
        .code(0);
    let history = fixture.path.join(".azusa/history.jsonl");
    assert!(history.is_file());
}

#[test]
fn plan_exports_artefact_and_emits_event() {
    let fixture = Fixture::new();
    let output = azusa()
        .arg("plan")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    let plan = &parsed["plan"];
    assert_eq!(plan["schema_version"], 1);
    assert!(
        plan["steps"]
            .as_array()
            .expect("steps")
            .iter()
            .any(|s| s["kind"] == "noop")
    );
    let export_path = std::path::PathBuf::from(parsed["export"].as_str().expect("export path"));
    assert!(
        export_path.is_file(),
        "plan artefact missing: {export_path:?}"
    );
    let events =
        std::fs::read_to_string(fixture.path.join(".azusa/events.jsonl")).expect("events log");
    assert!(
        events.contains("\"azusa.plan.generated\""),
        "plan event should be emitted: {events}"
    );
}

#[test]
fn plan_is_blocked_by_policy_violation() {
    let fixture = Fixture::new();
    std::fs::write(fixture.path.join("feature.rs"), "pub fn feature() {}\n").expect("write");
    fixture.git(&["add", "."]);
    fixture.git(&["commit", "-m", "feat: feature"]);
    std::fs::write(
        fixture.path.join("azusa.toml"),
        "[policy]\nmax_drift = 0.1\n",
    )
    .expect("write");

    let output = azusa()
        .arg("plan")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    let gates = parsed["plan"]["gates"].as_array().expect("gates");
    assert!(
        gates
            .iter()
            .any(|g| g["blocking"] == true
                && g["condition"].as_str().expect("cond").contains("policy")),
        "policy gate should block: {gates:?}"
    );
}

#[test]
fn watch_first_cycle_analyses_and_persists_state() {
    let fixture = Fixture::new();
    let output = azusa()
        .arg("watch")
        .arg("--max-cycles")
        .arg("1")
        .arg("--interval")
        .arg("0")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    assert_eq!(parsed["changed"], true);
    assert_eq!(parsed["stage"], 1);
    assert!(fixture.path.join(".azusa/watch.json").is_file());
    let history =
        std::fs::read_to_string(fixture.path.join(".azusa/history.jsonl")).expect("history");
    assert_eq!(
        history.lines().count(),
        1,
        "one analysis should append one history record"
    );
}

#[test]
fn watch_idle_cycle_skips_analysis() {
    let fixture = Fixture::new();
    for _ in 0..2 {
        azusa()
            .arg("watch")
            .arg("--max-cycles")
            .arg("1")
            .arg("--interval")
            .arg("0")
            .arg("--json")
            .current_dir(&fixture.path)
            .assert()
            .code(0);
    }
    let output = azusa()
        .arg("watch")
        .arg("--max-cycles")
        .arg("1")
        .arg("--interval")
        .arg("0")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    assert_eq!(parsed["changed"], false);
    assert_eq!(parsed["stage"], 0, "idle cycle de-escalates to stage 0");
    let history =
        std::fs::read_to_string(fixture.path.join(".azusa/history.jsonl")).expect("history");
    assert_eq!(
        history.lines().count(),
        1,
        "idle cycle must not append history"
    );
}

#[test]
fn watch_escalates_and_generates_plan_on_growth() {
    let fixture = Fixture::new();
    azusa()
        .arg("watch")
        .arg("--max-cycles")
        .arg("1")
        .arg("--interval")
        .arg("0")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(0);

    // Local uncommitted work: drift grows from the idle baseline.
    std::fs::write(fixture.path.join("dirty.rs"), "pub fn dirty() {}\n").expect("write");
    let output = azusa()
        .arg("watch")
        .arg("--max-cycles")
        .arg("1")
        .arg("--interval")
        .arg("0")
        .arg("--json")
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .get_output()
        .stdout
        .clone();
    let parsed: serde_json::Value = serde_json::from_slice(&output).expect("json");
    assert_eq!(
        parsed["stage"], 2,
        "score growth should escalate to stage 2"
    );
    let plan_path = std::path::PathBuf::from(parsed["plan"].as_str().expect("plan path"));
    assert!(plan_path.is_file(), "escalated cycle should export a plan");
    let events =
        std::fs::read_to_string(fixture.path.join(".azusa/events.jsonl")).expect("events log");
    assert!(
        events.contains("\"azusa.plan.generated\""),
        "escalation should emit the plan event: {events}"
    );
}

#[test]
fn intent_is_reported_for_structural_drift() {
    let fixture = Fixture::new();
    // Feature-branch-style work: new module + tests, committed.
    fixture.git(&["checkout", "-b", "feature/engine"]);
    std::fs::create_dir_all(fixture.path.join("crates/engine/src")).expect("mkdir");
    std::fs::write(
        fixture.path.join("crates/engine/Cargo.toml"),
        "[package]\nname = \"engine\"\nversion = \"0.1.0\"\n",
    )
    .expect("write");
    std::fs::write(
        fixture.path.join("crates/engine/src/lib.rs"),
        "pub fn e() {}\n",
    )
    .expect("write");
    std::fs::write(
        fixture.path.join("Cargo.toml"),
        "[workspace]\nmembers = [\"crates/engine\"]\n",
    )
    .expect("write");
    fixture.git(&["add", "."]);
    fixture.git(&["commit", "-m", "feat: engine crate"]);

    azusa()
        .arg("explain")
        .current_dir(&fixture.path)
        .assert()
        .code(1)
        .stdout(contains("INTENT"))
        .stdout(contains("feature/engine"));
}

#[test]
fn works_from_subdirectory_of_repository() {
    let fixture = Fixture::new();
    let sub = fixture.path.join("src");
    azusa()
        .arg("status")
        .current_dir(&sub)
        .assert()
        .code(0)
        .stdout(contains("branch:      main"));
}

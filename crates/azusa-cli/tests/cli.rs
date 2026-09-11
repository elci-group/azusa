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
    assert!(output.is_empty());
}

#[test]
fn help_exits_zero() {
    azusa().arg("--help").assert().code(0);
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

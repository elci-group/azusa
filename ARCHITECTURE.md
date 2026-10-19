# Azusa Architecture

Azusa models repositories as **states**, not Git trees, and comparison as a
**layered engine** whose output is an evidence-first drift report.

## Crates

| Crate | Responsibility |
| --- | --- |
| `azusa-model` | Domain types: `RepositoryState`, `DriftReport`, observations, scores. No I/O. |
| `azusa-git` | Git state provider (worktree / index / HEAD / upstream / remote refs) via libgit2. |
| `azusa-fs` | Filesystem state provider: ignore-aware walk, content hashing, incremental cache. |
| `azusa-diff` | Fileset diff, rename detection, line deltas, change clustering. |
| `azusa-graph` | Structural analysis: module topology, dependency graphs, config/behaviour inventory. |
| `azusa-core` | Comparison engine: layers L0–L9, scoring, direction, intent, risk, plans, config. |
| `azusa-policy` | Policy engine: drift limits checked by `azusa policy check` (CI gate). |
| `azusa-elci` | ELCI universal event contract, observation schema, export artefacts. |
| `azusa-store` | Local append-only evidence store (`.azusa/`): report history, state digests. |
| `azusa-cli` | The `azusa` binary. |

## Comparison layers

```text
L0 Identity        same repository? fork? unrelated?
L1 Ancestry        merge-base, ahead/behind, committed vs uncommitted
L2 Filesystem      files added / deleted / modified / renamed
L3 Structure       modules, crates, directory topology
L4 Dependencies    manifest + lockfile divergence
L5 Configuration   project configuration files
L6 Interfaces      public API surface
L7 Behaviour       tests, build scripts, entry points
L8 Semantics       security-/behaviour-critical regions (heuristic, evidence-first)
L9 Operations      deployment and service configuration
```

Each layer emits `DriftObservation`s carrying category, severity, confidence
and concrete evidence. Scores never replace evidence: the composite drift
index is always accompanied by the observations that produced it.

## Determinism

Given the same states, configuration and tool version, azusa produces the
same structural result. Scoring uses fixed saturating formulas over
observation counts (`10·c/(c+k)` per layer, constants in `azusa-core`'s
scoring module, each documented and unit-tested):

- **ancestry** — `c = commits_ahead + commits_behind + 0.5·uncommitted`, k=12
- **filesystem** — `c = added + modified + 2·deleted + 0.5·renamed`, k=40
- composite — weighted mean over **analyzed** layers only (empty facets are
  excluded, never silently zero)

Timestamps appear only in provenance, never inside the deterministic
analysis path.

## Structural layers (L3–L5)

`azusa-graph::enrich` fills the structure, dependency and configuration
facets from the filesystem listing and `captured_text` (retained UTF-8 of
analysis-interest files, ≤128 KiB each):

- **L3 structure** — Cargo manifests → modules (workspace / library /
  binary, refined by `src/lib.rs` autodetection), directory topology.
- **L4 dependencies** — declared deps per manifest with lockfile-resolved
  versions; version transitions classified major/minor/patch; lockfile
  divergence tracked separately.
- **L5 configuration** — config files classified by kind
  (ci-workflow, dockerfile, kubernetes, systemd-unit, compose, formatter,
  linter, environment, cargo-manifest); kind weights severity
  (deployment-config changes outweigh formatter tweaks).

Layer scores: structure `10·c/(c+10)` with `c = added + 2·removed +
0.2·dir-churn`; dependency `10·c/(c+8)` with `c = 3·major + 2·minor +
1·patch + 1.5·added-or-removed + 1·lockfile-diverged`; configuration
`10·c/(c+6)` with per-kind weights (2.5 deploy/CI … 0.5 formatting).

## Behavioural layers (L6–L7)

Providers capture the behavioural surface alongside configuration:
`build.rs`, library and binary roots (`src/lib.rs`, `src/main.rs`,
`src/bin/*.rs`), integration tests (`tests/*.rs`) and benches
(`benches/*.rs`). `azusa-graph::behaviour` derives the facet as a pure
function of the listing plus retained text:

- **path classification** — test, bench, entry-point and build-script files;
- **text signals** — `#[test]` declaration counts, files carrying a
  `#[cfg(test)]` module, and per-file public item counts
  (`pub fn/struct/enum/trait/mod/const/static/type/use`).

`azusa-core::behaviour` compares facets:

- **L6 interfaces** — files and per-file public item counts that exist on
  only one side. Severity is direction-aware: items on the side that
  reconciliation would overwrite (behind the drift source) would vanish
  from the interface and rate medium (high at ≥5 items); items on the
  drift source's side rate low. Bidirectional drift rates both sides
  medium; unknown direction rates low. Score: `10·c/(c+6)` with
  `c = 2·removed-files + added-files + 2·items-only-in-A +
  items-only-in-B`.
- **L7 behaviour** — removed tests rate medium (high at ≥3 files), removed
  `#[test]` declarations medium, entry-point removals medium, changed
  build scripts medium (they execute at compile time); additions and
  benchmark churn rate low. Score: `10·c/(c+8)` with `c = 1.5·tests-
  removed + tests-added + 0.2·declarations-removed + 0.1·added +
  2·entry-removals + entry-additions + 2·build-script-changes +
  0.5·bench-churn`.

Both layers stay silent when both facets are empty (not analyzed) and are
excluded from the composite rather than counted as zero drift.

## Operational layer (L9)

`azusa-graph::enrich` derives the operations facet from the configuration
facet: every configuration file whose kind is operational (CI workflow,
Dockerfile/Containerfile, kubernetes manifest, systemd unit, compose stack)
becomes an *operational unit* carrying path, kind and content digest.

`azusa-core::structural::compare_operations` frames the comparison as
deployment topology: units present on only one side (side-attributed in
the title) and units whose digest or kind changed (reconfigured).
Side-attributed units follow [`side_severity`](#behavioural-layers-l6l7):
units on the side reconciliation would overwrite rate medium (high at
≥5), units on the drift source's side rate low, bidirectional rates both
medium. Reconfigured units rate medium — the running infrastructure may
diverge from the repository — high at ≥3. Score: `10·c/(c+5)` with
`c = 2·removed + 2·reconfigured + 1.5·added`. Like all layers, silence on
empty facets means "not analyzed", not zero.

## Reconciliation planning

`azusa-core::plan::generate` derives a [`ReconciliationPlan`] from a
finished report plus the policy verdict — a pure function, deterministic
like the report it plans from. Steps follow the drift direction:
in-sync → no-op; a-to-b → (commit if uncommitted) → (review per cluster)
→ (run-tests if behaviour/interface moved) → push; b-to-a → fetch →
fast-forward (approval required when it would delete files);
bidirectional → fetch → per-cluster human review → merge → push;
unrelated/unknown → manual inspection only.

Safety is structural, not advisory: steps that rewrite history or publish
(`merge`, `rebase`, `push`) always carry `requires_approval`. Blocking
gates: policy violations, high/critical risk, any bidirectional drift,
unrelated states. Advisory gates: accidental-intent drift. `azusa plan`
exports the plan as a canonical artefact and emits `azusa.plan.generated`.
Azusa plans; Kaptaind executes — the planner never mutates anything.

## Watch mode (continuous drift)

`azusa watch` loops a staged escalation ladder, persisted in
`.azusa/watch.json` (`azusa_store::WatchState`):

- **stage 0** — a stat-only worktree fingerprint
  (`azusa_fs::stat_digest`: BLAKE3 over sorted `(path, size, mtime)`
  triples) matches the last full analysis: no analysis, no history churn;
  streaks reset.
- **stage 1** — the fingerprint differs: run the full deterministic
  pipeline (history + ELCI events via the normal `drift` path).
- **stage 2** — the score grew past the previous run or risk is
  high/critical: additionally generate a reconciliation plan (policy
  verdict included) and emit `azusa.plan.generated`.

Escalation is sticky while drift persists (streak counted), idle cycles
de-escalate to stage 0, and the state file lets a restarted watcher resume
without re-analysing unchanged trees. Determinism holds per cycle: mtimes
drive only the change detector, never the analysis path, and every
analysis run is the same pure pipeline as a one-shot `azusa drift`.

## Change clustering

`azusa-diff::cluster` assigns every changed path to a labeled cluster
(lockfiles, manifests, tests, docs, operations, configuration,
`crates/<name>`, top-level module). Clusters are part of the report
artefact and power `azusa explain`.

## Intent assessment

`azusa-core::intent` estimates whether drift looks deliberate from
deterministic signals only: cluster composition, generated-content markers
in retained text (`@generated` / `DO NOT EDIT` in the first lines), branch
naming conventions on side A, and uncommitted hygiene. Signals carry fixed
weights; the normalised sum maps onto five classes (`intentional`,
`probably-intentional`, `indeterminate`, `probably-accidental`,
`accidental`) with a confidence derived from signal count. No signals or
in-sync states produce no assessment at all — "indeterminate" is a
first-class answer, never a forced choice.

## Policy engine

`azusa-policy` checks a finished `DriftReport` against the `[policy]` block
of `azusa.toml` (`max_drift`, dependency, structure and configuration
rules). It is an evidence check over the report — it never re-analyses
states, so a policy verdict is as deterministic as the report it audits.
Violations carry stable ids (`POLICY-0001`…), severities and references to
the offending observations; `azusa policy check` exits 3 on any violation
and emits `azusa.policy.failed`.

## Evidence store and ELCI events

`azusa-store` maintains an append-only trail under `.azusa/`:
`history.jsonl` (one record per analysis run), `events.jsonl` (ELCI event
log) and `exports/` (canonical report artefacts written by `azusa export`).
`azusa-elci` defines the event contract (`schema_version 1`): `Event`
records with event name, RFC 3339 timestamp, repository, state selectors,
severity, score, evidence count and tool version, serialised as NDJSON.
Timestamps live in the event/provenance layer only; the deterministic
analysis path never reads the store. All store writes are best-effort —
analysis never fails because the store is unwritable.

## State resolution

`worktree` · `index` · `head` · `upstream` · `remote` (origin/HEAD →
origin/main → origin/master) · any libgit2 revision · any directory path.
The default pair is `worktree` vs `upstream` with documented fallbacks,
each carried into the report as an observation.

## Hashing and incrementality

File content is hashed with BLAKE3 (`azusa-model::hash`). Fileset digests
hash a canonical `<mode> <hash> <path>` listing, so two states are identical
iff their digests match. A stat-keyed cache (`(path, size, mtime) → hash`,
persisted at `.azusa/cache/fs-hashes.json`) makes repeat scans incremental;
mtime never enters the state itself. Worktree snapshots share one walker
(`azusa-fs`) across providers so ignore rules behave identically.

## Platform readiness

Azusa is Linux-first but Windows-portable at the source level:

- Every unix-only API is `#[cfg(unix)]`-gated with a `#[cfg(not(unix))]`
  fallback (executable-bit detection in `azusa-fs` degrades to `false` on
  Windows); symlink handling uses cross-platform std APIs.
- Filesystem-state keys are canonical repo-relative forward-slash paths on
  every host OS; lookups must be built through the same canonical join —
  never the host separator.
- Git mode strings (`100644`, `100755`, `120000`) are git's convention,
  emitted as a string contract — they describe analyzed content, not the
  host filesystem.
- `ConfigurationKind::SystemdUnit` (`systemd-unit`), `CiWorkflow`, and
  friends are content-classification vocabulary: they name file kinds
  found in *analyzed repositories*, not runtime services of the host.
  A Windows host classifying a Linux repository's systemd units is a
  supported, tested path.

Platform-portability scans (edwardian) flag the classifier vocabulary and
cfg-gated unix test code as blockers; those are scanner artifacts, not
runtime dependencies. A standalone Windows fork can be produced with
`edwardian release` when a Windows target is actually needed.

## Safety

Azusa observes, analyses and directs. It does not execute repository
mutations: reconciliation plans are handed to Kaptaind. Destructive
operations (force push, reset, branch deletion, history rewriting) are
refused unless explicitly authorised by policy.

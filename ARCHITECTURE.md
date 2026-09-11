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

## Change clustering

`azusa-diff::cluster` assigns every changed path to a labeled cluster
(lockfiles, manifests, tests, docs, operations, configuration,
`crates/<name>`, top-level module). Clusters are part of the report
artefact and power `azusa explain`.

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

## Safety

Azusa observes, analyses and directs. It does not execute repository
mutations: reconciliation plans are handed to Kaptaind. Destructive
operations (force push, reset, branch deletion, history rewriting) are
refused unless explicitly authorised by policy.

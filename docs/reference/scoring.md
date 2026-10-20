# Scoring Reference

Every number Azusa reports comes from a fixed, documented formula over
evidence counts. Scores are reproducible, monotonic in change volume, and
never depend on wall-clock time or environment. This page is the audit
trail for every constant. Implementation: `azusa-core::scoring` (unit tests
per formula).

## The saturating curve

```text
score(c, k) = 10 · c / (c + k)      (0 when c ≤ 0)
```

- `c` — a weighted change count for the layer (composition documented
  below).
- `k` — the layer's **saturation constant**: the count at which the layer
  scores 5.0. Smaller `k` ⇒ faster saturation ⇒ the same volume scores
  higher.
- Scores approach 10.0 asymptotically and never exceed it; monotonic in
  `c`.

Reference points (`10c/(c+k)`): at `c = k` the score is 5.0; at `c = 3k`
it is 7.5; at `c = k/4` it is 2.0.

## Per-layer formulas

### L1 Ancestry — k = 12

```text
c = commits_ahead + commits_behind + 0.5 · uncommitted_paths
```

Calibration: 3 novel commits ≈ 1.9; 12 ≈ 5.0; 36 ≈ 7.5.
Uncommitted work counts at half weight — it is real drift but not yet
landed. Rationale for the low default weight (0.5): ancestry volume is a
fact, but volume is not severity.

### L2 Filesystem — k = 40

```text
c = added + modified + 2 · deleted
      + Σ renames: exact-identity ×0.5 · content-similarity ×0.75
                 · path-heuristic       ×1.5
```

Calibration: 10 changed files ≈ 2.0; 40 ≈ 5.0; 120 ≈ 7.5.
Deletions weigh **double** — removals carry more risk than additions.
Rename weights follow evidence tier: an exact-identity move is cheap to
reconcile; a path-heuristic pairing is treated almost like a distinct
delete + add, because the association may simply be wrong. Severity of a
false rename attribution is worse than reporting two events.

### L3 Structure — k = 10

```text
c = added_modules + 2 · removed_modules + 0.2 · directory_churn
```

Module topology from manifests and the tree (`azusa-graph::enrich`).
Removals weigh double; directory churn (moves/renames of directories)
weighs lightly.

### L4 Dependency — k = 8

```text
c = 3 · major + 2 · minor + 1 · patch
      + 1.5 · added_or_removed + 1 · lockfile_diverged
```

Semver-aware version transitions (manifest declarations with
lockfile-resolved versions). Major jumps weigh triple; a diverged lockfile
is its own unit of concern.

### L5 Configuration — k = 6

```text
c = Σ per-kind weights over changed configuration files
```

Kind weights span **2.5** (deployment-facing: CI workflows, Dockerfiles,
kubernetes, systemd units, compose stacks) down to **0.5** (formatter
config). A deployment-config change outweighs a formatter tweak by 5×.

### L6 Interface — k = 6

```text
c = 2 · symbols_removed + 3 · signatures_changed + symbols_added
```

Public-API surface at **symbol identity** level (crate-relative fully
qualified name, item kind, normalized declaration signature; restricted
visibility excluded). A parameter-type change registers as
`signatures_changed` even when the symbol count is unchanged — hence its
triple weight. Severity is direction-aware (`side_severity`, below).

### L7 Behaviour — k = 8

```text
c = 1.5 · test_files_removed + 1 · test_files_added
      + 0.2 · test_declarations_removed + 0.1 · test_declarations_added
      + 2 · entry_point_removals + 1 · entry_point_additions
      + 2 · build_script_changes + 0.5 · benchmark_churn
```

The verification surface: tests, benches, executable entry points, build
scripts (`build.rs` executes at compile time — hence weight 2). This layer
inventories verification **infrastructure**; runtime-behaviour comparison
belongs to future L8. The JSON layer id stays `behaviour` for
compatibility.

### L9 Operations — k = 5

```text
c = 2 · units_removed + 2 · units_reconfigured + 1.5 · units_added
```

Deployment/service surface framed as *operational units* (CI workflows,
containers, kubernetes manifests, systemd units, compose stacks; each with
path, kind, content digest). Reconfigured units weigh full — the running
infrastructure may already diverge from the repository. Smallest `k` of
any layer: operational drift saturates fastest.

### L8 Semantics

Planned (security-critical regions). Reserved weight 2.0; not yet emitted.

## Composite index

```text
overall = round2( Σ_layer weight·score  /  Σ_layer weight )
          over analysed layers only
```

- **Analysed only.** A layer missing from the score map (empty facets on
  both sides) is excluded from numerator and denominator — never silently
  counted as zero drift. Silence means "not analysed".
- **Weights** come from `[drift]` in `azusa.toml`; the security-sensitive
  defaults: ancestry 0.5, filesystem/structure/dependency/configuration/
  interface 1.0, behaviour 1.5, operational 1.5, semantic 2.0.
  A weight of 0.0 removes a layer from the composite.
- `round2` — two decimal places, for display and JSON stability.

## Direction-aware severity (`side_severity`)

Used by L6 (interfaces) and L9 (operations) for items present on only one
side:

| Situation | Severity |
| --- | --- |
| Items on the side reconciliation would **overwrite** (they would vanish) | `medium`; `high` at ≥ 5 items |
| Items on the drift source's side (growth) | `low` |
| Bidirectional (either side could be the casualty) | both sides `medium` |

The *breaking* case is the side behind the drift source: for `a-to-b`
that is side A's items; for `b-to-a`, side B's.

## Other fixed severity rules

| Observation | Severity |
| --- | --- |
| Removed test files | `medium`; `high` at ≥ 3 files |
| Removed `#[test]` declarations | `medium` |
| Removed entry points; changed build scripts | `medium` |
| Test/bench/entry additions; benchmark churn | `low` |
| Changed-signature symbols | `medium`; `high` at ≥ 5 |
| Reconfigured operational units | `medium`; `high` at ≥ 3 |
| Renames (any tier) | discounted in scoring per tier (L2 formula) |

## Risk classification

Risk is a whole-report classification (`low` → `critical`) with an explicit
rationale and traceable factors. Two standing floors:

- **Bidirectional drift floors risk at `medium`** — no fast-forward path
  exists, so reconciliation always involves human judgement.
- **High/critical risk** blocks reconciliation plans and escalates watch
  mode.

## Determinism contract

- Fixed formulas, fixed constants, fixed ordering; `c` derives only from
  captured state content.
- No timestamps, randomness or environment reads in the scoring path.
- All floats rounded deterministically (`round2` for JSON stability).
- Identical states + identical `azusa.toml` + identical tool version ⇒
  byte-identical report.

## Worked example

Worktree vs upstream: 4 commits ahead, 6 behind, 2 uncommitted files;
5 files added, 3 modified, 2 deleted, 1 exact-identity rename;
1 major dependency bump; 1 kubernetes manifest changed; 2 public symbols
removed (breaking side), 1 signature changed.

```text
ancestry     c = 4 + 6 + 0.5·2 = 11      → 10·11/23 = 4.78
filesystem   c = 5 + 3 + 2·2 + 0.5 = 12  → 10·12/52 = 2.31
dependency   c = 3·1 = 3                 → 10·3/11  = 2.73
configuration c = 2.5                    → 10·2.5/8.5 = 2.94
interface    c = 2·2 + 3·1 = 7           → 10·7/13  = 5.38
overall      = weighted mean of the analysed layers above (defaults)
```

Layer weights make the composite a judgement, not a sum: see
[configuration](../guides/configuration.md) to retune them — and note the
config digest in every report's provenance.

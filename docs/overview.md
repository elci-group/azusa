# Azusa Overview

**Repository drift intelligence.**

Azusa answers one question precisely:

> How different is the repository I have from the repository I believe I
> have?

It compares two arbitrary repository states — a local worktree, the index, a
commit, an upstream branch, a remote branch, or a plain filesystem directory
— and produces a quantified, explainable model of the drift between them:
what changed, in which direction, how severe it is, why it matters, and what
a safe reconciliation would look like.

## What Azusa is not

Azusa is **not** a `git diff` wrapper. Git is one state provider among
several. The fundamental abstraction is the *state* — identity, ancestry,
filesystem content, structure, dependencies, configuration, interfaces,
verification surface, operations — and the *drift model* produced by
comparing two states. Two directories with no Git at all can drift; so can a
worktree and a tarball-extracted copy of an old release.

## The mental model

```text
  state A  ─┐
            ├─►  layered comparison  ─►  DriftReport  ─►  policy verdict
  state B  ─┘        (L0–L9)              (evidence      ─►  ReconciliationPlan
                                            first)            (Kaptaind executes)
```

1. **States.** A selector (`worktree`, `index`, `head`, `upstream`,
   `remote`, any Git revision, any directory path) resolves to a captured
   `RepositoryState`: content hashes (BLAKE3), ancestry facts, and enriched
   structural facets.
2. **Comparison.** The comparison engine runs ten layers, L0–L9, each
   emitting *observations* — evidence-first facts with severity, confidence,
   side, epistemic strength and machine-checkable references.
3. **The report.** A deterministic `DriftReport` (schema version 2):
   direction, per-layer and composite scores, risk, headline numbers,
   clusters, intent estimate, recommendations, provenance. The JSON
   serialisation is a contract; the human rendering is presentation.
4. **Policy.** `azusa policy check` audits the finished report against the
   `[policy]` block of `azusa.toml` and exits 3 on violation. Policy is an
   evidence check — it never re-analyses states.
5. **Planning.** `azusa plan` derives a deterministic reconciliation plan
   with explicit safety gates. Steps that rewrite history or publish work
   require human approval. **Azusa plans; Kaptaind executes.**

## Why "drift" is first-class

Most tooling answers "what changed between two commits?" — an ancestry
question. Drift is broader and more operational:

- A teammate force-pushed a rewritten history: your local branch and the
  remote have diverged in ways `ahead/behind` numbers hide.
- Generated code was regenerated and committed on one side only.
- A dependency major bump landed upstream while a lockfile was edited locally.
- Production manifests were modified in the worktree and never committed.
- Two checkouts of the same project were provisioned differently.

These situations mix committed and uncommitted work, git and non-git trees,
and structural/dependency/configuration dimensions. Azusa models all of it
in one report, with one score, one direction, and evidence you can check.

## Key concepts

### Layers (L0–L9)

| Layer | Question | Basis |
| --- | --- | --- |
| L0 Identity | Is this even the same repository? | content overlap |
| L1 Ancestry | Ahead/behind, committed vs uncommitted | merge-base |
| L2 Filesystem | Files added / deleted / modified / renamed | BLAKE3 hashes |
| L3 Structure | Modules, crates, directory topology | manifests + tree |
| L4 Dependencies | Manifest + lockfile divergence | semver-aware |
| L5 Configuration | Project configuration files | kind-weighted |
| L6 Interfaces | Public API surface | symbol identities |
| L7 Behaviour | Verification surface: tests, benches, entry points, build scripts | facet comparison |
| L8 Semantics | Security-critical regions | planned |
| L9 Operations | Deployment / service configuration | operational units |

A layer that was not analysed (empty facets on both sides) is excluded from
the composite score — it is never silently counted as zero drift.

### Direction

`in-sync`, `a-to-b`, `b-to-a`, `bidirectional`, `unrelated`, `unknown` —
each with a confidence. Direction is first-class because reconciliation
strategy depends on it: a fast-forward is not a merge, and bidirectional
drift always floors risk at medium (no fast-forward path exists).

### Severity vs epistemic strength

Two orthogonal axes on every observation:

- **Severity** — how much the change matters: `info`, `low`, `medium`,
  `high`, `critical`.
- **Strength** — how the evidence was obtained: `fact` (verified from
  content), `derived` (parsed from structured declarations), `heuristic`
  (inferred by proxy), `speculative` (reserved for semantic layers).

Autonomous consumers gate on strength: act on `fact`/`derived`, route
`heuristic` to human review.

### Intent

Every report carries an intent estimate — does the drift look deliberate or
incidental? — computed from deterministic signals only, in five classes
(`intentional`, `probably-intentional`, `indeterminate`,
`probably-accidental`, `accidental`) with a confidence and the full signal
list. `indeterminate` is a first-class answer. Intent never modifies score
or risk; the inference chain is strictly
`drift → risk → intent → policy → plan`.

### Determinism

Given identical states, configuration and tool version, the report is
byte-identical. Scoring uses fixed saturating curves; observation and
violation ids are assigned after stable sorts; timestamps live only in
provenance and events, never in the analysis path. Reports declare
`analysis_mode: "deterministic"` explicitly.

### Safety

Azusa observes, analyses and plans. It never executes repository mutations:
reconciliation plans are handed to Kaptaind, and destructive operations
(force push, hard reset, branch deletion, history rewriting) are refused by
policy unless explicitly authorised.

## The toolchain context

Azusa is part of a local-first tool ecosystem and speaks the **ELCI**
universal event contract, so its findings flow into shared tooling:

| Tool | Relationship |
| --- | --- |
| Kaptaind | Executes reconciliation plans Azusa generates; manages versioning |
| baby | Builds and installs the `azusa` binary |
| theosis | Reconciles the installed version against this repository |
| Amber, Fract, Traci, Isopod, Bart | Consume Azusa's ELCI events as a shared data product |

## Design rules

1. Determinism is a contract: same inputs → same report, byte for byte.
2. Evidence first: every material score carries the observations behind it.
3. Azusa plans; Kaptaind executes — no destructive Git operations, ever.
4. Local-first: no cloud services, no mandatory daemons, works offline.

## Glossary

| Term | Definition |
| --- | --- |
| **State** | A captured snapshot of a repository: identity, ancestry, filesystem listing, enriched structural facets. |
| **State selector** | A name that resolves to a state: `worktree`, `index`, `head`, `upstream`, `remote`, a Git revision, or a directory path. |
| **Provider** | A component that captures states. Two exist: Git (libgit2) and filesystem. |
| **Drift** | The measurable difference between two states of the same repository. |
| **Layer (L0–L9)** | One dimension of comparison (identity, ancestry, filesystem, structure, dependency, configuration, interface, behaviour, semantics, operations). |
| **Observation** | One evidence-backed fact about divergence, with severity, confidence, side, strength and stable id (`DRIFT-nnnn`). |
| **Evidence** | A machine-checkable datum behind an observation (kind, description, references). |
| **Strength** | How the evidence was obtained: `fact`, `derived`, `heuristic`, `speculative`. |
| **Direction** | The shape of divergence: `in-sync`, `a-to-b`, `b-to-a`, `bidirectional`, `unrelated`, `unknown`. |
| **Drift index** | The composite 0–10 score, a weighted mean over analysed layers. |
| **Risk** | Classified reconciliation risk: `low`, `medium`, `high`, `critical`. |
| **Cluster** | A logical grouping of changed paths (`lockfiles`, `tests`, `crates/azusa-core`, …). |
| **Intent** | The estimate of whether drift is deliberate, with confidence and signals. |
| **Policy** | Configurable limits checked against a finished report (`azusa policy check`). |
| **Violation** | One policy breach with a stable id (`POLICY-nnnn`), rule, severity and references. |
| **Plan** | A deterministic, ordered sequence of reconciliation steps plus safety gates (`azusa plan`). |
| **Gate** | A condition that must hold before a plan may run; `blocking` gates prevent execution. |
| **ELCI** | The universal event contract Azusa emits (`azusa.drift.detected`, …). |
| **Evidence store** | The append-only local trail under `.azusa/` (history, events, exports, watch state, hash cache). |
| **Rename tier** | Rename confidence: `exact-identity`, `content-similarity`, `path-heuristic`. |
| **Kaptaind** | The executor of reconciliation plans; Azusa never mutates a repository itself. |

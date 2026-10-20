# Azusa

**Repository drift intelligence.**

> How different is the repository I have from the repository I believe I have?

Azusa compares arbitrary repository states — local worktree, index, HEAD,
upstream branch, remote branch, or plain filesystem trees — and produces a
quantified, explainable model of how they have drifted apart: what changed,
in which direction, how severe it is, and what to do about it.

It is **not** a `git diff` wrapper. Git is one state provider among several;
the fundamental abstraction is the *state* — identity, ancestry, filesystem,
structure, dependencies, configuration, behaviour — and the *drift model*
produced by comparing two of them.

## Quickstart

```console
$ azusa                    # fast path: worktree vs upstream
$ azusa drift origin/main  # worktree vs an explicit state
$ azusa drift head HEAD~3  # any two states
$ azusa drift ./old-copy   # a filesystem path is a state too
$ azusa explain            # why does it matter? clusters + full evidence
$ azusa status             # identity + quick health
$ azusa inspect --json     # full state dump (machine contract)
$ azusa score              # the drift index only
$ azusa policy check       # CI gate: exit 3 when configured limits are violated
$ azusa export             # canonical drift-report artefact under .azusa/exports/
$ azusa plan               # reconciliation plan: steps + safety gates (Kaptaind executes)
$ azusa watch              # continuous drift monitoring with staged escalation
```

State selectors: `worktree` · `index` · `head` · `upstream` · `remote` · any
revision libgit2 resolves (`main`, `origin/main`, `HEAD~3`, an object id) ·
any directory path. With no arguments, azusa compares `worktree` against
`upstream`, falling back to the remote default branch, then `head` — every
fallback is carried in the report as evidence.

## Install & update

Azusa ships as a single static binary. Two repo-native tools manage its
lifecycle:

```console
$ baby --user             # build + install to ~/.local/bin (recipe: .baby.toml)
$ theosis check           # compare the installed version with this remote
$ theosis update          # reconcile: clone, build via baby, install, verify
```

Theosis clones this repository's default branch into a transaction
directory, reads the kaptaind-managed root `VERSION` (the same number the
binary self-reports — `azusa --version`), delegates the build to
`.baby.toml`, installs, and verifies before declaring success. It never
downgrades automatically (exit 10 when the installed version is newer).
Update state lives under `.theosis/` and is never committed.

## Exit codes

| Code | Meaning |
| ---: | --- |
| 0 | states in sync — no drift |
| 1 | drift detected |
| 2 | high-risk / critical drift |
| 3 | policy violation |
| 4 | analysis failure (unresolvable state, provider error) |
| 5 | invalid invocation |

Thresholds are configurable (see below), making `azusa` a CI gate.

## Output contract

Every command speaks three dialects of the same facts:

```console
$ azusa drift --json    # canonical machine contract (schema_version 2)
$ azusa drift --ndjson  # one JSON record per observation + a summary line
$ azusa drift --quiet   # exit codes only
```

Human output is presentation; JSON is the contract. Reports are
**deterministic**: identical states, configuration and tool version always
produce byte-identical JSON (timestamps live in provenance only, never in
the analysis path). `analysis_mode: deterministic` says so explicitly.

## Drift model

Comparison runs in layers; each layer emits evidence-first observations —
the score never replaces the evidence that produced it:

```text
L0 Identity      same repository? fork? unrelated?        (content-overlap)
L1 Ancestry      ahead/behind, committed vs uncommitted   (merge-base)
L2 Filesystem    added / deleted / modified / renamed      (BLAKE3 hashes)
L3 Structure     modules, crates, topology                (manifests + tree)
L4 Dependencies  manifest + lockfile divergence           (semver-aware)
L5 Configuration project configuration files             (kind-weighted)
L6 Interfaces    public API surface                       (symbol identities)
L7 Behaviour     verification surface: tests, benches,     (facet comparison)
                 entry points, build scripts
L8 Semantics     security-critical regions                (planned)
L9 Operations    deployment/service surface               (operational units)
```

**Change clustering**: every diff is grouped into logical clusters
(`crates/azusa-core`, `tests`, `lockfiles`, `operations`, …) so
reconciliation can be decomposed — `azusa explain` shows them, and they ride
along in the JSON contract.

**Structural enrichment**: while hashing, providers retain the text of
analysis-interest files (manifests, lockfiles, toolchain/CI/deploy configs,
size-capped). `azusa-graph` turns that into module topology, dependency
declarations with lockfile-resolved versions, and configuration
classification — a pure function of the captured state, so enrichment never
breaks determinism.

**Behavioural enrichment**: the retained surface also covers the behavioural
files — `build.rs`, library and binary roots, integration tests and benches.
From those, `azusa-graph` derives the behaviour facet: test files, benchmark
files, executable entry points, build scripts, `#[test]` declaration counts
and per-file public item counts (`pub fn`, `pub struct`, …).

- **L6 interfaces** compares the public API at **symbol level**: each
  side's extracted symbols (crate-relative fully qualified name, item
  kind, normalized declaration signature) are diffed. Symbols present on
  only one side and symbols whose signature changed with the same identity
  are reported — a parameter-type change registers as interface drift even
  when the symbol count is unchanged. Restricted visibility
  (`pub(crate)`, `pub(in …)`) is excluded: it is not external API.
  Severity is direction-aware: symbols on the side reconciliation would
  overwrite rate **medium** (high at ≥5), drift-source-side symbols rate
  **low**; changed signatures rate **medium** (high at ≥5).
- **L7 behaviour** compares the verification surface — tests, benches,
  entry points, build scripts: removed tests and deleted `#[test]`
  declarations rate **medium**, changed build scripts (which execute at
  compile time) rate **medium**, additions rate **low**. This is
  verification-infrastructure topology, not runtime-behaviour analysis;
  true behavioural drift (L8 semantics) is future work. The layer id stays
  `behaviour` in the JSON contract for compatibility.
- **L9 operations** compares the deployment/service surface — CI workflows,
  containers, kubernetes manifests, service units, compose stacks, framed
  as *operational units*. Units present on only one side follow the same
  direction-aware severity rule as interfaces (side reconciliation would
  overwrite: **medium**; drift source's side: **low**); reconfigured units
  (content changed) rate **medium**, high at ≥3, because running
  infrastructure may diverge from the repository.

Direction is first-class: `in-sync`, `a-to-b`, `b-to-a`, `bidirectional`,
`unrelated`, `unknown` — with a confidence. Bidirectional drift always
floors risk at medium, because no fast-forward path exists.

**Rename confidence is explicit.** Every detected rename carries a tier:
`exact-identity` (content hash identical — the move is a fact),
`content-similarity` (content changed, similarity ≥ 0.9 — probable move),
or `path-heuristic` (paired by path/size heuristics only — an association,
not evidence). Scoring discounts renames by tier (×0.5 / ×0.75 / ×1.5):
a weak pairing is treated almost like a distinct delete + add, because
false rename attribution is worse than reporting the two events.

## Epistemic strength

Severity says how much a change matters; **strength** says how much the
claim can be trusted. Every observation carries one of four grades:

| Grade | Meaning | Examples |
| --- | --- | --- |
| `fact` | verified from content | digests, merge-base, exact renames, manifest/lockfile deltas |
| `derived` | parsed from structured declarations | API symbol identities and signatures |
| `heuristic` | inferred by proxy | lexical test inventory, path-similarity renames, intent signals |
| `speculative` | pattern-based judgement | reserved for semantic layers |

Autonomous consumers gate on strength, not severity alone: Kaptaind may
act on `fact` and `derived` findings; `heuristic` findings warrant human
review. Intent assessment is fully orthogonal — it never modifies the
drift score or risk level.

Scores come from documented saturating curves (`10·c/(c+k)`) over weighted
change counts; deletions weigh double, renames half, uncommitted paths half.
Unanalyzed layers are excluded from the composite rather than counted as
zero drift.

## Intent assessment

Every drift report carries an intent estimate: does the drift look
**deliberate** or **incidental**? Azusa weighs deterministic signals —
cluster composition, generated-content markers, branch naming, uncommitted
hygiene — into one of five classes: `intentional`, `probably-intentional`,
`indeterminate`, `probably-accidental`, `accidental`, each with a confidence
and the full signal list. "Indeterminate" is a first-class answer; intent is
never asserted as certain.

## Policy as a CI gate

`azusa policy check` runs the normal drift analysis, then checks the report
against the `[policy]` block of `azusa.toml`: maximum overall drift score,
dependency rules (major bumps, new dependencies), structure rules (new
crates) and configuration rules (production-facing changes). Violations list
stable ids (`POLICY-0001`…), severities and references; any violation exits
3. Policies are evidence checks over the report — they never re-analyse
states.

## Continuous drift (watch mode)

`azusa watch [--interval <secs>] [--max-cycles <n>]` monitors drift
continuously, escalating attention in stages:

- **Stage 0 — idle.** A cheap stat-only fingerprint (`path, size, mtime`)
  of the worktree; unchanged means no analysis and no history churn.
- **Stage 1 — analysed.** The fingerprint changed: run the full pipeline
  (history + ELCI events).
- **Stage 2 — escalated.** The drift score grew beyond the previous run,
  or risk is high/critical: additionally generate a reconciliation plan
  and emit `azusa.plan.generated`.

State persists under `.azusa/watch.json`, so a restarted watcher resumes
where it left off; idle cycles de-escalate back to stage 0. Each cycle's
analysis stays deterministic — only the loop itself is temporal, and
mtimes feed the change detector, never the analysis path. With
`--max-cycles`, watch exits with the drift mapping of its last analysis
cycle, so a bounded watch is CI-usable like `azusa drift`.

## Reconciliation planning

`azusa plan` derives a deterministic reconciliation plan from the drift
report and the policy verdict: ordered steps (commit, fetch, fast-forward,
merge, push, review, run-tests), each with a risk, a rationale and an
explicit approver. Steps that rewrite history or publish work — merge,
rebase, push — **require human approval**; Azusa plans, Kaptaind executes.

Safety gates must pass before a plan may run:

- a failing `azusa policy check` **blocks** the plan;
- high/critical risk, or any bidirectional drift (a human merge decision is
  unavoidable), **blocks** until a human accepts the risk;
- accidentally-classified drift raises an advisory gate to confirm the
  accidental side is safe to discard;
- unrelated states **block** pending manual verification.

The plan exports as a canonical artefact under `.azusa/exports/` and emits
`azusa.plan.generated`. Plans are pure functions of the report — identical
reports always produce identical plans.

## Local evidence store

Analysis writes an append-only evidence trail under `.azusa/` (never
duplicating report content):

```text
.azusa/history.jsonl   one JSONL record per analysis run
.azusa/events.jsonl    ELCI event log (NDJSON, schema_version 1)
.azusa/exports/        canonical artefacts: drift reports (`azusa export`)
                       and reconciliation plans (`azusa plan`, watch stage 2)
.azusa/watch.json      watch-mode escalation state
.azusa/cache/          stat-keyed hash cache (incremental scans)
```

Events follow the ELCI universal event contract (`azusa.drift.detected`,
`azusa.drift.threshold_exceeded`, `azusa.policy.failed`, …) so other ELCI
tools — Amber, Fract, Traci, Isopod, Bart — can consume them as a shared
data product. Every analysis run appends to the trail, best-effort: analysis
never fails because the store is unwritable.

## Configuration (optional `azusa.toml`)

```toml
[drift]           # layer weights (defaults shown)
ancestry = 0.5
filesystem = 1.0
structure = 1.0
dependency = 1.0
configuration = 1.0
interface = 1.0
behaviour = 1.5
semantic = 2.0    # security-sensitive default profile
operational = 1.5

[rename]          # rename-detection thresholds
path_threshold = 0.85
size_ratio_threshold = 0.5
min_similarity_size = 16

[exit]            # exit code mapping
medium = 1
high = 2
critical = 2

[relatedness]
min_overlap = 0.15

[policy]          # enforced by `azusa policy check` (defaults shown)
max_drift = 5.0   # maximum tolerated overall drift score

[policy.dependencies]
allow_major = false
allow_new = true

[policy.structure]
allow_new_crates = true

[policy.configuration]
production_changes = "RequireReview"    # Forbid | RequireReview | Allow
```

## Performance

- Incremental by design: stat-keyed hash cache under `.azusa/cache/` means
  unchanged files are never re-read on repeat scans.
- Parallel content hashing (rayon); large files stream rather than buffer.
- Performance targets (clean status <100 ms, structural drift <500 ms) are
  tracked toward v1; see `ARCHITECTURE.md`.

## Roadmap

| Phase | Scope | Status |
| --- | --- | --- |
| 1 | Foundation: providers, fileset comparison, ancestry, scoring, CLI, JSON | ✅ |
| 2 | Structural intelligence: renames ✓, module topology, dependencies, config, clustering | ✅ |
| 3 | ELCI intelligence: policy engine, intent detection, events, export, plans | ✅ |
| 4 | Behavioural drift: tests ✓, build ✓, interfaces ✓ | ✅ |
| 5 | Operational drift: deployment/service configuration | ✅ |
| 6 | Reconciliation planning with safety gates (Kaptaind executes) | ✅ |
| 7 | Continuous drift: watch mode with staged escalation | ✅ |

## Design rules

- Determinism is a contract: same inputs → same report, byte for byte.
- Evidence first: every material score carries its observations.
- Azusa observes, analyses and plans; it never executes destructive Git
  operations. Kaptaind executes.
- Local-first: no cloud services, no mandatory daemons, works offline.

## Documentation

The full documentation anthology lives under [`docs/`](docs/README.md) —
role-oriented guides, machine-contract references, decision records and
operational material:

| Start here | For |
| --- | --- |
| [Documentation index](docs/README.md) | the map and reading paths |
| [Quickstart](docs/guides/quickstart.md) | first fifteen minutes |
| [CLI reference](docs/guides/cli-reference.md) | commands, flags, exit codes |
| [Configuration](docs/guides/configuration.md) | every `azusa.toml` knob |
| [Drift report schema](docs/reference/drift-report-schema.md) | the JSON contract |
| [Scoring reference](docs/reference/scoring.md) | every formula and constant |
| [Operations runbook](docs/operations/runbook.md) | triage and recovery |

## License

MIT — see `LICENSE`.

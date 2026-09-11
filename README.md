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
$ azusa status             # identity + quick health
$ azusa inspect --json     # full state dump (machine contract)
$ azusa score              # the drift index only
```

State selectors: `worktree` · `index` · `head` · `upstream` · `remote` · any
revision libgit2 resolves (`main`, `origin/main`, `HEAD~3`, an object id) ·
any directory path. With no arguments, azusa compares `worktree` against
`upstream`, falling back to the remote default branch, then `head` — every
fallback is carried in the report as evidence.

## Exit codes

| Code | Meaning |
| ---: | --- |
| 0 | states in sync — no drift |
| 1 | drift detected |
| 2 | high-risk / critical drift |
| 3 | policy violation *(phase 3)* |
| 4 | analysis failure (unresolvable state, provider error) |
| 5 | invalid invocation |

Thresholds are configurable (see below), making `azusa` a CI gate.

## Output contract

Every command speaks three dialects of the same facts:

```console
$ azusa drift --json    # canonical machine contract (schema_version 1)
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
L3 Structure     modules, crates, topology                (phase 2)
L4 Dependencies  manifest + lockfile divergence           (phase 2)
L5 Configuration project configuration files             (phase 2)
L6 Interfaces    public API surface                       (phase 4)
L7 Behaviour     tests, build scripts, entry points       (phase 4)
L8 Semantics     security-critical regions                (phase 4)
L9 Operations    deployment and service configuration     (phase 5)
```

Direction is first-class: `in-sync`, `a-to-b`, `b-to-a`, `bidirectional`,
`unrelated`, `unknown` — with a confidence. Bidirectional drift always
floors risk at medium, because no fast-forward path exists.

Scores come from documented saturating curves (`10·c/(c+k)`) over weighted
change counts; deletions weigh double, renames half, uncommitted paths half.
Unanalyzed layers are excluded from the composite rather than counted as
zero drift.

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
| 2 | Structural intelligence: renames ✓, module topology, dependencies, config, clustering | ⬜ |
| 3 | ELCI intelligence: policy engine, intent detection, events, export, plans | ⬜ |
| 4 | Behavioural drift: tests, build, interfaces | ⬜ |
| 5 | Operational drift: deployment/service configuration | ⬜ |
| 6 | Reconciliation planning with safety gates (Kaptaind executes) | ⬜ |
| 7 | Continuous drift: watch mode with staged escalation | ⬜ |

## Design rules

- Determinism is a contract: same inputs → same report, byte for byte.
- Evidence first: every material score carries its observations.
- Azusa observes, analyses and plans; it never executes destructive Git
  operations. Kaptaind executes.
- Local-first: no cloud services, no mandatory daemons, works offline.

## License

MIT — see `LICENSE`.

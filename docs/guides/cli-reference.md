# CLI Reference

The `azusa` binary. Global flags apply to every command; JSON is the
machine contract, human output is presentation, exit codes are the verdict.

```text
azusa [GLOBAL FLAGS] [COMMAND] [ARGS]
```

With no subcommand, `azusa` runs `drift` (the fast path: worktree vs
upstream).

## Global flags

| Flag | Effect |
| --- | --- |
| `--json` | Machine-readable JSON (pretty-printed) instead of human output. |
| `--ndjson` | Newline-delimited JSON — one record per line. |
| `--quiet` | Suppress output entirely; communicate through exit codes only. |
| `--version` | Print the tool version (the root `VERSION` file) and exit 0. |
| `--help` | Print help and exit 0. |

The three output flags are mutually exclusive; `--json` and `--ndjson`
conflict with each other, and `--quiet` conflicts with both.

## Exit codes

| Code | Meaning | Notes |
| ---: | --- | --- |
| 0 | states in sync — no drift | `--help` / `--version` also exit 0 |
| 1 | drift detected | medium risk, or low risk that is not in-sync; configurable via `[exit].medium` |
| 2 | high-risk / critical drift | configurable via `[exit].high` / `[exit].critical` |
| 3 | policy violation | `azusa policy check` on any violation |
| 4 | analysis failure | unresolvable state, provider error, unreadable `azusa.toml` |
| 5 | invalid invocation | clap usage errors |

Exit-code mapping for drift results is risk-driven:

- `low` risk + `in-sync` direction → 0
- `low` risk + any other direction → `[exit].medium` (default 1)
- `medium` risk → `[exit].medium` (default 1)
- `high` risk → `[exit].high` (default 2)
- `critical` risk → `[exit].critical` (default 2)

A bounded `azusa watch` exits with the drift mapping of its last *analysis*
cycle (idle cycles never overwrite it; a watch that only saw idle cycles
exits 0).

## State selectors

Every comparison command takes 0–2 positional selectors:

| Form | Resolves via | Examples |
| --- | --- | --- |
| Git keywords | Git provider | `worktree`, `index`, `head`, `upstream`, `remote` |
| Git revisions | libgit2 | `main`, `origin/main`, `HEAD~3`, an object id |
| Directory paths | filesystem provider | `.`, `./old-copy`, `/srv/checkouts/v1` |

`remote` resolves origin/HEAD, falling back to `origin/main`, then
`origin/master`.

Selector count semantics:

- **0** — the default pair: `worktree` vs `upstream`, falling back to
  `remote`, then `head`. Every fallback is recorded in the report as an
  observation (fallbacks are evidence, never silent).
- **1** — `worktree` vs `<spec>`.
- **2 or more** — `<spec-1>` vs `<spec-2>` (extras are ignored).

A directory path resolves through the filesystem provider even when it is
not a Git repository; mixed comparisons (git state vs plain directory) are
supported.

## Commands

### `azusa status [SPEC]`

Identity and quick health of one state (default: `worktree`). Always exits
0 unless resolution fails (then 4). Machine format: the full
`RepositoryState` JSON.

### `azusa inspect [SPEC]`

Full state dump of one state (default: `worktree`) — every facet: identity,
ancestry, filesystem, structure, dependencies, configuration. This is the
machine contract for "what does this state look like". Human output
appends quick counts (modules, declared dependencies, lockfiles, config
files).

### `azusa drift [SPEC…]` *(default command)*

The core comparison: resolve the pair, run layers L0–L9, print the report.
Exit code from the risk mapping above. The report is recorded to the local
evidence store (`.azusa/history.jsonl`) and ELCI events are emitted
(`azusa.drift.detected`; `azusa.drift.threshold_exceeded` when risk ≥ high)
— best-effort, never failing the analysis.

Output dialects:

| Flag | Shape |
| --- | --- |
| *(human)* | headline block: direction, score, risk, summary, top observations |
| `--json` | the complete `DriftReport` (schema version 2), pretty-printed |
| `--ndjson` | one record per observation (`"record": "observation"`, …) then one summary line (`"record": "summary"`) |
| `--quiet` | nothing; exit code only |

### `azusa score [SPEC…]`

The drift index only. Human output lists the overall score and per-layer
scores; `--json` emits exactly the `DriftScore` object (`overall` +
`layers`); `--quiet` is exit-code-only. Analysis side-effects (history,
events) are identical to `drift`.

### `azusa explain [SPEC…]`

Why does it matter? Human output is the full report rendering — every
observation with complete detail and evidence, all clusters, the intent
assessment with its signal list, recommendations. `--json` is the complete
report (identical bytes to `drift --json`).

### `azusa policy check [SPEC…]`

Drift analysis plus policy enforcement. Runs the normal pipeline, audits
the report against the `[policy]` block of `azusa.toml`, and:

- exits **3** on any violation (regardless of drift risk);
- exits the normal drift mapping when compliant;
- emits `azusa.policy.failed` to the event log on violation.

Output: human rendering lists each violation (`id [severity] rule:
description` plus up to four evidence references); `--json` emits
`{"compliant": bool, "violations": [...], "report": {...}}`; `--quiet`
suppresses everything.

See [reference/policy-rules.md](../reference/policy-rules.md).

### `azusa export [SPEC…]`

Runs the analysis, writes the canonical drift-report artefact to
`.azusa/exports/drift-<UTC-stamp>-<digest8>.json` (envelope: schema
version, `kind: "azusa.drift-report"`, timestamp; payload: full report),
and emits `azusa.inspect.completed`. Exits with the normal drift mapping.

### `azusa plan [SPEC…]`

Derives the deterministic reconciliation plan (report + policy verdict),
exports it to `.azusa/exports/plan-<UTC-stamp>-<repo>-<direction>.json`
(`kind: "azusa.reconciliation-plan"`), and emits `azusa.plan.generated`.
Human output lists gates (blocking vs advisory) and steps (id, kind, risk,
rationale, approval requirement). `--json` emits
`{"plan": {...}, "export": "...", "events": "..."}`. Exits with the
underlying drift mapping (the plan itself does not change the exit code).

See [reference/plan-schema.md](../reference/plan-schema.md).

### `azusa watch [--interval SECS] [--max-cycles N]`

Continuous drift monitoring with staged escalation. `--interval` is the
seconds between quick checks (default 5); `--max-cycles` bounds the run
(default 0 = run until interrupted). Per-cycle output in machine mode is a
JSON object:

```json
{"cycle": 1, "changed": false, "stage": 0}
{"cycle": 2, "changed": true, "stage": 1, "score": 2.4, "risk": "medium"}
{"cycle": 3, "changed": true, "stage": 2, "score": 4.1, "risk": "high",
 "plan": ".azusa/exports/plan-...-.json"}
```

Full behaviour, the stage ladder and operational guidance:
[watch-mode.md](watch-mode.md).

## Machine-contract summary

Every command speaks three dialects of the same facts — human (presentation),
JSON (contract), exit codes (verdict). The canonical artefacts are:

| Artefact | Producer | Schema |
| --- | --- | --- |
| Drift report | `drift`, `explain`, `score` (`--json` subsets), `export` | [schema v2](../reference/drift-report-schema.md) |
| Reconciliation plan | `plan`, watch stage 2 | [schema v1](../reference/plan-schema.md) |
| Policy verdict | `policy check --json` | `{compliant, violations, report}` |
| ELCI events | all analysis commands | [schema v1](../reference/elci-events.md) |
| History record | all analysis commands | [evidence store](../reference/evidence-store.md) |

## Invocation examples

```console
$ azusa                                    # worktree vs upstream
$ azusa drift --json > report.json         # canonical artefact to a file
$ azusa drift --quiet; echo $?             # 0/1/2 verdict in scripts
$ azusa drift worktree index --quiet       # did I stage everything?
$ azusa plan origin/main --json | jq .plan.steps
$ azusa policy check && echo ship-it
$ azusa watch --interval 60 --max-cycles 1440   # one day, one-minute cadence
```

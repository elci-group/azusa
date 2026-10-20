# Quickstart

A guided first pass over every command, run against a real repository. All
commands are read-only with respect to your repository; the only things
Azusa writes live under `.azusa/` (which is git-ignored).

## 0. Install

```console
$ baby --user          # build + install to ~/.local/bin
$ azusa --version
azusa 0.1.39
```

## 1. Identity and quick health

```console
$ azusa status
```

Reports the resolved state's identity — repository name, provider, label,
fileset digest — plus quick health numbers. The optional argument is any
state selector:

```console
$ azusa status head
$ azusa status origin/main
$ azusa status /somewhere/else/checkouts/old-copy
```

## 2. The fast path: worktree vs upstream

```console
$ azusa
```

A bare `azusa` is the default drift comparison: **worktree vs upstream**.
If no upstream is configured it falls back to the remote default branch
(`remote`), then to `head` — and every fallback is carried in the report as
an observation, never silently. Human output shows the direction, the
composite drift index, risk, headline numbers and the top observations.

## 3. Compare any two states

```console
$ azusa drift origin/main           # worktree vs an explicit state
$ azusa drift head HEAD~3           # any two revisions
$ azusa drift ./old-copy            # a filesystem path is a state too
$ azusa drift worktree index        # staged vs unstaged reality
```

State selectors: `worktree` · `index` · `head` · `upstream` · `remote` ·
any revision libgit2 resolves (`main`, `origin/main`, `HEAD~3`, an object
id) · any directory path.

Exit codes carry the verdict, so scripts need no output parsing:

| Code | Meaning |
| ---: | --- |
| 0 | states in sync |
| 1 | drift detected (medium, or low-but-drifting) |
| 2 | high-risk / critical drift |
| 3 | policy violation (`azusa policy check` only) |
| 4 | analysis failure |
| 5 | invalid invocation |

## 4. Why does it matter?

```console
$ azusa explain
```

Full evidence for every observation — complete detail text, all evidence
references, change clusters (logical groupings like `crates/azusa-core`,
`tests`, `lockfiles`), the intent estimate with its signal list, and
recommendations.

## 5. The number only

```console
$ azusa score
```

Prints the composite drift index and the per-layer scores. In `--json`
mode, emits exactly the `DriftScore` object — handy for dashboards.

## 6. The machine contract

```console
$ azusa drift --json      # canonical machine contract (schema_version 2)
$ azusa drift --ndjson    # one JSON record per observation + a summary line
$ azusa drift --quiet     # exit codes only
```

Reports are deterministic: identical states, configuration and tool version
always produce byte-identical JSON. `analysis_mode: "deterministic"` says
so explicitly. The full field reference lives in
[reference/drift-report-schema.md](../reference/drift-report-schema.md).

## 7. Full state dump

```console
$ azusa inspect --json
```

Dumps one resolved state — all facets: identity, ancestry, filesystem
listing, structure, dependencies, configuration — as the machine contract
other tools can consume.

## 8. The CI gate

```console
$ azusa policy check
```

Runs the normal drift analysis and then audits the report against the
`[policy]` block of `azusa.toml` (maximum drift score, dependency rules,
structure rules, production configuration rules). Any violation **exits 3**
and lists violations with stable ids (`POLICY-0001`…). See
[ci-integration.md](ci-integration.md) for pipeline recipes and
[reference/policy-rules.md](../reference/policy-rules.md) for rule semantics.

## 9. A reconciliation plan

```console
$ azusa plan
```

Derives the deterministic reconciliation plan from the report and the
policy verdict: ordered steps (`commit`, `fetch`, `fast-forward`, `merge`,
`push`, `review`, `run-tests`, …), each with a risk, a rationale and an
explicit approver. Steps that rewrite history or publish work — merge,
rebase, push — **require human approval**. Blocking safety gates (policy
violations, high/critical risk, bidirectional drift, unrelated states) must
pass before the plan may run. The plan exports as a canonical artefact
under `.azusa/exports/`.

**Azusa plans; Kaptaind executes.** Running the plan is never Azusa's job.

## 10. Export an audit artefact

```console
$ azusa export
exported: .azusa/exports/drift-20261020T120000-1a2b3c4d.json
events:   .azusa/events.jsonl
```

Writes the canonical drift-report artefact (envelope + full report) and
emits ELCI events to the local event log.

## 11. Continuous watch

```console
$ azusa watch --interval 30
```

Monitors drift continuously with staged escalation: idle cycles do a cheap
stat-only check (no analysis, no history churn); a detected change runs the
full pipeline; a growing score or high/critical risk additionally generates
a reconciliation plan. See [watch-mode.md](watch-mode.md).

## 12. What got written where

```text
.azusa/
├── history.jsonl     one record per analysis run
├── events.jsonl      ELCI event log (NDJSON)
├── exports/          canonical drift-report and plan artefacts
├── watch.json        watch-mode escalation state
└── cache/            stat-keyed hash cache (incremental scans)
```

All of it is best-effort, git-ignored, and safe to delete.

## Next steps

- Tune thresholds and weights: [configuration.md](configuration.md)
- Wire the gate into CI: [ci-integration.md](ci-integration.md)
- Integrate on the JSON contract: [reference/drift-report-schema.md](../reference/drift-report-schema.md)
- Understand every number: [reference/scoring.md](../reference/scoring.md)

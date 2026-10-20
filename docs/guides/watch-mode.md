# Watch Mode (Continuous Drift)

`azusa watch` monitors drift continuously, escalating attention in stages
instead of spamming analyses. It exists for operators who want a standing
sentinel over a long-lived checkout: a working tree mid-development, a
deployment staging checkout, a hotfix environment.

```console
$ azusa watch [--interval <secs>] [--max-cycles <n>]
```

- `--interval <secs>` — seconds between quick checks (default 5).
- `--max-cycles <n>` — stop after this many cycles (default 0 = run until
  interrupted).

## The escalation ladder

```text
stage 0 ──change──► stage 1 ──score grew / risk ≥ high──► stage 2
  ▲                    │                                      │
  └──── no change ─────┴──────────── no change ───────────────┘
```

### Stage 0 — idle

A cheap **stat-only fingerprint** of the worktree (`path, size, mtime`
triples, BLAKE3-digested, sorted) is compared with the fingerprint of the
last full analysis. Unchanged means: no analysis, no history churn, streaks
reset. This is what makes a long-running watch affordable — the common
cycle costs one directory stat pass.

### Stage 1 — analysed

The fingerprint changed: run the **full deterministic pipeline** — the same
one-shot `azusa drift` path, including history append and ELCI events.
Nothing about the analysis is cheaper or weaker than a manual run.

### Stage 2 — escalated

The drift score **grew beyond the previous run**, or risk is high/critical:
additionally generate a reconciliation plan (policy verdict included),
export it under `.azusa/exports/`, and emit `azusa.plan.generated`. The
plan is generated only on the transition into stage 2, not on every
escalated cycle.

Escalation is **sticky while drift persists**: consecutive escalated cycles
increment a streak. Idle cycles de-escalate back to stage 0.

## State and restarts

State persists in `.azusa/watch.json`:

```json
{
  "schema_version": 1,
  "last_full_quick_digest": "…",
  "stage": 2,
  "streak": 3,
  "last_score": 4.1,
  "last_risk": "high"
}
```

A restarted watcher resumes where it left off — unchanged trees are not
re-analysed, and stage/streak/score history survives restarts. A corrupt
state file resets the watcher to idle rather than failing it (best-effort
store semantics).

## Output

Human:

```text
watch cycle 12: no change (stage 0, idle)
watch cycle 13: change detected (stage 1, analysed)
  drift 2.4/10  risk medium
watch cycle 14: change detected (stage 2, escalated — plan generated)
  drift 4.1/10  risk high
  plan: .azusa/exports/plan-20261020T121500-azusa-bidirectional.json
```

Machine (`--json` or `--ndjson`): one JSON object per cycle —

```json
{"cycle": 14, "changed": true, "stage": 2, "score": 4.1, "risk": "high",
 "plan": ".azusa/exports/plan-….json"}
```

Idle cycles emit `{"cycle": n, "changed": false, "stage": 0}`. `--quiet`
suppresses all output.

## Exit codes

Without `--max-cycles`, watch runs until interrupted. **With
`--max-cycles`, watch exits with the drift mapping of its last *analysis*
cycle** — so a bounded watch is CI-usable exactly like `azusa drift`
(0 in-sync, 1 medium, 2 high/critical; 4 on analysis failure). A watch that
only saw idle cycles exits 0.

## Determinism guarantees

The loop is temporal; **the analysis never is**:

- mtimes feed only the stage-0 change detector, never the analysis path;
- every full cycle is the same pure pipeline as one-shot `azusa drift`;
- per-cycle reports remain byte-reproducible for identical states.

## Operational recipes

**Long-lived checkout sentinel:**

```console
$ azusa watch --interval 60
```

**One working day, one-minute cadence, CI-usable verdict:**

```console
$ azusa watch --interval 60 --max-cycles 480 --quiet; echo $?
```

**Pipeline tail: escalate to a plan artefact, then archive exports:**

```sh
azusa watch --interval 30 --max-cycles 16
rsync -a .azusa/exports/ audit-host:/srv/azusa-exports/$(hostname)/
```

## Tuning

| Concern | Adjustment |
| --- | --- |
| Watch too chatty on busy trees | raise `--interval`; stage 0 still makes each idle cycle cheap |
| Plans generated too eagerly | stage 2 requires score *growth* or high risk — persistent medium drift stays at stage 1 by design; loosen `[policy]`/`[exit]` if stage 1 alarms |
| Resume after reboot | nothing to do — `watch.json` restores stage/streak/digest |
| Full reset | `rm .azusa/watch.json` (safe; the watcher starts idle) |

## Boundaries

- Watch analyses the **worktree** (default pair) only; it does not loop
  over arbitrary selector pairs — use scheduled `azusa drift <specs>` for
  that.
- The interval is a floor, not a deadline: each full analysis takes as long
  as it takes; the quick check runs again `interval` seconds after the
  cycle completes.
- Watch is single-instance per repository by convention (the state file is
  not locked); running two watchers on one checkout will interleave stage
  transitions harmlessly but doubles work.

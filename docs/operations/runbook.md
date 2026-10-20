# Operations Runbook

Operational guidance for running Azusa on live repositories: reading
verdicts, triaging failures, and recovering from odd states.

## Reading a verdict in 30 seconds

```console
$ azusa status                 # 1. what am I looking at?
$ azusa                        # 2. worktree vs upstream: the verdict
$ azusa explain                # 3. the receipts
```

Decision table for `azusa` / `azusa drift` exit codes:

| Exit | Meaning | Action |
| ---: | --- | --- |
| 0 | in sync | nothing to do |
| 1 | medium drift (or low-but-drifting) | review at convenience; `azusa plan` shows the path |
| 2 | high/critical risk | **review before acting**; a plan will be gated |
| 3 | (policy check) policy violation | address the violation or make a documented policy exception |
| 4 | analysis failure | triage below |
| 5 | invalid invocation | fix the command line (`azusa --help`) |

## Triage: exit 4 — analysis failure

| Cause | Symptom | Fix |
| --- | --- | --- |
| Unresolvable selector | `unresolved state …` | `azusa status <spec>` to test; check `git branch -a`, spelling, object ids |
| Not a repository, no path | `no provider` | run inside the repo, or pass a directory selector |
| Malformed `azusa.toml` | `azusa: <path>: <parse error>` | fix the TOML (unknown keys are rejected by design) |
| Unreadable worktree | provider error | check permissions; symlink loops; extremely large generated trees (use ignore rules) |
| Git object database issues | git provider error | `git fsck`; Azusa reads via libgit2 and never repairs |

## Triage: surprising verdicts

**"Drift but I expected in-sync"**

- You likely compare **worktree vs upstream**, and uncommitted work counts.
  `azusa drift head upstream` compares committed state only.
- Generated files churn: check the clusters (`azusa explain`) — generated
  content also feeds the intent signal list.
- Line-ending or permission churn shows up as modification; check
  `git config core.autocrlf` / file modes.

**"In-sync but I expected drift"**

- The compared states may share content even if refs differ (same tree).
  Read `state_a.digest` / `state_b.digest`: equal digests ⇒ identical
  filesets by definition.
- A layer with empty facets on both sides is *not analysed* and excluded
  from the composite — silence means "not analysed", never zero drift.

**"Score changed but files did not"**

- The effective `azusa.toml` changed (check `provenance.config_digest`) or
  the tool version changed (check `provenance.tool_version`). Scores are
  comparable across runs only under equal digests.

**"Observation looks wrong"**

- Check `strength`. `heuristic` observations (path-similarity renames,
  lexical test inventory, intent signals) are explicitly not facts.
  `fact` observations are content-verified; if one is genuinely wrong,
  that is a bug — file it with the report JSON attached.

## Watch mode operations

- **Reset escalation:** `rm .azusa/watch.json` (safe; watcher starts idle).
- **Busy tree, noisy output:** raise `--interval`; idle cycles are cheap
  (stat-only) but not free.
- **Two watchers on one checkout:** unsupported; interleaved stage
  transitions are harmless but wasteful.
- **Watcher died mid-escalation:** restart it — `watch.json` resumes stage,
  streak and last score.

## Evidence store maintenance

- **Grow control:** rotate during downtime — archive `.azusa/exports/`,
  truncate `history.jsonl` / `events.jsonl`. Analysis never reads the
  store; truncation is safe.
- **Full reset:** `rm -rf .azusa` (loses history, events, exports, watch
  state, hash cache; nothing else).
- **Slow repeat scans:** the hash cache is stat-keyed; after operations
  that touch many mtimes (branch switches, big rebases) the first scan
  re-hashes changed entries and rebuilds speed.

## CI integration failures

- **Exit 3 on a legitimate change:** the policy caught a configured
  violation (major bump, new crate, production manifest). Fix the change or
  make a *targeted, commented, dated* policy exception
  ([policy rules](../reference/policy-rules.md)).
- **Exit 3 with "max_drift" on a huge refactor:** the composite legitimately
  exceeded budget. Split the change, or raise `max_drift` consciously.
- **Flaky gate:** not possible for identical inputs — flakiness means the
  *inputs* moved (upstream advanced mid-pipeline) or the config diverged
  between runs (compare `config_digest`).
- **Shallow clone surprises:** ancestry needs history; use full fetch or
  explicit selectors ([ci-integration](../guides/ci-integration.md)).

## Known boundaries

- Watch analyses the worktree default pair only.
- L8 semantics is planned, not implemented; the `semantic` weight is
  reserved.
- `azusa.inspect.started` is defined in the event contract but not yet
  emitted.
- Single-instance watch per checkout (no state-file locking).

## Escalation to humans (when autonomous)

Autonomous consumers (Kaptaind, CI bots) must escalate to humans when:

- any gate is `blocking` (policy violation, high/critical risk,
  bidirectional drift, unrelated states);
- observations carry `strength: "heuristic"` and the action is destructive
  or hard to reverse;
- exit code 4 repeats on a healthy-looking repository (infrastructure
  problem, not drift).

The standing rule: **Azusa plans; Kaptaind executes** — and anything
that rewrites history or publishes work requires human approval,
always.

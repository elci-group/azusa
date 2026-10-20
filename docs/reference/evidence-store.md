# Evidence Store (`.azusa/`)

Every analysis run appends to a local, **append-only evidence trail** under
`.azusa/` at the repository root. The store is an audit product, not a
source of truth: it never duplicates repository content — records reference
state digests, not file trees.

```text
.azusa/
├── history.jsonl     one record per analysis run (store schema_version 1)
├── events.jsonl      ELCI event log (NDJSON, event schema_version 1)
├── exports/          canonical artefacts: drift reports and reconciliation plans
├── watch.json        watch-mode escalation state
└── cache/
    └── fs-hashes.json   stat-keyed hash cache (incremental scans)
```

`.azusa/` is git-ignored and safe to delete at any time: deleting it loses
history, events, exports, watch escalation state and the hash cache —
nothing else. Analysis is **never** a function of the store: the
deterministic path never reads it.

## Write discipline

All store writes are **best-effort**: analysis never fails because the
store is unwritable (read-only checkout, full disk, permissions). A corrupt
`watch.json` resets the watcher to idle rather than failing it.

## `history.jsonl`

One compact record per analysis run (`azusa-drift`-family commands,
including every watch stage-1+ cycle):

```json
{
  "schema_version": 1,
  "timestamp": "2026-10-20T12:00:00Z",
  "repository": "azusa",
  "state_a": "worktree",
  "state_b": "upstream",
  "direction": "bidirectional",
  "score": 4.25,
  "risk": "high",
  "intent": "probably-accidental",
  "observations": 12,
  "clusters": 5,
  "digest_a": "9f2c…",
  "digest_b": "1a7e…"
}
```

Records are trimmed to what history needs: identity, verdict, counts and
digests. Full evidence for a run lives in the report itself (re-run the
same comparison, or `azusa export` to persist it canonically). Unknown or
malformed lines are skipped on read — history survives partial corruption.

## `events.jsonl`

The ELCI event stream. Format, catalogue and consumption:
[elci-events.md](elci-events.md).

## `exports/`

Canonical artefacts written by `azusa export`, `azusa plan` and watch
stage 2, named with a UTC stamp:

```text
drift-20261020T120000-9f2ca1b3.json    kind: "azusa.drift-report"
plan-20261020T121500-azusa-bidirectional.json
                                       kind: "azusa.reconciliation-plan"
```

Each artefact is an envelope (`schema_version`, `kind`, `timestamp`) around
the full payload (report or plan). Exports are the recommended way to
persist a specific analysis for audit, since a report's JSON output is
reproducible only while the compared states still exist.

## `watch.json`

Watch-mode escalation state (`WatchState`, schema version 1):

| Field | Type | Meaning |
| --- | --- | --- |
| `schema_version` | int | `1`. |
| `last_full_quick_digest` | string | Stat-only worktree fingerprint at the last full analysis. |
| `stage` | int | 0 idle · 1 analysed · 2 escalated. |
| `streak` | int | Consecutive cycles with drift at stage 2 or above. |
| `last_score` | float or null | Overall score of the last full analysis. |
| `last_risk` | string | Risk level of the last full analysis. |

mtime feeds only the stage-0 change detector — never the deterministic
analysis path. See [watch-mode guide](../guides/watch-mode.md).

## `cache/fs-hashes.json`

The incremental-scan cache: `(path, size, mtime) → content hash`. On repeat
scans, files whose stat triple is unchanged are not re-read or re-hashed.
Consequences:

- mtime influences **cache validity only**; it never enters a state or a
  score;
- editing a file without changing its size/mtime (same-second writes) is
  reconciled by content hashing on the next change that *does* surface;
- the cache is a pure speed optimisation — deleting it only costs time.

## Operating notes

| Task | How |
| --- | --- |
| Inspect history | `jq -c . .azusa/history.jsonl | tail` |
| Count runs by risk | `jq -s 'group_by(.risk) \| map({risk: .[0].risk, n: length})' .azusa/history.jsonl` |
| Archive audits | copy `exports/` off-host; envelopes embed provenance |
| Reset the watcher | `rm .azusa/watch.json` |
| Full reset | `rm -rf .azusa` (loses history/events/exports/cache) |
| CI artefacts | upload `exports/*.json`; optionally `history.jsonl` |

## Retention

The store grows append-only and unbounded. For long-lived sentinels
(watch mode), schedule rotation: archive `exports/` and truncate
`history.jsonl` / `events.jsonl` during downtime — consumers should
checkpoint by offset, and the analysis path is indifferent to truncation.

# ELCI Event Contract (schema version 1)

Azusa emits events following the **ELCI universal event contract** so other
ELCI tools — Amber, Fract, Traci, Isopod, Bart — can consume them as a
shared data product. Events are the temporal layer: the deterministic
analysis path never reads them.

- **Format:** NDJSON — exactly one JSON object per line.
- **Sink:** `.azusa/events.jsonl` (append-only, created on demand).
- **Canonical constant:** `EVENT_SCHEMA_VERSION = 1` (`azusa-elci`).
- **Delivery:** best-effort. A failure to write events never fails
  analysis.

## Event catalogue

| Event name | Emitted when |
| --- | --- |
| `azusa.drift.detected` | An analysis found drift (direction not `in-sync` and not `unrelated`). |
| `azusa.drift.threshold_exceeded` | The same, and risk ≥ `high`. Emitted immediately after `azusa.drift.detected`. |
| `azusa.policy.failed` | `azusa policy check` produced at least one violation. |
| `azusa.plan.generated` | A reconciliation plan was generated (`azusa plan`; watch stage-2 transition). |
| `azusa.inspect.completed` | A canonical drift-report artefact was exported (`azusa export`). |
| `azusa.inspect.started` | Defined in the contract; reserved — the current CLI emits it on no path. |

## Event record

```json
{
  "schema_version": 1,
  "event": "azusa.drift.detected",
  "timestamp": "2026-10-20T12:00:00Z",
  "repository": "azusa",
  "state_a": "worktree",
  "state_b": "upstream",
  "severity": "high",
  "score": 4.25,
  "evidence": 12,
  "tool_version": "0.1.39",
  "analysis_mode": "deterministic"
}
```

| Field | Type | Notes |
| --- | --- | --- |
| `schema_version` | int | `1`. |
| `event` | string | One of the catalogue names. |
| `timestamp` | string | RFC 3339 / UTC, second precision, deterministic calendar math. Event layer only — never the analysis path. |
| `repository` | string | Repository name. |
| `state_a` / `state_b` | string | State selectors compared. |
| `severity` | string | Risk level at emission time (`low`…`critical`). |
| `score` | float | Overall drift score at emission time. |
| `evidence` | int | Number of observations backing the event. |
| `tool_version` | string | Emitting azusa version. |
| `analysis_mode` | string | `deterministic` (or `augmented`, reserved). |

## Which commands emit what

| Command | Events |
| --- | --- |
| `drift`, `score`, `explain` (any analysis run) | `azusa.drift.detected`; plus `azusa.drift.threshold_exceeded` when risk ≥ high |
| `policy check` | `azusa.policy.failed` (on violation only) |
| `export` | `azusa.inspect.completed` |
| `plan` | `azusa.plan.generated` |
| `watch` | stage-1 cycles like `drift`; `azusa.plan.generated` on stage-2 transition |
| `status`, `inspect` | none (pure state reads) |

## Consuming the stream

```console
$ tail -f .azusa/events.jsonl | jq -c 'select(.event=="azusa.policy.failed")'
$ jq -s 'map(select(.event=="azusa.drift.detected")) | length' .azusa/events.jsonl
$ jq -s 'group_by(.repository) | map({repo: .[0].repository,
        runs: length, max: map(.score) | max})' .azusa/events.jsonl
```

Integration guidance:

- **Treat lines as immutable facts.** The file is append-only; consumers
  should checkpoint by byte offset, not by rewriting.
- **Gate on `analysis_mode`.** Only `deterministic` events are
  byte-reproducible products of their states.
- **`evidence` is a count, not the data.** Follow the reference to the
  report (history/exports) for the observations themselves.
- **Severity is risk vocabulary** (`low`…`critical`), mirroring
  `DriftRisk.level` at emission time.

## Design notes

- Events intentionally carry **summary scalars only** (name, timestamp,
  repository, selectors, risk, score, evidence count, versions). Full
  evidence lives in reports and exports; the event stream stays compact.
- The contract is shared with the wider ELCI ecosystem; event names are
  namespaced `azusa.*` and additive evolution (new names, new optional
  fields) does not bump `schema_version`.
- Timestamps are produced with deterministic calendar math (no external
  time database), so formatting is stable across platforms.

# Drift Report Schema (JSON contract, version 2)

The drift report is Azusa's primary artefact. Its JSON serialisation is a
**contract**: identical states, configuration and tool version produce
byte-identical output (`analysis_mode: "deterministic"` says so
explicitly). Human output is presentation and may change; this schema
changes only with a version bump.

- **Emitted by:** `azusa drift --json`, `azusa explain --json`,
  `azusa export` (as the `report` payload), `azusa policy check --json`
  (as the `report` payload).
- **Canonical constant:** `DRIFT_REPORT_SCHEMA_VERSION = 2`
  (`azusa-model::drift`).
- **Serde style:** enums are `kebab-case`; field order follows the struct
  definitions below; maps use `BTreeMap` (layer scores are emitted in
  canonical layer order).

## Top-level object

```json
{
  "schema_version": 2,
  "repository":   { "name": "azusa", "root": "/home/sal/azusa" },
  "state_a":      { "spec": "worktree", "provider": "git",
                    "label": "worktree", "digest": "9f2c…" },
  "state_b":      { "spec": "upstream", "provider": "git",
                    "label": "origin/main", "digest": "1a7e…" },
  "direction": "bidirectional",
  "direction_confidence": 0.85,
  "score":       { "overall": 4.25, "layers": { "ancestry": 1.25, "filesystem": 2.31 } },
  "risk": { "level": "high", "rationale": "…", "factors": ["…"] },
  "summary": { "…": 0 },
  "observations": [ /* DriftObservation */ ],
  "intent": { "classification": "probably-accidental", "confidence": 0.62,
              "signals": [ { "description": "…", "weight": -1.0 } ] },
  "recommendations": [ { "action": "…", "rationale": "…", "priority": "high" } ],
  "clusters": [ /* ChangeCluster */ ],
  "provenance": {
    "tool_version": "0.1.39",
    "analysis_mode": "deterministic",
    "captured_at": "2026-10-20T12:00:00Z",
    "config_digest": "b3:…"
  }
}
```

| Field | Type | Notes |
| --- | --- | --- |
| `schema_version` | int | `2`. Bumps on any breaking contract change. |
| `repository` | object | `name` (directory or remote-derived), `root` (nullable). |
| `state_a` / `state_b` | StateRef | Conventionally A = local side, B = remote side. |
| `direction` | enum | `in-sync` · `a-to-b` · `b-to-a` · `bidirectional` · `unrelated` · `unknown`. |
| `direction_confidence` | float 0–1 | Confidence in the direction determination. |
| `score` | DriftScore | Composite index; see below. |
| `risk` | DriftRisk | Classified reconciliation risk with rationale. |
| `summary` | DriftSummary | Headline counters. |
| `observations` | array | Evidence-first facts; the score never replaces these. |
| `intent` | object or null | Present when the intent stage produced an assessment; `null` when it did not run (e.g. no signals, in-sync). |
| `recommendations` | array | Ordered suggested actions. |
| `clusters` | array | Logical change clusters. |
| `provenance` | object | How/when/with what the report was produced. |

## `StateRef`

| Field | Type | Notes |
| --- | --- | --- |
| `spec` | string | The selector requested (by user or defaults). |
| `provider` | enum | `git` · `filesystem`. |
| `label` | string | Human label, e.g. `worktree`, `origin/main`, a path. |
| `digest` | string | Fileset digest of the resolved state (BLAKE3 over the canonical `<mode> <hash> <path>` listing). Two states are identical iff their digests match. |

## `DriftScore`

| Field | Type | Notes |
| --- | --- | --- |
| `overall` | float | 0.0–10.0, rounded to two decimals. Weighted mean over **analysed** layers only. |
| `layers` | map | Layer name → score (0.0–10.0). Only analysed layers appear. |

Layer keys (canonical order): `identity`, `ancestry`, `filesystem`,
`structure`, `dependency`, `configuration`, `interface`, `behaviour`,
`semantic`, `operational`. `identity` is never scored. An absent layer
means "not analysed" — never zero drift.

Formulas for every score: [scoring.md](scoring.md).

## `DriftRisk`

| Field | Type | Notes |
| --- | --- | --- |
| `level` | enum | `low` · `medium` · `high` · `critical`. |
| `rationale` | string | Why this level was chosen. |
| `factors` | array[string] | Contributing factors, traceable to observations. |

Bidirectional drift always floors the level at `medium`.

## `DriftSummary`

| Field | Type | Notes |
| --- | --- | --- |
| `files_added` | int | Present only in state B (absent in A). |
| `files_deleted` | int | Present only in state A (absent in B). |
| `files_modified` | int | Content changed. |
| `files_renamed` | int | Renamed/moved between states. |
| `commits_ahead` | int or null | Git states only. |
| `commits_behind` | int or null | Git states only. |
| `uncommitted_a` | bool | State A carries uncommitted work. |
| `uncommitted_b` | bool | State B carries uncommitted work. |

## `DriftObservation`

```json
{
  "id": "DRIFT-0007",
  "category": "filesystem",
  "severity": "medium",
  "confidence": 1.0,
  "title": "3 file(s) deleted",
  "detail": "…",
  "side": "a",
  "strength": "fact",
  "evidence": [
    { "kind": "file",
      "description": "content hash removed from listing",
      "references": ["src/legacy/auth.rs"] }
  ]
}
```

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Stable id `DRIFT-nnnn`, assigned deterministically **after** the observation list is sorted. Identical inputs → identical ids. |
| `category` | enum | The ten layer names. |
| `severity` | enum | `info` · `low` · `medium` · `high` · `critical` — how much the change matters. |
| `confidence` | float 0–1 | Confidence in the observation. |
| `title` / `detail` | string | Short title; longer explanation. |
| `side` | enum | `a` · `b` · `both` — which side of the comparison. |
| `strength` | enum | `fact` · `derived` · `heuristic` · `speculative` — how the evidence was obtained. Default `fact` (field tolerated missing on deserialize for forward compatibility). |
| `evidence` | array | Machine-checkable backing data. |

### `Evidence`

| Field | Type | Notes |
| --- | --- | --- |
| `kind` | enum | `commit` · `branch` · `file` · `manifest` · `lockfile` · `config` · `test` · `rename` · `path-pattern` · `marker`. |
| `description` | string | Human statement of the evidence. |
| `references` | array[string] | Paths, commit ids, names. Policy rules match on these (e.g. `"serde: 1.0.1 → 2.0.0 (major)"`, references containing `prod`). |

### Strength semantics

| Strength | Obtained by | Examples |
| --- | --- | --- |
| `fact` | verified from content | digests, merge-base, exact-identity renames, manifest/lockfile deltas |
| `derived` | parsed from structured declarations | API symbol identities and signatures |
| `heuristic` | inferred by proxy | lexical test inventory, content-similar and path-heuristic renames, intent signals |
| `speculative` | pattern-based judgement | reserved for semantic layers (L8) |

Consumers gate on strength: autonomous agents act on `fact`/`derived` and
route `heuristic` to human review.

## `IntentAssessment`

| Field | Type | Notes |
| --- | --- | --- |
| `classification` | enum | `intentional` · `probably-intentional` · `indeterminate` · `probably-accidental` · `accidental`. |
| `confidence` | float 0–1 | Derived from signal count. |
| `signals` | array | `{description, weight}`; positive weights support "intentional", negative support "accidental". |

`indeterminate` is a first-class answer. Intent is orthogonal: it never
modifies score or risk. The inference chain is
`drift → risk → intent → policy → plan`.

## `ChangeCluster`

| Field | Type | Notes |
| --- | --- | --- |
| `label` | string | e.g. `crates/azusa-core`, `tests`, `lockfiles`, `ci`. |
| `kind` | enum | `lockfile` · `manifest` · `tests` · `documentation` · `configuration` · `operations` · `module` · `other`. |
| `paths` | array[string] | Changed paths in this cluster (capped listing). |
| `files` / `added` / `deleted` / `modified` / `renamed` | int | Counters. |

## `Provenance`

| Field | Type | Notes |
| --- | --- | --- |
| `tool_version` | string | The `azusa` version that produced the report. |
| `analysis_mode` | enum | `deterministic` (today's only mode) · `augmented` (reserved: deterministic core + recorded, confidence-scored model interpretation). |
| `captured_at` | string or null | RFC 3339 wall-clock capture time. **Display only — never part of scoring.** |
| `config_digest` | string or null | Digest of the effective configuration, for reproducibility. |

## NDJSON dialect

`azusa drift --ndjson` flattens the same report into one record per
observation plus a final summary line:

```json
{"record":"observation","id":"DRIFT-0001","category":"ancestry",
 "severity":"medium","confidence":1.0,"title":"…","detail":"…",
 "side":"both","evidence":[…],"strength":"fact",
 "repository":"azusa","state_a":"worktree","state_b":"upstream"}
{"record":"summary","repository":"azusa","state_a":"worktree",
 "state_b":"upstream","direction":"bidirectional",
 "direction_confidence":0.85,"score":{…},"risk":"high","summary":{…},
 "schema_version":2}
```

## Versioning policy

- Additive optional fields do **not** bump the schema version (consumers
  must ignore unknown fields).
- Renames, removals, semantic changes bump the version and are announced
  in the changelog with migration notes.
- `schema_version: 2` exists for the introduction of `strength`; reports
  deserialize v1 reports as `strength: "fact"`.

## Worked consumer checks

```console
$ azusa drift --json | jq '.direction, .score.overall, .risk.level'
$ azusa drift --json | jq '[.observations[] | select(.severity=="high" or .severity=="critical")]'
$ azusa drift --json | jq '[.observations[] | select(.strength=="heuristic")] | length'
$ azusa drift --ndjson | jq -c 'select(.record=="summary")'
```

# Reconciliation Plan Schema (version 1)

A reconciliation plan is an ordered, evidence-linked sequence of steps that
would reconcile two drifted states, plus the safety gates that must hold
before those steps run. A plan is a **pure function of the drift report and
the policy verdict** — identical reports always produce identical plans.

- **Emitted by:** `azusa plan --json` (as the `plan` payload), watch mode
  stage 2, and the exported artefact `kind: "azusa.reconciliation-plan"`.
- **Canonical constant:** `PLAN_SCHEMA_VERSION = 1` (`azusa-model::plan`).
- **Executor:** none. **Azusa plans; Kaptaind executes.** A plan never
  mutates anything by itself.

## Top-level object

```json
{
  "schema_version": 1,
  "repository": "azusa",
  "state_a": "worktree",
  "state_b": "upstream",
  "direction": "a-to-b",
  "drift_score": 3.4,
  "risk": "medium",
  "intent": "probably-intentional",
  "steps":  [ /* PlanStep, ordered */ ],
  "gates":  [ /* SafetyGate */ ],
  "summary": "commit local work, review tests cluster, push to origin"
}
```

| Field | Type | Notes |
| --- | --- | --- |
| `schema_version` | int | `1`. |
| `repository` | string | Repository name from the report. |
| `state_a` / `state_b` | string | Selectors compared. |
| `direction` | string | The drift direction the plan reconciles. |
| `drift_score` | float | Overall score of the underlying report. |
| `risk` | string | Risk level of the underlying report. |
| `intent` | string or null | Intent classification, when the report carried one. |
| `steps` | array | Ordered `PlanStep`s. |
| `gates` | array | `SafetyGate`s that must hold before execution. |
| `summary` | string | One-line summary. |

## `PlanStep`

```json
{
  "id": "PLAN-0001",
  "kind": "commit",
  "description": "commit 3 uncommitted file(s)",
  "rationale": "state a carries uncommitted work; reconciliation would lose it",
  "risk": "low",
  "requires_approval": false,
  "executed_by": "kaptaind"
}
```

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Stable id `PLAN-nnnn`, in step order. |
| `kind` | enum | See below. |
| `description` | string | What to do. |
| `rationale` | string | Why this step exists (evidence-linked). |
| `risk` | enum | `low` · `medium` · `high` · `critical`. |
| `requires_approval` | bool | **True for every step that rewrites history or publishes work** (`merge`, `rebase`, `push`), and for fast-forward when it would delete files. |
| `executed_by` | string | `kaptaind` (automated) or `human`. |

### Step kinds

| Kind | Meaning |
| --- | --- |
| `noop` | Nothing to do; states agree. |
| `commit` | Commit or stash local work. |
| `fetch` | Fetch the other side. |
| `fast-forward` | Fast-forward to the other side. |
| `rebase` | Rebase local work (rewrites local history). Always requires approval. |
| `merge` | Merge the two sides (creates a merge commit). Always requires approval. |
| `push` | Push local work to the remote. Always requires approval. |
| `review` | Review a change cluster by hand. |
| `run-tests` | Run the test suite. |
| `manual` | Human inspection required; no automated action exists. |

## `SafetyGate`

```json
{
  "id": "GATE-0001",
  "condition": "policy check passes",
  "reason": "a failing policy verdict blocks the plan",
  "blocking": true
}
```

| Field | Type | Notes |
| --- | --- | --- |
| `id` | string | Stable id `GATE-nnnn`. |
| `condition` | string | What must hold. |
| `reason` | string | Why the gate exists. |
| `blocking` | bool | `true`: the plan **may not run** until the gate passes. `false`: advisory — confirm before proceeding. |

## Plan derivation rules

Steps follow the drift direction:

| Direction | Step shape |
| --- | --- |
| `in-sync` | no-op |
| `a-to-b` | (commit if uncommitted) → (review per cluster) → (run-tests if behaviour/interface moved) → push |
| `b-to-a` | fetch → fast-forward (approval required when it would delete files) |
| `bidirectional` | fetch → per-cluster human review → merge → push |
| `unrelated` / `unknown` | manual inspection only |

### Blocking gates (plan not executable until passed)

| Gate | Trigger |
| --- | --- |
| Policy violation | A failing `azusa policy check` blocks the plan. |
| High/critical risk | Risk at or above `high` blocks until a human accepts the risk. |
| Bidirectional drift | Any bidirectional drift means a human merge decision is unavoidable. |
| Unrelated states | Blocks pending manual verification. |

### Advisory gates

| Gate | Trigger |
| --- | --- |
| Accidental-intent drift | Confirm the accidental side is safe to discard. |

`is_executable` is exactly: **every gate has `blocking == false`**.

## Export artefact

`azusa plan` and watch stage 2 write:

```json
{
  "schema_version": 1,
  "kind": "azusa.reconciliation-plan",
  "timestamp": "2026-10-20T12:15:00Z",
  "plan": { "…": "the full plan object" }
}
```

File name: `.azusa/exports/plan-<UTC-stamp>-<repository>-<direction>.json`
(colons and dashes stripped from the stamp).

## Worked consumer checks

```console
$ azusa plan --json | jq '[.plan.gates[] | select(.blocking)] | length'   # 0 ⇒ executable
$ azusa plan --json | jq '[.plan.steps[] | select(.requires_approval)] | map(.kind)'
$ azusa plan --json | jq -r '.plan.summary'
```

Consumers must treat `requires_approval` and `blocking` as hard
preconditions: an executor that skips them is operating outside the
contract — and outside Azusa's safety model
(see [operations/security.md](../operations/security.md)).

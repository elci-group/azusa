# Policy Rules Reference

`azusa policy check` runs the normal deterministic drift analysis, then
audits the **finished report** against the `[policy]` block of
`azusa.toml`. Policies are evidence checks over the report — they never
re-analyse states — so a policy verdict is as deterministic as the report
it audits. Any violation exits **3** and emits `azusa.policy.failed`.

## Rule catalogue

| Rule | Config | Default | Fires when | Severity |
| --- | --- | --- | ---: | --- |
| `policy.max_drift` | `[policy] max_drift` | 5.0 | overall score **>** maximum | high |
| `policy.dependencies.allow_major` | `[policy.dependencies] allow_major` | `false` | any dependency transitioned across a **major** boundary | high |
| `policy.dependencies.allow_new` | `[policy.dependencies] allow_new` | `true` | a dependency is **declared only on one side** | medium |
| `policy.structure.allow_new_crates` | `[policy.structure] allow_new_crates` | `true` | **modules present only on one side** | medium |
| `policy.configuration.production_changes=forbid` | `[policy.configuration] production_changes` | `"RequireReview"` | production-facing configuration changed | **critical** |
| `policy.configuration.production_changes=require-review` | same | — | production-facing configuration changed | **medium** |
| `policy.configuration.production_changes=allow` | same | — | never fires (changes permitted silently) | — |

Configuration reference with examples:
[guides/configuration.md](../guides/configuration.md#policy--ci-gate-rules).

## Violation records

```json
{
  "id": "POLICY-0001",
  "rule": "policy.dependencies.allow_major",
  "description": "1 major dependency bump(s) not permitted",
  "severity": "high",
  "references": ["serde: 1.0.1 → 2.0.0 (major)"]
}
```

- **Stable ids.** Violations are sorted by severity (descending), then rule
  name (ascending), then numbered `POLICY-0001`… — identical reports always
  produce identical ids.
- **References** point at the offending evidence (dependency transitions,
  configuration paths, layer scores for `max_drift`), so a violation can be
  verified against the report without re-running anything.

## What counts as "production-facing" configuration

A configuration-layer observation is production-facing when **either**:

1. its title names an operational configuration kind — `dockerfile`,
   `kubernetes`, `systemd-unit`, `compose` — or
2. any of its evidence references contains `prod`.

The kinds are derived from the shared `ConfigurationKind` vocabulary, so
the policy can never drift from the classifier's canonical names.
Non-production kinds (formatter, linter, environment, cargo-manifest, plain
CI) never trip the rule.

## Enforcement semantics

- **Gate, not score.** Policy verdicts do not alter the drift score or
  risk; they are a separate verdict layered over the report. `policy check`
  exits 3 on any violation *regardless of drift risk*; when compliant it
  exits with the normal drift mapping.
- **Evidence check.** The engine reads observations from the report; it
  performs no I/O on repository states.
- **Determinism.** Same report + same policy ⇒ same verdict, same ids.

## The inference chain

```text
drift (what changed) → risk (consequences) → intent (why we suspect it)
                     → policy (is this permitted) → plan (what happens next)
```

Policy is the fourth link: a failing verdict **blocks** any reconciliation
plan (a blocking gate), and `azusa plan` embeds the policy outcome via that
gate. See [plan-schema.md](plan-schema.md#blocking-gates-plan-not-executable-until-passed).

## Worked examples

**Major bump under defaults** (`allow_major = false`):

```console
$ azusa policy check
POLICY CHECK — azusa
────────────────────────────────────────────
drift: 3.40/10  risk: medium
violations: 1 — NON-COMPLIANT
  POLICY-0001 [high] policy.dependencies.allow_major:
        1 major dependency bump(s) not permitted
        serde: 1.0.1 → 2.0.0 (major)
$ echo $?
3
```

**Production manifest edited under `RequireReview`:**

```text
POLICY-0001 [medium] policy.configuration.production_changes=require-review:
      production-facing configuration changed: 1 configuration changed: kubernetes
      ~k8s/prod/deploy.yml
```

**Strict profile** — `production_changes = "Forbid"` turns the same
observation critical; `max_drift = 4.0` adds a score violation when the
composite exceeds 4.

## Tuning guidance

| Goal | Change |
| --- | --- |
| Strict score budget for release branches | lower `max_drift` (e.g. 3.0–4.0) |
| Freeze dependency surface during a release | `allow_major = false`, `allow_new = false` |
| Freeze architecture during a migration | `allow_new_crates = false` |
| Protect production manifests | keep `RequireReview`; use `Forbid` only where review is impossible |
| Documented exceptions | raise the specific knob with a comment and a review date — never disable the whole gate |

Anti-pattern: raising `max_drift` to silence violations. The gate then
masks exactly the drift it exists to catch; prefer targeted, commented,
dated exceptions.

## Machine consumption

```console
$ azusa policy check --json | jq '.compliant'
$ azusa policy check --json | jq -c '.violations[] | {id, rule, severity}'
```

The JSON payload is `{"compliant": bool, "violations": [...],
"report": {…}}` — the report is embedded so consumers can verify every
violation against its evidence in one read.

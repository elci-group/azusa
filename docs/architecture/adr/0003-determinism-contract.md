# ADR 0003 — The Determinism Contract

- **Status:** Accepted
- **Date:** 2026-09-14 (retroactive record)
- **Deciders:** Azusa core maintainers

## Context

Azusa's output is consumed by machines: CI gates, policy verdicts,
reconciliation plans, autonomous agents. A report that could differ
between two runs of the same comparison is useless for gating — flapping
verdicts are worse than no verdict. Time, locale, environment and
concurrency are the usual culprits.

## Decision

**Determinism is a contract**: given identical states, configuration and
tool version, the report is byte-identical.

- Scores come from fixed saturating formulas with documented, unit-tested
  constants; no wall-clock, randomness or environment reads in the
  analysis path.
- Observation, violation, step and gate ids are assigned **after stable
  sorts**, so identical inputs yield identical ids.
- Timestamps exist only in provenance (`captured_at`) and the event layer —
  never inside scoring, ordering or ids.
- mtime drives only cache validity and watch-mode's stage-0 change
  detector, never state content or scores.
- Reports declare `analysis_mode: "deterministic"` explicitly, so a future
  `augmented` mode can never be mistaken for a pure result.
- Floating-point results are rounded deterministically (`round2`) for JSON
  stability.

## Consequences

- `azusa drift --json | jq …` is a stable integration surface; consumers
  can hash reports.
- Features that want temporal behaviour (watch, events, history) must
  quarantine time at the edges — the loop is temporal, the analysis never
  is.
- Any change to a constant, formula or ordering is a contract change: it
  needs tests, and where outputs change, a documented note
  ([reference/scoring.md](../../reference/scoring.md)).

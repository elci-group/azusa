# ADR 0002 — Evidence-First Observations

- **Status:** Accepted
- **Date:** 2026-09-12 (retroactive record)
- **Deciders:** Azusa core maintainers

## Context

Scoring tools that emit only numbers are unauditable: a "7.2/10 drift"
with no receipts cannot be verified, disputed, or acted on safely —
especially by autonomous consumers. If the score is the artefact, every
consumer must trust the scorer blindly, and every scoring change silently
changes downstream verdicts.

## Decision

Every layer emits **observations** — evidence-first facts with stable ids,
category, severity, confidence, side, concrete evidence (kind, description,
machine-checkable references) and epistemic strength. **The score never
replaces the evidence that produced it**: the composite index is always
accompanied by the observations behind it, in the same artefact. Scores
are computed *from* observation counts via documented formulas; they are a
view, not the record.

## Consequences

- Any number in a report can be traced to checkable evidence
  (`azusa explain`).
- Policy rules, plans and intents reference observations rather than raw
  scores, so verdicts remain grounded when weights change.
- Reports are larger than a bare score — accepted; the NDJSON dialect and
  summary fields serve consumers who want only the headline.
- Documentation and tests must keep formulas and evidence aligned; the
  scoring reference ([reference/scoring.md](../../reference/scoring.md)) is
  part of the contract.

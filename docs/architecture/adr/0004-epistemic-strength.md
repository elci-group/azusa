# ADR 0004 — Epistemic Strength on Every Observation

- **Status:** Accepted
- **Date:** 2026-10 (schema version 2)
- **Deciders:** Azusa core maintainers

## Context

Severity says how much a change matters; it says nothing about how much
the *claim* can be trusted. A content-hash mismatch (`fact`) and a
path-similarity rename guess (`heuristic`) may both be "medium", but an
autonomous agent should not act on them identically. Collapsing trust into
severity forces conservative behaviour everywhere or reckless behaviour
somewhere.

## Decision

Every observation carries an orthogonal **`EvidenceStrength`**:
`fact` (verified from content — digests, merge-base, exact identity),
`derived` (parsed from structured declarations — symbol tables,
manifests), `heuristic` (inferred by proxy — lexical inventories,
path similarity, intent signals), `speculative` (pattern-based judgement,
reserved for semantic layers). Strength never modifies severity, confidence
or scores. Consumers gate on strength: autonomous consumers act on
`fact`/`derived` and route `heuristic` to human review (e.g. Kaptaind's
contract). Its introduction bumped the report schema to **version 2**, with
deserialize-level tolerance for reports missing the field (defaulting to
`fact`).

## Consequences

- Trust-based automation policies become expressible without duplicating
  Azusa's internals.
- Rename tiers align with strength: `exact-identity` renames are facts,
  weaker tiers are heuristics — and scoring discounts accordingly.
- The four-grade scale is deliberately coarse; finer grades would invite
  false precision. `speculative` exists so future semantic layers cannot
  quietly masquerade as facts.
- Documentation must keep the strength vocabulary visible wherever
  heuristics are described (behavioural inventory, rename detection,
  intent).

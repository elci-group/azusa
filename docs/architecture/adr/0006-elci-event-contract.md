# ADR 0006 — ELCI Events as a Shared Data Product

- **Status:** Accepted
- **Date:** 2026-10 (phase 3)
- **Deciders:** Azusa core maintainers

## Context

Drift intelligence is more useful when it feeds a wider toolchain (Amber,
Fract, Traci, Isopod, Bart…). Ad-hoc per-tool integrations produce N×M
glue, and proprietary webhooks couple consumers to Azusa's release cycle.
The ecosystem already shares a universal event contract — ELCI.

## Decision

Emit **ELCI universal event contract** records (schema version 1) as NDJSON
into the local evidence trail (`.azusa/events.jsonl`): namespaced event
names (`azusa.drift.detected`, `azusa.drift.threshold_exceeded`,
`azusa.policy.failed`, `azusa.plan.generated`, `azusa.inspect.completed`;
`azusa.inspect.started` reserved), each carrying summary scalars only —
timestamp, repository, selectors, risk, score, evidence count, tool
version, analysis mode. Full evidence stays in reports and exports; the
stream stays compact. Events are the **temporal layer**: the deterministic
analysis path never reads the store, and all writes are best-effort.

## Consequences

- Any ELCI-speaking tool consumes Azusa's findings with zero Azusa-specific
  glue; conversely, Azusa's event names are a public vocabulary — additive
  evolution only (new names/fields), breaking changes bump the schema.
- A local append-only file is the lowest-common-denominator transport:
  offline-friendly, inspectable with `jq`/`tail`, trivially shipped by any
  forwarder.
- Best-effort delivery means the stream is an audit convenience, not a
  guarantees-grade queue; consumers needing delivery guarantees must tail
  durably and checkpoint offsets.
- Timestamp precision is seconds (deterministic calendar math); consumers
  needing ordering must not rely on intra-second monotonicity.

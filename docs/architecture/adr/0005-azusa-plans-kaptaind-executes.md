# ADR 0005 — Azusa Plans; Kaptaind Executes

- **Status:** Accepted
- **Date:** 2026-10 (phase 6)
- **Deciders:** Azusa core maintainers

## Context

A drift tool that can reconcile drift is two tools: a measurement instrument
and a repository mutator. Mutating repositories autonomously — merging,
rebasing, pushing, resetting — is exactly the power that makes a tool too
dangerous to run widely, and turns every analysis bug into a potential
destructive operation. The alternative (a purely advisory tool) loses the
ability to close the loop safely.

## Decision

Split the loop at the sharpest possible boundary: **Azusa observes,
analyses and plans; it never executes repository mutations.** The planner
emits a deterministic `ReconciliationPlan` — ordered steps with explicit
risk, rationale and approver — plus structural safety gates. Execution
belongs to **Kaptaind**, which enforces:

- `requires_approval` on every step that rewrites history or publishes
  work (`merge`, `rebase`, `push`; fast-forward when it would delete
  files);
- blocking gates before any execution: policy violations, high/critical
  risk, any bidirectional drift, unrelated states;
- advisory gates (accidental-intent drift) requiring confirmation.

Destructive operations (force push, hard reset, branch deletion, history
rewriting) are refused by policy unless explicitly authorised — and they
are never *Azusa's* action to take under any policy.

## Consequences

- The planner can be aggressive in *recommending* because it is incapable
  of *doing*; plans are pure functions of the report and are byte-stable.
- Every step names its approver (`kaptaind` or `human`), so accountability
  is an artefact property, not a runtime convention.
- A compliant-but-wrong plan still cannot mutate anything without an
  executor choosing to run it — the executor is an independent trust
  boundary.
- Cost: reconciliation is never one-command convenience. Accepted; drift
  reconciliation should be deliberate.

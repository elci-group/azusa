# Azusa Documentation Anthology

This directory is the canonical documentation set for **Azusa — repository
drift intelligence**. It complements the top-level [`README.md`](../README.md)
(product tour) and [`ARCHITECTURE.md`](../ARCHITECTURE.md) (system design)
with role-oriented guides, machine-contract references, decision records and
operational material.

> How different is the repository I have from the repository I believe I
> have? Azusa measures, classifies, explains and plans the reconciliation of
> that difference — deterministically, locally, and without ever mutating
> your repository.

## Document map

| Document | Audience | Purpose |
| --- | --- | --- |
| [overview.md](overview.md) | everyone | What Azusa is, the mental model, key concepts, glossary |
| [guides/installation.md](guides/installation.md) | operators | Build, install, update, versioning (`baby` / `theosis`) |
| [guides/quickstart.md](guides/quickstart.md) | new users | First fifteen minutes: every command against a live repository |
| [guides/cli-reference.md](guides/cli-reference.md) | all users | Commands, flags, state selectors, output dialects, exit codes |
| [guides/configuration.md](guides/configuration.md) | power users | Complete `azusa.toml` reference with defaults and semantics |
| [guides/ci-integration.md](guides/ci-integration.md) | release engineers | Using Azusa as a CI gate (GitHub Actions and generic recipes) |
| [guides/watch-mode.md](guides/watch-mode.md) | operators | Continuous drift monitoring and staged escalation |
| [reference/drift-report-schema.md](reference/drift-report-schema.md) | integrators | The canonical JSON contract (schema version 2), field by field |
| [reference/plan-schema.md](reference/plan-schema.md) | integrators | Reconciliation plan artefact (schema version 1) |
| [reference/elci-events.md](reference/elci-events.md) | integrators | ELCI event contract, event catalogue, consumption guidance |
| [reference/evidence-store.md](reference/evidence-store.md) | integrators, operators | The `.azusa/` evidence trail: history, events, exports, watch state, cache |
| [reference/scoring.md](reference/scoring.md) | auditors | Every scoring formula, weight and severity rule, with rationale |
| [reference/policy-rules.md](reference/policy-rules.md) | release engineers | Policy rules, violation ids, severity matrix, examples |
| [architecture/adr/](architecture/adr/) | architects | Architecture decision records |
| [operations/runbook.md](operations/runbook.md) | operators | Exit-code triage, failure modes, recovery procedures |
| [operations/security.md](operations/security.md) | security reviewers | Safety model, threat model, vulnerability reporting |
| [development/contributing.md](development/contributing.md) | contributors | Workspace layout, invariants, testing and review standards |
| [development/release.md](development/release.md) | maintainers | Version authority, release checklist, update channel |

## Reading paths

- **I want to try it** → [installation](guides/installation.md) →
  [quickstart](guides/quickstart.md).
- **I wire tools into pipelines** → [cli-reference](guides/cli-reference.md) →
  [ci-integration](guides/ci-integration.md) →
  [drift-report-schema](reference/drift-report-schema.md).
- **I build on Azusa's output** → [drift-report-schema](reference/drift-report-schema.md)
  → [elci-events](reference/elci-events.md) →
  [evidence-store](reference/evidence-store.md).
- **I run Azusa for a team** → [watch-mode](guides/watch-mode.md) →
  [runbook](operations/runbook.md) → [configuration](guides/configuration.md).
- **I audit the scoring and policy decisions** →
  [scoring](reference/scoring.md) → [policy-rules](reference/policy-rules.md) →
  the [ADR set](architecture/adr/).
- **I want to change Azusa itself** → [contributing](development/contributing.md)
  → [ARCHITECTURE.md](../ARCHITECTURE.md) → [release](development/release.md).

## Documentation invariants

These rules keep the anthology trustworthy; contributors are expected to
uphold them (see [contributing](development/contributing.md)):

1. **The code is the truth.** Every documented default, threshold, exit code
   and schema field must match the source; when they disagree, fix the docs
   or the code in the same change.
2. **JSON is the contract; human output is presentation.** Contract documents
   describe exactly what the serialisers emit.
3. **Determinism claims are testable claims.** Any statement of the form
   "identical inputs produce identical output" must be backed by a test.
4. **Azusa plans; Kaptaind executes.** Documentation never implies that
   Azusa performs repository mutations.
5. **Docs travel with behaviour changes.** `README.md`, `ARCHITECTURE.md` and
   the affected pages here are updated in the same commit.

## Related documents at the repository root

- [`README.md`](../README.md) — product overview, quickstart, roadmap.
- [`ARCHITECTURE.md`](../ARCHITECTURE.md) — crate map, layer design, determinism.
- [`SECURITY.md`](../SECURITY.md) — security policy (supported versions, reporting).
- [`LICENSE`](../LICENSE) · [`NOTICE`](../NOTICE) ·
  [`THIRD_PARTY_NOTICES`](../THIRD_PARTY_NOTICES) — licensing.
- [`AGENTS.md`](../AGENTS.md) — guidance for automated agents working in this
  repository.

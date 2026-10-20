# Contributing to Azusa

Development guide for working in this repository. The product contract
lives in [README.md](../../README.md) and [ARCHITECTURE.md](../../ARCHITECTURE.md);
the documentation standards live in [docs/README.md](../README.md).

## Workspace layout

Rust workspace, ten crates under `crates/`, layered strictly:

```text
azusa-model     domain types: RepositoryState, DriftReport, observations, plans. No I/O.
azusa-diff      fileset diff, rename detection (tiers), line deltas, clustering
azusa-fs        filesystem provider: ignore-aware walk, BLAKE3 hashing, stat-keyed cache
azusa-git       git provider: worktree/index/commit/upstream/remote snapshots (libgit2)
azusa-graph     structural analysis: topology, dependencies, config/behaviour/operations facets
azusa-core      comparison engine: layers L0–L9, scoring, direction, intent, risk, plans, config
azusa-policy    policy engine (evidence checks over finished reports)
azusa-elci      ELCI event contract, export artefacts
azusa-store     local append-only evidence store (.azusa/)
azusa-cli       the azusa binary
```

Dependency direction: `model` ← `diff`/`fs`/`git` ← `graph` ← `core` ←
`policy`/`elci`/`store` ← `cli`. Do not introduce upward or cross-cutting
dependencies without an ADR.

## Invariants (break these and the product breaks)

1. **Determinism.** No timestamps, randomness, environment reads or
   unordered iteration in the analysis path. Ids are assigned after stable
   sorts. See [ADR 0003](../architecture/adr/0003-determinism-contract.md).
2. **Evidence first.** Scores derive from observations; a score must never
   appear without the observations that produced it.
3. **Azusa plans; Kaptaind executes.** No code path mutates repository
   state — no checkouts, resets, commits, pushes. Ever.
4. **Silence ≠ zero.** Unanalysed layers are excluded from composites,
   never counted as zero drift.
5. **JSON is the contract.** Changes to serialised shapes bump
   `DRIFT_REPORT_SCHEMA_VERSION` / `EVENT_SCHEMA_VERSION` /
   `PLAN_SCHEMA_VERSION` per the versioning rules in
   [reference/drift-report-schema.md](../reference/drift-report-schema.md).
6. **Strength honesty.** Anything inferred by proxy is `heuristic`, never
   `fact` ([ADR 0004](../architecture/adr/0004-epistemic-strength.md)).
7. **Best-effort periphery.** Store/event failures never fail analysis.

## Workflow

```console
$ cargo fmt --all
$ cargo clippy --workspace --all-targets -- -D warnings
$ cargo test --workspace
```

All three must pass; CI enforces them plus the checks below
(`.github/workflows/ci.yml`):

1. `cargo fmt --all --check`
2. version authority check: `./scripts/sync-version.sh` then
   `git diff --exit-code Cargo.toml`
3. `cargo clippy --workspace --all-targets -- -D warnings`
4. `cargo test --workspace`
5. **Azusa gates itself**: `azusa policy check --quiet` must not exit ≥ 3.

Commit only what you intend to commit; never commit secrets or the
generated state directories (`.azusa/`, `.theosis/`, `.kaptaind/`, `target/`
are ignored).

## Code conventions

- MIT copyright header on every source file
  (`// Copyright (c) 2026 The azusa Authors` + SPDX line).
- Crate-level `//!` docs stating responsibility and contract; every public
  item documented; `/// # Errors` sections on fallible public functions.
- No logic comments where code can say it; comments carry *why*, not *what*.
- Naming follows the domain vocabulary (`state`, `observation`, `evidence`,
  `strength`, `tier`, `gate`) — the glossary in [docs/overview.md](../overview.md)
  is the shared language; rename code terms and docs together.
- Enums carry `as_str()` with the canonical kebab-case serialisation —
  that string is contract; changing it is a schema change.

## Testing standards

- `cargo test --workspace` is the gate for any change (repository policy).
- Unit-test every scoring constant and formula (`azusa-core::scoring` is
  the reference style).
- Git behaviour tests build real fixture repositories
  (`azusa-git::testutil::FixtureRepo`), including remote fixtures for
  upstream/remote selectors.
- CLI integration tests live in `crates/azusa-cli/tests/cli.rs` and assert
  on **JSON contracts and exit codes**, not prose.
- Determinism tests assert byte-equality of serialised reports for
  identical inputs.
- Golden expectations for policy ids and plan shapes keep the stable-id
  guarantees honest.

## Documentation duties

Docs are part of the definition of done. A behaviour change updates, in
the same change:

- `README.md` (user-visible behaviour) and `ARCHITECTURE.md` (design);
- the affected pages under `docs/` — especially
  [guides/cli-reference.md](../guides/cli-reference.md),
  [guides/configuration.md](../guides/configuration.md) and the
  [reference/](../) contracts;
- the [scoring reference](../reference/scoring.md) for any constant or
  formula change, alongside its unit tests;
- an [ADR](../architecture/adr/) for any decision that constrains future
  changes.

Documentation invariants (code is the truth; contracts match serialisers;
determinism claims are test-backed) are listed in
[docs/README.md](../README.md#documentation-invariants).

## Extending the system

| Change | Touch points |
| --- | --- |
| New comparison layer | `azusa-model::Layer`, engine layer runner, scoring function + tests, weights default in config, docs (`Layer` order, scoring reference) |
| New policy rule | `azusa-policy` check + tests (including stable-id ordering), `[policy]` config surface, policy reference, README policy section |
| New plan step kind | `azusa-model::plan` enum + `as_str`, derivation rules in `azusa-core::plan`, plan schema docs |
| New event | `azusa-elci::events` constant, emission site, event catalogue docs (additive — no schema bump) |
| New state selector | `azusa-git::selector` or `StateResolver`, resolution tests, selector table in the CLI reference |
| New CLI flag/command | `azusa-cli` parser, command implementation, output dialects, exit-code table, CLI reference |

## Local quality tooling

The repository is itself measured by the ecosystem (see
`docs/uni-*.md` for historical analyses): `fract` (module health),
`amber` (dependency audit), `ferret` (review findings). Keep `clippy
--pedantic` clean for library crates; the historical baseline is zero
warnings.

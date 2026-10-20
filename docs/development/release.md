# Release and Versioning

How Azusa versions, releases and updates.

## Version authority

The root **`VERSION`** file is the single version authority (currently
`0.1.39`):

- `scripts/sync-version.sh` mirrors it into `Cargo.toml`'s
  `[workspace.package] version`; member crates use
  `version.workspace = true`.
- The binary embeds it; `azusa --version` self-reports the same number.
- `theosis` reconciles the installed binary against this number.
- CI **fails** if `VERSION` and `Cargo.toml` diverge: every version bump
  must land together with its Cargo propagation
  (`git diff --exit-code Cargo.toml` after running the script).

## Version scheme

Semantic versioning, `MAJOR.MINOR.PATCH`. While in `0.x`, the minor
segment carries breaking semantics:

| Bump | When |
| --- | --- |
| patch | fixes with no contract change |
| minor | new features; **any breaking contract change** (schema bumps, default changes, exit-code semantic changes) |

Contract bumps ride explicit schema versions
(`DRIFT_REPORT_SCHEMA_VERSION`, `EVENT_SCHEMA_VERSION`,
`PLAN_SCHEMA_VERSION`) — a code release that changes a serialised shape
must bump the corresponding schema and document migration notes
([reference/drift-report-schema.md](../reference/drift-report-schema.md#versioning-policy)).

## Release checklist

1. `cargo test --workspace` green; `cargo clippy --workspace
   --all-targets -- -D warnings` clean; `cargo fmt --all --check` clean.
2. Behaviour changes documented: `README.md`, `ARCHITECTURE.md`, affected
   `docs/` pages (see [contributing](contributing.md#documentation-duties)).
3. Bump root `VERSION`; run `./scripts/sync-version.sh`; commit both
   together.
4. Confirm the self-gate: `azusa policy check --quiet` on the release tree
   exits < 3.
5. Build and install through the supported channel:
   `baby --user` (recipe `.baby.toml`).
6. Verify the installed binary self-reports the new version:
   `azusa --version`.
7. Tag/annotate per your release process; the kaptaind-managed versioning
   flow derives change analysis from the working tree and history.

## Distribution and updates

- **Install:** `baby --user` builds and installs the static binary to
  `~/.local/bin` ([installation guide](../guides/installation.md)).
- **Update:** `theosis update` clones the default branch into a
  transaction directory, reads root `VERSION`, builds via `baby`, installs
  and verifies before declaring success. It **never downgrades
  automatically** (exit 10 when installed > remote).
- **Verify:** `theosis check` compares installed vs remote without
  changing anything.

## Compatibility promises

- **JSON contracts.** `drift --json` (schema 2), plans (schema 1), events
  (schema 1), history records (schema 1): additive fields do not bump;
  renames/removals/semantic changes do, with migration notes.
- **Exit codes.** 0/1/2/3/4/5 are stable; `[exit]` thresholds remap only
  the drift codes by explicit configuration.
- **Layer and event names.** kebab-case `as_str()` values are contract
  (`behaviour` remains the L7 id even as semantics evolve).
- **Determinism.** Same states + config + version ⇒ byte-identical
  reports; any formula/constant change is a behaviour change requiring
  tests and documentation updates.

## Hotfix flow

Fix on top of the current minor; patch bump; expedite only steps 1, 3, 5,
6 of the checklist. If the hotfix touches a contract, it is by definition
not a hotfix — follow the full checklist.

# AGENTS.md

Guidance for agents working in this repository.

## Project

Azusa — repository drift intelligence. Rust workspace under `crates/`.
Measures, classifies, explains and plans reconciliation of drift between
repository states. See `README.md` and `ARCHITECTURE.md`.

## Policies

- Never commit secrets, API keys, or credentials. Use environment variables
  or a secret manager.
- Run `cargo test --workspace` before considering a change complete.
- Keep `README.md` and `ARCHITECTURE.md` in sync with behavior changes.
- Determinism is a contract: scoring, observation IDs and JSON output must be
  reproducible for identical inputs. No timestamps or randomness inside the
  deterministic analysis path.
- Never silently execute destructive Git operations (force push, hard reset,
  branch deletion, history rewriting). Azusa plans; Kaptaind executes.

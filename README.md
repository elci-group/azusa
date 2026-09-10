# Azusa

**Repository drift intelligence.**

> How different is the repository I have from the repository I believe I have?

Azusa compares arbitrary repository states — local worktree, index, HEAD,
upstream branch, remote branch, or plain filesystem trees — and produces a
quantified, explainable model of how they have drifted apart: what changed,
in which direction, how severe it is, whether it looks intentional, and what
to do about it.

It is **not** a `git diff` wrapper. Git is one state provider among several;
the fundamental abstraction is the *state* — identity, ancestry, filesystem,
structure, dependencies, configuration, interfaces, behaviour — and the
*drift model* produced by comparing two of them.

## Status

Work in progress — building toward the v1 acceptance criteria. See
`ARCHITECTURE.md` for the layer model and `ROADMAP` section below for phase
status.

| Phase | Scope | Status |
| --- | --- | --- |
| 1 | Foundation: providers, fileset comparison, ancestry, basic scoring, CLI, JSON | 🚧 |
| 2 | Structural intelligence: renames, module topology, dependency & config comparison, clustering | ⬜ |
| 3 | ELCI intelligence: policy engine, intent detection, events, export, plans | ⬜ |
| 4 | Behavioural drift: tests, build, interfaces | ⬜ |
| 5 | Operational drift: deployment/service configuration | ⬜ |
| 6 | Reconciliation planning with safety gates | ⬜ |
| 7 | Continuous drift: watch mode, events | ⬜ |

## License

MIT — see `LICENSE`.

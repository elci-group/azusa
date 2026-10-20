# ADR 0001 — States, Not Git Trees

- **Status:** Accepted
- **Date:** 2026-09-10 (retroactive record)
- **Deciders:** Azusa core maintainers

## Context

Repository-difference tooling overwhelmingly treats Git as the universe:
diff two commits, two trees, two refs. That framing fails for a class of
operational questions that matter — uncommitted work, ignored build
artefacts, a directory that is not a repository at all (an extracted
release, a staging checkout), mixed comparisons (my worktree vs an old
copy on disk), and repositories whose remotes have been rewritten.
A `git diff` wrapper cannot answer *"how does my working reality differ
from where the team thinks we are?"*

## Decision

The fundamental abstraction is the **state**: a captured snapshot carrying
identity, ancestry, filesystem listing (BLAKE3 hashes) and enriched
structural facets. Git is one **provider** among several; the filesystem
provider captures any directory. Selectors (`worktree`, `index`, `head`,
`upstream`, `remote`, any libgit2 revision, any directory path) resolve
uniformly through `StateResolver`, and every comparison is
state-vs-state — never ref-vs-ref.

## Consequences

- Mixed comparisons (Git state vs plain directory) are first-class, not
  hacks.
- Uncommitted and ignored-file reality is representable, because the
  worktree state is captured as content, not derived from commits.
- Ancestry facts are *optional* facets (`commits_ahead: null` for
  filesystem states) rather than structural assumptions.
- The diff engine is provider-agnostic; adding a tarball or container-image
  provider later requires no engine changes.
- Cost: states are captured in full before comparison (memory and hashing
  cost), mitigated by the stat-keyed incremental cache and streaming hash
  of large files.

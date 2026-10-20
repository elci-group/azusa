# Security and Safety Model

This document describes Azusa's safety posture for security reviewers and
operators. For the disclosure process, see [`SECURITY.md`](../../SECURITY.md).

## The prime directive

**Azusa observes, analyses and plans. It never executes repository
mutations.**

- No command mutates the working tree, the index, refs or remotes.
- Reconciliation plans are inert data (`ReconciliationPlan` artefacts);
  execution belongs to Kaptaind, a separate trust boundary.
- Steps that rewrite history or publish work (`merge`, `rebase`, `push`;
  fast-forward that would delete files) carry `requires_approval: true` —
  a structural property of the artefact, enforced by the executor.
- Destructive operations (force push, hard reset, branch deletion, history
  rewriting) are refused unless explicitly authorised by policy — and are
  never Azusa's action under any policy.

## Writes: everything, and only, under `.azusa/`

| Path | Content | Risk |
| --- | --- | --- |
| `.azusa/history.jsonl` | run summaries | none; contains no file content |
| `.azusa/events.jsonl` | ELCI events | none; scalars only |
| `.azusa/exports/` | full reports/plans | **may embed paths, commit ids, dependency names** — treat as internal data when sharing |
| `.azusa/watch.json` | escalation state | none |
| `.azusa/cache/fs-hashes.json` | `(path, size, mtime) → hash` | **paths and hashes** — hash discloses *that* two contents are equal, not the content |

The directory is git-ignored. All writes are best-effort: an unwritable
store degrades the audit trail, never the analysis.

## Threat model

| Threat | Posture |
| --- | --- |
| Malicious repository content | Parsed with structured parsers (TOML, manifests, lockfiles), never evaluated; no shells out, no code execution from analysed data. Retained text is size-capped (≤ 128 KiB/file) UTF-8 read for classification only. |
| Malicious `azusa.toml` | Strict parsing; unknown fields rejected; parse failure is a loud exit 4, never a silent fallback. The config digests into provenance. |
| Network attack surface | **None.** No network access anywhere in the analysis path; local-first by design. Update tooling (`theosis`) is the only component that clones the repository's own origin. |
| Supply chain | Small dependency surface (libgit2, serde, toml, blake3, rayon, clap); Dependabot weekly; workspace builds pinned by `Cargo.lock`. |
| Tampering with `.azusa/` | The store is evidence, not authority: analysis never reads it, so poisoning it cannot alter verdicts. Reports embed provenance (tool version, analysis mode, config digest) for integrity review. |
| Plan misuse | Plans are inert until an executor runs them; executors must enforce `requires_approval` and `blocking` gates. A plan with any blocking gate is *not executable* by contract. |
| Determinism subversion (hidden model calls) | Reports declare `analysis_mode: "deterministic"`; there are no LLM or network calls in the binary to subvert. `augmented` mode is reserved and, if ever introduced, must be recorded and confidence-scored per its contract. |

## Data sensitivity

Reports and exports contain:

- file **paths** (including potentially sensitive names, e.g. `deploy/prod/...`);
- dependency names and versions;
- commit ids and branch names;
- public API symbol names;
- content **hashes** (BLAKE3) — equality oracles, not content;
- repository name and root path;
- configuration digest and tool version;
- capture timestamp (provenance only).

They never contain file *content*. Treat exports as internal artefacts:
strip or review before attaching to public trackers.

## Secrets handling

Azusa never reads, emits or stores credential material. Hashes and
metadata only. If a secret is present in a *path name* it can appear in a
report reference — do not encode secrets in paths.

## Autonomous-consumer contract

Tools acting on Azusa output should:

1. gate on `strength` — act on `fact`/`derived`, review `heuristic`;
2. treat `blocking` gates as absolute preconditions;
3. require human approval for every `requires_approval` step;
4. verify provenance (`analysis_mode`, `tool_version`, `config_digest`)
   before acting on a report;
5. never act on reports with a different config digest than the policy
   they believe they are enforcing.

## Vulnerability reporting

Report privately per [`SECURITY.md`](../../SECURITY.md): do not open
public issues with exploit details. Include affected versions,
reproduction steps and impact. Security fixes target the latest supported
release.

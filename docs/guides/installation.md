# Installation and Updates

Azusa ships as a **single static binary** with no runtime dependencies, no
daemon and no network access. This guide covers building, installing,
updating and verifying.

## Requirements

- Rust (stable toolchain) — build time only.
- A Git repository to analyse is optional: directory-vs-directory
  comparisons work without any Git present. Git state providers use
  libgit2, compiled in — no system `git` required at runtime.

## Install with `baby` (recommended)

The repository carries a native build recipe (`.baby.toml`) that builds the
`azusa` binary and installs it to `~/.local/bin`, cleaning build output by
default:

```console
$ baby --user
```

Verify:

```console
$ azusa --version
azusa 0.1.39
$ which azusa
/home/sal/.local/bin/azusa
```

Ensure `~/.local/bin` is on your `PATH`.

## Alternative: plain Cargo

```console
$ cargo install --path crates/azusa-cli
```

This installs the same binary; use `baby` when you want the repo-native
recipe and automatic build-artefact cleanup.

## Updating with `theosis`

The lifecycle is managed by two repo-native tools: `baby` builds and
installs; `theosis` reconciles the installed version against this
repository.

```console
$ theosis check     # compare the installed version with this remote
$ theosis update    # clone, build via baby, install, verify
```

What `theosis update` does, in order:

1. Clones the repository's default branch into a **transaction directory**.
2. Reads the kaptaind-managed root `VERSION` file — the same number the
   binary self-reports via `azusa --version`.
3. Delegates the build to the `.baby.toml` recipe.
4. Installs and **verifies before declaring success**.

Guarantees:

- `theosis` **never downgrades automatically** (exit code 10 when the
  installed version is newer than the remote default branch).
- Update state lives under `.theosis/` and is never committed.

## Version authority

The root `VERSION` file is the single version authority. It is propagated
into the workspace `Cargo.toml` by `scripts/sync-version.sh` (CI enforces
that they never diverge), embedded into the binary, and echoed by
`azusa --version`. See [development/release.md](../development/release.md).

## Shell completion

The CLI is built with clap; completions follow your shell's standard
mechanism. `azusa --help` lists every command; each subcommand accepts
`--help` as well.

## Uninstall

Remove the binary and, if desired, the per-repository evidence trail:

```console
$ rm ~/.local/bin/azusa
$ rm -rf /path/to/repo/.azusa   # per-repo evidence store (see reference/evidence-store.md)
```

`.azusa/` is git-ignored and always safe to delete: it is an append-only
audit trail and cache, never a source of truth. Deleting it loses report
history and watch-mode escalation state, nothing else.

## Troubleshooting installation

| Symptom | Cause | Fix |
| --- | --- | --- |
| `azusa: command not found` | `~/.local/bin` not on `PATH` | add it to your shell profile |
| version mismatch after update | stale binary earlier on `PATH` | `which -a azusa`; remove the stale copy |
| `theosis update` exit 10 | installed version newer than remote | intentional no-downgrade guard; pin or wait |
| build fails on toolchain | outdated stable Rust | `rustup update stable` |

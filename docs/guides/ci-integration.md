# CI Integration

Azusa is designed to be a **CI gate**: deterministic reports, exit codes as
verdicts, and a policy engine tuned for pipeline enforcement. This guide
covers the recipes.

## The gate: `azusa policy check`

```console
$ azusa policy check --quiet
$ echo $?
0        # compliant (and drift below configured exit thresholds)
1        # compliant, but drift at medium (see note)
2        # compliant, but drift at high/critical (see note)
3        # POLICY VIOLATION — the gate signal
4        # analysis failure — treat as pipeline error
5        # invalid invocation — treat as pipeline error
```

> **Note.** When compliant, `policy check` exits with the normal drift
> mapping (0/1/2), so a *compliant but medium-risk* repository exits 1.
> If you want the policy check alone to decide pass/fail, treat **3** as
> the failure code and route 0–2 per your drift appetite (the reference
> Azusa repository's own CI fails only on ≥ 3).

What a violation means: the finished drift report breached a configured
limit — overall score above `max_drift`, a major dependency bump, a new
crate where forbidden, or a production configuration change where not
permitted. Violations are evidence checks over the report; they never
re-analyse states, so the verdict is as deterministic as the report.

## Installing the binary in CI

Any static-binary distribution works. Two supported patterns:

**Build in-pipeline (no artefact management):**

```yaml
- run: cargo install --path crates/azusa-cli
```

**Build once, cache the binary** — Azusa is a single static binary, so
caching per tool version is trivial. Key the cache on `VERSION`.

## GitHub Actions

The Azusa repository gates itself this way (`.github/workflows/ci.yml`):

```yaml
name: drift-gate
on: [pull_request]

jobs:
  policy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with: { fetch-depth: 0 }        # full history: ancestry + upstream states
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Install azusa
        run: cargo install --path crates/azusa-cli
      - name: Policy gate
        run: |
          azusa policy check --quiet
          code=$?
          if [ "$code" -ge 3 ]; then
            echo "::error::azusa policy violation (exit $code)"
            exit 1
          fi
          echo "azusa policy check: compliant"
```

**Why `fetch-depth: 0`:** the default comparison is worktree vs upstream;
shallow clones cannot express ahead/behind ancestry. If full history is
unacceptable, compare against explicit refs instead:

```yaml
- run: azusa policy check origin/${{ github.base_ref }} --quiet
```

### Drift gate (warnings on drift, failure on violation)

```yaml
- name: Drift report
  run: |
    azusa drift --json > drift-report.json || code=$?
    echo "exit=$code" >> "$GITHUB_OUTPUT"
- uses: actions/upload-artifact@v4
  if: always()
  with:
    name: drift-report
    path: drift-report.json
```

### Enforcing an explicit upstream comparison

```yaml
- run: azusa policy check worktree origin/main --quiet
```

## Generic pipeline recipe (any CI)

```sh
set -eu
azusa export --json > /dev/null          # 1. produce the canonical artefact
azusa policy check --quiet               # 2. gate: exit 3 on violation
latest=$(ls -t .azusa/exports/drift-*.json | head -1)
cp "$latest" "$ARTIFACTS_DIR/"           # 3. attach the audit artefact
```

## CI as a state: what the pipeline actually compares

Two useful framings:

1. **PR gate (default pair).** `azusa policy check` in a PR pipeline
   compares the *worktree as checked out and built* against *upstream* —
   i.e. "does this change violate policy relative to where the branch
   targets?".
2. **Environment watch.** `azusa watch --max-cycles 1` inside a scheduled
   job gives a cheap one-shot re-check with the same exit semantics;
   stage-0 idle cycles exit 0 without analysing.

## Policy that fits pipelines

Keep the default policy in `azusa.toml` at the repository root so local
runs and CI enforce the identical contract (the config digest rides in
every report's provenance).

A typical stricter CI profile:

```toml
[policy]
max_drift = 4.0

[policy.dependencies]
allow_major = false
allow_new   = true

[policy.structure]
allow_new_crates = true

[policy.configuration]
production_changes = "RequireReview"
```

Violations list stable ids (`POLICY-0001`…), severities, rules and evidence
references — paste them straight into the PR review. Rule semantics:
[reference/policy-rules.md](../reference/policy-rules.md).

## The audit trail

Every analysis run in CI appends to the evidence store (best-effort — an
unwritable store never fails the run):

- `.azusa/history.jsonl` — one compact record per run;
- `.azusa/events.jsonl` — ELCI events (`azusa.policy.failed`,
  `azusa.drift.threshold_exceeded`, …);
- `.azusa/exports/` — canonical artefacts from `azusa export` / `azusa plan`.

`.azusa/` is git-ignored. Upload exports as pipeline artefacts when you
need cross-run auditing; the artefact embeds full provenance (tool
version, analysis mode, config digest, capture timestamp).

## Performance

Clean `status` targets sub-100 ms; structural drift analysis sub-500 ms.
The stat-keyed hash cache (`.azusa/cache/`) makes repeat scans incremental.
On ephemeral CI runners the cache is cold each run — the analysis is still
fast; on persistent runners, cache `.azusa/cache/` keyed on tool version.

## Failure handling

| Exit | CI handling |
| ---: | --- |
| 0 | compliant — proceed |
| 1 / 2 | compliant, drift at medium/high — proceed or warn per appetite (3 is the gate) |
| 3 | **violation — fail the pipeline** |
| 4 | analysis failure (bad selector, unwritable worktree, malformed config) — fail as infrastructure error, not as policy breach |
| 5 | invocation bug in the pipeline definition — fix the command line |

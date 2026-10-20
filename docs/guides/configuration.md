# Configuration Reference (`azusa.toml`)

Azusa needs **no configuration** — every knob has a documented default.
When defaults are not enough, an optional `azusa.toml` at the repository
root (or any parent directory) tunes weights, thresholds, exit-code
mapping and policy.

## Discovery and parsing rules

- The file is named exactly `azusa.toml`.
- `AzusaConfig::load` walks from the working root **upward** and uses the
  nearest `azusa.toml` found.
- A missing file means defaults; a **malformed file is an error** (exit 4),
  never silently ignored.
- **Unknown keys are rejected** (`deny_unknown_fields` everywhere): a typo
  like `[drift] semantc = 2.0` fails loudly instead of silently scoring
  with defaults.
- The effective configuration is serialised and digested into the report's
  `provenance.config_digest`, so a report always carries the configuration
  that produced it.

## Complete example

```toml
[drift]                              # layer weights (defaults shown)
ancestry      = 0.5
filesystem    = 1.0
structure     = 1.0
dependency    = 1.0
configuration = 1.0
interface     = 1.0
behaviour     = 1.5
semantic      = 2.0                  # security-sensitive default profile
operational   = 1.5

[rename]                             # rename-detection thresholds (defaults shown)
path_threshold        = 0.85         # min full-path similarity for pairing
size_ratio_threshold  = 0.5          # min size ratio for pairing
min_similarity_size   = 16           # smaller files never pair by path alone

[exit]                               # exit-code mapping (defaults shown)
medium   = 1
high     = 2
critical = 2

[relatedness]                        # state relatedness (defaults shown)
min_overlap = 0.15                   # min common-path overlap before "unrelated"

[policy]                             # enforced by `azusa policy check`
max_drift = 5.0                      # maximum tolerated overall drift score

[policy.dependencies]
allow_major = false                  # major version bumps permitted?
allow_new   = true                   # new dependencies permitted?

[policy.structure]
allow_new_crates = true              # new crates/modules permitted?

[policy.configuration]
production_changes = "RequireReview" # Forbid | RequireReview | Allow
```

Every section and every key is optional; omitted values take the defaults
shown.

## `[drift]` — layer weights

Higher weight pulls the composite drift index toward that layer. Weights
participate in the weighted mean over **analysed** layers only
(`overall = Σ weight·score / Σ weight`). Setting a weight to `0.0` disables
the layer's contribution entirely.

| Key | Default | Notes |
| --- | ---: | --- |
| `ancestry` | 0.5 | Commits ahead/behind are facts, but volume ≠ severity; deliberately under-weighted. |
| `filesystem` | 1.0 | |
| `structure` | 1.0 | |
| `dependency` | 1.0 | |
| `configuration` | 1.0 | |
| `interface` | 1.0 | |
| `behaviour` | 1.5 | Losing verification surface is costlier than code churn. |
| `semantic` | 2.0 | Reserved for L8 (planned); highest default weight follows the security-sensitive profile. |
| `operational` | 1.5 | Deployment surface divergence is expensive to discover late. |

The defaults encode a **security-sensitive profile** — behaviour,
operations and (future) semantics outweigh raw file churn.

## `[rename]` — rename detection

`azusa-diff` pairs deletions with additions in two stages: exact content
identity (always, hash-equal) and a path/size heuristic stage.

| Key | Default | Meaning |
| --- | ---: | --- |
| `path_threshold` | 0.85 | Minimum full-path similarity for a similarity-based pairing. |
| `size_ratio_threshold` | 0.5 | Minimum size ratio (min/max) for a pairing. |
| `min_similarity_size` | 16 | Files smaller than this (bytes) never pair by path similarity alone. |

Every detected rename carries an explicit tier — `exact-identity`,
`content-similarity` (similarity ≥ 0.9 with changed content) or
`path-heuristic` — and scoring discounts by tier: ×0.5 / ×0.75 / ×1.5. A
weak pairing costs nearly as much as an unexplained delete + add, because
false rename attribution is worse than reporting the two events.

## `[exit]` — exit-code mapping

| Key | Default | Meaning |
| --- | ---: | --- |
| `medium` | 1 | Exit code for medium risk (and low-but-drifting results). |
| `high` | 2 | Exit code for high risk. |
| `critical` | 2 | Exit code for critical risk. |

Codes 0, 3, 4 and 5 are reserved (in-sync, policy violation, analysis
failure, invalid invocation) and cannot be remapped.

## `[relatedness]` — identity

| Key | Default | Meaning |
| --- | ---: | --- |
| `min_overlap` | 0.15 | Minimum common-path overlap ratio before two states are declared `unrelated` (L0). |

## `[policy]` — CI gate rules

Checked by `azusa policy check` **after** the deterministic analysis, as an
evidence check over the finished report — policies never re-analyse states,
so a policy verdict is as deterministic as the report it audits.

| Key | Default | Rule |
| --- | ---: | --- |
| `max_drift` | 5.0 | Violation `policy.max_drift` (severity **high**) when overall score exceeds the maximum. |
| `[policy.dependencies] allow_major` | false | Violation `policy.dependencies.allow_major` (**high**) when any dependency transitioned across the major boundary. |
| `[policy.dependencies] allow_new` | true | Violation `policy.dependencies.allow_new` (**medium**) when a dependency was declared only on one side. |
| `[policy.structure] allow_new_crates` | true | Violation `policy.structure.allow_new_crates` (**medium**) when modules appear on only one side. |
| `[policy.configuration] production_changes` | `"RequireReview"` | Treatment of production-facing configuration changes: `Forbid` (violation, **critical**), `RequireReview` (violation, **medium**), `Allow` (no violation). |

"Production-facing" configuration kinds: `dockerfile`, `kubernetes`,
`systemd-unit`, `compose` — plus any evidence reference containing `prod`.

Violations are ordered by severity (descending) then rule name, and
assigned stable ids `POLICY-0001`… — identical reports always produce
identical ids. Full semantics: [reference/policy-rules.md](../reference/policy-rules.md).

## Provenance and reproducibility

The effective configuration is digested (BLAKE3 over the canonical
serialisation) into `provenance.config_digest`. Two reports with different
config digests were produced under different rules — compare their scores
with care. Reproducing a report byte-for-byte requires the same states,
the same `azusa.toml` content, and the same tool version.

## Anti-patterns

- **Setting `max_drift = 10.0` "temporarily".** The gate stops gating;
  make the intent explicit with a dated policy exception instead.
- **Zeroing `behaviour` or `operational` for speed.** The layers are cheap;
  the defaults are the security-sensitive profile for a reason.
- **Typos in keys.** They fail the run (exit 4) by design — fix the key,
  do not bypass with a duplicate config.

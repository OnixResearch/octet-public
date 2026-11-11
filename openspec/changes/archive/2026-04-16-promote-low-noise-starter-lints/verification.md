# Verification

## Exact replay path

```bash
bash openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/replay-aspen-promoted-starter.sh
```

The replay uses the archived Aspen phase-1 reference from
`openspec/changes/archive/2026-04-13-aspen-consumer-rollout/` and checks out
Aspen commit `909e5e9fa` with sibling `aspen-dns` and `aspen-wasm-plugin`
symlinks in a disposable sandbox.

## Saved artifacts

- `evidence/replay-aspen-promoted-starter.sh`
- `evidence/check-aspen-promoted-starter.py`
- `evidence/aspen-promoted-starter-config.toml`
- `evidence/aspen-promoted-starter-default.txt`
- `evidence/aspen-promoted-starter-coordination.txt`
- `evidence/aspen-promoted-starter-counts.txt`
- `evidence/templates-dylint.diff`

## Promotion summary

Observed promoted-family counts from `evidence/aspen-promoted-starter-counts.txt`:

| Family | Starter scope (`aspen-time`, `aspen-hlc`, `aspen-core`) | `aspen-coordination --lib` | Starter status |
| --- | ---: | ---: | --- |
| `explicit_defaults` | 1 | 2 | promoted |
| `unbounded_channel` | 0 | 0 | promoted |
| `usize_in_public_api` | 3 | 0 | promoted |
| `negated_predicate` | 0 | 2 | promoted |
| `unbounded_collection_growth` | 0 | 0 | inventory-first (disabled in promoted starter profile) |
| `too_many_parameters` | 0 | 0 | inventory-first (disabled in promoted starter profile) |

Template proof:

- `evidence/templates-dylint.diff` shows the exact `templates/dylint.toml` hunk
  that removed the four promoted families from the example `disabled_lints`
  list while keeping `unbounded_collection_growth` and
  `too_many_parameters` deferred.

Starter-profile totals in the saved transcripts:

- default starter scope: 10 warnings total
  - 6 `ambient_clock`
  - 1 `explicit_defaults`
  - 3 `usize_in_public_api`
- focused `aspen-coordination --lib` spotcheck: 26 warnings total
  - 21 `ambient_clock`
  - 1 `compound_assertion`
  - 2 `explicit_defaults`
  - 2 `negated_predicate`

## Validation commands

```bash
openspec validate --specs
openspec validate promote-low-noise-starter-lints
python3 openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence/check-aspen-promoted-starter.py \
  openspec/changes/archive/2026-04-16-promote-low-noise-starter-lints/evidence
```

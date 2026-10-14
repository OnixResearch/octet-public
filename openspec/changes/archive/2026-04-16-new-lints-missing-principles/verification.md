# Verification

## Local lint crate proof

Run inside the repo devshell:

```bash
nix develop -c bash -c 'cargo test --lib'
nix develop -c bash -c 'cargo test --test ui -- --ignored --nocapture'
```

The UI corpus covers the new lint fixtures, config fixtures, and deterministic
fixture inventories already checked into `tests/fixtures/`.

## Aspen downstream evidence

Exact replay path:

```bash
bash openspec/changes/archive/2026-04-16-new-lints-missing-principles/evidence/replay-aspen-new-lints.sh
```

Saved artifacts:

- `evidence/aspen-new-lints-meta.txt`
- `evidence/validation-counts.txt`
- `evidence/check-aspen-new-lints.py`
- `evidence/config-<lint>.toml`
- `evidence/aspen-<lint>-default.txt`
- `evidence/aspen-<lint>-coordination.txt`
- `evidence/cargo-test-lib.txt`
- `evidence/cargo-test-ui-bless.txt`
- `evidence/cargo-test-ui-ignored.txt`
- `evidence/ui-new-lints-snapshots.txt`

## Aspen classification summary

Low-noise rule for this change:

- `starter-candidate`: <= 3 findings on starter scope (`aspen-time`,
  `aspen-hlc`, `aspen-core`) and <= 5 findings on `aspen-coordination`
- `inventory-first`: anything larger

Observed counts:

| Lint | Starter scope | `aspen-coordination` | Classification |
| --- | ---: | ---: | --- |
| `explicit_defaults` | 1 | 2 | `starter-candidate` |
| `unbounded_channel` | 0 | 0 | `starter-candidate` |
| `unbounded_collection_growth` | 0 | 8 | `inventory-first` |
| `usize_in_public_api` | 0 | 0 | `starter-candidate` |
| `too_many_parameters` | 0 | 15 | `inventory-first` |
| `negated_predicate` | 0 | 2 | `starter-candidate` |

Current rollout decision:

- keep all six families out of copied starter-profile configs for now
- treat `explicit_defaults`, `unbounded_channel`, `usize_in_public_api`, and
  `negated_predicate` as low-noise starter candidates for a later promotion
  change
- keep `unbounded_collection_growth` and `too_many_parameters` inventory-first
  until a later cleanup/promotion pass reduces the Aspen coordination backlog

## Context

Each new lint follows the repo's standard lint-adding checklist (skill `octet-repo`): implement under the right domain module, register in `src/lib.rs`, add diagnostic metadata in `src/diagnostics.rs`, add UI fixture coverage, update `tests/fixtures/ui_fixture_set.json` and `tests/fixtures/lint_emitting_fixtures.json`, and add config keys where configurable. Existing neighbors give the exact shape: `too_many_parameters` for `fat_trait`, `explicit_defaults` for `concrete_construction_in_core`, `module_file_count` for `type_accumulation`, and the allow-by-default `assertion_density` for the advisory `defaulting_trait_method`.

## Approach

1. **`fat_trait`** — count `fn` items in each `trait` declaration HIR node; warn when count exceeds `trait_method_max` (default 7, mirroring `max_parameters` = 5 with headroom). Default `warn`. Remediation: split the trait into cohesive role interfaces.
2. **`concrete_construction_in_core`** — in declared core scopes (from the `crate-boundary` mechanism, falling back to none), flag method calls resolving to `<ExternalType>::new()` / `::default()` where the type is not in the domain crate set and not `std`/`core`. Reuse the HIR `MethodCall` resolution path already exercised by `clone_then_borrow`. Default `warn`. Without a declared core scope the lint is a no-op so existing consumers are unaffected.
3. **`type_accumulation`** — count distinct `impl` blocks per type and unrelated `pub` items per module; `allow` by default given high false-positive risk. Configurable thresholds `impl_blocks_max` (default 4) and `module_pub_items_max` (default 12).
4. **`defaulting_trait_method`** — flag `trait` method items carrying a default body; `allow` by default. Remediation: make the method required or document why a default is safe. This is the LSP-adjacent signal that defaults can quietly diverge across implementors.

## Risks

- `concrete_construction_in_core` depends on knowing the core scope; before `policy-driven-crate-boundaries` lands it must no-op cleanly with an empty scope so it does not break the existing lint set.
- `type_accumulation` "unrelated" is a heuristic. Keep it `allow` and base the count on distinct `impl` blocks and `pub` item count, not semantic relatedness, to keep the check deterministic.
- `defaulting_trait_method` will fire on many legitimate stdlib-style traits. `allow` default plus a scoped `#[allow(octet::defaulting_trait_method)]` keeps it advisory.
- All four add to the lint count claimed in `README.md` and `docs/lint-reference.md` (currently 38); update the catalog summary and domain counts.

## Validation

- Each lint gets a positive UI fixture (fires on the target shape) and a negative UI fixture (clean on the safe shape), per the repo's both-kinds requirement.
- `BLESS=1 cargo test --test ui ui -- --ignored --nocapture` produces stable `.stderr` snapshots.
- `cargo test --lib` covers config parsing for the new keys.
- The `autofix_machine_applicable` rail passes for any lint emitting a machine-applicable suggestion.
- `nix run path:/path/to/git/cairn#cairn -- validate --root .` and the three Cairn gates pass.

## Why

Haskell-style applicative validation is valuable for Octet because humans author and override config, policy, fixture manifests, and evidence declarations. A fail-fast parser often forces maintainers to fix one issue at a time, while a deterministic accumulated diagnostic set can show every independent problem in one review pass.

Octet should adapt this idea without importing Haskell terminology into user-facing docs. The Rust implementation should use explicit validation structs, typed diagnostics, and source locations. It should still fail closed before any dependent stage runs.

## What Changes

- Define when validation should accumulate independent semantic errors after syntax parsing succeeds.
- Add typed diagnostic aggregation for config, policy, fixture, and manifest validation seams where multiple independent errors can be reported safely.
- Use explicit parser boundaries for non-trivial mini-grammars instead of ad-hoc split/substring chains.
- Preserve fail-fast behavior for syntax errors and for dependent stages that would become unsafe or misleading after an invalid prerequisite.
- Add positive and negative tests proving multiple independent errors are reported deterministically.

## Impact

- **Files**: likely `src/config.rs`, `cargo-octet/src/config.rs`, policy/standards validation helpers, fixture manifest validators, Nickel export/freshness helpers, docs for config diagnostics, and tests.
- **Testing**: valid config/manifests pass, malformed syntax fails closed, multiple independent semantic errors accumulate in stable order, dependent stages do not run after invalid validation, and Cairn validate/gates.

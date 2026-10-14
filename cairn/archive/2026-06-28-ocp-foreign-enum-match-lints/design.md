## Context

`catch_all_on_enum` already detects `_` match arms over enums but does not classify the enum's ownership (same-crate vs foreign). Whether a catch-all is a defect depends on who owns the enum: a foreign enum's `_` arm hides variants the author cannot see being added; an own enum's exhaustive match is the author's own closed decision but is fragile to future additions. Distinguishing crate ownership is the missing ingredient and is available from `tcx` def-path / crate-of-origin for HIR `match` scrutinees.

## Approach

1. **`foreign_enum_catch_all`** — for each `match` over an enum, resolve the enum's defining crate via `tcx`; if it differs from the match's crate and a `_` arm is present, emit an `allow`-level diagnostic. Reuse the existing `catch_all_on_enum` arm-detection logic.
2. **`fragile_exhaustive_enum_match`** — for an exhaustive `match` over a same-crate enum (no `_` arm, all variants listed), emit an `allow`-level diagnostic marking it fragile. Honor an opt-in marker attribute so authors can mark deliberately-closed enums.
3. Add a config key `sealed_enum_marker` (default `octet::sealed_enum`) so the fragile-match lint can be suppressed per-enum without a blanket lint allow.
4. Keep both lints `allow` by default; promote only after fixture evidence, per the repo's advisory→hard staging pattern.

## Risks

- Crate-of-origin resolution for macro-generated enums can misfire. Restrict to enums whose `def_id` crate is unambiguous and skip expansion-generated scrutinees, mirroring the `unbounded_loop` desugaring guard.
- Exhaustiveness detection without rustc's match-exhaustiveness analysis is incomplete. Use a heuristic: no `_` arm plus all variants named; accept false negatives over false positives at `allow` level.
- Adding two lints bumps the catalog count; update `README.md` and `docs/lint-reference.md` (38, or 42 if `solid-structure-lints` lands first).

## Validation

- Positive UI fixture: foreign enum + `_` arm fires `foreign_enum_catch_all`; same-crate exhaustive match fires `fragile_exhaustive_enum_match`.
- Negative UI fixture: own enum + `_` arm is clean for the foreign lint; foreign exhaustive match is clean for the fragile lint.
- Cross-crate fixture (a sibling fixture-local crate exporting an enum) proves crate-of-origin classification.
- Opt-in marker fixture proves the fragile-match diagnostic is suppressed for `#[octet::sealed_enum]` enums.
- `BLESS=1` snapshot refresh; `cargo test --lib`; `nix run path:/path/to/git/cairn#cairn -- validate --root .` and the three Cairn gates.

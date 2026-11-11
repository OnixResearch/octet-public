## Why

The Octet rename is archived and the lint set now has a stable public surface. Comparing Octet with `indexable-inc/clippy` showed that Octet already covers many mainstream Clippy policy areas under Tiger/Octet names, but it does not yet cover the fork-specific Indexable lints that encode useful structure, naming, and integer-fallback policies.

These lints are small, concrete, and align with Tiger Style: make failure explicit, keep modules navigable, avoid ambiguous numeric constants, and keep imports/names grep-friendly.

## What Changes

- **Safety additions**: add policy coverage for explicit dropping of must-use values and fallible integer conversions hidden behind sentinel/default fallbacks.
- **Structure additions**: add policy coverage for excessive file length, excessive module file count, and inline magic numeric literals.
- **Naming/import additions**: add policy coverage for underscore module filenames, non-trait imports, renamed imports, and path-segment repetition.
- **Metadata and compatibility**: register each new lint in Octet inventory, diagnostics metadata, domain groups, config validation, JSON output metadata, and UI/evidence harnesses.

## Capabilities

### New Capabilities

- `indexable-safety-policy-lints`: detects `drop()` on must-use values and fallible integer conversion fallbacks.
- `indexable-structure-policy-lints`: detects magic numbers and oversized files/modules.
- `indexable-naming-import-policy-lints`: detects module/import/name patterns that reduce grepability and path clarity.

### Modified Capabilities

- `safety`: gains explicit must-use-drop and fallible-integer-fallback requirements.
- `structure`: gains file/module-size and magic-number requirements.
- `naming`: gains path-aware module/import/name requirements.

## Impact

- **Files**: `src/safety`, `src/structure`, `src/naming`, `src/lib.rs`, `src/diagnostics.rs`, `ui_tests`, `tests/ui.rs`, and OpenSpec specs.
- **APIs**: no public Rust API break; new lint names become user-facing config/source-attribute surfaces.
- **Dependencies**: no new runtime dependencies expected.
- **Testing**: focused UI fixtures for each new lint, config/group registration checks, and saved deterministic evidence artifacts.

## Out Of Scope

- Wholesale vendoring of Clippy or `indexable-inc/clippy` implementation code.
- Renaming existing Octet lints to Clippy names.
- Enabling broad upstream Clippy lint families that Octet already delegates to Clippy or covers semantically.

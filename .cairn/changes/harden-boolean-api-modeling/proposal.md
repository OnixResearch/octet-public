## Why

Octet checks boolean local-binding names, parameter count, and several type-shape rules. It does not reject a public API such as `deploy(true, false)`.

Bare boolean arguments hide domain meaning at call sites. They also allow unrelated modes, states, roles, or actions to share one primitive type.

Radiance provides a concise style reference for explicit option records and single state types. Octet needs a bounded Rust rule, not a broad copy of that style guide.

## What Changes

- Add a `raw_bool_parameter` structure lint for public functions, methods, and crate-owned trait declarations.
- Report non-public callables that contain at least the named minimum count of direct boolean parameters.
- Reuse `clippy::fn_params_excessive_bools` as independent co-reporting evidence.
- Recommend enums, option records, typestate, or named wrappers according to domain meaning.
- Skip generated code and externally owned trait signatures through explicit bounded rules.
- Require narrow justified source expectations for unavoidable compatibility surfaces.
- Add positive and negative UI fixtures, lint metadata, profile registration, guidance, and migration evidence.
- Enable the lint in the full deny-all catalog only after Octet itself and selected consumers migrate cleanly.

## Impact

- **Specs**: `solid-structure-lints`
- **Files**: Dylint pass, registration, metadata, profiles, UI fixtures, snapshots, docs, templates, migration notes, and release notes
- **Consumers**: public APIs with direct `bool` parameters need domain types or reviewed exceptions
- **Autofix**: none, because type selection requires domain judgment

## Non-Goals

- Do not ban boolean fields, return values, predicates, or local values.
- Do not infer domain semantics from parameter names alone.
- Do not rewrite APIs automatically.
- Do not copy the Radiance ban on small one-use helpers.
- Do not adopt Radiance panic guidance.
- Do not claim that replacing a boolean proves correct state modeling.

## Compatibility Boundaries

- Externally owned trait implementation signatures remain compatible.
- Generated code remains outside direct findings.
- Narrow source expectations require a specific reason and owner.
- Existing `bool_naming` behavior remains unchanged.

## Verification Expectations

Public direct boolean parameters and excessive private boolean parameters must produce stable diagnostics. Domain wrappers and allowed compatibility signatures must remain clean.

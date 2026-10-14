## Why

Tigerstyle already encodes core/shell, safety, boundedness, and lint guidance, but two external references point at high-ROI gaps: Valgrind's framing as a suite of debugging/profiling tools with clear run timing, and Rust Design Patterns' Rust-native mechanisms for making safety contracts structural. Capturing these as an OpenSpec prevents ad-hoc importing of generic patterns and keeps future docs/lints tied to Tiger Style outcomes.

## What Changes

- **Tool-suite profiles**: Add a Valgrind-inspired documentation model for when to run static lint profiles, dynamic profilers, race/model checks, and release evidence.
- **Rust idiom standards**: Add Tigerstyle guidance for newtypes, RAII guards, contained unsafe modules, borrowed argument types, temporary mutability, clone-avoidance, recoverable consumed-argument errors, constrained builders, command/effects-as-data, FFI wrappers/errors, Deref restraint, small-crate boundaries, and functional API design patterns that encode protocol differences in types.
- **Policy/lint roadmap**: Classify each imported pattern as hard, advisory, or human-process-only before adding lints.

## Capabilities

### Modified Capabilities
- `standards-codification`: Extends the policy registry and standards docs with external-source incorporation rules and candidate evidence profiles.
- `documentation`: Adds operator-facing guidance/checklists that explain what to run, when, and why.

## Impact

- **Files**: Expected implementation touches `docs/`, `templates/`, `standards/policies/`, and possibly `src/` only after policy classification.
- **APIs**: No immediate public API change; future lints may add diagnostics/config entries through separate slices.
- **Dependencies**: Dynamic evidence docs may name optional tools such as Valgrind, Miri, sanitizers, heap/profile tools, and Loom, but this change does not require adding them as hard dependencies.
- **Testing**: Validate OpenSpec, policy registry exports, documentation link/checklist coverage, and any future lint fixtures separately.

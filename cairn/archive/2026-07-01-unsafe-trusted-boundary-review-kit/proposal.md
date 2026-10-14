## Why

Salt's unsafe story is simple for readers: raw pointer arithmetic and FFI live behind explicit unsafe or trusted boundaries, and every boundary is expected to name its safety preconditions. Octet already has a `public_unsafe_api` lint, unsafe guidance in review docs, and dynamic evidence profiles for unsafe or FFI boundaries, but those pieces are not yet a first-class review kit.

Octet should adapt the explicit-boundary part of Salt's story for Rust consumers: public APIs should not leak unsafe obligations by default, FFI and raw-pointer seams should have safe wrappers or documented exceptions, and high-risk unsafe changes should name the evidence or human review still required. This remains lint/review/evidence guidance, not a claim that Octet verifies unsafe code.

## What Changes

- Add an unsafe/trusted-boundary review contract with fields for boundary name, public entry point, unsafe operation, invariant, caller obligation, safe wrapper, evidence tier, dynamic evidence, reviewer action, and explicit non-claim.
- Link the contract to existing `public_unsafe_api`, Rust idiom guidance, concurrency unsafe-quarantine guidance, and dynamic evidence profiles.
- Define FFI/trusted imports as review boundaries that require stable handles/error mapping and ownership/lifetime notes.
- Add deterministic docs checks for required boundary fields and forbidden claims that Octet verifies unsafe code or provides Salt-style `@trusted` semantics.

## Non-Goals

- Do not add a new `trusted` Rust attribute or Salt-style `@trusted` syntax.
- Do not claim Octet proves unsafe code correct.
- Do not require Miri, sanitizers, or Valgrind for every consumer run.
- Do not change the current `public_unsafe_api` lint implementation in this change.

## Impact

- **Files**: likely `docs/rust-idiom-guidance.md`, `docs/concurrency-state-review.md`, `docs/dynamic-evidence-profiles.md`, `docs/lint-reference.md`, `docs/safety-critical.md`, and deterministic docs fixtures/tests.
- **Behavior**: documentation and validation only unless a later implementation change chooses to tighten lint metadata or runner output.
- **Testing**: positive inventory for unsafe-boundary fields and links to existing surfaces; negative wording check for verified-unsafe and Salt-style trusted overclaims; Cairn validation and gates.

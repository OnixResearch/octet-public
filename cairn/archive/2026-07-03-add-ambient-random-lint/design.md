## Context

Random APIs make code non-replayable unless the random source is explicit. Octet should not ban random generation globally; it should make ambient generation visible and keep deterministic cores honest.

## Decisions

### 1. Detect direct randomness sources

**Choice:** Match direct calls to common random sources and ID constructors, including `rand` thread-local RNGs, `getrandom`, OS RNG constructors, and UUID v4 generation.

**Rationale:** These APIs are strongly associated with hidden input and have clear remediations.

### 2. Allow injected generators

**Choice:** Do not flag functions that receive `Rng`-like parameters, seed values, nonce bytes, or already-generated IDs.

**Rationale:** Explicit inputs preserve deterministic test seams even when the shell uses randomness.

### 3. Keep cryptographic policy out of scope

**Choice:** This lint only enforces explicit randomness boundaries; it does not evaluate RNG quality.

**Rationale:** Crypto selection is a separate policy and should not be conflated with purity.

## Risks / Trade-offs

- Crate APIs change names. Keep the matcher table small and fixture-backed.
- Some tests intentionally sample randomness. Test-context suppression should keep those clean.

## Validation

Add classifier unit tests for the path table and UI fixtures for positive and negative source shapes.

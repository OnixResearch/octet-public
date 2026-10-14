## Context

Octet's pure-core direction makes deterministic behavior easier to test without rustc traversal, filesystem setup, or process execution. Law tests should capture invariants that example tests rarely exhaust: stable output for equivalent inputs, order-insensitive normalization where promised, precedence rules, idempotence, and fail-closed handling of malformed data.

## Decisions

### 1. Start from a law inventory

**Choice:** Each property-test adoption pass SHOULD name the pure core, its public law, the domain of generated inputs, and the existing concrete examples that remain as regression tests.

**Rationale:** Property tests without named laws become random fuzzing. The useful Haskell adaptation is explicit laws, not broad generation for its own sake.

### 2. Keep generated tests deterministic and bounded

**Choice:** Property suites SHOULD use deterministic runner configuration, named case budgets, and saved minimal failing examples where the framework supports them.

**Rationale:** Octet gates and evidence artifacts must be reproducible. A failing generated input needs to be replayable by reviewers.

### 3. Pair laws with negative tests

**Choice:** Every law suite SHOULD include malformed or adversarial inputs that verify rejection, diagnostic stability, or non-panic behavior.

**Rationale:** Haskell-style totality maps to Rust cores returning typed errors rather than panicking or silently accepting invalid states.

### 4. Preserve fixture rails

**Choice:** Property tests complement, but do not replace, UI fixtures, checked-in snapshots, CLI golden tests, proof receipts, or artifact verification.

**Rationale:** Property tests show invariants over generated input. Fixtures and transcripts still prove user-visible source shapes and output contracts.

### 5. Avoid over-generalized abstractions

**Choice:** Reusable generators and assertion helpers SHOULD stay close to the core they test until multiple cores share a stable domain model.

**Rationale:** Premature generic testing frameworks can hide scenario intent and make failures harder to review.

## Risks / Trade-offs

- Property tests can slow focused test runs if case budgets are too broad; keep budgets named and intentional.
- Poor generators can miss realistic edge cases; keep concrete fixtures for known regressions.
- Failing generated cases must be easy to reproduce in Nix and local devshells.

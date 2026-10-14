## Context

Pure functions must not depend on process-wide mutable storage. Some shell layers need singletons for caches or clients, but core logic should accept state as input and return updated state or commands.

## Decisions

### 1. Check declarations, not every use

**Choice:** The first lint flags global mutable/lazy state declarations in configured core scopes.

**Rationale:** Declaration sites are high signal and avoid expensive alias/use analysis.

### 2. Distinguish constants from mutable state

**Choice:** `const` items and immutable `static` data without interior mutability stay clean.

**Rationale:** Shared immutable data does not violate purity by itself.

### 3. Classify global state with a pure item classifier

**Choice:** Extract a deterministic classifier for static kind, initializer type, and macro shape before emitting.

**Rationale:** Unit tests can cover the policy table without a full lint run.

## Risks / Trade-offs

- `OnceLock` and `LazyLock` can hold immutable configuration but still hide initialization state. Core-scope diagnostics should recommend explicit injection.
- Macro expansion shapes vary; start with known macro names and fixture them.

## Validation

Add positive and negative UI fixtures, classifier unit tests, and catalog freshness coverage.

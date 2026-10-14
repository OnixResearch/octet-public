## Context

Octet contains several large modules that combine parsing, domain decisions, filesystem traversal, process execution, rendering, and tests. This makes it difficult to review behavior in isolation and conflicts with the desired functional-core / imperative-shell architecture.

## Decisions

### 1. Extract by behavior, not by arbitrary line count

**Choice:** Split large files along stable behavior seams: CLI parsing, object model construction, supplement comparison, proof execution planning, artifact writing, and test fixture construction.

**Rationale:** A mechanical file-size split can create more indirection without improving testability. Behavior seams let reviewers test pure logic without standing up the world.

### 2. Pure core owns deterministic decisions

**Choice:** Put parsing, normalization, canonical rendering, validation, and comparison in pure modules that accept inputs and return outputs or diagnostics.

**Rationale:** These are the highest-value units to test with positive and negative cases. They also produce the data that receipts and evidence depend on.

### 3. Shell modules own effects

**Choice:** Keep filesystem traversal, process spawning, environment reads, stdout/stderr, and artifact writes in thin shell modules that call pure cores.

**Rationale:** This preserves reviewable side effects and makes it clear which code can mutate local state.

### 4. Test support becomes reusable

**Choice:** Move repeated fixture builders, command harnesses, and artifact assertions into test-support modules.

**Rationale:** Large integration tests should express scenarios, not repeat setup plumbing.

## Risks / Trade-offs

- Refactors can accidentally change CLI output. Mitigate with golden output and artifact-bundle tests before and after extraction.
- Moving test helpers can hide scenario intent. Mitigate by naming helper functions after the behavior they set up, not implementation details.
- Some modules depend on rustc-private or generated sources. Mitigate by keeping those dependencies at the shell or adapter boundary unless the pure core genuinely needs typed compiler data.

## Validation

- Positive: extracted core tests pass with representative valid inputs and existing CLI golden outputs stay stable.
- Negative: extracted validators reject malformed inputs, missing fields, unsupported modes, and inconsistent receipts without panicking.
- Regression: existing integration tests and release smoke transcript checks continue to pass.

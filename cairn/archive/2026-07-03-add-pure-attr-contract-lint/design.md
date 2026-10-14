## Context

Path-scoped purity rules help architecture, but they do not let maintainers mark an individual function as pure. An explicit marker creates an opt-in contract and can be stricter because the source author requested it.

## Decisions

### 1. Recognize a tool-scoped pure marker

**Choice:** Support Octet's current lint namespace marker shape, starting with `#[tigerstyle::pure]` while documenting any namespace migration behavior.

**Rationale:** Existing source attrs currently use the `tigerstyle` tool namespace; the contract should align with that reality until migration completes.

### 2. Enforce strict direct-effect checks

**Choice:** Marked functions emit when their body directly uses banned effects: platform IO, time/env/random, logging, spawning, global/interior mutation, mutating input capabilities, panics, or resource construction/return.

**Rationale:** The marker is opt-in, so violations should be treated as contract breaks rather than generic style warnings.

### 3. Reuse shared classifiers

**Choice:** Build the marker lint as a thin orchestrator over pure classifiers from the narrower purity lints.

**Rationale:** Shared classifiers keep behavior consistent and unit-testable.

## Risks / Trade-offs

- Tool-scoped marker attrs may need `register_tool` handling in standalone fixtures. Document the setup and fixture it.
- Direct-effect checks are not transitive proof. Diagnostics must say the contract checks direct source shapes, not whole-program purity.

## Validation

Add marked-function UI fixtures for each banned direct effect, clean pure examples, metadata/docs freshness, and guardrails for unmarked code.

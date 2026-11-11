## Context

The existing `checks.ui` nix gate uses a custom UI driver that accepts
prebuilt driver and library paths via environment variables. This avoids
the `dylint_testing` crate's dependency on `rustup`. Two fixtures exist
today: `ambient_clock.rs` and `nested_conditionals.rs`.

## Goals / Non-Goals

**Goals:** Every lint has at least one UI fixture exercising its primary
fire and primary should-not-fire scenarios.

**Non-Goals:** Exhaustive scenario coverage for every lint. This change
covers the baseline; further scenarios can be added incrementally.

## Decisions

### 1. One fixture file per lint

**Choice:** Each lint gets one `.rs` fixture file containing both positive
(should fire) and negative (should not fire) test cases, separated by
comments.

**Rationale:** Matches the existing pattern (`ambient_clock.rs` exercises
both fire and no-fire cases). One file per lint keeps the fixture count
manageable.

**Alternative:** Separate fire/no-fire files per lint — rejected because
it doubles the file count without adding clarity.

### 2. Configurable-lint fixtures use local workspace subdirectories

**Choice:** Lints with configurable thresholds or allowlists (`function_length`,
`assertion_density`, `nesting_max_depth`, `acronym_allowlist`) get a second fixture file with
a `dylint.toml` override in a subdirectory.

**Rationale:** Dylint resolves `dylint.toml` through `cargo metadata` rooted at the
fixture file's parent directory. A subdirectory with its own minimal `Cargo.toml`,
empty `src/lib.rs`, and local `dylint.toml` is the simplest isolation.

**Alternative:** Global config override flag — rejected because it would
require harness changes.

### 3. Late-pass lints need compilable fixtures

**Choice:** Fixtures for late-pass lints (those needing type info) must
be fully compilable Rust, not just syntax-valid.

**Rationale:** Late passes run after type checking. The fixture needs
real types to exercise the lint correctly.

**Implementation:** Each late-pass fixture includes minimal type
definitions (e.g., a `Mutex<T>` wrapper, a `Result` type) or uses `std`
types directly.

## Risks / Trade-offs

**[Nightly churn]** → Nightly rustc updates may change diagnostic
formatting. Mitigation: the UI harness already normalizes paths and line
numbers. Wording changes require `.stderr` updates but that's the point
of having fixtures.

**[Fixture maintenance burden]** → 25 base lint fixtures plus 4 config-override suites to maintain. Mitigation:
each fixture is minimal and self-contained. Failures point directly at
the lint or config case that changed.

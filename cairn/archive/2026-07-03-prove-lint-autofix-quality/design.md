## Context

The diagnostics module already records applicability labels, and the UI harness has an ignored autofix rail. The gap is coverage breadth: only a small number of machine-applicable fixes are proven through checked `.fixed` outputs.

## Decisions

### 1. Make MachineApplicable proof mandatory

**Choice:** Any lint whose emitted suggestion is classified `MachineApplicable` must have at least one `.fixed` fixture that the autofix rail applies, compares, reruns for idempotence, and checks warning-free.

**Rationale:** Machine applicability is a promise to tooling.

### 2. Require explicit rationale for non-machine tiers

**Choice:** `MaybeIncorrect` and `HasPlaceholders` suggestions must remain visible in `.stderr` with tier notes and may optionally include `.fixed` examples only when they are deterministic and safe.

**Rationale:** Users should understand which suggestions are sketches versus automatic rewrites.

### 3. Track fixed-fixture freshness in library tests

**Choice:** Extend existing rollout-scope tests so every machine-applicable lint maps to a checked fixed fixture or a named out-of-scope rationale.

**Rationale:** New suggestions should not bypass the quality contract.

## Risks / Trade-offs

- Some safe rewrites require multipart suggestions and may be hard to compare. Keep the first phase focused on existing single-span/local-rename fixes.
- Fixed artifacts can churn with rustfmt or diagnostics. The autofix rail should compare source output, then rerun check to prove no warning remains.

## Validation

Run `cargo test --test ui autofix_machine_applicable -- --ignored --nocapture`, the normal UI rail, and library freshness tests.

## Context

Octet has several human-authored or human-reviewed inputs: `dylint.toml`, Nickel-authored policy/config, fixture manifests, evidence manifests, proof manifests, and generated-artifact manifests. Some errors are independent and can be reported together. Others are dependent and should stop the pipeline immediately.

The useful Haskell idea is applicative validation: collect independent errors while still refusing to produce a valid value. In Rust, this should look like explicit validation results with typed diagnostics and deterministic ordering.

## Decisions

### 1. Separate parse errors from semantic validation

**Choice:** Syntax parsing MAY remain fail-fast when no reliable structure exists. After a parseable structure exists, independent semantic checks SHOULD accumulate diagnostics where doing so is safe.

**Rationale:** Accumulation is useful only when the validator can continue without inventing meaning for malformed syntax.

### 2. Use typed diagnostic aggregation

**Choice:** Validators SHOULD return typed diagnostics with field/path/source context, stable ordering, severity, and machine-readable error kinds where practical.

**Rationale:** Aggregated strings are hard to test and hard for tools to consume. Typed diagnostics keep the pure core deterministic.

### 3. Keep fail-closed stage boundaries

**Choice:** Accumulating diagnostics MUST NOT allow later execution, artifact acceptance, or evidence promotion when required inputs are invalid.

**Rationale:** Reporting more errors should improve review ergonomics, not weaken safety gates.

### 4. Add parser boundaries for mini-grammars

**Choice:** Non-trivial selector, scope, policy, or artifact locator grammars SHOULD be parsed through named parser functions or small parser modules that return typed structures and diagnostics.

**Rationale:** Ad-hoc string splitting spreads grammar rules across the codebase and makes negative cases fragile.

### 5. Preserve Nickel ownership boundaries

**Choice:** Human-authored composable config remains Nickel-preferred where appropriate, with deterministic generated artifacts consumed at runtime. Runtime Rust validators should validate generated or user-facing inputs without requiring dynamic Nickel evaluation in ordinary runner paths.

**Rationale:** This follows the repo's config/receipt source-of-truth policy and keeps runtime behavior deterministic.

## Risks / Trade-offs

- Aggregating too much after malformed syntax can produce confusing follow-on errors; keep syntax and semantic phases separate.
- Stable diagnostic ordering needs explicit sort keys.
- Parser modules should stay small and domain-specific rather than introducing a heavy parser-combinator framework prematurely.

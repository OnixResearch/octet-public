## Context

Octet's selected supplement evidence ladder now includes closed HIR/THIR/MIR fixture selectors and aggregate Valence replay gates. Existing supplement checks compare replay-critical fields, but invalid receipt shapes were not covered by a focused compatibility rail and malformed JSON could surface as a raw command error instead of a deterministic JSON check report.

## Goals / Non-Goals

Goals:
- Add deterministic positive and negative compatibility coverage for supplement receipt replay.
- Return typed `status = "invalid"` JSON reports for invalid supplement receipts.
- Wire a named focused Nix gate into aggregate Valence evidence.

Non-goals:
- No new fixture shape, extractor family, or generalized HIR/THIR/MIR extraction.
- No schema migration behavior beyond fail-closed validation.
- No correctness, semantic-equivalence, pure-cache, cross-machine, or cross-toolchain claims.

## Decisions

### 1. Reuse selected MIR fixture as representative receipt producer

**Choice:** Generate a positive `rustc-mir-fixture` `checked-arithmetic` receipt in the test and derive negative receipt shapes from it.

**Rationale:** The compatibility behavior is owned by generic supplement check validation and replay comparison. A generated v1 receipt avoids checked-in rustc/toolchain identity churn while still exercising the real CLI surface.

**Alternative:** Check in static JSON fixtures for every family. Rejected for this slice because rustc/toolchain fields are intentionally environment-scoped and would create fixture churn unrelated to compatibility semantics.

### 2. Invalid supplement receipts return typed check reports

**Choice:** `supplement check --output-format json` emits `status = "invalid"`, empty comparisons, typed `errors`, and non-proof boundary fields for unreadable, malformed, missing-field, unsupported-version, or wrong-family receipts.

**Rationale:** This matches the fail-closed object-corpus check pattern and gives CI deterministic fields to assert.

### 3. Boundary tamper remains a replay mismatch

**Choice:** Boundary tampering in an otherwise well-formed v1 receipt reports `status = "fail"` with a `boundary` comparison mismatch.

**Rationale:** The receipt is structurally valid but no longer matches current replay evidence, so comparison failure is the clearest operator signal.

## Risks / Trade-offs

- **Overclaim risk:** Mitigated by preserving existing non-proof boundary fields and adding no new extractor support.
- **Fixture churn risk:** Mitigated by generating the positive receipt dynamically from the selected fixture.
- **Aggregate gate cost:** Low; the focused rail is a narrow Cargo test filter.

## Validation Plan

- `TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet supplement_compatibility -- --nocapture'`
- `TMPDIR=/tmp nix build .#checks.x86_64-linux.supplement-compatibility-gate --no-link -L`
- `TMPDIR=/tmp nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`
- `openspec validate add-supplement-compatibility-fixtures --strict --json`
- `openspec validate --all --strict --json`
- `git diff --check`

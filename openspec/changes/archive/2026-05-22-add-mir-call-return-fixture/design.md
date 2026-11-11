## Context

Octet has a fail-closed supplement model for HIR/THIR/MIR evidence. HIR and THIR selected fixtures already demonstrate more than one fixture shape, while MIR currently demonstrates only `checked-arithmetic`. The existing MIR implementation already has a selected fixture inventory, parser-selected paths, deterministic fixture contract hashes, replay checks, and an aggregate Nix gate.

## Goals / Non-Goals

**Goals:**
- Add exactly one new MIR selector: `call-return`.
- Bind it to `cargo-octet/tests/fixtures/mir-spike/call_return.rs::call_then_return`.
- Update parser, runtime fixture registry, contract metadata, docs, and tests in one slice.
- Keep non-fixture/default MIR status `unsupported-planned`.

**Non-Goals:**
- No arbitrary MIR extractor.
- No acceptance of MIR selector aliases.
- No changes to HIR/THIR selected selector sets.
- No stronger proof claims.

## Decisions

### 1. Reuse the selected fixture contract path

**Choice:** Add `RUSTC_MIR_SPIKE_CALL_*` constants and a second `RUSTC_MIR_SPIKE_FIXTURES` entry, reusing the current `rustc_mir_spike_supplement` contract-hash path.

**Rationale:** The current design already hashes `(path, function, shape, function_source)` and emits selected-fixture-only observations; adding a second fixture exercises the same deterministic contract without broadening support.

**Alternative:** Introduce a separate MIR extractor path. Rejected because this slice is only a fixture-backed contract expansion.

**Implementation:** Add a checked-in `call_return.rs` fixture with a single selected function and map parser `call-return` to that path/function for `rustc-mir-fixture`.

### 2. Keep MIR exact selectors only

**Choice:** Continue rejecting `checked` and any other alias for MIR, while accepting exact `checked-arithmetic` and `call-return`.

**Rationale:** Recent MIR contract work intentionally rejected aliases so inventory selector values are the only accepted CLI values.

**Alternative:** Mirror HIR/THIR aliases. Rejected to preserve closed selector semantics and avoid compatibility ambiguity.

### 3. Preserve unsupported-planned metadata for default MIR

**Choice:** Update metadata selected fixture lists and replay text, but keep the MIR supplemental family status `unsupported-planned`.

**Rationale:** Selected fixture runtime support does not imply default or general MIR support.

**Alternative:** Mark the family pilot-supported. Rejected as an overclaim.

## Risks / Trade-offs

**Fixture drift** → Mitigate with parser tests, runtime emit/check tests, and named Nix MIR gate.

**Archive spec replacement drops prior scenarios** → Preserve the existing main `r[hir-mir-supplement-pilot]` scenarios and only modify/add MIR selector scenarios.

**Aggregate object evidence fixture staleness** → Refresh object corpus fixtures only if DTO output changes require it, then rerun aggregate gate.

## Validation plan

- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_parse_accepts_supplement_emit -- --nocapture'`
- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'`
- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_corpus_contract -- --nocapture'`
- `nix build .#checks.x86_64-linux.mir-supplement-fixture-gate --no-link -L`
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`
- `openspec validate --all --strict --json`
- `git diff --check`

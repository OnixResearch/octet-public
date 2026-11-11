## Context

Octet's Valence object-evidence ladder uses bounded checked-in fixture contracts for compiler-derived supplemental evidence. HIR already includes a `branch-control-flow` fixture shape. THIR has a named fixture inventory rail and selected fixture roundtrip coverage for `checked-arithmetic` and `call-return`, but lacks an equivalent control-flow fixture.

## Goals

- Add THIR selected-fixture evidence for `branch-control-flow`.
- Keep inventory, CLI, tests, docs, and aggregate evidence aligned.
- Preserve fail-closed defaults: non-fixture THIR remains `unsupported-planned`.
- Keep MIR branch-control-flow unsupported for this change.

## Non-Goals

- General THIR or MIR extraction.
- Semantic-equivalence or correctness proof claims.
- Changing schema versions or default supplement behavior.

## Decisions

### 1. Use a checked-in THIR fixture mirroring the HIR control-flow shape

**Choice:** Add a THIR fixture source under `cargo-octet/tests/fixtures/thir-spike/` with a branch/control-flow function selected by `--fixture-shape branch-control-flow`.

**Rationale:** A checked-in source fixture keeps the contract replayable and bounded. It lets the inventory and command roundtrip tests exercise the same operator-visible fixture selector as HIR without claiming arbitrary THIR extraction.

**Rejected:** Reusing the HIR fixture path for THIR. That would blur family-specific fixture contracts and make future THIR/MIR evidence boundaries harder to audit.

### 2. Extend only the THIR selected fixture inventory

**Choice:** Update THIR fixture inventory and CLI selector errors to name exactly `checked-arithmetic`, `call-return`, and `branch-control-flow`.

**Rationale:** The evidence ladder needs operator-visible inventory exactness. Updating the CLI messages prevents stale fail-closed diagnostics from incorrectly presenting `branch-control-flow` as HIR-only.

**Rejected:** Adding MIR `branch-control-flow` in the same change. MIR has separate risk and should receive its own fixture/evidence step after THIR is stable.

### 3. Verification remains evidence replay, not proof

**Choice:** Focus verification on inventory exactness, emit/check roundtrip, fail-closed MIR non-selection, aggregate gate composition, and docs boundary strings.

**Rationale:** These are the actual claims being made. Broader compiler-semantic or cross-toolchain properties would overclaim the fixture evidence.

## Risks and Mitigations

- **Risk:** Docs or tests accidentally imply general THIR support. **Mitigation:** Add a docs boundary checker and human boundary review artifact.
- **Risk:** MIR inventory broadens accidentally. **Mitigation:** Negative assertions keep MIR fixture inventory exactly `checked-arithmetic` and `call-return`.
- **Risk:** Aggregate gate output changes incompatibly. **Mitigation:** Preserve existing aggregate PASS lines and add no rename in this change.

## Validation Plan

- `cargo test -p cargo-octet thir_inventory -- --nocapture` for inventory/roundtrip/boundary tests.
- `nix build .#checks.x86_64-linux.thir-supplement-inventory-gate --no-link -L` for the named THIR rail.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L` for aggregate visibility.
- `python3 openspec/changes/archive/2026-05-22-add-thir-branch-control-flow-fixture/evidence/check-doc-boundaries.py` for overclaim prevention.
- `openspec validate --all --strict --json` and `git diff --check` before archive/commit.

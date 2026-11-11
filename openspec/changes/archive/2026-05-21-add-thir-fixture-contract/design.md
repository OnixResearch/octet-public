## Context

`--family thir` currently emits fail-closed `unsupported-planned` receipts. HIR already has a selected fixture inventory and a `rustc-hir-fixture` scope. THIR needs a similarly bounded rail, but it must not borrow HIR's support claims or imply arbitrary rustc THIR extraction.

## Goals / Non-Goals

Goals:
- Add one closed THIR fixture selector: `checked-arithmetic`.
- Bind it to one exact fixture/function pair: `cargo-octet/tests/fixtures/thir-spike/checked.rs::checked`.
- Emit `pilot-supported` THIR supplements only for that selected fixture scope.
- Keep default `--family thir` behavior fail-closed and unsupported-planned.
- Add positive and negative replay evidence plus metadata/docs guardrails.

Non-goals:
- No general THIR extractor.
- No correctness, semantic-equivalence, pure-cache, or cross-toolchain claim.
- No broad fixture discovery beyond the closed selector list.

## Decisions

### 1. Separate THIR fixture scope

**Choice:** Add `rustc-thir-fixture` as a distinct supplement scope instead of overloading `rustc-hir-fixture`.

**Rationale:** The support boundary is different. A separate scope keeps CLI help, parser errors, contract metadata, and negative tests explicit.

**Implementation:** Extend `SupplementScopeArg`, parser help, fixture inventory, and `supplement_records` dispatch. The scope accepts only `--family thir` and exact selector `checked-arithmetic`, fixture path `cargo-octet/tests/fixtures/thir-spike/checked.rs`, and function `checked`.

**User-visible contract:** Root and object help list `--spike rustc-hir-fixture|rustc-thir-fixture`, fixture selectors `--fixture-shape checked-arithmetic|branch-control-flow`, and THIR fixture inventory command `cargo octet object supplement fixtures --family thir --spike rustc-thir-fixture --output-format human|json`. Object help must say: `THIR default supplements are fail-closed unsupported-planned evidence, with only selected rustc-thir-fixture cases reporting pilot-supported. Supplements are not correctness proofs.` Parser errors must include: `object supplement fixtures requires --spike rustc-hir-fixture or --spike rustc-thir-fixture`, `object supplement fixtures for --family thir requires --spike rustc-thir-fixture`, `--fixture-shape requires --spike rustc-hir-fixture or --spike rustc-thir-fixture`, `unsupported supplement spike ... expected rustc-hir-fixture or rustc-thir-fixture`, and `unsupported THIR supplement fixture shape ... expected checked-arithmetic`. Docs must include both `cargo octet object supplement fixtures --family thir --spike rustc-thir-fixture --output-format json` and the default-boundary sentence `The default THIR runtime seam remains fail-closed`.

### 2. Fixture-backed status promotion only

**Choice:** Keep runtime pilot THIR `unsupported-planned`; only the fixture scope emits `pilot-supported`.

**Rationale:** This prevents support creep. A receipt is pilot-supported only when replay inputs bind to the closed fixture selector and exact function.

**Implementation:** Add a dedicated fixture supplement constructor with exact observation keys `fixture_context`, `thir_fixture_contract_hash`, `typed_expression_shape`, and `support_boundary`. The fixture receipt must also preserve receipt-critical fields `schema`, `schema_version`, `family`, `status`, `function_object_hash`, `source_path`, `function_name`, `rustc_version`, `target_triple`, `extractor`, `observations`, `observation_hash`, `extraction_caveats`, and `boundary`. Reuse existing supplement check comparison fields so tampering with schema/status/family/object/function/observations fails closed.

### 2a. Spec-home oracle checkpoint

**Choice:** Keep this delta under `openspec/specs/standards-codification/spec.md` for this change.

**Rationale:** The canonical `HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]` requirement already contains the supplement metadata/CLI contract and the prior THIR fail-closed archive extended that requirement. Moving only this slice would split one supplement contract across two spec homes.

**Implementation:** This change modifies that existing requirement and records the checkpoint in `proposal.md`. A future spec-home cleanup must migrate the whole requirement and all scenarios together, not only this THIR fixture scenario.

### 3. Contract metadata names both support and limits

**Choice:** Update the THIR contract metadata from no selected shapes to the single selected shape while keeping unsupported reasons for non-fixture requests.

**Rationale:** Operators need a machine-readable distinction between selected fixture support and general THIR non-support.

**Implementation:** Contract metadata sets `status=pilot-supported`, `selected_fixture_shapes=["checked-arithmetic"]`, `toolchain_scope=rustc-version-target-and-thir-fixture-contract-scoped`, `fixture_replay_command` with `--family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic`, and false `claims_correctness`, `claims_semantic_equivalence`, and `claims_cross_toolchain_stability`. Evidence snapshots are the saved command transcripts `evidence/thir-fixture-tests.txt` and `evidence/supplement-regression-tests.txt`; the tests assert the emitted JSON fields rather than relying on a separate checked-in golden receipt.

## Risks / Mitigations

- **Overclaiming THIR support:** docs/spec/tests require non-proof and selected-fixture-only wording.
- **Parser/help drift:** parser unit tests and nested help smoke tests assert `rustc-thir-fixture` surfaces.
- **Fixture mismatch false positives:** negative tests exercise wrong path/function/scope mismatches.
- **Archive merge drift:** after archive, compare the canonical spec requirement against `HEAD` and preserve existing scenarios.

## Verification Plan

- Focused supplement tests covering THIR fixture emit/check and default unsupported-planned behavior.
- Contract metadata tests asserting the THIR selected fixture shape and non-proof fields.
- OpenSpec proposal/design/tasks gates.
- `openspec validate --all --strict --json` and `git diff --check` before commit.

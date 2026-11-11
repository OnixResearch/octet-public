## Context

The current THIR implementation has two distinct states that must stay separate: default THIR emit/check and object corpus contract metadata are `unsupported-planned`, while the selected runtime fixture path `--family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic` can emit/check `pilot-supported` evidence for exactly one checked-in fixture.

## Goals / Non-Goals

**Goals:**
- Add exactly one new selected THIR fixture shape, `call-return`.
- Bind it to `cargo-octet/tests/fixtures/thir-spike/call_return.rs::call_then_return`.
- Make inventory, parser/help, docs, runtime metadata, and tests agree on the closed selector set: `checked-arithmetic`, `call-return`.
- Keep default/non-fixture THIR metadata and runtime behavior `unsupported-planned`.

**Non-Goals:**
- No general THIR extractor.
- No branch-control-flow support for THIR; HIR `branch-control-flow` remains rejected for THIR.
- No semantic correctness, cross-toolchain equivalence, dependency resolution, pure-cache, or proof support.

## Decisions

### 1. Closed selector extension

**Choice:** Extend the THIR fixture inventory table with a second `RustcThirSpikeFixture` entry using shape `call-return`, fixture path `cargo-octet/tests/fixtures/thir-spike/call_return.rs`, and function `call_then_return`.

**Rationale:** The existing table-driven path already provides closed selector inventory, fixture/function admission checks, deterministic contract hashing, and negative replay coverage. Extending it keeps the expansion small and reviewable.

**Alternative:** Accept arbitrary THIR fixture paths. Rejected because it would overclaim THIR support and bypass the bounded fixture contract.

### 2. Parser remains explicit and family-aware

**Choice:** `parse_supplement_fixture_shape` accepts `call-return` only for `RustcThirFixtureSpike`, continues requiring an explicit shape for THIR fixture use, and continues rejecting `branch-control-flow` for THIR.

**Rationale:** The parser is the first fail-closed boundary. Keeping shape selection explicit prevents accidental implicit pilot-supported evidence.

### 3. Shape-specific observation detail

**Choice:** Runtime THIR fixture observations will name the selected shape in `fixture_context`, `thir_fixture_contract_hash`, and `typed_expression_shape` detail, with wording broad enough for selected shapes but narrow enough to avoid general THIR claims.

**Rationale:** Existing check comparisons include observation data, so shape/function/path mismatches become deterministic failures.

## Exact Contracts

- Parser accepted THIR fixture selector values added by this change: `call-return` for `--family thir --spike rustc-thir-fixture`; normalized THIR inventory shape output is exactly `checked-arithmetic` or `call-return`. The pre-existing `checked-arithmetic` selector remains supported for the checked fixture.
- Parser rejected THIR fixture selector: `branch-control-flow` with diagnostic containing `unsupported THIR supplement fixture shape `branch-control-flow`; expected `checked-arithmetic` or `call-return``.
- Explicit-shape rule: `--family thir --spike rustc-thir-fixture` without `--fixture-shape` is rejected with a diagnostic containing `requires explicit `--fixture-shape checked-arithmetic` or `--fixture-shape call-return``.
- Inventory contract: JSON `schema=octet.thir-fixture-inventory.v1`, `family=thir`, `scope=rustc-thir-fixture-spike`, `default_shape=checked-arithmetic`, and `shapes` exactly `checked-arithmetic` -> `cargo-octet/tests/fixtures/thir-spike/checked.rs::checked` plus `call-return` -> `cargo-octet/tests/fixtures/thir-spike/call_return.rs::call_then_return`.
- Runtime observation contract: `fixture_context.detail` names the selected `fixture_shape`, fixture path, function, and object hash; `typed_expression_shape.detail` names the selected shape; `support_boundary.detail` contains `selected-fixture-only`.
- Corpus contract: THIR supplemental evidence remains `status=unsupported-planned`; `selected_fixture_shapes` may list `checked-arithmetic` and `call-return` without making the family generally supported.
- Documentation contract: `docs/contributor-guide.md` and `docs/content-addressed-rust-evidence-boundaries.md` name exact commands `cargo octet object supplement emit --family thir --spike rustc-thir-fixture --fixture-shape call-return --output-format json` and `cargo octet object supplement check --family thir --spike rustc-thir-fixture --fixture-shape call-return --receipt RECEIPT.json --output-format json`, bind `call_return.rs::call_then_return`, and state that only explicit selected `rustc-thir-fixture` runtime paths may emit `pilot-supported`; default/non-fixture THIR remains `unsupported-planned`.

## Verification Strategy

- `TMPDIR=/tmp nix develop -c bash -c 'cargo fmt --check && cargo test -p cargo-octet thir_fixture -- --nocapture'` proves inventory, `checked-arithmetic`, `call-return`, and tampered-replay behavior.
- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_parse_accepts_supplement_emit -- --nocapture'` proves parser acceptance/rejection and explicit-shape diagnostics.
- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet supplement -- --nocapture && cargo test -p cargo-octet object_corpus_contract -- --nocapture'` proves docs, inventory smoke, and default THIR contract metadata stay fail-closed.
- `openspec validate add-thir-call-return-fixture --strict --json`, proposal/design/tasks gates, final `openspec validate --all --strict --json`, `git diff --check`, and done-review close the OpenSpec/archive workflow.

## Oracle Checkpoints

- Proposal gate verifies scope stays closed to selected THIR fixture expansion.
- Design gate verifies parser/inventory/runtime/docs/test surfaces are explicitly synchronized.
- Tasks gate verifies each scenario maps to exact command/input/expect/evidence rows.
- Done-review re-checks implementation/spec/archive wording for overclaims before commit/push.

## Risks / Trade-offs

- **Overclaim risk:** Docs or metadata could imply the THIR family is now supported. Mitigation: keep corpus contract status `unsupported-planned` and state selected fixture runtime support only.
- **Selector drift:** Parser, inventory, docs, tests, and spec text could disagree. Mitigation: add positive inventory assertions for both shapes and negative parser coverage for THIR `branch-control-flow`.
- **Archive merge risk:** OpenSpec archive can reshape canonical scenarios. Mitigation: inspect and preserve existing scenarios before commit, then rerun strict all-spec validation and done-review.

## Context

Octet now has selected HIR and THIR fixture rails that prove a narrow pattern: closed selector inventory, parser admission checks, deterministic emit/check receipts, and explicit non-proof boundaries. MIR currently exists as a generic supplement family, but the CLI fixture scope only names HIR/THIR scopes and MIR fixture inventory is rejected. The next step is to add MIR's first selected fixture without implying general MIR extraction.

## Goals / Non-Goals

**Goals:**
- Add exactly one selected MIR fixture shape, `checked-arithmetic`.
- Bind it to `cargo-octet/tests/fixtures/mir-spike/checked.rs::checked`.
- Make parser/help, fixture inventory, runtime metadata, docs, tests, and aggregate gate agree on the closed MIR selector set.
- Keep default/non-fixture MIR metadata and runtime behavior `unsupported-planned`.

**Non-Goals:**
- No general MIR extractor or arbitrary source-path MIR support.
- No branch-control-flow or call-return MIR fixture support in this slice.
- No semantic correctness, cross-toolchain equivalence, dependency resolution, pure-cache, or proof support.

## Decisions

### 1. Add a MIR-specific rustc fixture scope

**Choice:** Introduce `rustc-mir-fixture` / `rustc-mir-fixture-spike` as a distinct CLI scope accepted only with `--family mir`.

**Rationale:** Reusing HIR/THIR scopes would blur extraction semantics and make future evidence hard to audit. A MIR-specific scope keeps the family contract closed and discoverable.

**Alternative:** Let `--family mir --spike pilot` produce fixture-backed evidence. Rejected because the pilot scope is already the fail-closed generic path.

### 2. Table-driven closed selector inventory

**Choice:** Add a MIR fixture inventory table with exactly one entry: shape `checked-arithmetic`, fixture path `cargo-octet/tests/fixtures/mir-spike/checked.rs`, function `checked`, status `pilot-supported`, and selected-fixture-only boundary wording.

**Rationale:** The HIR/THIR fixture tables already make parser, inventory, and runtime evidence deterministic. MIR should use the same shape instead of a separate ad hoc implementation.

**Alternative:** Accept arbitrary fixture paths with `--family mir`. Rejected because it would overclaim MIR support and bypass selected-fixture evidence.

### 3. Runtime supplement remains non-proof evidence

**Choice:** The selected MIR fixture supplement uses schema `octet.function-object-mir-supplement.v1`, status `pilot-supported`, Function Object v1 hash linkage, MIR fixture observation details, and boundary booleans all false for correctness/semantic-equivalence/pure-cache/cross-toolchain claims.

**Rationale:** This aligns MIR with Valence evidence rails while preserving the narrow identity/provenance claim.

### 4. Aggregate gate composition after focused proof

**Choice:** First prove parser/inventory/emit/check behavior with focused `cargo-octet` tests, then extend the aggregate `valence-object-evidence` Nix check to include MIR fixture proof.

**Rationale:** Focused tests isolate selector and receipt bugs; aggregate gate integration gives operators one stable evidence entry point after the slice lands.

## Exact Contracts

- Parser accepted MIR fixture selector values added by this change: `checked-arithmetic` only for `--family mir --spike rustc-mir-fixture`.
- Parser rejected MIR fixture selectors: `branch-control-flow` and `call-return`, with diagnostics naming expected `checked-arithmetic`.
- Explicit-shape rule: `--family mir --spike rustc-mir-fixture` without `--fixture-shape` is rejected with a diagnostic containing `requires explicit --fixture-shape checked-arithmetic`.
- Inventory contract: JSON `schema=octet.mir-fixture-inventory.v1`, `family=mir`, `scope=rustc-mir-fixture-spike`, `default_shape=checked-arithmetic`, and `shapes` exactly `checked-arithmetic` -> `cargo-octet/tests/fixtures/mir-spike/checked.rs::checked`.
- Runtime observation contract: `fixture_context.detail` names selected `fixture_shape`, fixture path, function, and object hash; `mir_fixture_contract_hash.detail` names `fixture_shape=checked-arithmetic` and a BLAKE3 `b3:` contract hash over the selected fixture contract; `support_boundary.detail` contains `selected-fixture-only`.
- Corpus contract: MIR supplemental evidence remains `status=unsupported-planned`; selected fixture metadata may list `checked-arithmetic` without making the family generally supported.
- Documentation contract: operator docs name exact emit/check/fixtures commands and state only explicit selected `rustc-mir-fixture` runtime paths may emit `pilot-supported`.
- Unsupported-shape diagnostic contract: rejected MIR selectors include the stable substring `unsupported MIR supplement fixture shape` and `expected `checked-arithmetic``; missing explicit shape includes the stable substring `requires explicit `--fixture-shape checked-arithmetic``.

## Verification Strategy

- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet mir_fixture -- --nocapture'` proves inventory, emit/check, selected fixture metadata, negative shape rejection, and tampered replay behavior.
- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet object_parse_accepts_supplement_emit -- --nocapture'` proves parser acceptance/rejection and explicit-shape diagnostics.
- `TMPDIR=/tmp nix develop -c bash -c 'cargo test -p cargo-octet supplement -- --nocapture && cargo test -p cargo-octet object_corpus_contract -- --nocapture'` proves docs and default MIR contract metadata stay fail-closed.
- `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L` proves the aggregate Valence evidence rail includes the MIR fixture proof after focused tests pass.
- `openspec validate add-mir-fixture-contract --strict --json`, proposal/design/tasks gates, final `openspec validate --all --strict --json`, `git diff --check`, and done-review close the OpenSpec/archive workflow.

## Oracle Checkpoints

- Proposal gate verifies scope stays closed to selected MIR fixture expansion.
- Design gate verifies parser/inventory/runtime/docs/test surfaces are explicitly synchronized.
- Tasks gate verifies each scenario maps to exact command/input/expect/evidence rows.
- Done-review re-checks implementation/spec/archive wording for overclaims before commit/push.

## Design-Critical Choice Record

- **Question:** Should the MIR fixture observation use a generic observation key or a MIR-specific contract-hash key?
- **Inspected evidence:** Existing THIR fixture observations use shape-specific runtime detail plus deterministic contract-hash wording in `cargo-octet/src/object.rs` and are asserted by `cargo-octet/tests/fingerprint_consumer.rs`.
- **Decision:** Use exact key `mir_fixture_contract_hash` and require its detail to name `fixture_shape=checked-arithmetic` plus a `b3:` contract hash.
- **Owner:** Octet implementation for this change.
- **Next action:** Implement the exact observation and diagnostic contracts before marking I2/I3 complete.

## Risks / Trade-offs

- **Overclaim risk:** Docs or metadata could imply MIR is generally supported. Mitigation: default MIR stays `unsupported-planned` and selected fixture wording is repeated in runtime/docs/spec.
- **Selector drift:** Parser, inventory, docs, tests, and spec text could disagree. Mitigation: focused tests assert exact selector set and aggregate gate composes the proof.
- **Receipt drift:** Adding CLI/object code can stale object corpus fixtures. Mitigation: rerun focused object-corpus refresh if the aggregate gate reports stale receipts.

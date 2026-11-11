## Context

The current selected THIR fixture set is exactly:

- `checked-arithmetic` -> `cargo-octet/tests/fixtures/thir-spike/checked.rs::checked`
- `call-return` -> `cargo-octet/tests/fixtures/thir-spike/call_return.rs::call_then_return`

HIR still owns `branch-control-flow`; THIR must not list or accept it. The inventory command is read-only operator metadata and must not imply general THIR support.

## Goals / Non-Goals

**Goals:**
- Make the THIR inventory rail explicitly covered by unit and integration tests.
- Assert exact selector set, fixture path/function binding, `pilot-supported` selected-fixture entries, and no-proof boundaries.
- Assert inventory-emitted emit/check commands roundtrip for both THIR shapes.

**Non-Goals:**
- Add a new fixture shape.
- Relax default THIR runtime/contract status.
- Claim broad THIR extraction or semantic proof support.

## Decisions

### 1. Reuse the existing fixture inventory surface

**Choice:** Keep `cargo octet object supplement fixtures --family thir --spike rustc-thir-fixture --output-format json|human`.

**Rationale:** Operators already have one read-only discovery surface for supplement fixture selectors. Extending coverage avoids creating a second list command.

**Implementation:** Unit tests inspect the Rust inventory report; integration tests execute the JSON command and then execute each emitted THIR `emit_command`/`check_command`.

### 2. Keep THIR selected-fixture-only status explicit

**Choice:** Inventory entries may report `pilot-supported` only for selected fixture commands; family contract metadata remains `unsupported-planned` outside explicit fixture runtime paths.

**Rationale:** This preserves the THIR fail-closed rail and prevents a discoverability feature from becoming an overclaim.

**Implementation:** Tests assert support boundaries contain selected checked-in rustc THIR fixture wording and boundary booleans remain false.

### 3. Negative control stays in inventory tests

**Choice:** Tests assert THIR inventory excludes `branch-control-flow`.

**Rationale:** `branch-control-flow` remains HIR-only. The inventory rail should make invalid THIR selector drift obvious.

## Verification Strategy

- `cargo test -p cargo-octet thir_fixture_inventory -- --nocapture` proves the Rust inventory report uses schema `octet.thir-fixture-inventory.v1`, lists exactly `checked-arithmetic` and `call-return`, binds fixture paths/functions, marks entries `pilot-supported`, excludes `branch-control-flow`, and preserves boundary booleans; save to `openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-fixture-inventory-tests.txt`.
- `cargo test -p cargo-octet thir_inventory_commands_roundtrip -- --nocapture` proves the JSON inventory command output can drive each emitted THIR `emit_command`/`check_command` and all comparisons match; save to `openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-inventory-roundtrip-tests.txt`.
- `cargo test -p cargo-octet object_corpus_contract_metadata_reports_pilot_and_planned_supplement_statuses -- --nocapture` proves THIR family contract metadata remains `unsupported-planned` while naming selected fixture shapes; save to `openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/thir-contract-boundary-tests.txt`.
- `openspec validate add-thir-fixture-inventory-rail --strict --json` and final `openspec validate --all --strict --json` prove OpenSpec structure; save validation transcripts under the change evidence directory.
- `git diff --check` proves whitespace hygiene; save to `openspec/changes/archive/2026-05-22-add-thir-fixture-inventory-rail/evidence/git-diff-check.txt`.

## Risks / Trade-offs

**Existing implementation already supports THIR inventory** -> This change is intentionally a hardening/evidence rail. It does not add a feature claim beyond focused tests/spec/evidence.

**Archive spec merge drift** -> Inspect canonical `standards-codification` after archive and preserve prior scenarios.

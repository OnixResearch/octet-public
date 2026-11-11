## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST expose bounded supplemental compiler-evidence families for selected Function Object v1 records while preserving Function Object v1 identity and non-proof boundaries. Fixture-backed families MAY report `pilot-supported` only for closed, deterministic fixture selectors with positive and negative replay evidence. Families or request shapes without deterministic fixture evidence MUST remain `unsupported-planned`.

#### Scenario: THIR fixture emits pilot-supported evidence [r[hir-mir-supplement-pilot.scenario.thir-fixture-pilot-supported]]

- GIVEN checked-in THIR fixture selector `checked-arithmetic` bound to exactly `cargo-octet/tests/fixtures/thir-spike/checked.rs::checked`
- WHEN Octet emits a supplement with `--family thir --spike rustc-thir-fixture --fixture-shape checked-arithmetic`
- THEN the supplement uses schema `octet.function-object-thir-supplement.v1` and status `pilot-supported`
- AND its observations name selected fixture shape `checked-arithmetic`, fixture path `cargo-octet/tests/fixtures/thir-spike/checked.rs`, function `checked`, and Function Object v1 hash
- AND its boundary fields continue to reject correctness, semantic-equivalence, pure-cache, and cross-toolchain claims

#### Scenario: THIR fixture check rejects mismatched receipts [r[hir-mir-supplement-pilot.scenario.thir-fixture-mismatch-fails]]

- GIVEN a saved THIR fixture-backed supplement receipt for the selected fixture/function pair
- WHEN Octet checks that receipt against a different function, path, family, scope, or tampered observation data
- THEN the check fails closed with at least one mismatched receipt-critical field
- AND it does not silently downgrade the selected fixture to general THIR support

#### Scenario: THIR default runtime remains unsupported-planned [r[hir-mir-supplement-pilot.scenario.thir-default-remains-planned]]

- GIVEN a non-fixture THIR supplement request using the default pilot scope
- WHEN Octet emits or checks the supplement
- THEN its status remains `unsupported-planned`
- AND docs, contract metadata, and tests distinguish that default behavior from selected fixture support

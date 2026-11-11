## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot supplement records MUST reference the canonical `octet.function-object.v1` object hash, record exact toolchain and extractor identity when available, emit closed schema/status values, and preserve no-correctness/no-semantic-equivalence boundaries. Fixture-backed families MAY report `pilot-supported` only for closed, deterministic fixture selectors with positive and negative replay evidence; families or request shapes without deterministic fixture evidence MUST remain `unsupported-planned`. THIR fixture support MUST stay selector-bounded: default/non-fixture THIR requests and object corpus contract metadata remain `unsupported-planned`, while explicit selected THIR fixture runtime selectors MAY report `pilot-supported` only for their checked-in fixture/function pairs.

#### Scenario: THIR fixture inventory lists selected shapes [r[hir-mir-supplement-pilot.scenario.thir-fixture-inventory-selected-shapes]]

- GIVEN the rustc THIR fixture contract supports checked-in selector values `checked-arithmetic` and `call-return`
- WHEN an operator requests fixture inventory for `--family thir --spike rustc-thir-fixture`
- THEN Octet lists exactly those selected THIR fixture shapes
- AND each entry names its fixture path, function, support status, and selected-fixture-only boundary

#### Scenario: THIR call-return fixture emits pilot-supported evidence [r[hir-mir-supplement-pilot.scenario.thir-call-return-fixture-pilot-supported]]

- GIVEN checked-in THIR fixture selector `call-return` bound to exactly `cargo-octet/tests/fixtures/thir-spike/call_return.rs::call_then_return`
- WHEN Octet emits a supplement with `--family thir --spike rustc-thir-fixture --fixture-shape call-return`
- THEN the supplement uses schema `octet.function-object-thir-supplement.v1` and status `pilot-supported`
- AND its observations name selected fixture shape `call-return`, fixture path `cargo-octet/tests/fixtures/thir-spike/call_return.rs`, function `call_then_return`, and Function Object v1 hash
- AND its boundary fields continue to reject correctness, semantic-equivalence, pure-cache, and cross-toolchain claims

#### Scenario: THIR rejects unsupported fixture shapes [r[hir-mir-supplement-pilot.scenario.thir-unsupported-fixture-shape-rejected]]

- GIVEN `branch-control-flow` remains a HIR-only fixture selector
- WHEN an operator requests `--family thir --spike rustc-thir-fixture --fixture-shape branch-control-flow`
- THEN Octet rejects the selector with a deterministic usage error
- AND no emitted THIR supplement or inventory output implies arbitrary THIR fixture support

#### Scenario: THIR default contract remains unsupported-planned after fixture expansion [r[hir-mir-supplement-pilot.scenario.thir-default-contract-remains-planned-after-expansion]]

- GIVEN selected THIR fixture shapes `checked-arithmetic` and `call-return` have runtime fixture evidence
- WHEN contract metadata or operator documentation describes the THIR family outside an explicit selected fixture runtime path
- THEN its status remains `unsupported-planned`
- AND the metadata names selected fixture shapes without claiming general THIR extraction support

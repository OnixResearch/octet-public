## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot records MUST preserve object-hash linkage, closed schemas/statuses, toolchain/extractor identity when available, and no-correctness/no-semantic-equivalence boundaries. Fixture-backed families MAY report `pilot-supported` only for closed selectors with positive and negative replay evidence; all other families or request shapes MUST remain `unsupported-planned`.

#### Scenario: MIR fixture inventory lists selected shapes [r[hir-mir-supplement-pilot.scenario.mir-fixture-inventory-selected-shapes]]

- GIVEN the rustc MIR fixture contract supports checked-in selector values `checked-arithmetic` and `call-return`
- WHEN an operator requests fixture inventory for `--family mir --spike rustc-mir-fixture`
- THEN Octet lists exactly the selected MIR fixture shapes `checked-arithmetic` and `call-return` in deterministic order
- AND each entry names its fixture path, function, support status, and selected-fixture-only boundary

#### Scenario: MIR call-return fixture emits pilot-supported evidence [r[hir-mir-supplement-pilot.scenario.mir-call-return-fixture-pilot-supported]]

- GIVEN checked-in MIR fixture selector `call-return` bound to exactly `cargo-octet/tests/fixtures/mir-spike/call_return.rs::call_then_return`
- WHEN Octet emits a supplement with `--family mir --spike rustc-mir-fixture --fixture-shape call-return`
- THEN the supplement uses schema `octet.function-object-mir-supplement.v1` and status `pilot-supported`
- AND its observations name selected fixture shape `call-return`, fixture path `cargo-octet/tests/fixtures/mir-spike/call_return.rs`, function `call_then_return`, and Function Object v1 hash
- AND its boundary fields continue to reject correctness, semantic-equivalence, pure-cache, and cross-toolchain claims

#### Scenario: MIR rejects unsupported fixture shapes [r[hir-mir-supplement-pilot.scenario.mir-unsupported-fixture-shape-rejected]]

- GIVEN `branch-control-flow` and selector aliases such as `checked` are not MIR fixture selectors
- WHEN an operator requests `--family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow` or `--fixture-shape checked`
- THEN Octet rejects the selector with a deterministic usage error
- AND no emitted MIR supplement or inventory output implies arbitrary MIR fixture support

#### Scenario: MIR default contract remains unsupported-planned after fixture expansion [r[hir-mir-supplement-pilot.scenario.mir-default-contract-remains-planned-after-expansion]]

- GIVEN selected MIR fixture shapes `checked-arithmetic` and `call-return` have runtime fixture evidence
- WHEN contract metadata or operator documentation describes the MIR family outside an explicit selected fixture runtime path
- THEN its status remains `unsupported-planned`
- AND the metadata names selected fixture shapes without claiming general MIR extraction support

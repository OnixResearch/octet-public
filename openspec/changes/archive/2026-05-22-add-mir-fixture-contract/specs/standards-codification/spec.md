## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot supplement records MUST reference the canonical `octet.function-object.v1` object hash, record exact toolchain and extractor identity when available, emit closed schema/status values, and preserve no-correctness/no-semantic-equivalence boundaries. Fixture-backed families MAY report `pilot-supported` only for closed, deterministic fixture selectors with positive and negative replay evidence; families or request shapes without deterministic fixture evidence MUST remain `unsupported-planned`. MIR fixture support MUST stay selector-bounded: default/non-fixture MIR requests and object corpus contract metadata remain `unsupported-planned`, while explicit selected MIR fixture runtime selectors MAY report `pilot-supported` only for their checked-in fixture/function pairs.

#### Scenario: MIR fixture inventory lists selected shapes [r[hir-mir-supplement-pilot.scenario.mir-fixture-inventory-selected-shapes]]

- GIVEN the rustc MIR fixture contract supports checked-in selector value `checked-arithmetic`
- WHEN an operator requests fixture inventory for `--family mir --spike rustc-mir-fixture`
- THEN Octet lists exactly the selected MIR fixture shape `checked-arithmetic`
- AND the entry names its fixture path, function, support status, and selected-fixture-only boundary

#### Scenario: MIR checked-arithmetic fixture emits pilot-supported evidence [r[hir-mir-supplement-pilot.scenario.mir-checked-arithmetic-fixture-pilot-supported]]

- GIVEN checked-in MIR fixture selector `checked-arithmetic` bound to exactly `cargo-octet/tests/fixtures/mir-spike/checked.rs::checked`
- WHEN Octet emits a supplement with `--family mir --spike rustc-mir-fixture --fixture-shape checked-arithmetic`
- THEN the supplement uses schema `octet.function-object-mir-supplement.v1` and status `pilot-supported`
- AND its observations name selected fixture shape `checked-arithmetic`, fixture path `cargo-octet/tests/fixtures/mir-spike/checked.rs`, function `checked`, and Function Object v1 hash
- AND its boundary fields continue to reject correctness, semantic-equivalence, pure-cache, and cross-toolchain claims

#### Scenario: MIR rejects unsupported fixture shapes [r[hir-mir-supplement-pilot.scenario.mir-unsupported-fixture-shape-rejected]]

- GIVEN `branch-control-flow` and `call-return` are not MIR fixture selectors
- WHEN an operator requests `--family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow` or `--fixture-shape call-return`
- THEN Octet rejects the selector with a deterministic usage error
- AND no emitted MIR supplement or inventory output implies arbitrary MIR fixture support

#### Scenario: MIR fixture scope rejects non-MIR family pairings [r[hir-mir-supplement-pilot.scenario.mir-fixture-scope-rejects-non-mir-family-pairings]]

- GIVEN `rustc-mir-fixture` is the MIR-only selected fixture scope
- WHEN an operator pairs `--spike rustc-mir-fixture` with `--family hir` or `--family thir`
- THEN Octet rejects the command with a deterministic usage error
- AND the diagnostic states the MIR fixture scope only supports `--family mir`

#### Scenario: MIR default contract remains unsupported-planned after fixture expansion [r[hir-mir-supplement-pilot.scenario.mir-default-contract-remains-planned-after-expansion]]

- GIVEN selected MIR fixture shape `checked-arithmetic` has runtime fixture evidence
- WHEN contract metadata or operator documentation describes the MIR family outside an explicit selected fixture runtime path
- THEN its status remains `unsupported-planned`
- AND the metadata names selected fixture shapes without claiming general MIR extraction support

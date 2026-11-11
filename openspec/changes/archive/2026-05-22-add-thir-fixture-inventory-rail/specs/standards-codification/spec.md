## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST keep HIR, MIR, and THIR supplement pilot evidence toolchain-scoped, optional, and non-proof. The THIR family MUST remain `unsupported-planned` by default and for non-fixture runtime paths, while selected rustc THIR fixture inventory entries MAY advertise `pilot-supported` only for explicitly listed checked-in fixture selectors.

#### Scenario: THIR fixture inventory lists selected shapes [r[hir-mir-supplement-pilot.scenario.thir-fixture-inventory-selected-shapes]]

- GIVEN the rustc THIR fixture contract supports checked-in selector values `checked-arithmetic` and `call-return`
- WHEN an operator requests fixture inventory for `--family thir --spike rustc-thir-fixture --output-format json`
- THEN Octet lists exactly those selected THIR fixture shapes
- AND each entry names its fixture path, function, support status `pilot-supported`, emitted replay commands, and selected-fixture-only boundary
- AND the THIR inventory does not list HIR-only selector `branch-control-flow`

#### Scenario: THIR inventory commands roundtrip selected fixture evidence [r[hir-mir-supplement-pilot.scenario.thir-inventory-command-roundtrip]]

- GIVEN the THIR fixture inventory emits replay commands for every listed shape
- WHEN the emitted `emit_command` and `check_command` are executed for each listed THIR selector
- THEN each emitted supplement reports status `pilot-supported` and fixture context for the same selector
- AND each corresponding check report returns `pass` with all compared fields matching

#### Scenario: THIR fixture inventory preserves unsupported-planned family boundary [r[hir-mir-supplement-pilot.scenario.thir-fixture-inventory-default-boundary]]

- GIVEN selected THIR fixture shapes are discoverable through the fixture inventory
- WHEN contract metadata or operator documentation describes the THIR family outside an explicit selected fixture runtime path
- THEN its status remains `unsupported-planned`
- AND the metadata names selected fixture shapes without claiming general THIR extraction, correctness, semantic-equivalence, pure-cache, or cross-toolchain support

## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot records MUST preserve object-hash linkage, closed schemas/statuses, toolchain/extractor identity when available, and no-correctness/no-semantic-equivalence boundaries. Fixture-backed families MAY report `pilot-supported` only for closed selectors with positive and negative replay evidence; all other families or request shapes MUST remain `unsupported-planned`.

#### Scenario: MIR fixture inventory lists selected shapes [r[hir-mir-supplement-pilot.scenario.mir-fixture-inventory-selected-shapes]]

- GIVEN the rustc MIR fixture contract supports checked-in selector values `checked-arithmetic`, `call-return`, and `branch-control-flow`
- WHEN an operator requests fixture inventory for `--family mir --spike rustc-mir-fixture`
- THEN Octet lists exactly the selected MIR fixture shapes `checked-arithmetic`, `call-return`, and `branch-control-flow` in deterministic order
- AND each entry names its fixture path, function, support status, replay commands, and selected-fixture-only boundary

#### Scenario: MIR fixture inventory commands roundtrip selected fixture evidence [r[hir-mir-supplement-pilot.scenario.mir-inventory-commands-roundtrip]]

- GIVEN the MIR fixture inventory emits replay commands for each selected MIR fixture shape
- WHEN the emitted command pair for each inventory row is executed
- THEN `checked-arithmetic`, `call-return`, and `branch-control-flow` each emit `pilot-supported` MIR supplement evidence and replay successfully
- AND no inventory output accepts or implies non-selected MIR fixture shapes

#### Scenario: MIR branch-control-flow selected fixture is replayable [r[hir-mir-supplement-pilot.scenario.mir-branch-control-flow-roundtrip]]

- GIVEN the operator invokes `cargo octet object supplement emit --family mir --spike rustc-mir-fixture --fixture-shape branch-control-flow`
- WHEN the generated receipt is checked with the matching MIR fixture selector
- THEN the check passes, the receipt names `mir-spike/branch_control_flow.rs::choose_branch`, and the boundary fields keep all correctness, semantic-equivalence, pure-cache, and cross-toolchain claims false.

#### Scenario: THIR fixture inventory lists selected branch-control-flow shape [r[hir-mir-supplement-pilot.scenario.thir-branch-control-flow-inventory]]

- GIVEN the rustc THIR fixture contract supports checked-in selector values `checked-arithmetic`, `call-return`, and `branch-control-flow`
- WHEN an operator requests fixture inventory for `--family thir --spike rustc-thir-fixture`
- THEN Octet lists exactly the selected THIR fixture shapes `checked-arithmetic`, `call-return`, and `branch-control-flow` in deterministic order
- AND each entry names its fixture path, function, support status, replay commands, and selected-fixture-only boundary

#### Scenario: THIR branch-control-flow commands roundtrip selected fixture evidence [r[hir-mir-supplement-pilot.scenario.thir-branch-control-flow-roundtrip]]

- GIVEN the THIR fixture inventory emits replay commands for `branch-control-flow`
- WHEN the emitted command pair for `branch-control-flow` is executed
- THEN it emits `pilot-supported` THIR supplement evidence and replays successfully
- AND the emitted supplement names `fixture_shape=branch-control-flow` while preserving no-correctness, no-semantic-equivalence, no-pure-cache, and no-cross-toolchain boundary fields

#### Scenario: Default MIR remains unsupported planned [r[hir-mir-supplement-pilot.scenario.default-mir-remains-unsupported-planned]]

- GIVEN the MIR supplemental evidence contract is emitted
- WHEN the contract is inspected
- THEN MIR remains `unsupported-planned` for default and non-fixture requests and does not claim general MIR extraction.

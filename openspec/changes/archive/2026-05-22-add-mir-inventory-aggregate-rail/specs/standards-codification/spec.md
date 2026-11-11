## MODIFIED Requirements

### Requirement: Aggregate Valence Object Evidence Gate [r[aggregate-valence-object-evidence-gate]]

The system MUST provide a named Nix check that aggregates Octet's selected Valence object-evidence rails for operator and CI use.

The aggregate gate MUST compose the checked-in internal object corpus replay rail, object corpus compatibility fixture rail, object corpus caveat taxonomy rail, bounded local fresh-build object corpus determinism rail, THIR supplement fixture inventory rail, and MIR supplement fixture inventory rail. It MUST preserve the existing non-proof boundaries: object evidence identifies reviewed evidence/provenance bytes and replay inputs, not behavioral correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, or cross-machine/cross-toolchain determinism. The THIR and MIR inventory rails MUST remain selected-fixture-only and MUST NOT imply general THIR or MIR extraction.

#### Scenario: Aggregate gate composes object evidence rails

- GIVEN a maintainer invokes the aggregate Valence object evidence Nix check
- WHEN the check evaluates and builds
- THEN it depends on the object corpus replay, compatibility, caveat taxonomy, fresh-build determinism, THIR supplement fixture inventory, and MIR supplement fixture inventory rails
- AND it exits successfully only when all composed rails succeed

#### Scenario: Aggregate gate composes MIR inventory rail [r[aggregate-valence-object-evidence-gate.scenario.mir-inventory-rail]]

- GIVEN the focused MIR supplement inventory rail checks the selected MIR fixture inventory and inventory-listed emit/check commands
- WHEN the aggregate Valence object evidence check builds
- THEN it depends on that MIR inventory rail
- AND its output reports `valence_object_evidence.mir_supplement_inventory_gate: PASS`
- AND the existing `valence_object_evidence.mir_supplement_fixture_gate: PASS` line remains available until any compatibility rename is explicitly retired

#### Scenario: Contributor docs name the aggregate command and MIR boundaries

- GIVEN a contributor reads the contributor guide
- WHEN they look for Valence object-evidence verification
- THEN the guide names the aggregate Nix check command
- AND it states the aggregate gate is bounded evidence replay rather than a correctness, semantic-equivalence, exhaustive-coverage, or cross-machine determinism proof
- AND it states the aggregate includes selected THIR and MIR fixture inventory coverage without implying general THIR or MIR extraction

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot records MUST preserve object-hash linkage, closed schemas/statuses, toolchain/extractor identity when available, and no-correctness/no-semantic-equivalence boundaries. Fixture-backed families MAY report `pilot-supported` only for closed selectors with positive and negative replay evidence; all other families or request shapes MUST remain `unsupported-planned`.

#### Scenario: MIR fixture inventory lists selected shapes [r[hir-mir-supplement-pilot.scenario.mir-fixture-inventory-selected-shapes]]

- GIVEN the rustc MIR fixture contract supports checked-in selector values `checked-arithmetic` and `call-return`
- WHEN an operator requests fixture inventory for `--family mir --spike rustc-mir-fixture`
- THEN Octet lists exactly the selected MIR fixture shapes `checked-arithmetic` and `call-return` in deterministic order
- AND each entry names its fixture path, function, support status, replay commands, and selected-fixture-only boundary

#### Scenario: MIR fixture inventory commands roundtrip selected fixture evidence [r[hir-mir-supplement-pilot.scenario.mir-inventory-commands-roundtrip]]

- GIVEN the MIR fixture inventory emits replay commands for each selected MIR fixture shape
- WHEN the emitted command pair for each inventory row is executed
- THEN `checked-arithmetic` and `call-return` each emit `pilot-supported` MIR supplement evidence and replay successfully
- AND no inventory output accepts or implies non-selected MIR fixture shapes

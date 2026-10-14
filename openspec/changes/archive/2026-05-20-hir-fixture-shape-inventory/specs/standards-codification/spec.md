## MODIFIED Requirements

### Requirement: Rustc-backed HIR extractor spike [r[rustc-hir-extractor-spike]]

Octet MUST provide bounded, fixture-backed rustc HIR extractor spike evidence for selected Function Object v1 records while preserving the v1 object hash as the canonical review/provenance identity. Each supported HIR spike fixture shape MUST be a checked-in, explicitly selected fixture with exact rustc/toolchain identity, extractor identity, schema version, fixture context, observation hash, and explicit non-proof boundaries. Octet MUST expose the closed rustc HIR fixture-shape inventory through a deterministic operator surface so valid selectors and their bounded meanings are discoverable without accepting arbitrary HIR inputs.

#### Scenario: HIR fixture inventory lists closed selectors

- GIVEN the rustc HIR spike supports the checked-in fixture shapes `checked-arithmetic` and `branch-control-flow`
- WHEN an operator requests the HIR fixture inventory for `--spike rustc-hir-fixture`
- THEN Octet lists exactly the supported closed selector values
- AND each entry includes fixture shape, source path, function name, support status, and whether it is the default shape

#### Scenario: HIR fixture inventory preserves bounded support claims

- GIVEN an operator reads fixture inventory output in JSON or human format
- WHEN the output describes rustc HIR spike support
- THEN it names the support as selected-fixture-backed and toolchain-scoped
- AND it preserves no-correctness, no-semantic-equivalence, no-pure-cache, and no-cross-toolchain-stability boundary fields
- AND it does not claim arbitrary HIR, THIR, MIR, exhaustive Rust coverage, or semantic proof support

#### Scenario: HIR fixture inventory rejects unscoped requests

- GIVEN fixture inventory is only meaningful for the rustc HIR fixture spike
- WHEN an operator requests fixture inventory without `--family hir` and `--spike rustc-hir-fixture`
- THEN Octet rejects the request with a deterministic usage error
- AND no inventory output implies support for arbitrary supplement families or spikes

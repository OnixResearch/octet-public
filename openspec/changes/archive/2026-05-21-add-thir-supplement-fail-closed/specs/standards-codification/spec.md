## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]
Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot supplement records MUST reference the canonical `octet.function-object.v1` object hash, record exact toolchain and extractor identity when available, emit closed schema/status values, and preserve no-correctness/no-semantic-equivalence boundaries.

#### Scenario: THIR supplement emits unsupported-planned evidence [r[hir-mir-supplement-pilot.scenario.thir-unsupported-planned]]

- GIVEN a selected Rust function that can emit a Function Object v1 record
- WHEN Octet emits a pilot THIR supplement for that function with `--family thir`
- THEN the supplement uses schema `octet.function-object-thir-supplement.v1` and status `unsupported-planned`
- AND it records observations or caveats explaining that THIR extraction is not implemented
- AND it preserves no-correctness, no-semantic-equivalence, no-pure-cache, and no-cross-toolchain-stability boundaries

#### Scenario: THIR supplement check replays fail-closed receipt [r[hir-mir-supplement-pilot.scenario.thir-check-replay]]

- GIVEN a saved THIR supplement receipt emitted by the fail-closed pilot path
- WHEN Octet checks that receipt against the same selected function with `--family thir`
- THEN the report status is `pass` only when compared receipt-critical fields match
- AND the report continues to describe the THIR family as unsupported-planned evidence rather than correctness, semantic-equivalence, or extractor support

## MODIFIED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support bounded, fail-closed supplemental evidence families for selected Function Object v1 records. Pilot records MUST preserve object-hash linkage, closed schemas/statuses, toolchain/extractor identity when available, and no-correctness/no-semantic-equivalence boundaries. Fixture-backed families MAY report `pilot-supported` only for closed selectors with positive and negative replay evidence; all other families or request shapes MUST remain `unsupported-planned`.

Supplement receipt checks MUST fail closed for malformed JSON, missing replay-critical fields, unsupported future schema versions, wrong receipt families, and boundary tampering while preserving no-correctness and no-semantic-equivalence boundaries.

#### Scenario: Supplement compatibility accepts positive v1 receipt [r[hir-mir-supplement-pilot.scenario.supplement-compat-positive-v1]]

- GIVEN a generated selected-fixture supplement v1 receipt for a supported family and selector
- WHEN `cargo octet object supplement check --output-format json` replays the receipt with the matching selector
- THEN the check exits successfully with `status = "pass"`
- AND the report contains no typed errors

#### Scenario: Supplement compatibility rejects invalid receipts [r[hir-mir-supplement-pilot.scenario.supplement-compat-invalid-receipts]]

- GIVEN malformed, missing-field, unsupported-future-version, or wrong-family supplement receipt inputs
- WHEN `cargo octet object supplement check --output-format json` validates them
- THEN the command exits non-zero with `status = "invalid"`
- AND the report contains typed errors naming the failing kind and field while preserving non-proof boundary fields

#### Scenario: Supplement boundary tamper fails closed [r[hir-mir-supplement-pilot.scenario.supplement-boundary-tamper]]

- GIVEN a selected-fixture supplement v1 receipt whose boundary claims are tampered
- WHEN `cargo octet object supplement check --output-format json` replays it with the matching selector
- THEN the command exits non-zero with `status = "fail"`
- AND the comparison list names the `boundary` field as a mismatch

### Requirement: Aggregate Valence Object Evidence Gate [r[aggregate-valence-object-evidence-gate]]

The system MUST provide a named Nix check that aggregates Octet's selected Valence object-evidence rails for operator and CI use.

The aggregate gate MUST compose the checked-in internal object corpus replay rail, object corpus compatibility fixture rail, object corpus caveat taxonomy rail, object corpus fixture freshness rail, supplement compatibility rail, bounded local fresh-build object corpus determinism rail, THIR supplement fixture inventory rail, and MIR supplement fixture inventory rail. It MUST preserve the existing non-proof boundaries: object evidence identifies reviewed evidence/provenance bytes and replay inputs, not behavioral correctness, semantic equivalence, pure-cache eligibility, exhaustive Rust coverage, or cross-machine/cross-toolchain determinism. The THIR and MIR inventory rails MUST remain selected-fixture-only and MUST NOT imply general THIR or MIR extraction.

#### Scenario: Aggregate gate composes supplement compatibility rail [r[aggregate-valence-object-evidence-gate.scenario.supplement-compatibility-rail]]

- GIVEN the focused supplement compatibility rail checks positive and negative supplement receipt replay behavior
- WHEN the aggregate Valence object evidence check builds
- THEN it depends on that supplement compatibility rail
- AND its output reports `valence_object_evidence.supplement_compatibility_gate: PASS`
- AND the aggregate non-claims continue to exclude correctness, semantic equivalence, exhaustive Rust coverage, cross-machine determinism, and cross-toolchain determinism

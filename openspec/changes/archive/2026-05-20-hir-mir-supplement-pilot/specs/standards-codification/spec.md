## ADDED Requirements

### Requirement: HIR/MIR supplement pilot [r[hir-mir-supplement-pilot]]

Octet MUST support a bounded, fixture-backed HIR/MIR supplement pilot for selected Function Object v1 records. Pilot supplement records MUST reference the canonical `octet.function-object.v1` object hash, record exact toolchain and extractor identity, emit closed schema/status values, and preserve no-correctness/no-semantic-equivalence boundaries.

#### Scenario: Pilot HIR supplement attaches to v1 object identity [r[hir-mir-supplement-pilot.scenario.hir-attaches-v1]]

- GIVEN a selected fixture function with an emitted Function Object v1 record
- WHEN Octet emits a pilot HIR supplement for that function
- THEN the supplement references the v1 `object_hash`, source path, and function name
- AND it records rustc/toolchain identity, extractor name/version, HIR supplement schema version, and unresolved dependency or macro-origin caveats
- AND it does not replace the v1 object hash as review/provenance identity

#### Scenario: Pilot MIR supplement records unsupported summary families [r[hir-mir-supplement-pilot.scenario.mir-unsupported-families]]

- GIVEN a selected fixture function where only some MIR summary families are supported by the pilot extractor
- WHEN Octet emits a pilot MIR supplement
- THEN the supplement records rustc/toolchain identity, extractor name/version, MIR supplement schema version, and supported call/panic/drop/async/unsafe/allocation/effect summaries where available
- AND every absent or incomplete summary family is represented by an explicit unknown or unsupported caveat
- AND the output does not claim cross-toolchain stability, behavioral correctness, semantic equivalence, or pure-cache eligibility

#### Scenario: Supplement check fails closed on mismatched context [r[hir-mir-supplement-pilot.scenario.check-fails-closed]]

- GIVEN a saved pilot supplement receipt whose object hash, source path, toolchain identity, extractor version, or schema version differs from the current extraction context
- WHEN Octet checks the receipt against the current fixture input
- THEN the check fails with a deterministic mismatch report naming the compared field
- AND the report preserves the no-correctness and no-semantic-equivalence boundary fields

#### Scenario: Unsupported supplement family remains planned [r[hir-mir-supplement-pilot.scenario.unsupported-remains-planned]]

- GIVEN a HIR, THIR, or MIR supplement family without deterministic checked-in fixture evidence
- WHEN contract metadata or operator documentation describes that family
- THEN its status remains `unsupported-planned`
- AND the metadata names the required fixture evidence before support can be claimed

## MODIFIED Requirements

### Requirement: HIR/MIR object evidence supplement contract

Octet MUST define HIR/MIR object evidence as optional, toolchain-scoped supplements to `function-object-v1` rather than replacements for source-token object identity. For the bounded pilot, any emitted supplement MUST be attached to a selected Function Object v1 object, MUST include exact toolchain/extractor/schema identity, and MUST include explicit caveats for every unsupported or incomplete evidence family.

#### Scenario: HIR supplement preserves v1 identity boundary

- GIVEN a Function Object v1 record with source-token object identity
- WHEN a pilot HIR evidence supplement is attached to that object
- THEN the supplement records its rustc/toolchain identity, extractor identity, schema version, and extraction caveats
- AND the Function Object v1 `object_hash` remains a review/provenance identity rather than a correctness or semantic-equivalence proof

#### Scenario: MIR supplement is version and context scoped

- GIVEN a pilot MIR evidence supplement for an object
- WHEN the supplement records MIR-derived observations
- THEN it records the rustc version, extractor identity, MIR context or optimization assumptions, and unsupported-context caveats
- AND it does not claim cross-toolchain stability, behavioral correctness, or semantic equivalence from matching MIR evidence

#### Scenario: Supplement metadata fails closed before implementation claims

- GIVEN Octet has not implemented a HIR, THIR, or MIR extractor for a requested evidence family with deterministic fixture evidence
- WHEN contract metadata or operator documentation describes HIR/MIR evidence support
- THEN it reports the family as planned or unsupported rather than silently emitting partial support
- AND any future implementation must add deterministic fixture evidence before claiming support

### Requirement: HIR/MIR supplement contract metadata

Octet MUST expose HIR/MIR supplemental evidence families in object corpus contract metadata with closed support statuses. Families without deterministic extractor fixtures MUST remain `unsupported-planned`; bounded pilot families with positive and negative fixture evidence MAY report `pilot-supported` while preserving toolchain-scoped non-proof boundaries.

#### Scenario: Contract metadata names planned HIR/MIR supplements

- GIVEN `cargo octet object corpus contract --output-format json`
- WHEN the contract metadata is inspected
- THEN it lists HIR, THIR, and MIR supplemental evidence families with schema IDs, schema versions, required future fields, support status, and relationship to Function Object v1

#### Scenario: Planned supplements fail closed before extractor implementation

- GIVEN Octet has not implemented deterministic fixtures for a HIR, THIR, or MIR supplement family
- WHEN contract metadata describes that supplement status
- THEN the family reports `unsupported-planned` rather than supported or partial support
- AND the family records an unsupported reason and required deterministic fixture evidence before support can be claimed

#### Scenario: Pilot-supported metadata remains bounded

- GIVEN a supplement family has deterministic positive and negative pilot fixtures
- WHEN contract metadata reports that family as `pilot-supported`
- THEN it names the exact schema, toolchain scope, extractor identity expectations, required receipt/check fields, and fixture replay command
- AND it states the pilot is bounded supplement evidence, not correctness, semantic-equivalence, pure-cache, cross-toolchain, or exhaustive Rust coverage proof

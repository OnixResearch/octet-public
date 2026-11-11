## ADDED Requirements

### Requirement: HIR/MIR object evidence supplement contract

Octet MUST define HIR/MIR object evidence as optional, toolchain-scoped supplements to `function-object-v1` rather than replacements for source-token object identity.

#### Scenario: HIR supplement preserves v1 identity boundary

- GIVEN a Function Object v1 record with source-token object identity
- WHEN a future HIR evidence supplement is attached to that object
- THEN the supplement records its rustc/toolchain identity and extraction caveats
- AND the Function Object v1 `object_hash` remains a review/provenance identity rather than a correctness or semantic-equivalence proof

#### Scenario: MIR supplement is version and context scoped

- GIVEN a future MIR evidence supplement for an object
- WHEN the supplement records MIR-derived observations
- THEN it records the rustc version, MIR context or optimization assumptions, and unsupported-context caveats
- AND it does not claim cross-toolchain stability, behavioral correctness, or semantic equivalence from matching MIR evidence

#### Scenario: Supplement metadata fails closed before implementation claims

- GIVEN Octet has not implemented a HIR or MIR extractor for a requested evidence family
- WHEN contract metadata or operator documentation describes HIR/MIR evidence support
- THEN it reports the family as planned or unsupported rather than silently emitting partial support
- AND any future implementation must add deterministic fixture evidence before claiming support

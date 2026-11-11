## ADDED Requirements

### Requirement: Function Object Internal Corpus Pilot [r[function-object-internal-corpus-pilot]]

The system MUST support an Octet-internal function-object corpus pilot that emits deterministic machine-readable receipt summaries for a bounded set of Octet Rust source files.

The receipt MUST include the selected source paths, object count, deterministic object-set hash, caveat distribution, pure-cache blocking count, replay command, and an explicit boundary statement that the receipt is review/provenance evidence rather than a correctness or semantic-equivalence proof.

#### Scenario: Expanded internal corpus emits a deterministic receipt

- GIVEN a bounded Octet source corpus broader than the original object/fingerprint/CLI three-file pilot
- WHEN the corpus receipt command runs twice against the same inputs
- THEN both JSON receipts are byte-identical
- AND the receipt records all selected source paths and an object count greater than the original three-file pilot threshold

#### Scenario: Receipt preserves no-correctness boundary

- GIVEN the internal object corpus contains functions with unresolved dependency, effect, or macro caveats
- WHEN the corpus receipt is emitted
- THEN the receipt reports caveat counts and pure-cache blockers
- AND it states that object hashes are review/provenance evidence, not correctness proofs or semantic equivalence proofs

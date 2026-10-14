## Context

Octet is already evidence-heavy, but a reviewer often has to infer how an artifact relates to the policy or requirement it supports. external evidence tooling's provenance framing suggests making the chain explicit rather than relying on file names or docs prose.

The provenance model should be pure data: deterministic records over in-memory artifact metadata, with file discovery and hashing kept in the imperative shell. BLAKE3 is the default content hash for new artifact identity.

## Approach

1. Define a `ProvenanceRecord` DTO with source ID, checker ID, evidence tier, artifact locator, BLAKE3 hash, scope, and optional parent artifact references.
2. Keep the core validation function pure: it receives records and known registries, then returns validation diagnostics.
3. Let the runner or standards CLI shell gather files, compute BLAKE3 hashes, and serialize records.
4. Register the emitted record family in `standards/policies/registry.ncl` and keep generated contracts fresh.
5. Update release/operator docs to show requirement or policy ID to artifact lookup.

## Risks

- Existing artifacts may not have stable source IDs. Mitigate by applying the requirement to new or versioned evidence summaries first and documenting migration behavior.
- Hashing local generated artifacts can be noisy. Mitigate by hashing deterministic sidecars and transcripts, not volatile terminal output.
- Provenance can become too verbose. Mitigate by making parent references optional and keeping the required record compact.

## Validation

- Positive: a fixture provenance record with known source ID, checker ID, tier, locator, and BLAKE3 hash validates.
- Negative: missing source ID, unknown checker ID, and hash mismatch all fail closed with specific diagnostics.
- Cairn validation and gates pass for this change.

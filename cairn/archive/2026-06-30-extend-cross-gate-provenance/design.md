## Context

Octet's evidence stack has multiple producers. The current provenance model has fields for source ID, checker ID, evidence tier, artifact locator, BLAKE3 hash, checked scope, and parent artifacts, but only the runner artifact directory currently emits a sidecar.

Cross-gate provenance should reuse the same DTO and validation core where possible. Each producer shell should gather its artifacts and supply producer-specific source IDs while the pure core validates record consistency.

## Approach

1. Define the initial provenance producer set: `cargo-octet`, `octet-standards`, policy validation, fingerprint review manifests, transcript assertions, and generated policy artifacts.
2. Add a registry of known checker IDs and source ID families used by provenance verification.
3. Extend producer shells one at a time to emit or validate `ProvenanceRecord` values for deterministic artifacts.
4. Use `parent_artifacts` to link derived summaries, policy bundles, and release smoke receipts back to the artifacts they summarize.
5. Keep evidence tiers precise: standards and policy checks use `policy_check`, UI and transcript rails use their specific tiers, and only verifier-backed proof receipts may use `formal_proof`.

## Risks

- Cross-gate scope could become too broad. Mitigate by adding producers incrementally and keeping each record tied to deterministic artifacts only.
- Existing artifacts may lack stable source IDs. Mitigate with explicit source-family mapping and migration notes.
- Parent chains can be cyclic or noisy. Mitigate with validation that rejects missing parents and cycles while allowing compact optional parent lists.

## Validation

- Positive: a fixture release chain links runner output, policy validation, fingerprint review, and transcript assertion records with stable hashes and parent references.
- Negative: unknown checker ID, missing parent artifact, parent-cycle, hash mismatch, invalid tier assignment, and formal-proof overclaim all fail closed.
- Cairn validation and gates pass for this change.

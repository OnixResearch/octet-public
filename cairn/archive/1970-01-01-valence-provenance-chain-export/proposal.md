## Why

Octet already records the source/checker/tier/artifact identities needed for stack traceability. Valence should own cross-stack semantic verification, but it should not have to reverse-engineer Octet's artifact directory without a stable Octet-owned export seam.

This change adds an optional Valence provenance export from verified Octet artifacts. The export is adapter input for Valence, not a release claim by itself and not a Mantle dependency.

## What Changes

- Add an Octet command or artifact-verifier flag that emits a Valence-compatible provenance export after Octet artifact verification succeeds.
- Include Octet provenance records, proof cards, proof receipts, parent artifact links, checker IDs, evidence tiers, and BLAKE3 artifact identities.
- Preserve evidence-tier boundaries: validation/review evidence remains validation/review evidence; only validated verifier-backed receipts can support `formal_proof`.
- Fail closed when the export would reference missing parents, stale hashes, malformed proof cards, unknown checker IDs, or imported Trellis evidence that has not been rerun under an Octet-owned verifier.
- Document how the Valence adapter consumes the export and how Mantle bundles the final sidecar opaquely.

## Impact

- **Octet owns its artifact semantics**: Valence receives stable adapter input instead of reimplementing Octet-specific discovery rules.
- **Valence owns cross-stack verification**: the export feeds Valence but does not make Octet responsible for release-level stack traceability.
- **Mantle remains independent**: Mantle receives only the final sidecar as opaque external evidence.
- **Non-claims**: the export is provenance input, not proof of whole-program correctness, release eligibility, or semantic equivalence.

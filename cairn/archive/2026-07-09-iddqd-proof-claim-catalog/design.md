## Context

The proof claim catalog defines bounded proof/property claim ids, allowed aliases, evidence tiers, required artifacts, documentation anchors, and non-claim boundaries. It is reviewable JSON and should remain easy to diff. The weakness is in runtime indexing: validation currently builds independent sets and maps, while reference resolution scans claim and alias vectors.

## Design

### Catalog indexes

Build a pure catalog-index value after deserialization:

- an `IdOrdMap<ProofClaimEntry>` keyed by `claim_id`;
- an `IdOrdMap<ProofClaimAlias>` keyed by `alias_id`;
- deterministic diagnostics collected during index construction.

Use ordered maps for deterministic iteration and diagnostics. Do not use iddqd hash maps for externally supplied catalog ids. Use unique insertion so duplicate claim ids and duplicate aliases fail closed.

### Alias resolution

Reference validation resolves a proof/property id by first checking canonical claims, then aliases. Alias rows may target the same claim, so this is not a bijection. Keep target existence, self-target, alias-vs-claim conflict, status, tier-promotion, required artifact, documentation anchor, and non-claim checks intact.

### Serialization boundary

The public proof claim catalog remains vector-shaped JSON. iddqd indexes are built inside the pure validation core and are not emitted as the wire format.

### Fixtures

Positive fixtures cover canonical claim lookup, alias lookup, deprecated aliases, required artifacts, and documentation anchors. Negative fixtures cover duplicate claim ids, duplicate aliases, alias/claim id conflict, alias self-target, missing alias target, invalid status, invalid evidence tier, invalid formal-proof promotion, missing required artifact, missing anchor, and weakened non-claims.

## Non-goals

- Do not change proof-card, proof-ref, or provenance export JSON schemas.
- Do not promote imported Trellis artifacts or manual-review evidence to formal proof.
- Do not require aliases to be one-to-one with claims.

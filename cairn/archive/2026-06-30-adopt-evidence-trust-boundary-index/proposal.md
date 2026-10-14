## Why

The external evidence tooling pages are persuasive because each concept links to a concrete page: how it works, proofs, deployment, and research. Octet has comparable material spread across lint references, safety guidance, standards docs, rollout docs, and release smoke transcripts. A single index should connect these surfaces so readers can move from positioning to evidence without hunting.

This change is intentionally documentation-only. It creates an index for the adoption themes from the external evidence tooling review and helps future work land in the right place.

## What Changes

- Add a trust-boundary/evidence index that links checked-path framing, evidence stack, evidence tiers, provenance, scope boundaries, and controlled deployment.
- Keep the index as navigation, not a new claim surface.
- Ensure every linked theme points to an existing or planned Octet doc/artifact surface.

## Impact

- **Files**: `README.md`, `docs/consumer-guide.md`, or a new docs index page.
- **Testing**: positive link inventory; negative check that the index contains no unsupported verification claims.

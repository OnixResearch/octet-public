# Policy package customs gate

The local customs gate checks policy/config/template/proof-rail/generated-contract packages before promotion.

A package manifest uses `standards/policy-packages/contracts.ncl` and declares package ID, kind, owner, source paths, generated artifacts, registry family IDs, generation and validation commands, fixtures, provenance, license notes, non-claims, and manual review actions.

`octet-standards customs-gate` validates the manifest against:

- registry coverage in `standards/policies/registry.ncl`
- positive and negative fixture coverage
- generated artifact freshness through a BLAKE3 manifest
- checked-in source and fixture inventory
- no remote fetches during validation
- raw-secret exclusion or non-candidate rationale

The report is deterministic and local. Passing customs does not trust package authors, prove package safety, prove vulnerability freedom, or replace repository review.

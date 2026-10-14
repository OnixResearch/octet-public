# API contract rail

Octet adapts Oxide API tooling patterns as reference-only inputs for machine-contract evidence. The rail keeps Octet's source of truth explicit: runtime receipts and command JSON come from Rust DTOs, human-composed policy/config remains Nickel-authored or Nickel-exported, standards outputs name their checker, and external schemas are review inputs with declared boundaries.

## Reference inventory

| Reference | Use in Octet | Boundary |
| --- | --- | --- |
| Dropshot | explicit typed service/API surfaces and generated OpenAPI documents | reference only; no hidden middleware authority is imported |
| Progenitor | generated-client fixture pattern | client success proves contract consumption only |
| Typify | schema/Rust interop pattern | external schema inputs need declared source boundaries; Octet DTOs remain authoritative |
| Drift | compatibility diff pattern | diff output is review evidence, not release authorization |
| openapi-lint | contract linting pattern | lint success does not prove service behavior |
| dropshot-api-manager | managed promotion workflow pattern | Octet/Cairn gates remain the promotion authority |

## Compatibility classes

- `additive`: optional fields or supported non-breaking additions accepted by policy.
- `deprecated`: retained fields or aliases with a documented retirement path.
- `breaking`: removed required fields or tightened requirements; fails closed without an explicit change record.
- `malformed`: hidden source authority, missing supported claim, or missing non-claim boundary.
- `unsupported`: a generator, schema dialect, or fixture profile outside the declared rail.

Generated-client fixtures must carry the non-claim: generated client success proves contract consumption only, not API behavior correctness. Passing this rail does not prove service correctness, evidence truth, release eligibility, behavioral compatibility, or exhaustive API coverage.

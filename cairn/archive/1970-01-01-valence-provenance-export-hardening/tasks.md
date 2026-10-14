## Phase 1: Receipt envelope and edge vocabulary

- [x] [serial] r[octet.evidence_provenance.valence_export.replayable_receipts] Define the replayable receipt envelope fields in Rust-owned DTOs and generated Nickel contract coverage.
- [x] [serial] r[octet.evidence_provenance.valence_export.typed_edges] Add policy-owned typed edge kinds for Octet checks, proof receipts, imported Trellis references, parent artifacts, and Valence adapter inputs.
- [x] [serial] r[octet.evidence_provenance.valence_export.tier_preservation_hardened] Enforce that imported Trellis references remain manual-review or reference inputs unless an Octet-owned proof receipt validates them.

## Phase 2: Export validation and fixtures

- [x] [parallel] r[octet.evidence_provenance.valence_export.negative_fixtures] Add positive fixtures for valid lint, standards, and formal-proof exports.
- [x] [parallel] r[octet.evidence_provenance.valence_export.negative_fixtures] Add negative fixtures for stale digest, missing parent, unknown producer, unsupported edge kind, missing non-claim, and manual-review tier promotion.
- [x] [serial] r[octet.evidence_provenance.valence_export.replayable_receipts] Refuse to write the export when artifact verification, parent linkage, producer registry, or edge vocabulary checks fail.

## Phase 3: Operator docs and validation

- [x] [serial] r[octet.evidence_provenance.valence_export.boundary_hardened] Update operator documentation to state the export is Valence adapter input and not a release, whole-program, or Mantle bundle claim.
- [x] [serial] r[octet.evidence_provenance.valence_export.replayable_receipts] Run artifact verification tests, export fixture tests, policy freshness checks, and Cairn validation/gates before archive.

## Verification Coverage

| Scenario | Planned Evidence |
|---|---|
| Valid export with replayable receipts passes | Positive fixture test |
| Typed edges are preserved | Positive fixture test |
| Stale digest fails closed | Negative fixture test |
| Missing parent fails closed | Negative fixture test |
| Unknown producer fails closed | Negative fixture test |
| Unsupported edge kind fails closed | Negative fixture test |
| Manual-review tier promotion fails closed | Negative fixture test |
| Export boundary is documented | Docs check or transcript |

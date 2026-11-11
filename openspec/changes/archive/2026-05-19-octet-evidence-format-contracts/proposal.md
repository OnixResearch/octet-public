## Why

Octet owns Valence's Rust extraction/object/receipt surface, so the file-format policy for Octet evidence belongs in this repository. The current implementation uses Markdown for OpenSpec/evidence narratives and deterministic JSON for function objects, bundles, corpus receipts, corpus checks, and schema-hardened invalid-input reports. That is the right v1 operational default, but Octet needs an explicit contract so reviewers do not mistake `.json` or `.md` for the semantic compatibility source.

## What Changes

- Define Markdown as Octet's human review/lifecycle artifact format for proposals, designs, tasks, and evidence notes.
- Define deterministic JSON as the current v1 machine receipt, fixture, and CLI-output encoding for Octet object evidence.
- Require emitted receipt compatibility to be owned by Rust DTOs and versioned schema identifiers, with Nickel contract metadata used for human-authored policy/catalog registration where applicable.
- Preserve canonical hashing boundaries: deterministic content only, no self-referential hash fields, no ambient host entropy.
- Make future binary/packed encodings additive unless a later migration explicitly replaces the JSON review surface.
- Preserve non-claims: hashes and file formats identify evidence/provenance, not correctness or semantic equivalence.

## Capabilities

### Modified Capabilities
- `standards-codification`: Adds file-format and receipt-contract ownership rules for Octet content-addressed Rust evidence.

## Impact

- **Files**: Archived OpenSpec proposal/design/tasks/spec under `openspec/changes/archive/2026-05-19-octet-evidence-format-contracts/`; synced accepted spec on archive.
- **APIs**: No CLI behavior change in this slice.
- **Dependencies**: None.
- **Testing**: Validate this change strictly, validate all OpenSpec content, and run whitespace checks.

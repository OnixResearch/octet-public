## Phase 1: Implementation

- [x] I1 Add Nickel/Rust proof-bundle schema models and deterministic artifact emission (evidence: `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-export.txt`, `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/focused-checks.txt`)
- [x] I2 Add artifact-backed proof-bundle schema validation and CLI diagnostics (evidence: `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-unsupported-schema.txt`, `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-complete.txt`)
- [x] I3 Add positive and negative proof-bundle fixtures/tests (evidence: `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-focused-rust.txt`)

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Proof bundle export writes versioned artifact" with artifact export assertions; save output at `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-export.txt`
- [x] V2 Verify scenario "Artifact-backed check rejects unsupported proof bundle schema" with a CLI negative fixture; save output at `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-unsupported-schema.txt`
- [x] V3 Verify scenario "Artifact-backed check accepts complete proof bundle" with a CLI positive fixture; save output at `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-complete.txt`
- [x] V4 Verify focused Rust/Nickel/OpenSpec/whitespace checks; save output at `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/focused-checks.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`review diff for proof-bundle schema grouping`; input=`generated proof-bundle artifact and artifact-backed checker diff`; asserts=`proof metadata is grouped per policy/rail and not only flattened TOML arrays`; evidence=`openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/schema-grouping-review.txt`

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Proof bundle export writes versioned artifact` -> `V1` -> `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-export.txt`
- `Scenario: Artifact-backed check rejects unsupported proof bundle schema` -> `V2` -> `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-unsupported-schema.txt`
- `Scenario: Artifact-backed check accepts complete proof bundle` -> `V3` -> `openspec/changes/archive/2026-05-06-enforce-versioned-proof-bundle-schema/evidence/proof-bundle-complete.txt`

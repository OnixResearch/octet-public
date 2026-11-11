## Phase 1: Pilot surface

- [x] I1 Add bounded HIR/MIR supplement DTOs, closed status values, and canonical serialization helpers for selected fixture functions.
- [x] I2 Add pilot supplement emit/check command surface while preserving Function Object v1 as the canonical review/provenance identity.
- [x] I3 Update object corpus contract metadata so fixture-backed pilot families can report `pilot-supported` and all other HIR/THIR/MIR families remain `unsupported-planned`.
- [x] I4 Add contributor/operator documentation for the pilot command, replay boundaries, and non-proof language.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Pilot HIR supplement attaches to v1 object identity" with focused cargo-octet fixture tests; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/hir-pilot-attachment.txt`.
- [x] V2 Verify scenario "Pilot MIR supplement records unsupported summary families" with focused cargo-octet fixture tests; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/mir-pilot-caveats.txt`.
- [x] V3 Verify scenario "Supplement check fails closed on mismatched context" with negative receipt fixtures; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/supplement-check-negative.txt`.
- [x] V4 Verify scenario "Unsupported supplement family remains planned" with contract metadata fixture checks; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/unsupported-remains-planned.txt`.
- [x] V5 Verify scenario "HIR supplement preserves v1 identity boundary" with docs/output boundary checks; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/hir-boundary.txt`.
- [x] V6 Verify scenario "MIR supplement is version and context scoped" with docs/output boundary checks; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/mir-boundary.txt`.
- [x] V7 Verify scenario "Supplement metadata fails closed before implementation claims" with contract metadata fixture checks; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/fail-closed-metadata.txt`.
- [x] V8 Verify scenario "Contract metadata names planned HIR/MIR supplements" with `cargo octet object corpus contract --output-format json`; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/contract-metadata-planned.txt`.
- [x] V9 Verify scenario "Planned supplements fail closed before extractor implementation" with unsupported-family negative metadata checks; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/planned-fail-closed.txt`.
- [x] V10 Verify scenario "Pilot-supported metadata remains bounded" with contract metadata and contributor docs checks; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/pilot-supported-bounded.txt`.
- [x] V11 Verify aggregate Valence object evidence with `nix build .#checks.x86_64-linux.valence-object-evidence --no-link -L`; save output at `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/valence-object-evidence.txt`.

## Change Status

- Status: complete
- Open tasks: none

## Verification Coverage

- `Scenario: Pilot HIR supplement attaches to v1 object identity` -> `V1` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/hir-pilot-attachment.txt`
- `Scenario: Pilot MIR supplement records unsupported summary families` -> `V2` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/mir-pilot-caveats.txt`
- `Scenario: Supplement check fails closed on mismatched context` -> `V3` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/supplement-check-negative.txt`
- `Scenario: Unsupported supplement family remains planned` -> `V4` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/unsupported-remains-planned.txt`
- `Scenario: HIR supplement preserves v1 identity boundary` -> `V5` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/hir-boundary.txt`
- `Scenario: MIR supplement is version and context scoped` -> `V6` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/mir-boundary.txt`
- `Scenario: Supplement metadata fails closed before implementation claims` -> `V7` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/fail-closed-metadata.txt`
- `Scenario: Contract metadata names planned HIR/MIR supplements` -> `V8` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/contract-metadata-planned.txt`
- `Scenario: Planned supplements fail closed before extractor implementation` -> `V9` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/planned-fail-closed.txt`
- `Scenario: Pilot-supported metadata remains bounded` -> `V10` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/pilot-supported-bounded.txt`
- `Aggregate Valence object evidence gate` -> `V11` -> `openspec/changes/archive/2026-05-20-hir-mir-supplement-pilot/evidence/valence-object-evidence.txt`

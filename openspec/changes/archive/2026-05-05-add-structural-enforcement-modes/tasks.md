## Phase 1: Implementation

- [x] I1 Extend the policy registry schema/export with current and target enforcement modes plus mechanism metadata.
- [x] I2 Add Nickel closed contracts for policies, enforcement modes, mechanisms, evidence, suppressions, proof rails, and seed manifests.
- [x] I3 Add Nickel cross-field validation for structural mode, verified mode, deny severity, suppressions, and proof-relevant seeds.
- [x] I4 Implement policy validation for unknown modes, missing escalation criteria, missing mechanisms, and invalid fallback declarations.
- [x] I5 Implement the first structural core/shell checker slice for no-std core declarations, forbidden core effects, and shell-to-core dependency direction.
- [x] I6 Add capability-constructor validation for approved shell-owned authority wrappers and injected core values.
- [x] I7 Add type-state and bounded-primitive validation fixtures and checker hooks for configured structural policies.
- [x] I8 Add verified-mode metadata and proof-rail validation for backend, command, source/proof mapping, pass signal, and trust-boundary accounting.
- [x] I9 Add reference seed manifest support for optional imports from sibling repositories.
- [x] I10 Add minimal UCAN-style, verified-logic-style, and Vericoder-style seed fixtures or vendored examples only after manifest validation exists.
- [x] I11 Update docs, templates, and machine-output metadata to distinguish lint, gate, structural, verified, and human/oracle findings.

## Phase 2: Deterministic verification

- [x] V1 Verify scenario "Lint-only policy remains explicit" with policy-schema fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- [x] V2 Verify scenario "Target mode stronger than current mode records escalation" with policy-schema fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- [x] V3 Verify scenario "Closed Nickel policy contract rejects unknown fields" with Nickel negative policy fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- [x] V4 Verify scenario "Structural mode requires structural mechanism contract" with Nickel cross-field fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- [x] V5 Verify scenario "Verified mode requires proof rail contract" with Nickel cross-field fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- [x] V6 Verify scenario "Deny severity requires deterministic checker and evidence" with Nickel cross-field fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- [x] V7 Verify scenario "Crate boundary contract exports core and shell obligations" with Nickel mechanism export fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- [x] V8 Verify scenario "Capability contract exports constructor authority" with Nickel mechanism export fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- [x] V9 Verify scenario "Bounded and type-state contracts export API obligations" with Nickel mechanism export fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- [x] V10 Verify scenario "Seed corpus contract exports provenance and role" with Nickel seed manifest export fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- [x] V11 Verify scenario "No-std core boundary is enforced" with core/shell structural fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- [x] V12 Verify scenario "Shell-to-core dependency direction is enforced" with crate-graph fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- [x] V13 Verify scenario "Capability constructors are shell-owned" with positive and negative capability fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/capability-structural-fixtures.txt`
- [x] V14 Verify scenario "Type-state protocol surfaces reject raw operations" with positive and negative type-state fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- [x] V15 Verify scenario "Bounded primitive requirement rejects unbounded replacement" with positive and negative bounded primitive fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- [x] V16 Verify scenario "Verified policy requires proof command" with verified-mode manifest fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- [x] V17 Verify scenario "Proof-relevant code cannot be imported as trusted by default" with seed manifest negative fixture; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- [x] V18 Verify scenario "Vericoder seed is accepted as architecture or trajectory fixture" with seed manifest fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- [x] V19 Verify scenario "Verified-logic seed requires local proof boundary" with proof-relevant seed manifest fixtures; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- [x] V20 Verify scenario "UCAN seed demonstrates core-shell vendoring shape" with UCAN-style seed fixture; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- [x] V21 Verify scenario "Machine output classifies enforcement failure type" with machine-output fixture parsing; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/enforcement-output.txt`
- [x] V22 Verify scenario "Nix or CI gate fails on missing structural evidence" with local-first Nix/check fixture; save output at `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/enforcement-output.txt`

## Phase 3: Human/oracle checkpoints

- [x] H1 Human/oracle checkpoint: command=`manual review of seed manifest and vendored paths before accepting any sibling import`; input=`source commits and manifests for ../vericoder-mvp/, ../verified-logic/, and ../ucan/`; asserts=`license/provenance and role classification are sufficient for every vendored seed`; evidence=`openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/seed-provenance-review.txt`

## Change Status
- Status: complete
- Open tasks: none
## Verification Coverage

- `Scenario: Lint-only policy remains explicit` -> `V1` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- `Scenario: Target mode stronger than current mode records escalation` -> `V2` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/policy-mode-schema.txt`
- `Scenario: Closed Nickel policy contract rejects unknown fields` -> `V3` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- `Scenario: Structural mode requires structural mechanism contract` -> `V4` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- `Scenario: Verified mode requires proof rail contract` -> `V5` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- `Scenario: Deny severity requires deterministic checker and evidence` -> `V6` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-policy-contracts.txt`
- `Scenario: Crate boundary contract exports core and shell obligations` -> `V7` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- `Scenario: Capability contract exports constructor authority` -> `V8` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- `Scenario: Bounded and type-state contracts export API obligations` -> `V9` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- `Scenario: Seed corpus contract exports provenance and role` -> `V10` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/nickel-mechanism-contracts.txt`
- `Scenario: No-std core boundary is enforced` -> `V11` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- `Scenario: Shell-to-core dependency direction is enforced` -> `V12` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/core-shell-structural-fixtures.txt`
- `Scenario: Capability constructors are shell-owned` -> `V13` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/capability-structural-fixtures.txt`
- `Scenario: Type-state protocol surfaces reject raw operations` -> `V14` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- `Scenario: Bounded primitive requirement rejects unbounded replacement` -> `V15` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/typestate-bounded-fixtures.txt`
- `Scenario: Verified policy requires proof command` -> `V16` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- `Scenario: Proof-relevant code cannot be imported as trusted by default` -> `V17` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/verified-seed-proof-rail.txt`
- `Scenario: Vericoder seed is accepted as architecture or trajectory fixture` -> `V18` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- `Scenario: Verified-logic seed requires local proof boundary` -> `V19` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- `Scenario: UCAN seed demonstrates core-shell vendoring shape` -> `V20` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/reference-seed-manifests.txt`
- `Scenario: Machine output classifies enforcement failure type` -> `V21` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/enforcement-output.txt`
- `Scenario: Nix or CI gate fails on missing structural evidence` -> `V22` -> `openspec/changes/archive/2026-05-05-add-structural-enforcement-modes/evidence/enforcement-output.txt`

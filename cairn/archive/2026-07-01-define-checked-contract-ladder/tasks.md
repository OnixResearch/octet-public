## Contract

- [x] [serial] Define the progressive assurance ladder stages and evidence-tier boundaries. r[octet.checked_contract_ladder.stages]
- [x] [serial] Define checked-contract fields and mapping to existing Octet evidence surfaces. r[octet.checked_contract_ladder.contract_records]
- [x] [serial] Define high-confidence diagnostic UX expectations for deterministic failure shapes. r[octet.checked_contract_ladder.diagnostic_shapes]
- [x] [serial] Define forbidden wording for language-level contracts, solver completeness, runtime fallback, and blanket formal-proof claims. r[octet.checked_contract_ladder.boundaries]

## Implementation

- [x] [serial] Add the assurance-ladder explanation to the canonical trust-boundary or consumer documentation. r[octet.checked_contract_ladder.stages]
- [x] [serial] Add checked-contract field definitions and examples that reference existing lint, policy, proof-card, provenance, and artifact-verifier surfaces. r[octet.checked_contract_ladder.contract_records]
- [x] [serial] Add or update examples for high-confidence diagnostics that name failure shape and remediation boundary without SMT-counterexample wording. r[octet.checked_contract_ladder.diagnostic_shapes]
- [x] [serial] Link the ladder from README, lint reference, and Verus proof-path docs where appropriate. r[octet.checked_contract_ladder.stages]
- [x] [serial] Add deterministic documentation checks for ladder stages, required fields, diagnostic-shape wording, and forbidden overclaims. r[octet.checked_contract_ladder.boundaries]

## Verification

- [x] [serial] Run a positive ladder inventory proving every assurance stage is named with evidence tier, command or gate, and artifact/receipt surface. r[octet.checked_contract_ladder.stages]
- [x] [serial] Run a positive checked-contract inventory proving every required field appears in definitions and examples. r[octet.checked_contract_ladder.contract_records]
- [x] [serial] Run a positive diagnostic-shape check proving examples name invalid state, boundary, or missing guard when claimed. r[octet.checked_contract_ladder.diagnostic_shapes]
- [x] [serial] Run a negative wording check rejecting Salt-style syntax claims, solver-completeness claims, runtime-fallback claims, and blanket formal-proof claims. r[octet.checked_contract_ladder.boundaries]
- [x] [serial] Run `nix run .#cairn -- validate --root .` and Cairn gates for this change. r[octet.checked_contract_ladder.boundaries]

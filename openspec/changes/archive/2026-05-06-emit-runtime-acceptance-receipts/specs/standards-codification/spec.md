## MODIFIED Requirements

### Requirement: Structured Acceptance Receipts [r[structured-acceptance-receipts]]

The system MUST treat structured receipts, not raw logs, as acceptance records for proof/evidence/config review.

Acceptance receipts MUST include schema version, run or receipt ID, timestamps, tool/version or git revision, config digest for Nickel-authored inputs, inputs, stage outcomes, artifacts, status, warnings/errors, and verification commands or evidence references when applicable.

#### Scenario: Artifact export emits digest-bound receipt [r[structured-acceptance-receipts.scenario.artifact-export-emits-digest-bound-receipt]]

- GIVEN an exported Nickel run profile JSON and a policy registry JSON
- WHEN `tigerstyle-standards export-artifacts --profile-json <profile> --receipt-out <receipt>` runs
- THEN the command writes a structured acceptance receipt
- AND the receipt `config_digest` equals the BLAKE3 digest of the profile JSON bytes
- AND the receipt lists the generated artifact bundle and verification command evidence without embedding raw logs

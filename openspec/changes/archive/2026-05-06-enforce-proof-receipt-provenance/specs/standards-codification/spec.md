## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]

Verified enforcement MUST name the proof backend, source/proof mapping, expected command, accepted proof artifact shape, trust-boundary accounting, receipt binding, and receipt provenance obligations when the rail is promoted to hard verified enforcement. Verified enforcement MUST also provide artifact-backed checker diagnostics when declared local proof evidence is absent, does not contain the configured pass signal, has mismatched receipt binding, or lacks required provenance/trust-boundary lines.

#### Scenario: Proof receipt provenance obligations exported [r[verified-enforcement-rails.receipt-provenance-artifact]]

- GIVEN the Nickel registry declares proof receipt provenance obligations for a verified proof rail
- WHEN generated policy artifacts are exported
- THEN the check config and policy JSON expose the receipt schema, runner, environment, backend, and trust-boundary obligations for deterministic consumer checks

#### Scenario: Command-generated receipt includes provenance [r[verified-enforcement-rails.receipt-provenance-generated]]

- GIVEN a verified proof rail with receipt provenance obligations enabled
- WHEN `tigerstyle-standards check --execute-proof-rails` runs the local command and produces a proof receipt
- THEN the generated receipt includes deterministic schema, backend, runner, environment, and trust-boundary lines before validation
- AND the verified proof rail checker accepts the receipt only when those provenance lines match the policy contract

#### Scenario: Stale receipt without provenance rejected [r[verified-enforcement-rails.receipt-provenance-stale]]

- GIVEN a proof receipt with the expected pass signal and matching hash binding but missing required provenance or trust-boundary lines
- WHEN the verification-family checker evaluates the proof rail
- THEN the checker MUST emit a blocking diagnostic for mismatched receipt provenance

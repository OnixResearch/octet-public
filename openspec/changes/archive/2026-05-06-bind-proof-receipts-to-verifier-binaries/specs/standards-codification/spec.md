## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]
Verified enforcement MUST require deterministic proof rails that bind proof outcomes to checked source, proof commands, receipt provenance, verifier identity, verifier binary identity, and toolchain identity before the checker reports the policy as satisfied.

#### Scenario: Verified receipts account for verifier binary digests [r[verified-enforcement-rails.verifier-binary-accounting]]
- GIVEN a verified proof rail declares verifier binary path and digest metadata
- WHEN a proof receipt contains the pass signal and all other required binding metadata
- THEN the checker MUST also require exact verifier binary path and BLAKE3 digest receipt lines before accepting the receipt

#### Scenario: Command-generated receipts bind the running verifier binary [r[verified-enforcement-rails.generated-verifier-binary]]
- GIVEN `--execute-proof-rails` runs a verified proof command with configured verifier binary metadata
- WHEN the command-generated receipt is checked
- THEN the generated receipt MUST include the running verifier executable path and BLAKE3 digest metadata

#### Scenario: Generated artifacts expose verifier binary digest obligations [r[verified-enforcement-rails.export-verifier-binary]]
- GIVEN a policy registry declares verifier binary receipt metadata for a verified rail
- WHEN generated policy artifacts are exported
- THEN the checker configuration artifact MUST expose verifier binary path and digest obligations for downstream consumers

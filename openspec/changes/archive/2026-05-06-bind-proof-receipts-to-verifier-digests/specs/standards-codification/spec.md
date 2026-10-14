## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]
Verified enforcement MUST require deterministic proof rails that bind proof outcomes to checked source, proof commands, receipt provenance, verifier identity, and toolchain identity before the checker reports the policy as satisfied.

#### Scenario: Verified receipts account for verifier and toolchain identity [r[verified-enforcement-rails.verifier-toolchain-accounting]]
- GIVEN a verified proof rail declares verifier and toolchain receipt metadata
- WHEN a proof receipt contains the pass signal and all other required binding metadata
- THEN the checker MUST also require exact verifier and toolchain identity/digest receipt lines before accepting the receipt

#### Scenario: Receipt-only checks reject missing verifier/toolchain digests [r[verified-enforcement-rails.reject-missing-verifier-toolchain]]
- GIVEN a checked proof receipt has correct source hashes, command hashes, pass signal, and provenance lines
- WHEN the receipt omits the configured verifier or toolchain digest lines
- THEN receipt-only checking MUST emit a blocking verified diagnostic instead of accepting the receipt

#### Scenario: Generated artifacts expose verifier/toolchain digest obligations [r[verified-enforcement-rails.export-verifier-toolchain]]
- GIVEN a policy registry declares verifier and toolchain receipt metadata for a verified rail
- WHEN generated policy artifacts are exported
- THEN the checker configuration artifact MUST expose verifier and toolchain identity/digest obligations for downstream consumers

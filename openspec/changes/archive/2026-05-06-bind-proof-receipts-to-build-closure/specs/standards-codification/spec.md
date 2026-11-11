## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]
Tigerstyle policy artifacts MUST define verified-mode proof rails that bind proof evidence to deterministic commands, source coverage, receipt provenance, verifier identity, and build closure identity.

#### Scenario: Verified receipts account for verifier build closure identity [r[verified-enforcement-rails.build-closure-accounting]]
- GIVEN a verified proof rail with configured verifier build-closure identity and digest metadata
- WHEN a proof receipt contains the pass signal but omits or changes the build-closure receipt lines
- THEN the standards checker MUST emit a hard verified diagnostic for mismatched receipt verifier metadata

#### Scenario: Executed proof rails emit build closure receipt metadata [r[verified-enforcement-rails.generated-build-closure]]
- GIVEN an executed proof rail with build-closure manifest paths
- WHEN `tigerstyle-standards check --execute-proof-rails` synthesizes an in-memory receipt
- THEN the generated receipt MUST include `proof-verifier-build-closure:` and `proof-verifier-build-closure-blake3:` lines derived from the configured manifest

#### Scenario: Generated policy artifacts export build closure obligations [r[verified-enforcement-rails.export-build-closure]]
- GIVEN the Nickel policy registry declares verifier build-closure obligations
- WHEN `tigerstyle-standards export-artifacts` renders checker configuration
- THEN the generated artifact MUST include build-closure identity, digest, and manifest path fields for downstream checkers

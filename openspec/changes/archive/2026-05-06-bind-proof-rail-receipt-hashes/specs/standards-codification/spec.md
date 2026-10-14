## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]
Verified-mode policies MUST declare local proof rails with deterministic command metadata, covered source paths, proof receipt paths, expected pass signals, trust-boundary notes, and receipt binding when the rail is promoted to hard verified enforcement.

#### Scenario: Command and source hash-bound receipt accepted [r[verified-enforcement-rails.receipt-binding-pass]]

- GIVEN a verified proof rail with receipt binding enabled, a checked source file, and a proof command
- WHEN `tigerstyle-standards check --execute-proof-rails` runs the local command and produces a proof receipt
- THEN the receipt includes deterministic command and source BLAKE3 binding lines
- AND the verified proof rail checker accepts the receipt only when those binding lines match the checked command and source contents

#### Scenario: Stale proof receipt rejected [r[verified-enforcement-rails.receipt-binding-stale]]

- GIVEN a proof receipt with the expected pass signal but command or source hash lines that do not match the checked proof rail command or source contents
- WHEN the verification-family checker evaluates the proof rail
- THEN the checker MUST emit a blocking diagnostic for the mismatched receipt binding

#### Scenario: Receipt binding exported in policy artifacts [r[verified-enforcement-rails.receipt-binding-artifact]]

- GIVEN the Nickel registry declares a proof rail receipt binding mode
- WHEN generated policy artifacts are exported
- THEN the check config and policy JSON expose the receipt binding mode for deterministic consumer checks

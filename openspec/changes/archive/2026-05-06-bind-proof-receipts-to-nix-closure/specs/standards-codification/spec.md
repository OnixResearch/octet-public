## MODIFIED Requirements

### Requirement: Verified Enforcement Rails [r[verified-enforcement-rails]]

Verified enforcement rails MUST bind proof receipts to command/source hashes, receipt provenance, verifier/toolchain identity, verifier binary identity, build-closure identity, and Nix derivation/recursive closure identity when those obligations are configured by policy artifacts.

#### Scenario: Receipt records Nix derivation and recursive closure identity [r[verified-enforcement-rails.nix-closure-identity]]

- GIVEN a verified proof rail configured with verifier Nix derivation and recursive closure metadata
- WHEN receipt-only proof checking evaluates a passing proof receipt
- THEN the checker MUST require matching `proof-verifier-nix-derivation`, `proof-verifier-nix-derivation-blake3`, `proof-verifier-nix-closure`, and `proof-verifier-nix-closure-blake3` lines
- AND generated proof command receipts MUST include those lines when `--execute-proof-rails` is enabled
- AND generated policy artifacts MUST export the configured Nix derivation and closure obligations for downstream checkers

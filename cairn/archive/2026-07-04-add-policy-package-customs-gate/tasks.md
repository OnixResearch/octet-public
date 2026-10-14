## Contract

- [x] [serial] Define the local policy-package customs gate and package manifest fields. r[octet.policy_package_customs.gate]
- [x] [serial] Define fixture, export, generated-artifact, and registry coverage requirements for promotable packages. r[octet.policy_package_customs.fixtures]
- [x] [serial] Define package boundary, secret-exclusion, provenance, and non-claim requirements. r[octet.policy_package_customs.boundaries]

## Implementation

- [x] [serial] Add the package manifest contract and examples for policy, profile, template, and generated-contract packages. r[octet.policy_package_customs.gate]
- [x] [serial] Add pure customs validation for registry coverage, fixture coverage, generated artifact freshness, provenance, and package status. r[octet.policy_package_customs.fixtures]
- [x] [serial] Add the shell command or standards gate that reads manifests, invokes export/freshness checks when configured, and writes deterministic reports. r[octet.policy_package_customs.gate]
- [x] [serial] Add secret-exclusion checks and non-candidate rationale handling for packages that cannot be manifest-backed. r[octet.policy_package_customs.boundaries]
- [x] [serial] Document package submission, promotion, manual-review actions, and non-claims. r[octet.policy_package_customs.boundaries]

## Verification

- [x] [serial] Add a complete valid package fixture that passes customs and emits deterministic report output. r[octet.policy_package_customs.gate]
- [x] [serial] Add negative fixtures for missing registry entry, missing positive fixture, missing negative fixture, stale generated artifact, missing provenance, and unknown source path. r[octet.policy_package_customs.fixtures]
- [x] [serial] Add secret-bearing and remote-fetch fixtures that fail or require explicit non-candidate exclusion. r[octet.policy_package_customs.boundaries]
- [x] [serial] Run targeted customs core tests, shell/report tests, policy freshness checks, and Cairn validation/gates. r[octet.policy_package_customs.gate]

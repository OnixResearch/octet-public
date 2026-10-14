## Contract

- [x] [serial] Specify that owned workspace warning-clean checks are release-readiness evidence. r[octet.build_hygiene.warning_clean]
- [x] [serial] Specify the allowed classifications for generator/helper code that would otherwise be dead code. r[octet.build_hygiene.dead_code_classification]
- [x] [serial] Specify compatibility-binary warning coverage. r[octet.build_hygiene.alias_targets]

## Implementation

- [x] [serial] Classify `cargo-octet/src/evidence_contracts.rs` as live command code, test-only code, or obsolete code, and refactor accordingly. r[octet.build_hygiene.dead_code_classification]
- [x] [serial] Add a warning-deny check rail under the Nix devshell or flake checks. r[octet.build_hygiene.warning_clean]
- [x] [serial] Update contributor or release docs to name the warning-clean rail. r[octet.build_hygiene.alias_targets]

## Verification

- [x] [serial] Run `nix develop -c cargo check --workspace` and assert zero owned-code warnings. r[octet.build_hygiene.warning_clean]
- [x] [serial] Add a negative warning fixture or compile check proving new dead code fails the warning-deny rail. r[octet.build_hygiene.dead_code_classification]
- [x] [serial] Run the warning-deny rail for `cargo-octet`, `cargo-slotcar`, and `cargo-tigerstyle` targets. r[octet.build_hygiene.alias_targets]

## Contract

- [x] [serial] Specify warning-clean alias target declarations for canonical and compatibility binaries. r[octet.build_hygiene.alias_target_declarations]
- [x] [serial] Specify behavior preservation for canonical and compatibility binary version, help, exit-code, and command-routing surfaces. r[octet.build_hygiene.alias_target_behavior]
- [x] [parallel] Specify the warning text that the regression check must reject. r[octet.build_hygiene.alias_target_warning_regression]

## Implementation

- [x] [serial] Refactor `cargo-octet` so shared command execution lives behind a reusable entry point and each shipped binary has its own wrapper file. r[octet.build_hygiene.alias_target_declarations]
- [x] [serial] Refactor `octet-standards` aliases the same way without changing command semantics. r[octet.build_hygiene.alias_target_declarations]
- [x] [parallel] Update Nix/package smoke surfaces if binary paths or wrapper names change. r[octet.build_hygiene.alias_target_behavior]

## Verification

- [x] [serial] Add positive tests or smoke assertions for `cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`, `octet-standards`, `slotcar-standards`, and `tigerstyle-standards`. r[octet.build_hygiene.alias_target_behavior]
- [x] [serial] Add a negative regression test or check fixture proving duplicate-target warning text fails the warning-clean rail. r[octet.build_hygiene.alias_target_warning_regression]
- [x] [serial] Run `nix develop -c cargo check`, targeted package tests, release smoke if touched, and Cairn validation/gates for this change. r[octet.build_hygiene.alias_target_warning_regression]

## Contract

- [x] [serial] Specify that the release-readiness Clippy seam includes pedantic lint coverage. r[octet.build_hygiene.clippy_pedantic]
- [x] [serial] Specify that pedantic findings use narrow fixes or narrow justified allows instead of broad pedantic suppression. r[octet.build_hygiene.clippy_pedantic]

## Implementation

- [x] [serial] Fix current owned-code pedantic findings from the baseline run. r[octet.build_hygiene.clippy_pedantic]
- [x] [serial] Update `checks.clippy` to enable `clippy::pedantic` with warnings denied. r[octet.build_hygiene.clippy_pedantic]
- [x] [serial] Update contributor docs to name the pedantic Clippy rail. r[octet.build_hygiene.clippy_pedantic]

## Verification

- [x] [serial] Run `nix develop -c cargo clippy --workspace --all-targets -- -D warnings -W clippy::pedantic`. r[octet.build_hygiene.clippy_pedantic]
- [x] [serial] Run `nix build .#checks.x86_64-linux.clippy --no-link -L`. r[octet.build_hygiene.clippy_pedantic]
- [x] [serial] Run focused tests for touched logic plus Cairn validation/gates. r[octet.build_hygiene.clippy_pedantic]

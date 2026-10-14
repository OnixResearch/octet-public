## Why

Dependency vulnerability and license checks answer only part of the supply-chain question. Octet's evidence model should also support human dependency-audit coverage: which third-party crates were reviewed, which trust imports were accepted, which exceptions remain, and which reviewer action is still required. `cargo-vet` already provides the strongest Rust-native workflow for this and should be adopted as an optional receipt source.

The integration should keep dependency trust separate from source linting and separate from vulnerability scanning, while preserving local-first review evidence.

## What Changes

- Add optional `cargo-vet` receipt import or execution support.
- Record audit coverage, imports, exemptions, unaudited gaps, command argv, and config roots.
- Emit `manual_review_required` or `policy_check` evidence according to whether gaps remain.
- Document dependency trust evidence as review support, not a guarantee that dependencies are benign.

## Non-Goals

- Do not require all Octet consumers to use `cargo-vet`.
- Do not create an Octet web-of-trust service.
- Do not silently trust third-party imports without showing their source.
- Do not replace `cargo-deny` advisory/license checks.

## Impact

- **Files**: runner dependency-trust module, docs, artifact schemas, policy registry checker IDs, fixtures, tests, and Cairn specs.
- **Behavior**: optional dependency trust receipts can be bundled and verified with Octet evidence.
- **Testing**: positive fully-vetted fixture, negative unaudited dependency fixture, exemption/manual-review fixture, import identity fixture, artifact verification, and Cairn gates.

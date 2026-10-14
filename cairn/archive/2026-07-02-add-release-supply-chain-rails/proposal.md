## Why

Octet's release/operator smoke path already verifies local artifact bundles, but release artifacts also need supply-chain visibility after code leaves source form. `cargo-auditable`, `cargo audit bin`, Syft, and Grype cover complementary release questions: what Rust dependencies were embedded, whether built binaries carry auditable dependency metadata, what SBOM was generated, and which known vulnerabilities remain.

Octet should integrate these as optional release rails so controlled-deployment users can attach release evidence without requiring hosted scanners or rewriting their release pipeline.

## What Changes

- Add optional release rails for auditable Rust binaries, binary audit, SBOM generation, and SBOM/container vulnerability scans.
- Record tool identity, command argv, target artifacts, SBOM paths, vulnerability database identity when available, and result summaries.
- Emit local provenance for each release artifact and parent links back to source-check artifacts.
- Shape receipts so later SLSA or in-toto compatibility export is possible without making signing or hosted transparency logs mandatory.

## Non-Goals

- Do not require container image builds or SBOM generation for starter Octet users.
- Do not require Sigstore, hosted SLSA builders, or external attestation services.
- Do not claim SBOMs prevent supply-chain attacks.
- Do not scan secret source paths into release receipts.

## Impact

- **Files**: release runner code, artifact verifier, receipt schemas, docs, Nix wrapper inputs, tests, and Cairn specs.
- **Behavior**: optional release evidence bundle can include auditable binary, SBOM, and vulnerability-scan receipts.
- **Testing**: positive auditable-binary fixture, missing embedded metadata failure, SBOM receipt fixture, vulnerability finding fixture, parent provenance verification, and Cairn gates.

# Octet release candidate checklist

This document is the pre-tag release candidate receipt for Octet. It is intentionally a local verification checklist; it does not tag, publish, sign, or deploy artifacts.

## Candidate identity

- Candidate version: `0.1.0`
- Canonical Cargo package: `cargo-octet`
- Compatibility Cargo binaries: `cargo-slotcar`, `cargo-tigerstyle`
- Lint library package output: `octet` with compatibility aliases `slotcar` and `tigerstyle`

## Required pre-tag gates

Release operators should also review `docs/trust-boundary-evidence.md` for the
controlled-deployment posture, provenance fields, evidence tiers, and explicit
scope boundary behind release evidence claims.

Run these from the repository root before tagging:

```bash
nix build .#checks.x86_64-linux.release-candidate-smoke -L --no-link
nix flake check -L
cargo octet artifact verify --artifact-dir artifacts/octet
```

## Install smoke commands

The release candidate smoke check verifies these installed binary surfaces from the current flake build:

```bash
nix run .#cargo-octet -- --version
nix run .#cargo-slotcar -- --version
nix run .#cargo-tigerstyle -- --version
nix build .#cargo-octet .#cargo-slotcar .#cargo-tigerstyle .#octet .#slotcar .#tigerstyle --no-link
```

Expected version output:

```text
cargo-octet 0.1.0
```

## Release notes checklist

Before tagging, update release notes or the GitHub release body with:

- the candidate version (`0.1.0` for this candidate),
- the package outputs that shipped (`cargo-octet`, `cargo-slotcar`, `cargo-tigerstyle`, `octet`, `slotcar`, `tigerstyle`),
- the successful `release-candidate-smoke`, `nix flake check -L`, and local artifact verifier receipts,
- any known compatibility aliases or non-goals,
- the controlled-deployment and scope-boundary caveats for source/artifact locality and evidence claims.

## Acceptance signal

The candidate is ready to tag only after `checks.x86_64-linux.release-candidate-smoke` and `nix flake check -L` pass from a clean checkout and the OpenSpec evidence bundle records the exact commands and outputs.

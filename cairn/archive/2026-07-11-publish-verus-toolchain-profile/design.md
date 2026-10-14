# Design: Publish a stack Verus toolchain profile

## Source of truth

`standards/verus-toolchain/profile.ncl` is the human-authored source of truth and is checked by `contracts.ncl`. A deterministic checked-in JSON export is consumed by `flake.nix`; a freshness check rejects drift between Nickel and JSON.

The profile records:

- profile and schema identity;
- Verus release, release digest, Rust toolchain, and solver labels;
- Verus formatter release, upstream revision, target asset names, and SHA-256 fetch identities;
- explicit consumer compatibility rows;
- supported formatter targets and explicit unsupported targets;
- claim scope and non-claims.

Nix requires SHA-256 fixed-output hashes for fetching. Octet continues to compute BLAKE3 binary, solver, toolchain, closure, and receipt identities after materialization.

## Formatter package

The Nix package installs the upstream release binary under `libexec`, exposes a deterministic wrapper, rejects `--update`, and emits an identity record. The formatter check uses `--verus-only` so proof-macro formatting is checked without depending on ambient `rustfmt` discovery.

Positive fixtures must pass unchanged. A deliberately unformatted negative fixture must fail `--check`. All Octet-owned files under `verus/proofs/` must pass.

## Compatibility semantics

A compatibility row is inventory and review guidance only. `native` means the repository uses the profile's exact verifier release. `local-pin` means the repository intentionally uses another exact release and must keep its own receipts. `semantic-reference-only` means a semantic artifact models a different snapshot and cannot establish soundness for this profile.

## Functional boundaries

Nickel and pure field validation own profile semantics. Nix owns fixed-output fetching, executable wrapping, and binary identity materialization. Formatter and verifier execution remain opt-in proof/development rails and do not enter ordinary lint checks.

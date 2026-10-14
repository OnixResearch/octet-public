## Why

Proof receipts now bind to configured verifier and toolchain identity strings, but the verifier identity can still name a different executable at receipt time. A receipt should account for the actual verifier binary/build artifact, not only the logical verifier name.

## What Changes

- Add typed Nickel/Rust receipt verifier fields for verifier binary path and BLAKE3 digest.
- Require proof receipts to include exact `proof-verifier-binary` and `proof-verifier-binary-blake3` metadata when configured.
- Make `--execute-proof-rails` emit binary metadata from the running verifier executable, so generated receipts bind to the actual checker binary used.
- Export binary metadata obligations in generated policy artifacts and cover missing metadata with negative checks.

## Scope

In scope: local deterministic binary digest metadata for the standards verifier binary and generated artifact exposure.

Out of scope: full Nix closure attestation, remote builders, signing, or multiple verifier backend trust stores.

## Verification

Run focused Rust tests, policy validation, standards fixture checks, docs artifact checks, OpenSpec validation, and whitespace hygiene. Save transcripts under this change's `evidence/` directory before archiving.

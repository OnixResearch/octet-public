## Why

Tiger Style now declares Nickel-authored run profiles and Rust-derived acceptance receipt contracts, but the standards CLI only exports artifacts. Review evidence is stronger when the live CLI emits a structured receipt that binds the exact Nickel-exported profile digest to the produced artifact bundle.

## What Changes

- Add a Rust-owned acceptance receipt DTO for standards artifact export runs.
- Extend `tigerstyle-standards export-artifacts` with an optional receipt path and required exported profile JSON input.
- Record artifact paths, BLAKE3 digests, config digest, command evidence, and stage status in the emitted receipt.
- Keep raw logs out of the receipt and keep Nickel as the profile authoring surface.

## Verification

- Rust tests assert the live CLI writes a receipt with the expected schema and config digest.
- Nickel validates the emitted receipt against `standards/receipts/contracts.ncl`.
- Focused Rust/Nickel/OpenSpec checks pass before archive.

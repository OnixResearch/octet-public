# Proposal: Bind proof receipts to build closure identity

## Why

Verifier identity strings, verifier digests, and binary hashes prove which tool was named and which executable bytes were observed, but they do not account for the build/toolchain closure that produced or provisioned the verifier. A receipt can still replay with a matching binary label while depending on a different Nix/toolchain/dependency closure.

## What Changes

- Extend typed proof receipt verifier metadata with build-closure identity and digest fields.
- Allow proof rails to name source-root-relative build-closure manifest paths whose content is hashed into generated receipts.
- Require receipt-only checks to reject otherwise passing receipts missing the configured build-closure identity or digest.
- Export build-closure obligations through generated policy artifacts.

## Impact

- Affected files: Nickel contracts/registry, `tigerstyle-standards` Rust model/proof/artifact/CLI code, fixtures, docs, Nix assertions.
- Verification: focused Rust negative/generated tests, policy validation, artifact export assertions, Nix fixture/docs checks, OpenSpec strict validation, whitespace hygiene.

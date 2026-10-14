## Why

Mantle can enforce Rust compiler policy only if Octet exposes a stable provider boundary. Today Octet owns lint rules, Dylint/rustc-driver wiring, standards policy checks, and suppression validation, but downstream build systems have no single deterministic artifact that states which driver, lint library, config, standards policy, waiver contract, and non-claims define an enforceable Octet profile.

This change defines Octet's side of the integration: a compiler-policy provider manifest and enforcement profile that Mantle or another build orchestrator can consume without importing Octet internals.

## What Changes

- Define a deterministic Octet compiler-policy provider manifest schema.
- Export adapter identity material: compatible rustc/toolchain identity, Dylint driver path or derivation identity, Octet lint library path or derivation identity, lint config digest, standards policy artifact digest, invocation environment, and non-claim text.
- Define an Octet profile suitable for artifact acceptance: deny/error lints, structured in-code waivers, and FCIS standards checks where configured.
- Keep rule authority in Octet while allowing Mantle to bind the provider identity into build receipts.
- Preserve bounded claims: Octet can support proof that an artifact was accepted under a configured policy profile, not proof of program correctness.

## Impact

- **Files**: `cargo-octet/`, `src/config.rs`, `src/diagnostics.rs`, `crates/slotcar-standards/`, `standards/policies/`, templates, docs, Nix app/check surfaces, and new provider-manifest fixtures.
- **Testing**: positive and negative provider-manifest validation, waiver validation, standards artifact export, Dylint provider compatibility checks, downstream Mantle-style fixture smoke, `cairn validate --root .`, and Cairn gates for this change.

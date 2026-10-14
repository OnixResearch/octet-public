# Proposal: Publish a stack Verus toolchain profile

## Summary

Publish Octet's reviewed Verus verifier, Rust toolchain, solver, and Verus formatter identities as a typed Nickel compatibility profile and Nix package. Add a pinned `verusfmt` check that covers Octet-owned Verus proof sources with positive and negative fixtures.

## Motivation

Octet already packages the production Verus verifier and records verifier, solver, toolchain, and Nix closure identities, but the selected versions remain duplicated as local Nix constants. Downstream repositories need a reviewable compatibility artifact rather than silently mixing verifier releases. Octet also lacks a pinned formatter rail for its Verus proof sources.

## Scope

- Add a typed Nickel profile for verifier, Rust toolchain, solver, formatter release, supported targets, known consumer pins, and non-claims.
- Export deterministic JSON for Nix/runtime consumers and check export freshness.
- Drive Octet's production Verus Nix constants from the generated profile.
- Package the pinned upstream `verusfmt` release without enabling its self-update path.
- Add positive and negative `verusfmt --check --verus-only` fixtures and require Octet proof sources to pass.
- Register the Nickel-authored profile family in the policy registry.

## Non-goals

- Do not claim that a compatibility row transfers proof validity between verifier versions.
- Do not upgrade Trellis or downstream verifier pins automatically.
- Do not make Verus mandatory for ordinary `cargo octet check` starter use.
- Do not treat formatting as proof, semantic validation, or release eligibility.

## Validation

Nickel profile export and invalid-fixture rejection, Nix package identity checks, positive and negative formatter checks, production Verus positive/negative rails, policy validation, and Cairn gates.

## Why

Lean's Lake workflow makes proof projects reproducible by pinning package and toolchain material. Octet already records verifier identity, solver identity, command argv, source/proof bindings, and Nix closure identity for Verus rails, but operators still need a single lock-style record that captures the proof package used for a scoped proof run and can be checked offline before replay or promotion.

The goal is not to import Lake or make Lean part of Octet. The goal is a Lake-inspired proof package lock for Octet-owned proof rails: exact inputs, exact verifier material, exact command model, and deterministic stale-lock diagnostics.

## What Changes

- Define a proof package lock record for selected proof rails.
- Bind rail manifest identity, source/proof artifact identities, verifier/solver/toolchain identities, closure graph identity, command argv, environment allowlist, pass signal, and trust-boundary text.
- Add offline validation before proof execution or promotion so stale locks fail before producing misleading evidence.
- Keep starter `cargo octet check` independent from proof package locks.

## Impact

- **Files**: proof-rail manifest export, proof command planning, proof receipt validation, artifact verification, policy registry freshness, docs for proof replay, and lock fixtures.
- **Testing**: valid lock validation, stale source/proof/verifier/closure negative fixtures, offline validation without network, starter-path independence tests, and Cairn validate/gates.

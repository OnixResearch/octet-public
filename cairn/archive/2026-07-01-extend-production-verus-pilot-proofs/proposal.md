## Why

After the production rail has a positive path and a fail-closed negative path, Octet should prove the production pattern generalizes beyond the single addition-bound example. A second tiny proof target gives reviewers more confidence in manifest selection, receipt binding, policy registration, and artifact verification without claiming broad semantic correctness.

This change is intentionally sequenced after the negative-production-proof hardening change. The second proof should remain small, deterministic, and scoped to one reviewable property.

## What Changes

- Add one additional Octet-owned `src/verified/` source target with a tiny pure property suitable for Verus.
- Add a matching Verus proof under `verus/proofs/` and a production-mode proof rail that uses `octet-production-verus`.
- Register policy/manifest ownership so both production rails have independent command, source, proof, receipt, verifier, solver, toolchain, and Nix closure bindings.
- Update proof docs to show multiple production pilot rails while preserving scoped non-claim wording.
- Add tests and an end-to-end proof run proving both production receipts can coexist and verify independently.

## Non-Goals

- Do not verify broad Octet behavior, business semantics, or whole-program correctness.
- Do not add a large Verus proof or introduce proof maintenance burden beyond a small pilot.
- Do not require Verus for ordinary `cargo octet check` starter usage.
- Do not weaken production verifier identity requirements or fixture/production separation.
- Do not proceed until the negative production proof fail-closed behavior is specified and implemented.

## Impact

- **Files**: `src/verified/`, `verus/proofs/`, `standards/proof-rails/manifest.ncl`, `standards/proof-rails/contracts.ncl`, `standards/policies/registry.ncl`, docs, proof tests.
- **Behavior**: `cargo octet prove --source src/verified` can produce more than one independent production proof receipt when both pilot sources are selected.
- **Testing**: focused wrapper proof, multi-rail `cargo octet prove`, artifact verification, manifest/registry checks, no-Verus starter-path regression as needed.

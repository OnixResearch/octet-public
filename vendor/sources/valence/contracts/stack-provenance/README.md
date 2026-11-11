# Stack provenance Nickel contracts

These Nickel contracts harden the static metadata used by
`valence stack-provenance verify-built-binary`.

They validate:

- Octet Valence provenance export shape and schema string.
- Valence stack sidecar shape, claim scope, provenance chain schema, and non-claim boundary.
- Mantle release manifest view shape, `stack-provenance-trace` external evidence role, sidecar schema, BLAKE3 digest shape, and claim scope.
- Built-binary thread input metadata envelope.
- Built-binary thread report shape and bounded non-claim text.
- Negative cases for bad schema strings, malformed BLAKE3 digests, missing sidecar role, missing non-claims, stale parent links, Trellis/manual-review tier promotion, and overclaim wording.

Boundary: Nickel validates metadata shape and static invariants only. Rust remains the authority for reading binary bytes, computing BLAKE3, and comparing measured identity across the sidecar, provenance links, Octet export, and Mantle release metadata. Passing these contracts is not a behavioral-correctness, semantic-equivalence, requirement-satisfaction, release-eligibility, verifier-soundness, or whole-program-safety proof.

Run the deterministic contract gate with:

```bash
nix build .#checks.x86_64-linux.stack-provenance-nickel-contracts --no-link
```

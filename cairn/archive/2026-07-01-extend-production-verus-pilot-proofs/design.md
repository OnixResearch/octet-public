## Context

The first production pilot proves one tiny addition-bound property with real Verus and binds the resulting evidence to verifier, solver, Rust toolchain, Nix closure graph, command argv, source path, proof path, pass signal, checked scope, and trust boundary. That proves the production rail can work once, but it does not exercise multiple independent production rails or receipt coexistence.

This change should happen after `harden-production-verus-negative-proof`, so the fail-closed path is known before increasing production proof count.

## Approach

1. **Choose a second tiny pure property.** Prefer a small deterministic helper under `src/verified/` that can be specified without pulling in Octet runtime dependencies. Candidate shapes include a bounded length/index predicate, monotonic clamp, saturating cap, or simple ordering lemma.
2. **Keep proof source isolated.** Add a matching Verus proof file under `verus/proofs/` that imports or mirrors only the minimal property needed for the proof. Avoid broad Octet module dependencies.
3. **Add an independent production rail.** Declare a second production rail with a unique policy ID, receipt path, checked scope, trust boundary, and source/proof bindings. Reuse the pinned `octet-production-verus` identity and closure binding.
4. **Validate multi-rail receipts.** Extend tests so both production rails can run in one proof invocation, write separate receipts/log artifacts, and verify independently through `cargo octet artifact verify`.
5. **Document scope.** Update docs to say Octet has multiple tiny production pilots, each scoped to its declared property only.

## Decisions

### 1. Second proof must be smaller than the infrastructure around it

**Choice:** Pick a property that fits in one small source file and one small proof file.

**Rationale:** The change is meant to validate production rail repeatability, not to start a large proof-engineering effort.

### 2. Rail identity is unique even when verifier identity is shared

**Choice:** Reuse the same pinned production Verus wrapper while assigning a distinct policy ID, receipt path, checked scope, and proof artifact identity.

**Rationale:** Multiple rails should share trusted verifier provenance without making receipt/provenance records ambiguous.

### 3. Multi-rail proof output is the acceptance surface

**Choice:** Treat an end-to-end `cargo octet prove` run plus artifact verification as the primary evidence, rather than only direct wrapper output.

**Rationale:** The production value is in Octet's evidence pipeline handling multiple scoped formal proofs, not merely Verus accepting another file.

## Risks / Trade-offs

- **Proof creep:** A second proof can invite larger semantic claims. Mitigate by keeping property names and trust boundaries narrow.
- **Manifest drift:** Multiple production rails can duplicate verifier identity constants. Mitigate with manifest checks that bind rail fields to the generated closure manifest.
- **Receipt ambiguity:** Similar rail names can make provenance hard to inspect. Mitigate with distinct policy IDs, receipt filenames, checked scopes, and docs.

## Validation

- Direct `octet-production-verus` run passes for the second proof.
- Manifest and policy registry checks pass with two production pilot rails.
- `cargo octet prove --source src/verified` writes separate receipts for both production pilots.
- `cargo octet artifact verify` validates the multi-rail artifact bundle.
- Negative production proof hardening remains green.

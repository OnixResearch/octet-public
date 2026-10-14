## Why

The external evidence tooling proof page is useful because it shows one clear route from a requirement to proof or validation evidence. Octet already has the Verus execution and receipt machinery, but the reader still has to assemble the route from the README, `verus/README.md`, trust-boundary docs, and artifact-verifier docs.

Octet should add a concise Verus proof-path explainer that maps each scoped formal-proof claim from requirement or policy ID to Verus proof target, Rust source binding, verifier run, structured receipt, `formal_proof` provenance, and artifact verification. This improves operator review without changing proof execution semantics.

## What Changes

- Add documentation that presents the Verus formal-proof path as a bounded evidence lane inside Octet's trust stack.
- Use proof-vs-validation wording that preserves the existing evidence-tier taxonomy.
- Link the proof path to existing commands: `cargo octet prove`, `cargo octet prove --list`, `cargo octet prove --explain`, and `cargo octet artifact verify`.
- State what the Verus path does not claim: whole-program correctness, certification, business-domain validation, semantic equivalence, or full safety.

## Non-Goals

- Do not change Verus proof execution, receipt schemas, verifier identity requirements, or artifact verification behavior.
- Do not add new proof targets or broaden production Verus scope.
- Do not relabel lint, policy, transcript, or manual review evidence as formal proof.
- Do not introduce external evidence tooling, external reference, Lean, tensor, AD, or numerical-domain claims into Octet docs.

## Impact

- **Files**: `README.md`, `docs/trust-boundary-evidence.md`, `verus/README.md`, and any deterministic doc-check helper or fixture needed to prove the wording.
- **Behavior**: documentation-only; commands and proof artifacts remain unchanged.
- **Testing**: positive documentation inventory for the proof-path stages and commands; negative wording check for unqualified proof/correctness overclaims.

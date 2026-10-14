## ADDED Requirements

### Requirement: Multiple production Verus pilot rails

r[octet.verus_proof_rails.production_pilot_expansion] Octet SHOULD support more than one tiny production-mode Verus pilot rail only after production failure semantics are tested fail-closed, and each pilot rail MUST remain independently scoped.

#### Scenario: Second production pilot has independent identity

r[octet.verus_proof_rails.production_pilot_expansion.independent_identity]

- GIVEN Octet adds a second tiny production-mode Verus proof target
- WHEN the proof-rail manifest and policy registry are exported
- THEN the new rail MUST have a unique policy ID, checked scope, receipt path, source binding, proof binding, pass signal, and trust-boundary text.
- AND the rail MAY reuse the pinned `octet-production-verus` verifier identity only when the verifier, solver, Rust toolchain, and Nix closure graph bindings still match the production verifier manifest.

#### Scenario: Multiple production receipts verify independently

r[octet.verus_proof_rails.production_pilot_expansion.multi_receipt]

- GIVEN two tiny production-mode Verus pilot rails are selected in one proof run
- WHEN `cargo octet prove` writes artifacts and `cargo octet artifact verify` validates them
- THEN each rail MUST write a distinct structured receipt and proof artifact identity.
- AND each `formal_proof` provenance record MUST name only its own checked source scope and proof property.
- AND a stale or swapped receipt for one rail MUST fail with a deterministic diagnostic without making the other rail's identity ambiguous.

#### Scenario: Expanded pilot scope remains non-claiming

r[octet.verus_proof_rails.production_pilot_expansion.non_claiming]

- GIVEN documentation describes multiple production Verus pilot rails
- WHEN a reader inspects the proof-rail documentation
- THEN the documentation MUST state that each pilot proves only its declared property.
- AND it MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, or full safety.

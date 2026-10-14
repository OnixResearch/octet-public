# State machine exhaustiveness checks

Rust's native enum exhaustiveness already helps closed state machines when transition logic uses explicit matches without catch-all arms. Octet's current enum lints catch broad wildcard and enum-evolution hazards, but consumers still need a reviewable checked path for deliberately marked state-machine transition cores: no hidden fallback, explicit invalid-transition behavior, pure transition logic, and evidence that Valence can carry without turning the lint result into a correctness proof.

## Scope

- Add a marked state-machine transition-core contract for simple Rust enum state/event domains.
- Require supported transition cores to avoid `_` or catch-all arms and to route every supported state/event pair to either an explicit next state or an explicit invalid-transition error.
- Keep transition decisions in a pure core with mutation, I/O, clocks, process, network, and ambient state outside the transition checker surface.
- Emit deterministic lint diagnostics and reviewable receipts that can be included in Octet artifact bundles and exported to Valence.
- Document that this is source-shape and bounded static evidence unless paired with a separate Verus or Kani proof receipt.

## Impact

- **Octet** owns detection, diagnostics, UI fixtures, artifact receipts, and Valence export rows for state-machine exhaustiveness checks.
- **Valence** may consume the exported receipt as identity/linkage evidence with explicit non-claims.
- **Cairn** may gate lifecycle readiness on the presence of the receipt, but the receipt alone does not prove behavioral correctness or release eligibility.

## Non-claims

This change does not prove whole-program correctness, semantic equivalence, verifier soundness, liveness, deadlock freedom, protocol completeness outside the declared state/event domain, or correctness of unsupported transition shapes. It provides fail-closed source-shape evidence for marked transition cores and preserves separate formal-proof evidence when available.

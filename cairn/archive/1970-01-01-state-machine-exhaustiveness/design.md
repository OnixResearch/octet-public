# Design: State machine exhaustiveness checks

## Supported source shape

The first implementation targets one obvious functional-core shape: a transition function marked as a state-machine core whose typed inputs include a state enum and an event enum, and whose body makes the transition decision through a `match` over `(state, event)` or an equivalent tuple binding. Supported arms must enumerate concrete variants or explicit grouped variant pairs. Wildcards, rest patterns that hide variants, and fallback branches are blocking diagnostics.

Unsupported marked shapes fail closed with an `unsupported-state-machine-shape` diagnostic rather than being treated as checked. Examples include transition logic spread across mutable flags, trait-object dispatch, generated macro bodies whose variant domain cannot be inspected, or runtime string/tag tables without a typed enum domain.

## Exhaustiveness decision core

A pure decision core receives rustc-derived facts rather than reading files or querying the compiler itself:

- state type identity and variant inventory;
- event type identity and variant inventory;
- transition-arm patterns and branch outcomes;
- effect and mutation facts already available to Octet purity checks;
- source spans for deterministic diagnostics.

The core classifies each marked transition as passing, failing, or unsupported. A passing result means every supported state/event pair is explicitly handled as a next-state transition or an invalid-transition result, no fallback hides future variants, and the transition decision is not coupled to mutation or ambient effects.

## Diagnostics and fixtures

Positive fixtures cover a closed enum state machine with explicit valid and invalid transitions. Negative fixtures cover wildcard fallback, missing explicit invalid transition hidden by `_`, local enum evolution without an intentional closed marker, mutation-driven transition state, unsupported macro/generated shape, and overclaim text in emitted receipts.

Diagnostics must name the transition core, state type, event type, and the first blocking reason. When rustc already rejects a non-exhaustive match, Octet's fixture should preserve that compiler failure as the baseline and focus the lint on shapes that would otherwise compile by using fallback behavior.

## Evidence and Valence export

Octet records state-machine exhaustiveness as bounded source-shape evidence. The receipt binds the checker identity, transition core source span, state/event type identities, variant inventory digest, diagnostic status, artifact BLAKE3 identity, and explicit non-claims. Valence export rows preserve that receipt as adapter input and must not promote it to formal proof.

When a Verus or Kani proof rail proves an additional invariant about the same transition core, Octet links that proof receipt separately. The lint receipt may reference the proof as a parent or sibling artifact, but it does not inherit the formal-proof tier.

## Validation

Focused validation should run:

```bash
nix develop -c cargo test -p cargo-octet state_machine_exhaustiveness
nix develop -c cargo test --test ui -- --ignored --nocapture state_machine_exhaustiveness
nix run .#octet-policy-validate
nix run path:/path/to/git/OnixResearch/cairn#cairn -- validate --root .
nix run path:/path/to/git/OnixResearch/cairn#cairn -- gate proposal state-machine-exhaustiveness --root .
nix run path:/path/to/git/OnixResearch/cairn#cairn -- gate design state-machine-exhaustiveness --root .
nix run path:/path/to/git/OnixResearch/cairn#cairn -- gate tasks state-machine-exhaustiveness --root .
```

## Risks

- **False confidence from shape checks**: keep receipt non-claims explicit and require separate proof receipts for semantic invariants.
- **False positives on advanced patterns**: fail closed as unsupported for unclear marked shapes instead of guessing.
- **Namespace churn**: keep the state-machine marker configurable or policy-owned so the Octet namespace migration does not strand consumers.

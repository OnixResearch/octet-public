# Bounded evidence transforms

Octet treats tactic-like evidence derivation as named, bounded transforms, not as
unbounded tactics or hidden simplification. The first checked transform surface is
`octet.transform.claim-boundary-normalize.v1`, which applies the explicit
`octet.rewrite.claim-boundary.v1` rule set to claim-boundary text before a report
or derived artifact is reviewed.

A transform report names:

- transform ID and transform family;
- input and output contracts;
- explicit rule-set identity and applied rules;
- byte identities for input and parent artifacts;
- output artifact locator and BLAKE3 output identity;
- checked scope, evidence tier, diagnostics, and scoped non-claims;
- provenance linking the derived artifact back to parent artifacts.

The initial rule set is deliberately small: `trim-whitespace`,
`collapse-ascii-whitespace`, and `sort-non-claims`. Unsupported rules, hidden
rules, global simplification, stale parent identities, invalid tier promotion,
missing non-claims, and budget exhaustion fails closed with deterministic
diagnostics. The transform output is intended to be byte-stable for replay with
the same input identities, rules, and budget.

Bounded transforms are review aids only. They are not unbounded tactics, not
complete proof search, not solver completeness, not whole-program correctness,
not certification, not business-domain validation, not semantic equivalence, not
full safety, and not verified runtime proof. Any stronger claim must come from a
separately checked evidence tier and artifact chain.

## Why

FSM transition cores need the opposite default from ordinary open-ended same-crate enum evolution. For normal local enums, an exhaustive match can be fragile because adding a variant forces every call site to change. For a reviewed FSM core, that compile-time pressure is the point: adding a state or event must force every transition relation, denial reason, receipt, and coverage fixture to be reviewed.

Octet already has enum-match lints for catch-all arms, foreign enum catch-alls, and fragile local exhaustive matches. It needs a first-class FSM profile that composes those lints into an exhaustive-by-default policy without weakening the open/closed guidance for non-FSM code.

## What Changes

- Add an FSM exhaustiveness profile for marked transition cores and configured FSM source scopes.
- Treat catch-all enum arms in FSM transition code as fail-closed diagnostics unless a generated adapter is explicitly outside the pure transition core.
- Treat marked FSM state/event enums as deliberately closed so exhaustive matches are accepted instead of flagged as fragile local enum matches.
- Require FSM evidence to bind state/event variant inventories, transition-core identity, coverage summary identity, denial coverage, diagnostics, and bounded non-claims.
- Export a Valence-compatible state-machine exhaustiveness receipt while preserving Octet ownership of Rust source analysis and lint/provenance generation.
- Document how consumer repositories mark FSM enums, transition cores, generated adapters, and receipt-export paths.

## Impact

FSM owners get compiler- and Octet-backed pressure to review every state/event addition. Invalid transitions remain explicit denials, not wildcard defaults. Valence and Kamacite can consume the resulting receipts as bounded identity/linkage evidence without treating them as behavioral correctness, liveness, protocol-completeness, or release-readiness proofs.

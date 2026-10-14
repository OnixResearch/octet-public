## Design

Octet models exhaustive-by-default FSMs as a lint and receipt profile over Rust source facts. The profile is opt-in for consumers through configured source scopes and/or marker attributes, but once a transition core is in scope the policy is fail closed: wildcard handling is denied, FSM enums are treated as deliberately closed, and coverage evidence must bind the declared state/event universe.

## Decisions

### FSM identity is explicit

The profile recognizes three identities:

- FSM state/event enums, marked by the configured FSM enum marker or discovered through a marked transition core signature;
- pure transition cores, marked by the configured FSM transition marker or located under configured FSM core scopes;
- generated adapters, receipt shells, and parser/rendering code, which may be documented as outside the pure transition core.

The profile does not infer FSM semantics from names alone. A consumer must opt a core into the profile through config or markers.

### Existing enum lints compose differently in FSM cores

Inside FSM transition cores, `_` or otherwise broad catch-all enum arms are diagnostics because they hide future states/events. Exhaustive matches over marked FSM enums are accepted because they are the desired failure mode when the enum changes. Outside FSM cores, existing `catch_all_on_enum`, `foreign_enum_catch_all`, and `fragile_exhaustive_enum_match` behavior remains unchanged.

### Coverage evidence is bounded source-shape evidence

Octet records state/event variant inventories, transition-core identity, matrix or coverage-summary identity, status, diagnostics, parent artifact identity, and non-claims. The receipt proves only that Octet checked the declared source-shape profile and exported a stable identity. It does not prove protocol completeness, liveness, semantic correctness, verifier soundness, release readiness, or exhaustive Rust coverage.

### Functional core / shell split

Lint analysis and receipt normalization run as deterministic logic over rustc facts, parsed config, and in-memory receipt DTOs. File loading, Cargo invocation, artifact writing, Valence export location selection, and stdout/stderr remain in the runner shell.

## Validation

Positive coverage should include marked FSM enums with exhaustive transition matches, explicit invalid-transition denials, and passing Valence-compatible receipt export. Negative coverage should include wildcard FSM arms, missing state/event inventories, stale coverage summaries, unmarked fragile local enum matches outside FSM scope, foreign enum catch-all arms, and overclaiming receipts.

## Tasks

- [x] [serial] Define FSM profile config and marker contracts for FSM enum identity, transition-core scopes, generated-adapter exclusions, and receipt export profile. r[octet.ocp_enum_match.fsm_exhaustive_defaults.profile]
- [x] [serial] Compose enum-match lints so FSM transition cores deny wildcard enum arms while marked FSM enums are treated as deliberately closed for exhaustive matches. r[octet.ocp_enum_match.fsm_exhaustive_defaults.wildcards] r[octet.ocp_enum_match.fsm_exhaustive_defaults.sealed]
- [x] [parallel] Add positive UI/fixture coverage for marked FSM cores with exhaustive state/event matches, explicit invalid-transition denials, and no fragile-exhaustive diagnostics. r[octet.ocp_enum_match.fsm_exhaustive_defaults.fixtures.positive]
- [x] [parallel] Add negative UI/fixture coverage for wildcard FSM arms, missing enum markers, stale or missing coverage identities, foreign enum catch-alls, and overclaiming receipts. r[octet.ocp_enum_match.fsm_exhaustive_defaults.fixtures.negative]
- [x] [serial] Emit or normalize Valence-compatible FSM exhaustiveness receipts with transition-core identity, state/event variant inventories, coverage summary identity, diagnostics, artifact identity, and required non-claims. r[octet.ocp_enum_match.fsm_exhaustive_defaults.receipt]
- [x] [parallel] Document consumer setup in the lint reference/template config and state-machine/concurrency guidance. r[octet.ocp_enum_match.fsm_exhaustive_defaults.docs]
- [x] [serial] Run focused UI tests, receipt validation, Octet policy validation, Cairn validation, and proposal/design/tasks gates before archive. r[octet.ocp_enum_match.fsm_exhaustive_defaults.final_validation]

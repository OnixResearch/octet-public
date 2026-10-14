## Phase 1: Baseline and policy

- [ ] [serial] I1 Record baseline closure, artifact verification, policy freshness, workspace, and lifecycle results. r[octet.effect_closure.kamacite_fixtures]
- [ ] [depends:published-kamacite-semantic-operation-cohort] I2 Add the Nickel mapping profile, contract, generated runtime projection, and frozen Kamacite fixtures. r[octet.effect_closure.kamacite_mapping]
- [ ] [depends:kamacite-effect-mapping-profile] I3 Register the new config and receipt families with exact source-of-truth ownership. r[octet.effect_closure.kamacite_mapping] r[octet.effect_closure.kamacite_boundary]

## Phase 2: Core, runner, and handoff

- [ ] [depends:kamacite-effect-mapping-profile] I4 Implement pure ambient disposition checks, semantic-emission admission, identity checks, comparison classes, canonical ordering, and deterministic diagnostics. r[octet.effect_closure.kamacite_dispositions] r[octet.effect_closure.kamacite_comparison]
- [ ] [depends:kamacite-effect-comparison-core] I5 Add the optional `--kamacite-effect-profile PATH` runner path after successful direct and closure checks. r[octet.effect_closure.kamacite_cli]
- [ ] [depends:kamacite-effect-comparison-core] I6 Emit and verify the BLAKE3-bound Kamacite handoff projection without adding canonicalization authority to Octet. r[octet.effect_closure.kamacite_handoff] r[octet.effect_closure.kamacite_boundary]

## Phase 3: Verification and closeout

- [ ] [parallel] V1 Add positive fixtures for exact emissions, profile-local allocation, declared-not-emitted rows, order independence, and default compatibility. r[octet.effect_closure.kamacite_fixtures]
- [ ] [parallel] V2 Add negative fixtures for ambient-effect masquerade, stale identities, ambiguous emissions, denied or unknown effects, undeclared emissions, tampering, and role promotion. r[octet.effect_closure.kamacite_fixtures] r[octet.effect_closure.kamacite_boundary]
- [ ] [parallel] V3 Add property tests for canonical ordering, duplicate rejection, identity sensitivity, and deterministic comparison. r[octet.effect_closure.kamacite_comparison]
- [ ] [serial] I7 Document configuration, command usage, ownership, comparison classes, migration, and non-claims. r[octet.effect_closure.kamacite_boundary]
- [ ] [depends:kamacite-effect-closure-fixtures] V4 Run focused tests, UI fixtures, policy freshness, artifact verification, Clippy, Cairn validation, gates, and relevant Nix checks. r[octet.effect_closure.kamacite_fixtures] r[octet.effect_closure.kamacite_boundary]

## Verification Coverage

- `Scenario: Exact semantic emission is declared` -> I2, I4, V1
- `Scenario: Allocation remains profile-local` -> I2, I4, V1
- `Scenario: Ambient clock cannot become a semantic emission` -> I4, V2
- `Scenario: Unknown effect fails` -> I4, V2
- `Scenario: Emitted undeclared operation fails` -> I4, V2
- `Scenario: Declared operation is not emitted` -> I4, V1
- `Scenario: Optional profile emits a handoff` -> I5, I6, V1
- `Scenario: Default admission remains unchanged` -> I5, V1
- `Scenario: Semantic determinism claim fails` -> I6, V2, I7

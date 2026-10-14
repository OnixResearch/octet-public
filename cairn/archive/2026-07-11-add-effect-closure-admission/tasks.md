## Phase 1: Policy and pure core

- [x] [serial] Add the typed Nickel effect catalog and workspace-policy contract, valid/invalid fixtures, checked-in generated catalog JSON, copyable template, registry entry, and freshness checks. r[octet.effect_closure.catalog] r[octet.effect_closure.catalog.valid] r[octet.effect_closure.catalog.invalid]
- [x] [serial] Add pure Rust DTO validation for effect vocabulary, profile allowances, module roles, role edges, exact call summaries, deterministic ordering, non-claims, and receipt identities. r[octet.effect_closure.catalog] r[octet.effect_closure.roles] r[octet.effect_closure.dependency_summaries] r[octet.effect_closure.evidence]
- [x] [serial] Add conservative selected-source inference for direct effects, local calls, stable call-site identities, and fail-closed unsupported constructs. r[octet.effect_closure.inference] r[octet.effect_closure.inference.direct] r[octet.effect_closure.inference.unknown]
- [x] [serial] Add deterministic role-edge validation and finite-lattice fixed-point propagation over selected local calls, including convergent cycles. r[octet.effect_closure.roles] r[octet.effect_closure.roles.invalid] r[octet.effect_closure.transitive] r[octet.effect_closure.transitive.cycle]
- [x] [serial] Add exact Cargo/workspace/toolchain provider matching so missing, stale, duplicate, broad, or mismatched dependency-call summaries fail closed. r[octet.effect_closure.dependency_summaries] r[octet.effect_closure.dependency_summaries.valid] r[octet.effect_closure.dependency_summaries.invalid]

## Phase 2: CLI and evidence shell

- [x] [serial] Add `cargo octet admit --profile deterministic-core --closure`, run the existing strict check first, load selected sources/Cargo metadata/policy through a thin shell, and preserve existing check behavior. r[octet.effect_closure.cli] r[octet.effect_closure.cli.admitted] r[octet.effect_closure.cli.check_failure] r[octet.effect_closure.boundaries.default]
- [x] [serial] Emit profile/config/function-address/source/dependency/policy-bound closure artifacts and add `cargo octet admit verify` tamper-evident integrity verification. r[octet.effect_closure.evidence] r[octet.effect_closure.evidence.visible] r[octet.effect_closure.evidence.tamper]
- [x] [serial] Add deterministic human/JSON diagnostics and operator documentation for bootstrap, exact call summaries, role assignment, evidence verification, and bounded non-claims. r[octet.effect_closure.boundaries] r[octet.effect_closure.boundaries.non_claims]

## Phase 3: Adversarial verification and dogfood

- [x] [serial] Add positive and negative unit/integration fixtures for local calls, external dependencies, macros, trait/method ambiguity, `cfg`, FFI, unsafe, hidden panic paths, cycles, stale addresses, stale summaries, and artifact tampering. r[octet.effect_closure.verification] r[octet.effect_closure.verification.adversarial]
- [x] [serial] Run the initial Valence closure probe, retain its fail-closed unknown/unadmitted-edge evidence, add exact Nickel-authored role/call summaries, and retain a final verified bounded admission without broad suppressions. r[octet.effect_closure.dogfood] r[octet.effect_closure.dogfood.initial] r[octet.effect_closure.dogfood.final]
- [x] [serial] Run focused and broad Rust checks, Nickel fixture/freshness checks, package/downstream smokes, deterministic replay/tamper checks, formatting, Clippy, Cairn validation/gates, and record exact outcomes in `cairn/changes/add-effect-closure-admission/evidence/validation.md` before sync and archive. r[octet.effect_closure.verification] r[octet.effect_closure.verification.adversarial]

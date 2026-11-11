# Tasks: Resolve open assurance issues

## Policy execution coverage

- [x] [serial] Add typed policy execution status, ownership, scope, checker, evidence identity, and diagnostic linkage to the standards report. r[octet.open_issue_resolution.policy_execution]
- [x] [serial] Map local checker coverage explicitly and classify unimplemented local, external, and human policies truthfully. r[octet.open_issue_resolution.policy_execution]
- [x] [parallel] Add checker-failure input, `tool_error`, `not_applicable`, and strict coverage exit behavior. r[octet.open_issue_resolution.policy_execution]
- [x] [parallel] Cover the timeout/serial gap, passing, failed, missing execution, family exclusion, and external/human cases. r[octet.open_issue_resolution.policy_execution]

## Negative controls

- [x] [serial] Add v2 typed verifier evidence, mutation identity, measured source/deck identities, and outcome distinctions. r[octet.open_issue_resolution.negative_control]
- [x] [parallel] Require a passing baseline and attributed mutant rejection for `property_demonstrated`; keep v1 readable but inconclusive. r[octet.open_issue_resolution.negative_control]
- [x] [parallel] Recompute the negative-control decision during proof-ref validation and detect changed artifacts. r[octet.open_issue_resolution.negative_control]
- [x] [serial] Add positive and negative tests for every acceptance bullet. r[octet.open_issue_resolution.negative_control]

## Requirement contracts and freshness

- [x] [serial] Define the Nickel-authored contract, evidence bundle, and Rust decision DTOs with spec/source/clause/assumption/exclusion identities. r[octet.open_issue_resolution.requirement_contract]
- [x] [parallel] Implement `requirements check`, `verify`, and decision recomputation with fail-closed dispositions and negative controls. r[octet.open_issue_resolution.requirement_contract]
- [x] [parallel] Implement `requirements freshness` over the evidence codebase index with direct bindings, bounded BFS, and incomplete/stale/failed/unsupported outcomes. r[octet.open_issue_resolution.freshness]
- [x] [serial] Add the reviewed Aeneas fixture contract, empirical assumption check, exclusion, and freshness index fixture. r[octet.open_issue_resolution.requirement_contract]
- [x] [serial] Add tests for missing evidence, changed sources/spec/assumptions, empirical-for-formal rejection, duplicate IDs, empty inventory, tampering, and freshness distinctions. r[octet.open_issue_resolution.freshness]

## Executed replay

- [x] [serial] Preserve `--observed` comparison and label it `supplied_observation`. r[octet.open_issue_resolution.executed_replay]
- [x] [parallel] Add closed `check-v1` and `fingerprint-production-v1` profiles with fixed argv, allowlisted environment, limits, isolated output, and expected outputs. r[octet.open_issue_resolution.executed_replay]
- [x] [parallel] Add the versioned execution receipt, drift outcomes, named failure contracts, and input/receipt validation. r[octet.open_issue_resolution.executed_replay]
- [x] [serial] Test launch, fabricated observations, source/config/toolchain/argv/output/exit drift, timeout/overflow, unsupported profiles, path escape, and expected-failure mismatch. r[octet.open_issue_resolution.executed_replay]

## Admission authentication

- [x] [serial] Add pinned artifact-auth path dependencies and consumer-owned policy/evidence DTOs outside the candidate bundle. r[octet.open_issue_resolution.admission_auth]
- [x] [parallel] Verify detached signatures with `verify_and_evaluate` and bind bundle/provenance identities into the canonical scope. r[octet.open_issue_resolution.admission_auth]
- [x] [parallel] Report integrity, signature, authorization, currentness, checker, and overall outcomes separately with a recomputable decision receipt. r[octet.open_issue_resolution.admission_auth]
- [x] [serial] Test authorized success, unknown/unauthorized producers, wrong purpose/scope/subject/generation, threshold, altered bytes, forged observations, and whole-bundle replacement. r[octet.open_issue_resolution.admission_auth]

## Local scanners

- [x] [serial] Add Nickel Semgrep/CodeQL producer profiles and generated JSON with pinned executable, rules/query packs, scope, and limits. r[octet.open_issue_resolution.local_scanners]
- [x] [parallel] Add bounded producer execution, fixed argv, source snapshot, tool/rule identity, CodeQL database sidecar, and execution receipt. r[octet.open_issue_resolution.local_scanners]
- [x] [parallel] Reuse the SARIF importer and distinguish imported from executed evidence; missing tools stay unsupported. r[octet.open_issue_resolution.local_scanners]
- [x] [serial] Test clean/violation, missing/wrong tool, changed rules, source mutation, stale database, failed/empty report, timeout/overflow, exclusions, malformed SARIF, unsupported scope, replacement, and mislabeled import. r[octet.open_issue_resolution.local_scanners]
- [x] [parallel] Retain real Semgrep 1.180.0 and CodeQL 2.27.2 clean and violating reports, accept the `Semgrep OSS` driver spelling, correct the query-pack pin to `0.1.46`, and document the retained fixtures. r[octet.open_issue_resolution.local_scanners]

## Cross-tool benchmark

- [x] [serial] Keep the same-command comparator unchanged and add a separate experiment contract with tasks, applicability, corpus, assurance cases, and adoption targets. r[octet.open_issue_resolution.cross_tool_benchmark]
- [x] [parallel] Add bounded recorded/execute modes, retained observations, comparable-subset metrics, unavailable-tool handling, and source-linked report. r[octet.open_issue_resolution.cross_tool_benchmark]
- [x] [parallel] Verify derived fields from retained inputs and implement the eight negative controls. r[octet.open_issue_resolution.cross_tool_benchmark]
- [x] [serial] Add the compiling corpus, frozen expected outcomes, checked-in artifact/report, and CLI verify surface. r[octet.open_issue_resolution.cross_tool_benchmark]

## Public release cohort

- [x] [serial] Author the Nickel release-cohort manifest and generator with exact public/development source and export identities. r[octet.open_issue_resolution.release_cohort]
- [x] [parallel] Add offline cross-field validation, inventory comparison, notice checks, receipt freshness, and blocked qualification. r[octet.open_issue_resolution.release_cohort]
- [x] [parallel] Add the nine required negative controls and current public inventory/toolchain fixtures. r[octet.open_issue_resolution.release_cohort]
- [x] [serial] Document the supported channel, copyable public install command, walkthrough fix, and explicit blockers. r[octet.open_issue_resolution.release_cohort]

## Verification

- [x] [serial] Run `cargo test -p cargo-octet --lib` and `cargo test -p octet-standards --lib`. r[octet.open_issue_resolution.policy_execution]
- [x] [serial] Run workspace Clippy with `-D warnings` and `cargo fmt --all --check`. r[octet.open_issue_resolution.requirement_contract]
- [x] [serial] Verify Nickel export freshness and registry typecheck for every changed generated artifact. r[octet.open_issue_resolution.release_cohort]
- [x] [serial] Run the requirements check/verify/freshness and benchmark verify CLI surfaces end to end. r[octet.open_issue_resolution.freshness]

## Verification Coverage

- `Scenario: Unsupported local policies block strict coverage` -> `unimplemented_hard_local_policies_are_unsupported`
- `Scenario: External and human checks stay separate` -> `external_and_human_policies_are_not_run_and_not_completed_local_checks`
- `Scenario: Passing and failing local checks retain scope` -> `implemented_passing_policy_reports_passed_with_scope`, `checker_violation_reports_failed_execution`
- `Scenario: Unattributed failure is inconclusive` -> `missing_verifier_evidence_is_inconclusive`, `unverifiable_mutant_outcomes_are_inconclusive`
- `Scenario: Retained artifacts are recomputed` -> `verification_detects_changed_source_and_verifier_output`, `verification_detects_changed_mutation_obligation_and_spec`
- `Scenario: Missing clause evidence fails` -> `missing_evidence_fails_closed`
- `Scenario: Empirical evidence cannot satisfy formal proof` -> `empirical_evidence_cannot_satisfy_formal_clause`
- `Scenario: Changed assumptions and decisions are detected` -> `changed_assumption_identity_cannot_inherit_acceptance`, `tampered_decision_bytes_are_rejected`
- `Scenario: Fresh and stale receipts are distinguished` -> `stale_binding_and_missing_freshness_are_distinct`
- `Scenario: Incomplete inventory is not missing` -> `freshness_uses_incomplete_when_inventory_is_partial`
- `Scenario: Fabricated observation cannot satisfy executed replay` -> `executed_replay_cannot_be_satisfied_by_a_fabricated_observation`
- `Scenario: Drift and expected failure are precise` -> `executed_replay_reports_source_argv_output_and_exit_drift`, `executed_replay_requires_the_named_failure_contract`
- `Scenario: Candidate trust roots are rejected` -> `candidate_files_cannot_supply_trust_roots_or_observations`
- `Scenario: Decisions are recomputable and separated` -> `authorized_verification_binds_two_distinct_keys_and_recomputes_the_decision`
- `Scenario: Missing tool is not clean` -> `missing_and_mis_versioned_tools_never_produce_clean_evidence`
- `Scenario: CodeQL database drift fails` -> `stale_or_mismatched_codeql_database_fails`
- `Scenario: Unsupported scope is neither clean nor a false negative` -> `architecture_and_assurance_metrics_stay_separate_and_do_not_upgrade_unsupported_scope`
- `Scenario: Artifact verification recomputes derived fields` -> `checked_in_observations_artifact_and_report_verify`, `negative_controls_are_contained_and_tampering_is_rejected`
- `Scenario: Public inventory and notices are checked` -> `release_cohort::tests::checked_in_cohort_validates_and_stays_blocked`, `release_cohort::tests::documented_claims_match_the_public_inventory`
- `Scenario: Development receipts are not public evidence` -> `release_cohort::tests::negative_controls_are_rejected`

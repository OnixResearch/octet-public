# Aeneas Lean Proof Rail Specification

## Purpose

Defines the `aeneas-lean-proof-rail` capability.

## Requirements

### Requirement: Aeneas Lean provider contract

r[octet.aeneas_lean_proof_rail.provider_contract] Octet MUST define an opt-in `aeneas-lean` formal-proof provider whose typed configuration binds the exact Aeneas release, required Charon revision, OCaml toolchain, Lean toolchain, Aeneas Lean library, checked source surface, command argv, execution limits, environment allowlist, effective extraction policy, assumption policy, semantic-reference metadata, and non-claims.

#### Scenario: Complete provider profile is accepted

- GIVEN a provider profile contains every required identity and policy field
- WHEN policy export and provider validation run
- THEN the generated runtime profile MUST be deterministic and MUST preserve the reviewed source-surface and non-claim boundaries.

#### Scenario: Provider pin or policy is incomplete

- GIVEN the provider omits a tool identity, uses a Charon revision different from the reviewed Aeneas pin, enables an unsupported backend, or weakens required non-claims
- WHEN provider validation runs
- THEN validation MUST fail before any proof-stage process executes.

### Requirement: Aeneas stage chain is exact and linked

r[octet.aeneas_lean_proof_rail.stage_chain] Octet MUST bind the checked Rust source, effective Charon extraction contract, LLBC output, Aeneas command, `translation.json`, generated Lean output, hand-maintained theorem source, theorem-adequacy decision, imported external models, trust closure, Lean package/toolchain, and final checker result through ordered typed input/output identities using BLAKE3 for stack-owned content.

#### Scenario: Matching stage chain passes

- GIVEN every stage names the exact previous output as its input and all bound artifacts match their measured bytes
- WHEN the pure Aeneas receipt validator runs
- THEN it MUST accept the stage linkage and report every stage in deterministic order.

#### Scenario: Stale or swapped intermediate artifact fails

- GIVEN source, extraction contract, LLBC, translation manifest, generated Lean, theorem, model, package-lock, or checker identities are stale, missing, duplicated, swapped, or linked out of order
- WHEN receipt or artifact validation runs
- THEN validation MUST fail with a deterministic diagnostic naming the stage and field.

### Requirement: Effective Charon extraction contract is exact

r[octet.aeneas_lean_proof_rail.charon_contract] Octet MUST validate Charon's effective serialized options and extraction result, including `has_errors`, `skip_borrowck`, target triple, pointer width, endianness, MIR start stage, drop policy, `start_from`, include/exclude/opaque rules, transformation settings, and serialization format, before any Aeneas theorem can become property evidence.

#### Scenario: Reviewed extraction contract passes

- GIVEN `has_errors` is false, `skip_borrowck` is false, every effective option matches the reviewed provider profile, and target metadata matches the proof claim
- WHEN extraction-contract validation runs
- THEN the measured LLBC artifact MAY proceed to translation-manifest validation.

#### Scenario: Partial or weakened extraction fails

- GIVEN LLBC reports errors, borrow checking was skipped, target metadata drifted, or any effective extraction option is missing, unknown, hidden, or different from the reviewed profile
- WHEN extraction-contract validation runs
- THEN validation MUST fail before theorem checking and MUST NOT retain a prior property disposition.

### Requirement: Aeneas translation manifest is bound

r[octet.aeneas_lean_proof_rail.translation_manifest] Octet MUST measure and validate Aeneas `translation.json` as a distinct stage linking selected LLBC declarations to generated Lean declarations and files while preserving Rust names/spans, Lean names/files, `is_opaque`, `can_fail`, `can_diverge`, recursion, and loop-parent metadata.

#### Scenario: Complete declaration mapping passes

- GIVEN each selected LLBC declaration has a role-compatible manifest entry and every named generated Lean file matches measured bytes
- WHEN manifest validation runs
- THEN Octet MUST preserve the declaration linkage and generated-file inventory in deterministic order.

#### Scenario: Tool-local declaration ID is reused as stable identity

- GIVEN a manifest `def_id` is missing its current-run LLBC or generated declaration, or a receipt attempts to use `def_id` as a durable Octet or Valence stable ID
- WHEN manifest validation runs
- THEN validation MUST fail because declaration IDs are within-run join keys only and durable identity MUST remain BLAKE3-bound to source bytes/spans, provider profile, generated model, and theorem source.

### Requirement: Theorem adequacy and trust closure are explicit

r[octet.aeneas_lean_proof_rail.theorem_adequacy] Octet MUST require an approved theorem-statement BLAKE3, requirement ID, explicit claim class, generated-versus-maintained source classification, deterministic adequacy decision, and complete declared closure of assumptions, axioms, external models, opaque definitions, and admitted declarations before property promotion.

#### Scenario: Approved theorem and trust closure pass

- GIVEN the selected theorem statement matches the approved requirement and claim class, its closure contains no undeclared trust input, and the adequacy validator reports non-vacuous
- WHEN theorem admission runs
- THEN the theorem MAY proceed to Lean checking with its exact statement and trust identities bound.

#### Scenario: Vacuous or undeclared trust input fails

- GIVEN the selected property is a `True`-style vacuous statement for a stronger claim class, contains `sorry` or `admit`, depends on an undeclared axiom/model/opaque definition, or lacks an approved requirement or statement hash
- WHEN theorem admission runs
- THEN Octet MUST reject property promotion even if Lean accepts the encoded source.

### Requirement: Translation alone is not formal proof

r[octet.aeneas_lean_proof_rail.theorem_gate] Octet MUST emit `formal_proof` provenance for an Aeneas/Lean rail only when a bound named theorem over the generated model is present, its approved statement, requirement, claim class, theorem source, adequacy decision, assumptions, and trust closure are identified, Lean checks it successfully, and the complete stage chain validates.

#### Scenario: Named theorem is checked

- GIVEN Charon and Aeneas succeed, the configured theorem exists in the maintained theorem source, statement/requirement/claim identities and trust closure validate, Lean reports success, and every stage identity validates
- WHEN evidence-tier assignment runs
- THEN Octet MAY emit `formal_proof` provenance scoped to that theorem, source selection, generated model, assumptions, and receipt.

#### Scenario: Generated Lean compiles without the required theorem

- GIVEN Charon and Aeneas succeed but the required theorem is absent, admitted, stale, unbound, or not checked successfully
- WHEN evidence-tier assignment runs
- THEN generated artifacts MUST remain recorded-only and Octet MUST NOT emit `formal_proof` provenance.

### Requirement: Unsupported source surfaces fail without promotion

r[octet.aeneas_lean_proof_rail.unsupported_surface] Octet MUST classify unsafe Rust, concurrency, unsupported nested-loop control transfer, unresolved external definitions, unsupported language features, and translation failures as explicit blocked or failed provider outcomes.

#### Scenario: Unsupported source is selected

- GIVEN selected source contains a construct outside the reviewed Aeneas provider surface
- WHEN provider preflight or translation runs
- THEN the rail MUST report the unsupported construct or stage and MUST NOT claim that the source property passed or failed in Lean.

#### Scenario: Supported safe sequential fixture is selected

- GIVEN an Octet-owned fixture uses only the reviewed safe sequential subset and declared models
- WHEN provider preflight runs
- THEN the fixture MAY proceed to the stage-chain execution shell.

### Requirement: Aeneas proof execution is bounded

r[octet.aeneas_lean_proof_rail.safe_execution] Octet MUST run Charon, Aeneas, and Lean through explicit argv, pinned executable identities, deterministic working roots, named time/resource/log limits, an allowlisted environment, and network denial while keeping validation logic pure over in-memory observations.

#### Scenario: All stages execute within policy

- GIVEN the provider package and inputs match the package lock
- WHEN `cargo octet prove` executes the selected `aeneas-lean` rail
- THEN the shell MUST capture bounded stage results and pass typed observations to the pure validator.

#### Scenario: Process execution violates policy

- GIVEN a stage cannot spawn, times out, exceeds a declared bound, requests network access, resolves an unpinned executable, or exits unsuccessfully
- WHEN proof execution runs
- THEN the rail MUST fail closed with a stage-specific diagnostic and MUST NOT emit `formal_proof` provenance.

### Requirement: Existing Verus proof contracts remain compatible

r[octet.aeneas_lean_proof_rail.verus_compatibility] Octet MUST preserve existing Verus proof package-lock, receipt, proof-card, artifact-verification, and starter-path behavior through an explicit compatibility adapter rather than silently reinterpreting Verus v1 artifacts.

#### Scenario: Existing Verus artifacts remain valid

- GIVEN a currently valid Verus production or fixture artifact bundle
- WHEN common-envelope adaptation and artifact verification run
- THEN its accepted bytes, scoped status, and diagnostics MUST remain compatible.

#### Scenario: Aeneas fields are injected into a Verus receipt

- GIVEN a Verus v1 receipt contains undeclared Aeneas stage fields or is relabeled without a valid provider payload
- WHEN receipt validation runs
- THEN validation MUST fail rather than infer a different provider.

### Requirement: Aeneas proof artifacts remain reviewable

r[octet.aeneas_lean_proof_rail.artifact_evidence] Octet MUST persist provider receipts and proof cards that expose the theorem ID, source scope, stage identities, toolchain identities, model/assumption inventory, status, evidence tier, and non-claims, and `cargo octet artifact verify` MUST remeasure their declared local artifacts without mutation.

#### Scenario: Verified artifact directory passes

- GIVEN a passing Aeneas receipt, matching proof card, matching provenance, and untampered stage artifacts
- WHEN artifact verification runs
- THEN verification MUST accept the directory and report the theorem-scoped claim.

#### Scenario: Artifact or role is tampered

- GIVEN a stage artifact, theorem ID, assumption row, role, status, proof card, or receipt link changes after the proof run
- WHEN artifact verification runs
- THEN verification MUST fail and MUST NOT retain the prior `formal_proof` disposition.

### Requirement: Valence handoff preserves provider boundaries

r[octet.aeneas_lean_proof_rail.valence_handoff] Octet SHOULD export a frozen Valence adapter input for a verified Aeneas receipt that preserves intermediate translation stages as recorded-only or boundary evidence and identifies the final Octet-owned theorem receipt as the sole property candidate.

#### Scenario: Verified receipt exports

- GIVEN Octet artifact verification succeeds for the selected Aeneas rail
- WHEN the Valence adapter export runs
- THEN the export MUST bind every stage, role, digest, theorem ID, receipt ID, and required non-claim.

#### Scenario: Unverified receipt cannot export property evidence

- GIVEN artifact verification is absent or failed, or any translation stage is stale
- WHEN export is requested
- THEN Octet MUST refuse property-role export and MUST NOT imply Valence acceptance.

### Requirement: Pin upgrades are fixture-driven

r[octet.aeneas_lean_proof_rail.pin_upgrade_matrix] Octet MUST gate any Aeneas, Charon, rustc, Lean, Aeneas-library, schema, or provider-profile upgrade on a deterministic compatibility matrix rather than branch availability, documentation, or issue status.

#### Scenario: Upgrade matrix passes

- GIVEN a proposed pin change reruns the positive theorem, false-theorem rejection, vacuity rejection, partial-output rejection, borrow-check bypass rejection, opacity/model cases, target-width/endianness cases, source-to-manifest-to-Lean freshness, and representative unsupported-feature canaries
- WHEN compatibility admission runs
- THEN the new profile MAY replace the reviewed profile only when every fixture has its declared outcome and a new compatibility receipt is recorded.

#### Scenario: Upgrade skips or changes a canary outcome

- GIVEN any required fixture is absent, stale, unexpectedly passes, unexpectedly fails, or produces an unreviewed diagnostic class
- WHEN compatibility admission runs
- THEN the proposed profile MUST remain blocked.

### Requirement: Semantic foundation references remain recorded-only

r[octet.aeneas_lean_proof_rail.semantic_reference] Octet MAY record an exact `mechanized-llbc` revision, license identity, and source sentinel as semantic-foundation reference metadata, but it MUST keep that material recorded-only and MUST NOT treat it as proof that the pinned Charon or Aeneas implementations conform to those semantics.

#### Scenario: Exact semantic reference is recorded

- GIVEN a reviewed provider profile names an exact `mechanized-llbc` revision with license and sentinel identities
- WHEN profile validation runs
- THEN Octet MAY preserve the reference as non-property provenance.

#### Scenario: Semantic reference is promoted

- GIVEN a receipt uses the reference corpus to claim implementation conformance, translation soundness, theorem truth, or verifier soundness
- WHEN claim validation runs
- THEN validation MUST fail.

### Requirement: Aeneas provider claims remain bounded

r[octet.aeneas_lean_proof_rail.docs] Octet documentation and output MUST state that an accepted Aeneas/Lean receipt supports only the named theorem over the exact bound generated model and source/toolchain chain.

#### Scenario: Operator inspects a passing proof card

- GIVEN a proof card reports a passing Aeneas theorem
- WHEN its supported claim is rendered
- THEN it MUST disclaim Aeneas, Charon, Lean, rustc, translation, external-model, and verifier soundness; semantic equivalence; whole-program correctness; unsafe/concurrent Rust; and release eligibility.

### Requirement: Aeneas provider validation covers success and failure

r[octet.aeneas_lean_proof_rail.final_validation] The Aeneas provider change MUST include a passing theorem fixture, an intentionally false theorem fixture, a vacuous `True` theorem fixture, extraction-contract and translation-manifest fixtures, stage-tamper fixtures, unsupported-surface fixtures, the pin-upgrade compatibility matrix, existing Verus compatibility checks, and starter-path independence evidence before archive.

#### Scenario: Focused provider matrix is complete

- GIVEN the provider implementation and fixtures are ready for review
- WHEN focused proof, policy, artifact, compatibility, and Cairn checks run
- THEN the valid theorem MUST pass, false and vacuous statements MUST be rejected for distinct expected reasons, every extraction/manifest/tamper negative MUST fail for its expected reason, the upgrade matrix MUST be complete, existing Verus rails MUST retain their scoped behavior, and ordinary `cargo octet check` MUST remain independent of Aeneas tooling.

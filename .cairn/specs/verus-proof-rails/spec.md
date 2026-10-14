# Verus Proof Rails Specification

## Purpose

Defines the `verus-proof-rails` capability.

## Requirements

### Requirement: Verus proof-rail contract

r[octet.verus_proof_rails.contract] Octet SHOULD support Verus as an opt-in formal-proof rail that is scoped to declared source paths, proof paths, verifier identity, pass signal, and trust boundaries.

#### Scenario: Pinned Verus rail is declared

r[octet.verus_proof_rails.contract.declared]

- GIVEN a policy declares a Verus proof rail
- WHEN Octet exports or validates the policy registry
- THEN the rail MUST declare `backend = "verus"`, a deterministic command, source paths, proof paths, an exact pass signal, and trust-boundary notes.
- AND the rail MUST NOT rely on an ambient unpinned `verus` binary as verifier identity for `formal_proof` evidence.

#### Scenario: Proof claim remains scoped

r[octet.verus_proof_rails.contract.scoped]

- GIVEN a Verus proof rail passes
- WHEN Octet renders proof evidence or documentation for that rail
- THEN the evidence MUST name the checked source scope, proof artifact identity, verifier identity, and covered property or policy ID.
- AND the evidence MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, or full safety without separate evidence.

### Requirement: Persistent Verus proof receipts

r[octet.verus_proof_rails.receipts] Octet MUST persist proof receipts for executed Verus rails when the caller requests proof artifacts, and those receipts MUST be validated before they can support `formal_proof` provenance.

#### Scenario: Matching proof receipt passes

r[octet.verus_proof_rails.receipts.matching]

- GIVEN a Verus proof rail has a persisted receipt containing the configured pass signal
- AND the receipt contains matching BLAKE3 command and source bindings when the rail enables command/source binding
- AND the receipt contains configured verifier, provenance, and trust-boundary lines
- WHEN proof receipt validation runs
- THEN validation MUST accept the receipt.
- AND Octet MAY emit `formal_proof` provenance for the proof artifact.

#### Scenario: Stale or incomplete proof receipt fails

r[octet.verus_proof_rails.receipts.fail_closed]

- GIVEN a Verus proof receipt is missing the configured pass signal, has a stale source BLAKE3 binding, has wrong verifier identity, omits a required trust-boundary line, or omits proof artifact identity
- WHEN proof receipt validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the offending receipt field.
- AND Octet MUST NOT emit `formal_proof` provenance for that receipt.

#### Scenario: Proof artifact verifier is read-only

r[octet.verus_proof_rails.receipts.artifact_verify]

- GIVEN an operator has a saved proof artifact directory
- WHEN `cargo octet artifact verify` validates proof receipts and provenance
- THEN the verifier MUST recompute BLAKE3 identities and validate provenance coverage without mutating the artifact directory.
- AND malformed, missing, stale, or unmatched proof artifacts MUST fail closed.

### Requirement: Proof command surface

r[octet.verus_proof_rails.prove_command] Octet SHOULD expose a `cargo octet prove` command that runs selected proof rails and writes reviewable proof artifacts without changing ordinary lint-check behavior.

#### Scenario: Prove command writes scoped artifacts

r[octet.verus_proof_rails.prove_command.artifacts]

- GIVEN a workspace declares a selected Verus proof rail
- WHEN the operator runs `cargo octet prove` with an artifact directory
- THEN Octet SHOULD run the pinned verifier command, write a proof receipt, write provenance with `evidence_tier = formal_proof` only for validated proof receipts, and report the checked scope.
- AND the command SHOULD return a non-zero status when any selected proof rail fails validation.

#### Scenario: Starter lint path remains unaffected

r[octet.verus_proof_rails.prove_command.starter_unaffected]

- GIVEN a consumer runs ordinary `cargo octet check` under a starter profile with no selected proof rails
- WHEN the check completes
- THEN Octet MUST NOT require Verus to be installed or executed.
- AND the absence of Verus artifacts MUST NOT be rendered as a formal-proof failure for that starter check.

### Requirement: Verus proof registry ownership

r[octet.verus_proof_rails.registry] Verus proof receipt and verifier identity families MUST be registered in `standards/policies/registry.ncl` with Rust-derived ownership or explicit non-candidate rationale.

#### Scenario: Registry covers proof receipt family

r[octet.verus_proof_rails.registry.covered]

- GIVEN Octet emits Verus proof receipts after execution
- WHEN config/receipt family validation runs
- THEN the Verus proof receipt family MUST be registered as Rust-derived.
- AND generated policy artifacts MUST include the proof-rail fields needed by downstream receipt validators.

#### Scenario: Raw verifier internals remain excluded

r[octet.verus_proof_rails.registry.non_candidate]

- GIVEN verifier internals, raw logs, raw environment state, or secrets are encountered during proof execution
- WHEN registry ownership is reviewed
- THEN those raw surfaces MUST be excluded from Nickel-owned config and receipts except as bounded paths, labels, digests, allowlists, or explicit non-candidate rationale.

### Requirement: Verified-module proof coverage

r[octet.verus_proof_rails.verified_module_coverage] Octet verification profiles MUST make unbacked `verified/` module or formal-proof claims explicit instead of silently treating purity lint success as proof.

#### Scenario: Covered verified module passes

r[octet.verus_proof_rails.verified_module_coverage.covered]

- GIVEN a source file under `verified/` is selected by a verification profile
- AND the policy registry declares a matching Verus proof rail for that source file
- WHEN verification-profile checks run
- THEN Octet MUST accept the verified-module coverage obligation only after the proof receipt validates.

#### Scenario: Unbacked verified claim is manual review or failure

r[octet.verus_proof_rails.verified_module_coverage.unbacked]

- GIVEN a source file, document, or artifact claims formal verification for a module but no matching proof rail or validated proof receipt exists
- WHEN verification-profile checks run
- THEN Octet MUST either fail the selected verification gate or emit `manual_review_required` provenance with a human-action message, according to the selected profile.
- AND Octet MUST NOT emit `formal_proof` provenance for the unbacked claim.

### Requirement: Production Verus verifier identity

r[octet.verus_proof_rails.verifier_identity] Octet MUST distinguish production Verus verifier identity from fixture-only verifier identity before emitting promotion-grade `formal_proof` evidence.

#### Scenario: Production verifier identity is complete

r[octet.verus_proof_rails.verifier_identity.production]

- GIVEN a Verus proof rail is eligible for production `formal_proof` provenance
- WHEN `cargo octet prove` records verifier identity for that rail
- THEN the identity MUST include the pinned verifier label, verifier binary digest, solver identity, Rust toolchain identity, and Nix closure or closure-graph digest.
- AND the identity MUST be recorded in the proof receipt and provenance record.

#### Scenario: Fixture verifier remains bounded

r[octet.verus_proof_rails.verifier_identity.fixture]

- GIVEN a proof run uses the fixture verifier wrapper
- WHEN Octet writes proof artifacts for that run
- THEN the receipt and summary MUST label the evidence as fixture-only.
- AND Octet MUST NOT promote that receipt as production `formal_proof` provenance unless the checked scope explicitly declares fixture-only proof plumbing.

### Requirement: Structured Verus proof receipts

r[octet.verus_proof_rails.structured_receipts] Octet MUST persist Verus proof receipts as structured Rust-owned records with deterministic validation before those receipts can support `formal_proof` provenance.

#### Scenario: Valid structured receipt passes

r[octet.verus_proof_rails.structured_receipts.valid]

- GIVEN a Verus proof rail has a structured receipt with schema version, policy ID, rail index, verifier identity, command identity, source bindings, proof bindings, trust boundaries, checked scope, pass signal, and proof artifact identity
- WHEN proof receipt validation runs
- THEN validation MUST accept the receipt.
- AND Octet MAY emit `formal_proof` provenance for the matching proof artifact.

#### Scenario: Malformed or stale structured receipt fails closed

r[octet.verus_proof_rails.structured_receipts.fail_closed]

- GIVEN a structured receipt omits a required field, uses an unsupported schema version, has stale source or proof bindings, has wrong verifier identity, omits proof artifact identity, or records fixture evidence as production evidence
- WHEN proof receipt validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field.
- AND Octet MUST NOT emit `formal_proof` provenance for that receipt.

#### Scenario: Receipt contract is registered

r[octet.verus_proof_rails.structured_receipts.contract]

- GIVEN Octet emits structured Verus proof receipts from Rust code
- WHEN policy freshness validation runs
- THEN the Verus proof receipt contract MUST be registered as a Rust-derived receipt family.
- AND the committed Nickel contract MUST match the Rust-owned receipt schema source.

### Requirement: Safe Verus proof command execution

r[octet.verus_proof_rails.safe_execution] Octet MUST execute proof rails through a deterministic command model instead of an unchecked shell command string.

#### Scenario: Proof command uses argv and bounded environment

r[octet.verus_proof_rails.safe_execution.argv]

- GIVEN a selected Verus proof rail declares an executable and arguments
- WHEN `cargo octet prove` runs the rail
- THEN the CLI shell MUST execute the pinned verifier with explicit argv, a deterministic working directory, an allowlisted environment, a configured timeout, and bounded stdout/stderr artifact capture.
- AND the pure validation core MUST receive only typed command results and in-memory receipt inputs.

#### Scenario: Proof command failures fail closed

r[octet.verus_proof_rails.safe_execution.failures]

- GIVEN proof command spawning fails, exits unsuccessfully, times out, exceeds bounded log capture, or resolves to an unpinned verifier
- WHEN `cargo octet prove` reports the run
- THEN the proof row MUST fail with a deterministic diagnostic naming the command failure class.
- AND Octet MUST NOT emit `formal_proof` provenance for that failed run.

#### Scenario: Starter lint path remains independent

r[octet.verus_proof_rails.safe_execution.starter_independent]

- GIVEN a consumer runs ordinary `cargo octet check` under a starter profile
- WHEN no proof rails are selected
- THEN Octet MUST NOT require Verus, solver binaries, proof manifests, or proof artifacts.
- AND missing proof tooling MUST NOT affect the starter lint result.

### Requirement: Normalized proof rail path matching

r[octet.verus_proof_rails.path_matching] Octet MUST bind proof rails to normalized repo-relative source and proof paths using exact or glob semantics rather than substring matching.

#### Scenario: Exact and glob matches are deterministic

r[octet.verus_proof_rails.path_matching.deterministic]

- GIVEN a proof rail declares exact source paths, exact proof paths, or explicit glob patterns
- WHEN rail selection and receipt validation run
- THEN Octet MUST normalize candidate paths to repo-relative form before matching.
- AND Octet MUST bind each matched path deterministically in sorted order for receipt hashing and diagnostics.

#### Scenario: Ambiguous or overbroad matches fail closed

r[octet.verus_proof_rails.path_matching.ambiguous]

- GIVEN a rail path pattern is empty, matches no selected source, relies only on substring matching, or matches multiple paths where the rail requires a single artifact
- WHEN rail selection or verified-module coverage runs
- THEN Octet MUST report a deterministic diagnostic naming the path pattern.
- AND Octet MUST NOT treat the rail as proof coverage for the affected source.

### Requirement: Nickel proof rail manifest

r[octet.verus_proof_rails.rail_manifest] Human-authored Verus proof rail declarations SHOULD be maintained as typed Nickel maps keyed by rail ID and exported as deterministic JSON for runtime Rust consumers.

#### Scenario: Valid Nickel manifest exports deterministic rails

r[octet.verus_proof_rails.rail_manifest.valid]

- GIVEN a maintainer declares Verus proof rails in the Nickel manifest under built-in or consumer-owned rail IDs
- WHEN the manifest export command runs
- THEN every exported rail MUST include backend, command argv, source paths, proof paths, verifier identity requirements, pass signal, trust boundaries, and receipt binding policy.
- AND the manifest contract MUST allow additional rail IDs without changing the contract for each new proof target.
- AND repeated exports from the same manifest MUST produce byte-identical JSON.

#### Scenario: Invalid Nickel manifest fails before execution

r[octet.verus_proof_rails.rail_manifest.invalid]

- GIVEN a manifest omits required rail fields, declares an unsupported backend, declares no rails, uses empty source paths, or provides malformed trust boundaries
- WHEN manifest validation runs
- THEN validation MUST fail before `cargo octet prove` executes any verifier.
- AND the diagnostic MUST name the rejected manifest field.

### Requirement: Verus proof operator feedback

r[octet.verus_proof_rails.operator_feedback] Octet SHOULD provide list and explain surfaces for proof rails without broadening formal-proof claims.

#### Scenario: Selected rails can be listed

r[octet.verus_proof_rails.operator_feedback.list]

- GIVEN a workspace has exported Verus proof rails
- WHEN an operator runs the proof rail listing surface
- THEN Octet SHOULD show each selected rail's policy ID, backend, source scope, proof path, verifier label, and fixture-or-production mode.
- AND the output MUST include scoped non-claim wording for formal proof evidence.

#### Scenario: Explicit rail selectors narrow execution

r[octet.verus_proof_rails.operator_feedback.rail_selector]

- GIVEN a workspace has multiple selected Verus proof rails
- WHEN an operator passes `cargo octet prove --rail <policy-id[#rail-N]>`
- THEN Octet SHOULD execute, list, or explain only rails matching the explicit policy selector after ordinary family and source selection.
- AND unmatched, malformed, wrong-family, or wrong-source selectors MUST produce deterministic diagnostics and MUST NOT execute a different verifier rail.

#### Scenario: Unbacked claims can be explained

r[octet.verus_proof_rails.operator_feedback.explain]

- GIVEN a verified module or formal-proof claim lacks a matching validated receipt
- WHEN Octet reports or explains the claim
- THEN the output SHOULD identify the unmatched source, missing or stale rail field, required command, and remediation.
- AND Octet MUST NOT emit `formal_proof` provenance for the unbacked claim.

### Requirement: Production Verus negative proof behavior

r[octet.verus_proof_rails.production_negative] Octet MUST prove that production-mode Verus proof failures fail closed before expanding production formal-proof scope.

#### Scenario: Invalid production proof is rejected

r[octet.verus_proof_rails.production_negative.direct_rejects]

- GIVEN an Octet-owned Verus proof fixture with an intentionally unprovable postcondition
- WHEN `octet-production-verus` runs that proof fixture through explicit argv
- THEN the verifier command MUST exit non-zero.
- AND the fixture MUST be labeled as failure-path evidence only, not as a valid formal proof.

#### Scenario: Failed production rail emits no formal proof

r[octet.verus_proof_rails.production_negative.no_formal_proof]

- GIVEN a selected production-mode Verus rail runs an invalid proof fixture
- WHEN `cargo octet prove` captures the failed verifier result
- THEN the proof run MUST report a deterministic verifier-failure diagnostic and return non-zero.
- AND the artifact directory MAY contain bounded failure stdout/stderr artifacts.
- AND Octet MUST NOT emit `formal_proof` provenance for the failed rail.

#### Scenario: Negative fixture cannot break ordinary proof runs

r[octet.verus_proof_rails.production_negative.isolated]

- GIVEN the repository contains an invalid Verus proof fixture for fail-closed testing
- WHEN an operator runs the normal positive production pilot proof scope
- THEN the invalid fixture MUST NOT be selected accidentally.
- AND the existing positive production proof rail MUST continue to validate with scoped `formal_proof` provenance.

### Requirement: Multiple production Verus pilot rails

r[octet.verus_proof_rails.production_pilot_expansion] Octet SHOULD support more than one tiny production-mode Verus pilot rail only after production failure semantics are tested fail-closed, and each pilot rail MUST remain independently scoped.

#### Scenario: Second production pilot has independent identity

r[octet.verus_proof_rails.production_pilot_expansion.independent_identity]

- GIVEN Octet adds a second tiny production-mode Verus proof target
- WHEN the proof-rail manifest and policy registry are exported
- THEN the new rail MUST have a unique policy ID, checked scope, receipt path, source binding, proof binding, pass signal, and trust-boundary text.
- AND the rail MAY reuse the pinned `octet-production-verus` verifier identity only when the verifier, solver, Rust toolchain, and Nix closure graph bindings still match the production verifier manifest.

#### Scenario: Multiple production receipts verify independently

r[octet.verus_proof_rails.production_pilot_expansion.multi_receipt]

- GIVEN two tiny production-mode Verus pilot rails are selected in one proof run
- WHEN `cargo octet prove` writes artifacts and `cargo octet artifact verify` validates them
- THEN each rail MUST write a distinct structured receipt and proof artifact identity.
- AND each `formal_proof` provenance record MUST name only its own checked source scope and proof property.
- AND a stale or swapped receipt for one rail MUST fail with a deterministic diagnostic without making the other rail's identity ambiguous.

#### Scenario: Expanded pilot scope remains non-claiming

r[octet.verus_proof_rails.production_pilot_expansion.non_claiming]

- GIVEN documentation describes multiple production Verus pilot rails
- WHEN a reader inspects the proof-rail documentation
- THEN the documentation MUST state that each pilot proves only its declared property.
- AND it MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, or full safety.

### Requirement: Verus proof-path explainer

r[octet.verus_proof_rails.proof_path_explainer] Octet SHOULD document the Verus formal-proof path as a scoped evidence lane from requirement or policy ID to artifact verification, while preserving proof-vs-validation and imported-vs-formal-proof tier boundaries.

#### Scenario: Proof path names every direct stage

r[octet.verus_proof_rails.proof_path_explainer.stages]

- GIVEN Octet documentation describes Verus proof rails for operators or reviewers
- WHEN a reader follows the direct Octet Verus proof path
- THEN the documentation SHOULD name the requirement or policy ID, Verus proof rail, checked Rust source scope, proof target, verifier/solver/toolchain identity, structured proof receipt, `formal_proof` provenance, and artifact-verifier stage.
- AND the documentation SHOULD link the stages to existing proof commands or artifact commands where those commands are the review surface.

#### Scenario: Imported Trellis branch stays content-addressed

r[octet.verus_proof_rails.proof_path_explainer.trellis_branch]

- GIVEN Octet documentation describes using Trellis proof artifacts as references
- WHEN a reader follows the imported proof branch
- THEN the documentation SHOULD name the Trellis repository revision, source artifact hash, proof artifact hash, bounded log hash, receipt hash when present, hash algorithm, imported bundle status, and optional Octet rerun stage.
- AND the documentation MUST state that BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness.
- AND the imported branch MUST remain external or `manual_review_required` unless Octet reruns the proof with pinned verifier identity and validates an Octet-owned receipt.

#### Scenario: Proof path preserves evidence tiers

r[octet.verus_proof_rails.proof_path_explainer.tiers]

- GIVEN the Verus proof-path explainer mentions proof, validation, lint, policy, transcript, imported Trellis bundle, or manual review evidence
- WHEN the wording assigns an evidence tier
- THEN validated Octet-owned Verus proof receipts MAY be described as supporting `formal_proof` provenance.
- AND lint findings MUST remain `static_lint`, policy checks MUST remain `policy_check`, transcript checks MUST remain `transcript_assertion`, imported Trellis bundles MUST remain external or `manual_review_required` until rerun, unresolved human obligations MUST remain `manual_review_required`, and validation evidence MUST NOT be relabeled as formal proof.

#### Scenario: Proof path stays non-claiming

r[octet.verus_proof_rails.proof_path_explainer.non_claiming]

- GIVEN documentation presents the Verus proof path
- WHEN documentation validation scans proof and correctness language
- THEN the proof claim MUST be scoped to the declared property, source binding, proof binding, verifier identity, receipt, and artifact identity.
- AND the documentation MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime without separate evidence.

### Requirement: Verus proof cards

r[octet.verus_proof_rails.proof_cards] Octet SHOULD provide receipt-backed proof cards for selected Verus proof rails and BLAKE3-identified imported proof references so reviewers can inspect each scoped proof claim without treating the card as a standalone proof artifact.

#### Scenario: Proof card contains required fields

r[octet.verus_proof_rails.proof_cards.fields]

- GIVEN Octet emits or documents a proof card for a Verus proof rail
- WHEN a reviewer inspects the card
- THEN the card SHOULD name the proof or property ID, source requirement or policy ID, checked Rust source scope, proof target path, verifier mode, verifier identity summary, solver/toolchain summary, receipt path, artifact BLAKE3 identities, evidence tier, status, trust-boundary notes, and non-claims.
- AND the card SHOULD link to the artifact-verifier command or receipt/provenance artifact that supports the status.

#### Scenario: Formal-proof card is receipt backed

r[octet.verus_proof_rails.proof_cards.receipt_backed]

- GIVEN a proof card reports `evidence_tier = formal_proof` or equivalent formal-proof status
- WHEN card validation runs
- THEN validation MUST confirm that the card is generated from, or matches, a validated structured proof receipt and matching provenance record for the same source scope, proof target, verifier mode, and artifact identity.
- AND a card without a matching validated receipt MUST NOT report `formal_proof` status.

#### Scenario: Fixture, production, and external modes stay distinct

r[octet.verus_proof_rails.proof_cards.mode_distinct]

- GIVEN proof cards exist for fixture-only, production, and imported external Verus evidence
- WHEN a reviewer compares the cards
- THEN each card MUST display its verifier mode and evidence origin.
- AND fixture-only cards MUST NOT be presented as production formal-proof evidence unless the checked scope explicitly declares fixture-only proof plumbing.
- AND imported Trellis cards MUST NOT be presented as Octet `formal_proof` evidence unless Octet reruns the proof with a pinned verifier and validates an Octet-owned receipt.

#### Scenario: Overclaiming card fails validation

r[octet.verus_proof_rails.proof_cards.no_overclaim]

- GIVEN a proof card mentions proof, safety, verification, correctness, certification, business-domain validation, or semantic equivalence
- WHEN card validation scans the claim language
- THEN the claim MUST be scoped to the declared property, source binding, proof binding, verifier identity, receipt, and artifact identity.
- AND cards MUST NOT claim whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime without separate evidence.

#### Scenario: Imported Trellis card preserves BLAKE3 scope

r[octet.verus_proof_rails.proof_cards.trellis_blake3]

- GIVEN a proof card references a Trellis proof bundle by BLAKE3 identity
- WHEN card validation runs
- THEN validation MUST require the Trellis repository revision, source artifact hash, proof artifact hash, bounded log hash, receipt hash when present, hash algorithm, and evidence-origin fields.
- AND the card MUST state that BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness.
- AND the card MUST remain non-`formal_proof` unless the referenced proof is rerun and receipt-validated by Octet.

### Requirement: Trellis BLAKE3 proof-bundle import

r[octet.verus_proof_rails.trellis_blake3_import] Octet SHOULD support imported Trellis proof-bundle references that bind external proof artifacts by BLAKE3 identity while keeping imported references separate from Octet-owned `formal_proof` provenance until rerun and receipt validation succeed.

#### Scenario: Imported bundle declares required identity fields

r[octet.verus_proof_rails.trellis_blake3_import.fields]

- GIVEN a maintainer declares an imported Trellis proof bundle
- WHEN import validation runs
- THEN the bundle MUST name the repository label, repository revision, source path, proof path or proof module, verifier command when known, hash algorithm, source artifact hash, proof artifact hash, bounded stdout/stderr hashes when logs are present, receipt hash when a receipt is present, evidence origin, intended evidence tier, and trust-boundary notes.
- AND paths MUST be normalized before hash comparison and diagnostics.

#### Scenario: BLAKE3 identity is not proof correctness

r[octet.verus_proof_rails.trellis_blake3_import.identity_boundary]

- GIVEN an imported Trellis bundle has matching BLAKE3 source, proof, log, or receipt hashes
- WHEN Octet renders a proof card, proof-path explainer, provenance summary, or artifact-verifier output for that bundle
- THEN the output MUST state or encode that the hashes identify exact artifact bytes.
- AND the output MUST NOT claim those hashes prove Verus success, solver soundness, model adequacy, Octet receipt validity, whole-program correctness, certification, semantic equivalence, business-domain validation, full safety, or a verified runtime.

#### Scenario: Imported bundle stays external until Octet rerun

r[octet.verus_proof_rails.trellis_blake3_import.promotion]

- GIVEN an imported Trellis bundle has not been rerun by Octet
- WHEN evidence-tier assignment runs
- THEN the bundle MUST remain external evidence or `manual_review_required`.
- AND Octet MUST NOT emit `formal_proof` provenance for the imported bundle.
- WHEN Octet reruns the referenced proof with pinned verifier, solver, toolchain, command argv, source/proof bindings, and validates an Octet-owned structured receipt
- THEN Octet MAY emit scoped `formal_proof` provenance for the Octet-owned receipt only.

#### Scenario: Malformed or stale import fails closed

r[octet.verus_proof_rails.trellis_blake3_import.fail_closed]

- GIVEN an imported Trellis bundle omits a required field, declares an unsupported hash algorithm, has a malformed digest, has a stale source/proof/log/receipt hash, duplicates an artifact identity where uniqueness is required, omits trust-boundary notes, or requests `formal_proof` without an Octet-owned validated receipt
- WHEN import validation runs
- THEN validation MUST fail closed with a deterministic diagnostic naming the rejected field or artifact.
- AND Octet MUST NOT render the bundle as valid proof-card evidence.

#### Scenario: Import manifest ownership is registered

r[octet.verus_proof_rails.trellis_blake3_import.registry]

- GIVEN Octet maintains human-authored Trellis imported-proof bundle declarations
- WHEN policy freshness and registry validation run
- THEN the import manifest MUST be typed Nickel and registered as Nickel-authored.
- AND generated JSON imports MUST be covered by freshness checks.
- AND Rust-derived import validation receipts, if emitted, MUST be registered as Rust-derived receipt families.
- AND raw Trellis logs, verifier internals, environment state, and secrets MUST remain non-candidate surfaces except as bounded hashes, paths, labels, allowlists, or explicit rationale.

### Requirement: Proof evidence references

r[octet.verus_proof_rails.proof_evidence_ref] Octet SHOULD provide a Rust-owned `ProofEvidenceRef` data contract that links local claims, documentation references, or future lint suppressions to existing proof-card evidence without introducing a consumer-implementable proof trait.

#### Scenario: Reference fields are explicit

r[octet.verus_proof_rails.proof_evidence_ref.fields]

- GIVEN a `ProofEvidenceRef` JSON record is authored
- WHEN Octet validates the record shape
- THEN the record SHOULD include schema version, reference ID, consumer context, proof or property ID, source requirement or policy ID, checked Rust source scope, expected evidence tier, required status, required artifacts, and explicit non-claims.
- AND when the reference links a source comment, documentation line, or suppression, it SHOULD include consumer source path, consumer source BLAKE3 identity, and exact consumer source anchor text.
- AND the record MUST NOT be represented as a Rust trait or marker that consumer code can implement to assert proof status.

#### Scenario: Reference is linkage not proof

r[octet.verus_proof_rails.proof_evidence_ref.boundary]

- GIVEN a `ProofEvidenceRef` validates against a proof card
- WHEN Octet reports the result
- THEN the report MUST state or encode that the reference validates linkage to existing proof-card evidence.
- AND the reference MUST NOT by itself emit `formal_proof` provenance, rerun Verus, certify a system, or broaden the proof beyond the matched proof card.

### Requirement: Proof-ref validation CLI

r[octet.verus_proof_rails.proof_evidence_ref.cli] Octet SHOULD expose `cargo octet proof-ref scaffold` and read-only `cargo octet proof-ref validate --proof-cards PATH --ref PATH` commands that create or validate a proof evidence reference against a saved `proof-cards.json` deck without rerunning Verus.

#### Scenario: Matching reference validates

r[octet.verus_proof_rails.proof_evidence_ref.cli.matching]

- GIVEN a proof-card deck contains a card with the referenced proof or property ID, policy ID, checked scope, evidence tier, status, artifacts, and non-claims
- AND the reference either omits consumer-source fields or names a readable source file whose BLAKE3 identity and exact anchor text match
- WHEN the operator runs `cargo octet proof-ref validate --proof-cards proof-cards.json --ref proof-ref.json`
- THEN the command SHOULD exit successfully.
- AND the human or JSON report SHOULD name the reference ID, proof or property ID, card match, optional consumer-source match, status, and zero diagnostics.

#### Scenario: Reference scaffold computes source identity

r[octet.verus_proof_rails.proof_evidence_ref.cli.scaffold]

- GIVEN a proof-card deck contains the requested proof or property ID and the consumer source file contains the requested exact anchor text
- WHEN the operator runs `cargo octet proof-ref scaffold --proof-cards proof-cards.json --proof-or-property-id ID --reference-id REF --consumer-context TEXT --consumer-source PATH --anchor TEXT`
- THEN the command SHOULD emit a `ProofEvidenceRef` JSON record populated from the matching proof card and consumer source BLAKE3 identity.
- AND missing proof cards or missing anchors MUST fail closed with deterministic diagnostics.
- AND scaffold MUST NOT rerun Verus, mutate the proof-card deck, emit `formal_proof` provenance, or claim more than linkage to the matched card.

#### Scenario: Validation is read-only

r[octet.verus_proof_rails.proof_evidence_ref.cli.read_only]

- GIVEN an operator runs proof-ref validation
- WHEN the command finishes
- THEN it MUST NOT mutate the proof-card deck, proof receipts, provenance, source files, or artifact directory.
- AND it MUST NOT execute verifier commands.

### Requirement: Proof-ref diagnostics

r[octet.verus_proof_rails.proof_evidence_ref.diagnostics] Proof evidence reference validation MUST fail closed with deterministic diagnostics when the referenced proof card is missing or mismatched.

#### Scenario: Missing or mismatched reference fails

r[octet.verus_proof_rails.proof_evidence_ref.diagnostics.fail_closed]

- GIVEN a `ProofEvidenceRef` names a proof or property ID absent from the proof-card deck, mismatches source requirement, checked scope, expected evidence tier, required status, required artifacts, or non-claims, or names a stale/missing consumer source identity or anchor
- WHEN proof-ref validation runs
- THEN validation MUST fail closed with diagnostics naming the rejected field.
- AND validation MUST NOT upgrade manual-review or external proof-card evidence to `formal_proof`.

#### Scenario: Overbroad reference wording fails

r[octet.verus_proof_rails.proof_evidence_ref.diagnostics.overclaim]

- GIVEN a `ProofEvidenceRef` non-claim or consumer-context field claims whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or verified-runtime behavior
- WHEN validation runs
- THEN validation MUST fail with a diagnostic naming the unsupported claim.

### Requirement: Proof package lock records

r[octet.verus_proof_rails.proof_package_lock] Octet SHOULD support Lake-inspired proof package lock records for selected proof rails, binding intended proof-run material before promotion-grade formal-proof evidence is accepted.

#### Scenario: Valid proof package lock is deterministic

r[octet.verus_proof_rails.proof_package_lock.valid]

- GIVEN a selected production proof rail declares a proof package lock
- WHEN lock validation runs
- THEN the lock SHOULD bind the rail manifest identity, source artifact identities, proof artifact identities, verifier identity, solver identity, Rust toolchain identity, Nix closure or closure-graph identity, command argv, working-directory policy, environment allowlist, pass signal, trust-boundary text, schema version, and lock artifact identity.
- AND repeated validation of the same local inputs SHOULD produce byte-stable machine-readable output.

#### Scenario: Stale or incomplete lock fails closed

r[octet.verus_proof_rails.proof_package_lock.stale]

- GIVEN a proof package lock has a stale source hash, stale proof hash, wrong verifier identity, wrong solver identity, wrong toolchain identity, malformed closure graph, mismatched command argv, invalid environment allowlist, missing pass signal, missing trust-boundary text, unsupported schema version, or unsupported backend
- WHEN lock validation runs
- THEN validation MUST fail closed with deterministic diagnostics naming the rejected lock field.
- AND Octet MUST NOT use the stale lock to support `formal_proof` provenance.

#### Scenario: Receipt must match declared lock

r[octet.verus_proof_rails.proof_package_lock.receipt_match]

- GIVEN a proof rail declares a proof package lock and a verifier run emits a structured proof receipt
- WHEN proof receipt validation considers `formal_proof` provenance
- THEN the receipt MUST match the lock's source, proof, verifier, solver, toolchain, closure, command, environment, pass-signal, and trust-boundary identities.
- AND a receipt from a different or stale proof package MUST fail without making other proof rails ambiguous.

#### Scenario: Lock validation is offline

r[octet.verus_proof_rails.proof_package_lock.offline]

- GIVEN proof package lock validation runs in CI or release review
- WHEN local checked-in artifacts and declared paths are available
- THEN validation MUST use local files, generated manifests, and recorded identities only.
- AND it MUST NOT fetch network schemas, package registries, remote repositories, tool documentation, or verifier metadata.

#### Scenario: Starter check path remains independent

r[octet.verus_proof_rails.proof_package_lock.starter_independent]

- GIVEN a consumer runs ordinary `cargo octet check` with no selected proof rail
- WHEN proof package locks are absent or stale
- THEN the starter lint result MUST remain independent from proof package lock validation.
- AND missing proof package locks MUST NOT be reported as starter-path failures.

### Requirement: Proof goal-state feedback

r[octet.verus_proof_rails.goal_state_feedback] Octet SHOULD provide Lean-inspired proof goal-state feedback for selected proof and evidence review surfaces without requiring Lean or broadening formal-proof claims.

#### Scenario: Satisfied proof goal is explainable

r[octet.verus_proof_rails.goal_state_feedback.satisfied]

- GIVEN a selected proof or proof-reference claim has matching source scope, proof binding, verifier identity, structured receipt, proof card, provenance record, and artifact identity
- WHEN the operator requests proof goal-state feedback
- THEN the output SHOULD name the checked claim, assumptions, required evidence, satisfied evidence, verifier mode, checked scope, receipt path, proof-card identity, provenance identity, and explicit non-claims in deterministic order.
- AND the output SHOULD make clear that the formal-proof claim is scoped to the matched receipt and artifact identities.

#### Scenario: Missing proof obligation is a blocker

r[octet.verus_proof_rails.goal_state_feedback.missing]

- GIVEN a proof or proof-reference claim has a missing receipt, stale source or proof binding, absent proof card, unsupported evidence tier, unmatched source anchor, or verifier failure
- WHEN proof goal-state feedback is computed
- THEN the output MUST report an explicit blocker naming the missing or stale obligation and the remediation surface when one exists.
- AND Octet MUST NOT emit `formal_proof` provenance for the blocked claim.

#### Scenario: Goal-state feedback remains non-claiming

r[octet.verus_proof_rails.goal_state_feedback.non_claim]

- GIVEN proof goal-state feedback mentions Lean inspiration, goals, hypotheses, holes, proof, verification, or correctness
- WHEN documentation and output claim-boundary checks run
- THEN the wording MUST NOT imply that Lean is required, that Octet performs unbounded proof search, that every obligation has a solver-complete answer, or that a scoped receipt proves whole-program correctness, certification, business-domain validation, semantic equivalence, full safety, or a verified runtime.

### Requirement: Proof claim identity index

r[octet.verus_proof_rails.iddqd_proof_claim_catalog.index] Octet MUST validate proof claim catalog entries through identity indexes whose keys are derived from claim and alias values rather than duplicated temporary map keys.

#### Scenario: Canonical and alias lookup passes

r[octet.verus_proof_rails.iddqd_proof_claim_catalog.fixtures.positive]

- GIVEN a proof claim catalog has unique canonical claim ids, unique alias ids, valid alias targets, required artifacts, documentation anchors, and required non-claims
- WHEN proof claim reference validation resolves canonical and alias ids
- THEN validation MUST preserve the target claim identity and accept matching references.

#### Scenario: Duplicate or broken catalog identity fails

r[octet.verus_proof_rails.iddqd_proof_claim_catalog.fixtures.negative]

- GIVEN a proof claim catalog has duplicate claim ids, duplicate alias ids, alias/claim id conflicts, self-targeting aliases, missing targets, invalid evidence tiers, invalid formal-proof promotion, missing required artifacts, missing anchors, or weakened non-claims
- WHEN proof claim catalog validation builds identity indexes
- THEN validation MUST fail closed with deterministic diagnostics and MUST NOT resolve by scan order or overwrite behavior.

### Requirement: Catalog wire compatibility

r[octet.verus_proof_rails.iddqd_proof_claim_catalog.validation] Octet MUST keep the proof claim catalog JSON vector shape compatible while using iddqd indexes only inside pure validation and resolution code.

#### Scenario: Artifact shapes remain stable

r[octet.verus_proof_rails.iddqd_proof_claim_catalog.docs]

- GIVEN proof-card, proof-reference, and provenance export artifacts are produced or validated
- WHEN the proof claim catalog index is used internally
- THEN public artifact schemas MUST remain stable and aliases MUST NOT be treated as one-to-one proof claims.

### Requirement: Final validation evidence

r[octet.verus_proof_rails.iddqd_proof_claim_catalog.final_validation] The change MUST include positive and negative proof claim catalog coverage plus focused Octet validation evidence before archive.

#### Scenario: Proof claim fixtures cover duplicate handling

r[octet.verus_proof_rails.iddqd_proof_claim_catalog.final_validation.fixtures]

- GIVEN valid and invalid proof claim catalog fixtures
- WHEN focused proof claim tests and Cairn gates run
- THEN unique claim and alias identities MUST pass and duplicate, missing, or overclaiming identities MUST fail closed.

### Requirement: Typed Verus toolchain compatibility profile
r[octet.verus_proof_rails.toolchain_profile] Octet MUST publish its reviewed Verus verifier, solver, Rust toolchain, formatter, fixed-output fetch identities, supported targets, claim scope, and non-claims through a typed Nickel profile with deterministic exported runtime data.

#### Scenario: Current profile exports deterministically
- GIVEN the checked Nickel profile and generated runtime export
- WHEN profile freshness validation runs
- THEN the export MUST match the Nickel source byte-for-byte after deterministic generation
- AND malformed or incomplete profiles MUST fail before verifier or formatter execution.

### Requirement: Compatibility rows do not transfer proof claims
r[octet.verus_proof_rails.toolchain_profile.compatibility] Compatibility rows MUST distinguish exact native use, intentional local pins, unsupported targets, and semantic-reference-only snapshots. A row MUST NOT imply that proof receipts, verifier soundness, or semantic coverage transfer across versions.

#### Scenario: Different consumer pin remains explicit
- GIVEN a consumer uses a verifier release different from the Octet profile
- WHEN the compatibility profile is reviewed
- THEN the consumer MUST remain labeled `local-pin` or otherwise non-native
- AND its proof artifacts MUST retain their own verifier, solver, toolchain, source, and receipt identities.

### Requirement: Pinned Verus formatter rail
r[octet.verus_proof_rails.verusfmt] Octet SHOULD package a pinned `verusfmt` release and require Octet-owned Verus proof sources to pass `verusfmt --check --verus-only` without using the formatter self-update path.

#### Scenario: Formatted proof sources pass
- GIVEN Octet-owned Verus proof files and the pinned formatter package
- WHEN the formatter rail runs
- THEN every selected proof file MUST pass without mutation
- AND the formatter identity MUST name the exact release and fetched asset.

### Requirement: Formatter failures fail closed
r[octet.verus_proof_rails.verusfmt.fail_closed] The formatter rail MUST include positive and negative fixtures and MUST reject unformatted Verus input with a non-zero result.

#### Scenario: Unformatted fixture is rejected
- GIVEN a deliberately unformatted valid Verus fixture
- WHEN `verusfmt --check --verus-only` runs
- THEN the command MUST fail
- AND the failure MUST remain formatting evidence only, not a failed or passed proof claim.

### Requirement: Toolchain profile final validation
r[octet.verus_proof_rails.toolchain_profile.final_validation] The change MUST preserve production Verus positive and negative behavior, policy validation, formatter fixture behavior, and profile freshness before archive.

#### Scenario: Focused validation passes
- GIVEN the completed profile and formatter package
- WHEN focused Nix, policy, proof, formatter, and Cairn checks run
- THEN positive checks MUST pass and negative fixtures MUST fail as declared
- AND starter `cargo octet check` behavior MUST remain independent of Verus tooling.

### Requirement: Exact package consumers are registered explicitly [r[octet.verus_proof_rails.toolchain_profile.consumer_registration]]

Octet MUST register every reviewed downstream consumer of its exact Verus toolchain packages in the typed compatibility inventory. Consumer identifiers MUST come from the reviewed closed set, and each row MUST state whether the relationship is native, locally pinned, semantic-reference-only, or unsupported.

A native consumer row MUST mean only that the consumer selects the exact Octet package profile. It MUST NOT transfer proof validity, discharge consumer-specific assumptions, or authorize runtime or release promotion.

#### Scenario: Exact downstream package consumer is admitted

- GIVEN OnixOS lifecycle proof execution or Aspen's node-replication probe selects the exact Octet profile and packages
- WHEN the compatibility profile is validated
- THEN the consumer MUST have a reviewed `native` row with a bounded rationale
- AND the row MUST preserve consumer-owned proof and readiness decisions

#### Scenario: Unknown consumer is rejected

- GIVEN a compatibility row uses an identifier outside the reviewed consumer inventory
- WHEN Nickel contract validation runs
- THEN profile export MUST fail before package or proof execution
- AND the unknown row MUST NOT inherit authority from a matching version string

### Requirement: Production verifier upgrades require compatibility evidence [r[octet.verus_proof_rails.production_upgrade_compatibility]]

Octet MUST bind a production verifier upgrade to an exact published asset, matching Rust closure, package identities, positive and negative Octet proof rails, and explicit registered-consumer compatibility evidence. Package availability MUST NOT be treated as proof compatibility.

#### Scenario: Candidate release passes exact rails

- GIVEN a SHA-256-pinned Verus release and matching Rust toolchain
- WHEN Octet proof rails and a registered consumer's exact proof receipt pass
- THEN the profile MAY select that release and mark that consumer `native`
- AND the row MUST retain a non-transfer rationale

#### Scenario: Adapted corpus source passes

- GIVEN a registered consumer binds an immutable upstream source and a separate compatibility patch identity
- WHEN its native verifier command exits zero without increasing assumptions or trusted markers
- THEN Octet MAY consume that result only as an adapted-source compatibility result
- AND MUST NOT report it as proof of the unmodified upstream revision

#### Scenario: Registered consumer remains blocked

- GIVEN the candidate package builds but a registered consumer proof fails
- WHEN compatibility rows are exported
- THEN that consumer MUST remain `local-pin`, `unsupported`, or another truthful non-native status
- AND the package build MUST NOT authorize downstream promotion

### Requirement: OnixOS May 17 admission follows local candidate execution [r[octet.verus_proof_rails.onixos_may17_compatibility]]

Octet MUST promote the `onixos-lifecycle` compatibility row only from an OnixOS-owned candidate receipt that binds the exact provider revision, source, package identities, execution transcript, and partial-chain non-claims.

#### Scenario: Candidate execution authorizes the row update

- GIVEN the OnixOS candidate receipt reports four verified obligations, zero errors, no timeout, and promotion eligibility
- WHEN Octet updates its consumer inventory
- THEN only `onixos-lifecycle` MAY become `native` at May 17
- AND Aspen and unrelated consumers MUST remain unchanged

#### Scenario: Candidate identity or result is invalid

- GIVEN the receipt hash, provider revision, result count, timeout status, or consumer-row boundary differs
- WHEN Octet evaluates admission
- THEN the OnixOS row MUST remain `local-pin`
- AND package availability MUST NOT substitute for downstream execution

#### Scenario: Authority does not transfer

- GIVEN Octet publishes a native compatibility row
- WHEN OnixOS consumes it
- THEN OnixOS MUST retain proof-source and receipt authority
- AND the row MUST NOT claim verifier soundness, model completeness, physical readiness, whole-chain completion, or release eligibility

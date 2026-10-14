# Function Address Export Specification

## Purpose

Defines Octet's optional Rust source-shape producer for content-addressed function inventories that downstream Kamacite, Valence, Mantle, and Cairn layers can consume without re-parsing Rust. The rail supports gradual adoption from observe-only inventory to warning diagnostics and deny-mode enforcement.

## Requirements

### Requirement: Function address receipt contract

r[octet.function_address_export.receipt_contract] Octet MUST emit a deterministic function-address receipt for each configured checked Rust scope. The receipt MUST bind producer identity, canonicalizer identity, source-envelope identities, function locators, source spans, function kinds, BLAKE3 function addresses, source-token dependency identities, caveats, artifact identity, and explicit non-claims.

#### Scenario: Receipt contains every supported function
r[octet.function_address_export.receipt_contract.every_supported_function]
- GIVEN a configured Rust scope contains supported free functions, inherent methods, trait methods with bodies, async functions, and const functions
- WHEN Octet generates a function-address receipt
- THEN every supported function MUST appear exactly once with a function locator, span, function kind, source-envelope identity, and BLAKE3 function address.

#### Scenario: Receipt binds producer and canonicalizer
r[octet.function_address_export.receipt_contract.producer_identity]
- GIVEN Octet emits a function-address receipt
- WHEN the receipt is inspected
- THEN it MUST identify the Octet producer, canonicalizer profile, parser/toolchain identity, schema version, and address-domain string used to compute each BLAKE3 address.

### Requirement: Function-address paths are workspace-relative

r[octet.function_address_export.portable_locators.relative_paths] Octet MUST emit function-address source-envelope and record paths as slash-normalized members relative to the Cargo workspace root.

#### Scenario: Relocated checkout has stable paths
r[octet.function_address_export.portable_locators.positive]
- GIVEN the same Rust source tree is checked in two different filesystem roots
- WHEN function-address export runs in both roots
- THEN emitted source paths and locators MUST match without embedding either absolute root.

### Requirement: Locators exclude host-specific prefixes

r[octet.function_address_export.portable_locators.locator] Octet MUST derive function locators from normalized relative source path and function display name, not from absolute host paths.

#### Scenario: Duplicate normalized locator fails closed
r[octet.function_address_export.portable_locators.negative]
- GIVEN two function records normalize to the same relative path and function name
- WHEN Octet builds the function-address report
- THEN Octet MUST report a duplicate-locator diagnostic before downstream receipts consume the report.

### Requirement: Absolute baseline compatibility is bounded

r[octet.function_address_export.portable_locators.baseline_compat] Octet MAY accept legacy absolute-path baselines only when each path resolves under the current workspace root and can be normalized to the new relative form.

#### Scenario: Escaping baseline is rejected
r[octet.function_address_export.portable_locators.validation]
- GIVEN a baseline entry contains an absolute path outside the workspace or a `..` escape
- WHEN function-address checking loads the baseline
- THEN Octet MUST reject the entry with a deterministic path-boundary diagnostic.

### Requirement: Function-address records carry byte spans

r[octet.function_address_export.source_spans.contract] Octet MUST include zero-based `start_byte` and `end_byte` offsets for each exported function-address record.

#### Scenario: Span points into source envelope
r[octet.function_address_export.source_spans.positive]
- GIVEN a function-address record references a source envelope
- WHEN downstream tooling reads the record
- THEN `start_byte..end_byte` MUST slice the referenced source envelope bytes for the scanner-selected function item.

### Requirement: Span validation fails closed

r[octet.function_address_export.source_spans.validation] Octet MUST validate exported spans against source envelope bytes before treating the report as a reviewed baseline or downstream receipt input.

#### Scenario: Bad span is rejected
r[octet.function_address_export.source_spans.negative]
- GIVEN a function-address record has an out-of-range, inverted, stale-source, or non-matching span
- WHEN span validation runs
- THEN Octet MUST emit a deterministic diagnostic naming the invalid record and field.

### Requirement: Byte spans do not imply Rust semantics

r[octet.function_address_export.source_spans.export] Octet MUST state that function-address spans describe source scanner identity only and do not claim HIR, MIR, compiler, or behavioral equivalence.

#### Scenario: Downstream smoke consumes spans directly
r[octet.function_address_export.source_spans.final_validation]
- GIVEN Octet emits function-address records with byte spans
- WHEN Kamacite and Valence build function-address receipts
- THEN they SHOULD consume the exported spans directly without recomputing source offsets.

### Requirement: Dependency identity kind is explicit

r[octet.function_address_export.dependency_boundary.kind] Octet MUST label function-address dependency identity with a kind that distinguishes source-token dependency sets from resolved semantic dependency closure.

#### Scenario: Token-set identity is visible
r[octet.function_address_export.dependency_boundary.fixtures]
- GIVEN a function-address report records dependency identity for a Rust function
- WHEN an operator inspects the record
- THEN the record MUST identify the dependency evidence as source-token/source-shape identity only.

### Requirement: Existing dependency hashes remain compatible

r[octet.function_address_export.dependency_boundary.compat] Octet SHOULD continue accepting existing `dependency_set_hash` baselines while adding explicit dependency identity kind metadata.

#### Scenario: Existing baseline loads
r[octet.function_address_export.dependency_boundary.validation]
- GIVEN a baseline has the existing dependency hash fields and no forbidden overclaim text
- WHEN Octet loads the baseline
- THEN Octet SHOULD accept it and render the explicit token-set dependency identity in the next report.

### Requirement: Dependency closure overclaims fail closed

r[octet.function_address_export.dependency_boundary.overclaim] Octet MUST reject function-address baseline or projection text that claims source-token dependency identity proves resolved call graph closure, Rust semantic dependencies, whole-program safety, or release eligibility.

#### Scenario: Overclaim is rejected
r[octet.function_address_export.dependency_boundary.docs]
- GIVEN a baseline or projection describes the dependency hash as complete semantic closure
- WHEN function-address validation runs
- THEN Octet MUST fail with a deterministic overclaim diagnostic.

### Requirement: Octet emits a complete Preserves handoff envelope

r[octet.function_address_export.preserves_handoff.contract] Octet MUST emit or expose all function-address handoff fields required for Kamacite to build a canonical Preserves receipt, including producer metadata, source envelope identity, repo-relative path, byte spans, function address, dependency identity kind, and non-claims.

#### Scenario: Kamacite encodes without reparsing Rust
r[octet.function_address_export.preserves_handoff.positive]
- GIVEN Octet has collected function-address records for a Rust workspace
- WHEN Kamacite builds a Preserves-backed function-address receipt from the Octet handoff
- THEN Kamacite MUST NOT need to reparse Rust source or recompute source spans to encode the receipt.

### Requirement: JSON is compatibility projection

r[octet.function_address_export.preserves_handoff.json_projection] Octet MUST document `function-addresses.json` as an operator/debug compatibility projection, not the canonical portable stack receipt.

#### Scenario: JSON projection keeps non-claims
r[octet.function_address_export.preserves_handoff.validation]
- GIVEN Octet emits JSON and handoff data for the same function-address run
- WHEN an operator inspects either output
- THEN both outputs MUST preserve the source-shape-only non-claim boundary.

### Requirement: Invalid handoff fails closed

r[octet.function_address_export.preserves_handoff.core] Octet MUST validate handoff completeness before downstream receipt promotion.

#### Scenario: Incomplete handoff is rejected
r[octet.function_address_export.preserves_handoff.negative]
- GIVEN a function-address record lacks a required span, uses an absolute path, omits dependency identity kind, has malformed BLAKE3, declares unsupported producer metadata, or includes forbidden overclaim text
- WHEN handoff validation runs
- THEN Octet MUST report deterministic diagnostics and MUST NOT promote the record as Preserves-ready.

### Requirement: Canonical function records

r[octet.function_address_export.canonicalization] Octet MUST compute function addresses from typed canonical function records rather than raw source text. The canonical record MUST include the reviewed function facts admitted by the canonicalizer and MUST treat display paths, labels, comments, and formatting as metadata unless policy explicitly includes them.

#### Scenario: Formatting-only churn preserves address
r[octet.function_address_export.canonicalization.formatting_stable]
- GIVEN two supported functions have equivalent canonical facts but different whitespace or comments
- WHEN Octet computes their function addresses
- THEN the function addresses MUST match.

#### Scenario: Function content changes alter address
r[octet.function_address_export.canonicalization.content_changes]
- GIVEN a supported function's canonical signature, admitted attributes, body facts, or source-envelope identity changes
- WHEN Octet computes its function address
- THEN the function address MUST change or the checker MUST fail closed with a canonicalization diagnostic.

### Requirement: Checked function scope

r[octet.function_address_export.scope] Octet MUST define the checked function universe for each enabled run and MUST reject unsupported, generated, macro-expanded, or excluded functions in enforcing modes unless the receipt carries a reviewed exclusion or caveat that keeps the all-supported-functions claim bounded.

#### Scenario: Unsupported syntax fails without exclusion
r[octet.function_address_export.scope.unsupported_fails]
- GIVEN a configured checked scope contains a function the canonicalizer cannot represent
- WHEN function-address verification runs in an enforcing mode without an explicit exclusion
- THEN verification MUST fail with a diagnostic naming the unsupported function locator.

#### Scenario: Reviewed exclusion is visible
r[octet.function_address_export.scope.reviewed_exclusion]
- GIVEN a generated or macro-heavy function is outside the supported universe by reviewed policy
- WHEN Octet emits the receipt
- THEN the receipt MUST include an exclusion or caveat with locator, reason, source identity, and non-claim text rather than silently omitting the function.

### Requirement: Gradual adoption modes

r[octet.function_address_export.adoption_modes] Octet MUST expose function-address adoption modes that let projects move from no enforcement to full enforcement without a flag day. Supported modes SHOULD include `off`, `observe`, `warn`, and `deny`, and the rail SHOULD support scoped rollout and ratchet policy for configured crates, packages, modules, paths, or changed functions.

#### Scenario: Observe mode records without failing
r[octet.function_address_export.adoption_modes.observe]
- GIVEN function-address mode is `observe`
- WHEN Octet checks a configured scope with missing, stale, or unsupported function-address evidence
- THEN Octet SHOULD emit inventory and baseline diagnostics without failing the run solely because function-address coverage is incomplete.

### Requirement: Observe mode emits artifacts with advisory debt

r[octet.function_address_export.observe_artifact_valid.core] Octet MUST emit a function-address artifact in observe mode when Cargo checking succeeds, even when functions lack reviewed markers or baseline entries.

#### Scenario: Unbaselined observe writes report
r[octet.function_address_export.observe_artifact_valid.positive]
- GIVEN a Rust workspace has functions and no function-address baseline
- WHEN `cargo octet check --function-addresses=observe` runs successfully
- THEN Octet MUST write `function-addresses.json` and MUST NOT fail because the missing-review findings are malformed.

### Requirement: Observe findings use the accepted output domain

r[octet.function_address_export.observe_artifact_valid.findings] Octet MUST encode observe-mode function-address findings using a severity, group, path, and identity shape accepted by the runner output validator.

#### Scenario: Advisory finding validates
r[octet.function_address_export.observe_artifact_valid.validation]
- GIVEN observe mode reports missing reviewed function-address coverage
- WHEN structured output and artifact-bundle validation run
- THEN every function-address finding MUST pass the output-domain validator and the artifact bundle MUST remain consumable.

#### Scenario: Warn mode emits lint-like diagnostics
r[octet.function_address_export.adoption_modes.warn]
- GIVEN function-address mode is `warn`
- WHEN Octet finds missing, stale, unsupported, or duplicate function-address evidence
- THEN Octet SHOULD emit lint-like warning diagnostics with stable lint IDs and SHOULD preserve Octet's warning-only exit behavior.

#### Scenario: Deny mode fails closed
r[octet.function_address_export.adoption_modes.deny]
- GIVEN function-address mode is `deny`
- WHEN Octet finds missing, stale, unsupported, duplicate, malformed, or overclaiming function-address evidence in the configured scope
- THEN Octet MUST fail the check with deterministic diagnostics.

#### Scenario: Ratchet mode blocks new debt
r[octet.function_address_export.adoption_modes.ratchet]
- GIVEN a baseline records reviewed existing function-address debt
- WHEN a new function is added or a touched function becomes missing, stale, unsupported, or malformed
- THEN Octet SHOULD fail configured ratchet policy while preserving the reviewed baseline debt as explicit context.

### Requirement: Fail-closed verification

r[octet.function_address_export.fail_closed] Octet MUST fail closed when function-address evidence is missing, stale, malformed, duplicated, unsupported, or overclaiming.

#### Scenario: Missing or stale function fails
r[octet.function_address_export.fail_closed.stale]
- GIVEN a checked Rust source changes after a function-address receipt is written
- WHEN Octet verifies the receipt against the current scope
- THEN verification MUST fail with a diagnostic naming the missing or stale function address.

#### Scenario: Duplicate locator fails
r[octet.function_address_export.fail_closed.duplicate]
- GIVEN two receipt entries resolve to the same function locator within one scope
- WHEN Octet verifies the receipt
- THEN verification MUST fail with a duplicate-locator diagnostic.

#### Scenario: Overclaim text fails
r[octet.function_address_export.fail_closed.overclaim]
- GIVEN a receipt claims Rust semantic correctness, compiler correctness, whole-program safety, release eligibility, or verifier soundness
- WHEN Octet verifies the receipt
- THEN verification MUST fail with an overclaim diagnostic.

### Requirement: Positive fixture coverage

r[octet.function_address_export.fixtures.positive] Octet MUST include positive fixtures proving supported function kinds, formatting-stable addresses, dependency-edge recording, source-span preservation, and Kamacite-compatible export.

#### Scenario: Positive fixture suite passes
r[octet.function_address_export.fixtures.positive.suite]
- GIVEN valid supported Rust fixture sources and expected function-address receipts
- WHEN the focused fixture suite runs
- THEN Octet MUST accept the receipts and report deterministic function-address identities.

### Requirement: Negative fixture coverage

r[octet.function_address_export.fixtures.negative] Octet MUST include negative fixtures for missing functions, stale source identities, stale addresses, duplicate locators, unsupported syntax without exclusion, malformed BLAKE3 digests, and overclaim text.

#### Scenario: Negative fixture suite rejects invalid receipts
r[octet.function_address_export.fixtures.negative.suite]
- GIVEN invalid function-address fixtures cover the fail-closed classes
- WHEN the focused fixture suite runs
- THEN each invalid fixture MUST fail with a deterministic diagnostic naming the rejected field or function locator.

### Requirement: Strict modes fail closed on bad baselines

r[octet.function_address_export.observe_artifact_valid.negative] Octet MUST keep warn and deny mode diagnostics for malformed, stale, or overclaiming reviewed baselines.

#### Scenario: Malformed baseline remains diagnostic
r[octet.function_address_export.observe_artifact_valid.docs]
- GIVEN a reviewed function-address baseline contains malformed BLAKE3, stale source identity, or forbidden claim text
- WHEN warn or deny mode checks the baseline
- THEN Octet MUST report deterministic diagnostics without weakening the function-address non-claim boundary.

### Requirement: Kamacite-compatible export

r[octet.function_address_export.kamacite_export] Octet SHOULD emit a Kamacite-compatible function-address handoff artifact only after Octet-owned verification succeeds. The export MUST preserve source-envelope identities, function addresses, dependency identities, caveats, producer roles, and non-claims without treating Octet JSON as the canonical portable receipt or treating Kamacite or Valence as the Rust analyzer.

#### Scenario: Export follows verification
r[octet.function_address_export.kamacite_export.after_verify]
- GIVEN Octet function-address verification succeeds
- WHEN the operator requests the Kamacite-compatible export
- THEN Octet SHOULD write the deterministic export artifact with the verified function inventory and receipt identity.

#### Scenario: Failed verification blocks export
r[octet.function_address_export.kamacite_export.blocks_failed]
- GIVEN function-address verification fails for stale, missing, malformed, unsupported, duplicate, or overclaiming evidence
- WHEN the operator requests the Kamacite-compatible export
- THEN Octet MUST refuse to write the export and report deterministic diagnostics.

### Requirement: Registry and contract ownership

r[octet.function_address_export.registry] Octet MUST register function-address receipt and policy families under the reviewed config/receipt registry with ownership classification, freshness checks, generation commands when applicable, and positive/negative validation fixtures.

#### Scenario: Registry covers receipt family
r[octet.function_address_export.registry.covered]
- GIVEN Octet emits function-address receipts
- WHEN policy or artifact manifest freshness validation runs
- THEN the receipt family MUST be covered as Rust-derived or Nickel-authored according to ownership and stale generated artifacts MUST fail validation.

### Requirement: Documentation and final validation

r[octet.function_address_export.docs] Octet documentation MUST describe the function-address rail as Rust source-shape/content-identity evidence and MUST preserve downstream ownership boundaries. r[octet.function_address_export.final_validation] The change MUST include focused tests, positive and negative fixtures, artifact verification, policy freshness, and Cairn validation before archive.

#### Scenario: Boundary is visible
r[octet.function_address_export.docs.boundary]
- GIVEN an operator reads the function-address documentation or receipt summary
- WHEN the supported claim is described
- THEN it MUST state that Octet produced Rust function inventory/address evidence and did not prove Rust semantic correctness, compiler correctness, whole-program safety, release eligibility, or downstream verifier soundness.

#### Scenario: Final validation is reviewable
r[octet.function_address_export.final_validation.reviewable]
- GIVEN the implementation is complete
- WHEN focused validation runs
- THEN positive fixtures MUST pass, negative fixtures MUST fail closed, policy/artifact freshness MUST pass, Cairn validation MUST pass, and the saved evidence MUST identify the command surfaces used.

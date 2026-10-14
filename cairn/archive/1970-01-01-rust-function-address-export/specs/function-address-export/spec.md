# Function Address Export Specification

## Purpose

Defines Octet's optional Rust source-shape producer for content-addressed function inventories that downstream Kamacite, Valence, Mantle, and Cairn layers can consume without re-parsing Rust. The rail supports gradual adoption from observe-only inventory to warning diagnostics and deny-mode enforcement.

## Requirements

### Requirement: Function address receipt contract

r[octet.function_address_export.receipt_contract] Octet MUST emit a deterministic function-address receipt for each configured checked Rust scope. The receipt MUST bind producer identity, canonicalizer identity, source-envelope identities, function locators, source spans, function kinds, BLAKE3 function addresses, optional dependency-closure identities, caveats, artifact identity, and explicit non-claims.

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

### Requirement: Kamacite-compatible export

r[octet.function_address_export.kamacite_export] Octet SHOULD emit a Kamacite-compatible function-address receipt artifact only after Octet-owned verification succeeds. The export MUST preserve source-envelope identities, function addresses, dependency identities, caveats, producer roles, and non-claims without treating Kamacite or Valence as the Rust analyzer.

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

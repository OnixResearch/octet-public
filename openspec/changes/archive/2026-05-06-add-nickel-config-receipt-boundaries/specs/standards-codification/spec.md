## ADDED Requirements

### Requirement: Config and Receipt Source-of-Truth Registry [r[config-receipt-source-of-truth-registry]]

The system MUST maintain a typed registry for Tiger Style configuration and receipt families before those families add Nickel contracts, generated artifacts, or runtime validation logic.

Each family entry MUST declare a stable family ID, classification (`nickel-authored`, `rust-derived`, or `non-candidate`), owner source path, generated artifact path when applicable, generation command, validation command, status, and exclusion rationale for non-candidates.

#### Scenario: New family declares one source of truth [r[config-receipt-source-of-truth-registry.scenario.declares-one-source]]

- GIVEN a contributor adds a new run-config, policy-config, proof-receipt, deploy-receipt, or evidence-receipt surface
- WHEN the registry validation command runs
- THEN the family has exactly one source-of-truth classification
- AND the entry names the owner source path and concrete validation command
- AND registry validation rejects duplicate family IDs or families with both Nickel-authored and Rust-derived ownership

#### Scenario: Non-candidates are explicit [r[config-receipt-source-of-truth-registry.scenario.non-candidates-explicit]]

- GIVEN a proposed surface contains raw secrets, cryptographic internals, serialized capabilities, wire discriminants, distributed runtime behavior, or hot-path constants
- WHEN the registry is reviewed
- THEN the family is classified as `non-candidate`
- AND the exclusion rationale states why Nickel contracts must not own that surface

### Requirement: Nickel-Authored Human Configuration [r[nickel-authored-human-configuration]]

The system MUST use Nickel-authored contracts for human-maintained Tiger Style configuration that benefits from defaults, docs, merge overlays, environment profiles, fixture matrices, or policy knobs.

Nickel-authored config MUST export deterministic machine formats for tools while keeping helper inputs, derived locals, and non-exportable metadata behind `not_exported` fields.

#### Scenario: Human config exports deterministic tool input [r[nickel-authored-human-configuration.scenario.exports-deterministic-tool-input]]

- GIVEN a Nickel-authored config with defaults, environment overlays, and helper fields
- WHEN the config export command runs
- THEN the exported machine input is deterministic
- AND the exported data omits `not_exported` helper fields
- AND the exported data preserves policy IDs, profile names, enabled checks, timeout/retry policy, and artifact-retention settings

#### Scenario: Invalid human config fails before runtime side effects [r[nickel-authored-human-configuration.scenario.invalid-config-fails-before-effects]]

- GIVEN a Nickel-authored config with an invalid timeout ordering, unknown profile name, invalid checker reference, or impossible retention policy
- WHEN validation runs before standards checks or proof rails execute
- THEN the command exits non-zero before runtime side effects
- AND the diagnostic identifies the invalid field and config family ID

### Requirement: Rust-Derived Receipt Contracts [r[rust-derived-receipt-contracts]]

The system MUST keep runtime-emitted proof, evidence, deploy, and check receipts as Rust-owned serialized DTOs and generate Nickel contracts from those Rust-owned shapes.

Generated receipt contracts MUST preserve Rust serde names, enum alternatives, optionality, schema versions, defaults when known, and closed-record behavior only when Rust declares unknown fields invalid.

#### Scenario: Receipt contract freshness detects drift [r[rust-derived-receipt-contracts.scenario.freshness-detects-drift]]

- GIVEN a Rust receipt DTO changes its serde field names, schema version, enum alternatives, or optionality
- WHEN the generated-contract freshness check runs without updating the Nickel artifact
- THEN the check fails with a diff or diagnostic naming the stale generated contract
- AND the registry still identifies the Rust source path as authoritative

#### Scenario: Receipt fixtures validate shape without owning behavior [r[rust-derived-receipt-contracts.scenario.fixtures-validate-shape-not-behavior]]

- GIVEN positive and negative receipt JSON fixtures for a Rust-derived receipt family
- WHEN Nickel contract validation runs against those fixtures
- THEN valid fixtures satisfy the generated contract
- AND invalid fixtures fail only for Rust-owned shape, schema-version, required-field, or explicitly modeled invariant violations
- AND Nickel validation does not claim to prove distributed behavior, cryptographic correctness, or runtime transition semantics

### Requirement: Structured Acceptance Receipts [r[structured-acceptance-receipts]]

The system MUST treat structured receipts, not raw logs, as acceptance records for proof/evidence/config review.

Acceptance receipts MUST include schema version, run or receipt ID, timestamps, tool/version or git revision, config digest for Nickel-authored inputs, inputs, stage outcomes, artifacts, status, warnings/errors, and verification commands or evidence references when applicable.

#### Scenario: Receipt binds emitted evidence to config digest [r[structured-acceptance-receipts.scenario.binds-evidence-to-config-digest]]

- GIVEN a standards proof rail or artifact-backed check consumes Nickel-authored config
- WHEN the receipt is emitted
- THEN the receipt records the config digest or exported config artifact identity
- AND the receipt records generated artifact paths or evidence references without requiring log scraping

#### Scenario: Raw logs remain debug aids [r[structured-acceptance-receipts.scenario.raw-logs-debug-only]]

- GIVEN a command emits human-readable logs and a structured receipt
- WHEN acceptance evidence is reviewed
- THEN the structured receipt is the acceptance record
- AND raw logs may supplement debugging but cannot be the only proof that required receipt fields were emitted

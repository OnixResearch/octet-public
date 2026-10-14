## Context

Current Tiger Style standards work already has several adjacent surfaces: Nickel-authored policy registries, generated policy artifacts, proof-bundle schemas, proof receipts, closure receipts, and operator-facing verification evidence. The next config/receipt expansion needs a clear ownership boundary before more schemas are added.

## Goals / Non-Goals

**Goals:**
- Make Nickel the source of truth for human-authored modular configuration, defaults, docs, overlays, and semantic config validation.
- Keep Rust as the source of truth for runtime-emitted receipt DTOs, serialization shape, schema versions, and compatibility-sensitive outputs.
- Require a registry entry for every config/receipt family before implementation chooses Nickel-authored or Rust-derived ownership.
- Define deterministic validation gates that catch generated-contract drift and invalid fixture data.

**Non-Goals:**
- Do not move raw secrets, private keys, bearer tokens, cryptographic internals, serialized capabilities, wire discriminants, distributed behavior, or hot-path runtime constants into Nickel.
- Do not treat raw logs as acceptance receipts.
- Do not make Nickel stricter than Rust for Rust-derived DTOs unless Rust metadata explicitly declares that invariant.

## Decisions

### 1. Registry-first source-of-truth classification

**Choice:** Add a typed config/receipt family registry with `nickel-authored`, `rust-derived`, and `non-candidate` classifications.

**Rationale:** The registry prevents duplicate sources of truth and gives reviewers one place to inspect owner module, artifact path, generation command, validation command, and exclusion rationale.

**Alternative:** Add Nickel files as each need appears. Rejected because it encourages hand-written schemas for Rust-owned receipts and leaves future maintainers guessing which side owns compatibility.

**Implementation:** Extend the standards policy/schema surface with family entries that include stable ID, owner, source path, artifact path, status, generation command, validation command, and prior-art/exclusion notes.

### 2. Nickel-authored human configuration

**Choice:** Human-authored run, policy, fixture, and profile configuration MUST be written as Nickel contracts with defaults and `not_exported` helpers, then exported to JSON/TOML/YAML for tools.

**Rationale:** Nickel's merge model, field contracts, defaults, documentation, and deterministic export are the useful part for operator-authored config. Generated formats remain tool input only.

**Alternative:** Keep authoring TOML/JSON and validate in Rust. Rejected because it loses merge/default metadata and tends to duplicate validation between docs and code.

### 3. Rust-derived receipt contracts

**Choice:** Runtime-emitted receipts remain Rust-owned DTOs; Nickel contracts are generated validators for external fixtures, docs, and artifact-backed review.

**Rationale:** Rust owns serde names, enum alternatives, optionality, schema versions, ordering-sensitive serialization tests, and compatibility behavior. Nickel should validate shape/invariants without becoming a second live behavior source.

**Alternative:** Hand-author receipt contracts in Nickel. Rejected because hand schemas drift from Rust serialization and can accidentally make compatibility promises Rust does not enforce.

## Risks / Trade-offs

**Generator drift** → Mitigate with `--check` freshness gates and generated artifact diff failures.

**Over-strict Nickel fixtures** → Mitigate by tying negative fixtures to Rust metadata such as schema versioning or closed DTO declarations.

**Receipt/log confusion** → Mitigate by requiring acceptance records to be structured receipts with schema/version/config digest fields; raw logs remain debug aids only.

**Secret exposure** → Mitigate by validating secret references, handles, and capability names only; raw secret values are explicitly non-candidates.

## Validation Plan

- Typecheck the registry and every touched Nickel contract.
- Export representative Nickel-authored config and assert deterministic machine input shape.
- Run generated-contract freshness checks for Rust-derived receipt contracts.
- Validate positive and negative Nickel fixtures for config and receipt families.
- Run Rust serde/schema round-trip tests for Rust-derived receipts.
- Run OpenSpec strict validation and `git diff --check` before implementation commits.

# Function Address Export Specification

## Purpose

Prepare Octet function-address output for Kamacite-owned canonical Preserves receipts.

## Requirements

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

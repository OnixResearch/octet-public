## Context

The generated artifact seam intentionally lets downstream consumers avoid evaluating Nickel directly. Prior hardening added a versioned proof-bundle artifact, but a consumer still trusts sibling files independently. If one generated file is edited without regenerating the full bundle, the checker can combine registry data from one generation with check/proof metadata from another.

## Goals / Non-Goals

**Goals:**
- Emit a deterministic manifest next to generated artifacts.
- Include schema ID, source registry schema, artifact filename, BLAKE3 digest, and byte length.
- Validate the manifest before `--policy-artifacts-dir` proceeds.
- Report operator-readable diagnostics for missing, unsupported, stale, or tampered artifacts.

**Non-Goals:**
- Cryptographic signing, remote transparency logs, or key management.
- Changing the default read-only behavior of `check`.
- Requiring downstream consumers to evaluate Nickel.

## Decisions

### 1. Manifest file next to generated artifacts

**Choice:** write `tigerstyle-artifact-manifest.json` with schema `tigerstyle-artifact-manifest/v1`.

**Rationale:** the artifact directory is already the downstream handoff unit. A colocated manifest makes drift visible without adding an external service.

**Alternative:** embed hashes in each artifact. Rejected because consumers need a single root to validate the complete set before mixing artifacts.

### 2. Hash exact serialized bytes

**Choice:** hash the exact UTF-8 file contents written by the exporter, including the final newline, and record byte length.

**Rationale:** exact byte binding catches whitespace-only local edits and stale rewrites while remaining deterministic.

**Alternative:** hash parsed JSON/TOML structures. Rejected because TOML/JSON canonicalization differences would make partial edits harder to diagnose.

### 3. Validate before policy checks

**Choice:** `tigerstyle-standards check --policy-artifacts-dir` loads the manifest and validates every expected generated artifact before running standards checks.

**Rationale:** fail fast before consuming inconsistent policy/proof data.

**Alternative:** validate only proof bundles. Rejected because policy and checker config artifacts can drift too.

## Risks / Trade-offs

**Manifest can be regenerated with malicious artifacts.** This slice detects local drift/tampering, not provenance. A later signing slice can bind the manifest to a trusted producer.

**Manifest schema churn.** Version the schema and reject unsupported versions so future changes are explicit.

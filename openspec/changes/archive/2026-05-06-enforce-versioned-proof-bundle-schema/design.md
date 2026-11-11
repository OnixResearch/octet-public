## Context

The current verified proof-rail checker has enough metadata to validate rich receipts, including `proof-receipt-schema`, runner/environment, command/source BLAKE3 bindings, verifier binary, toolchain, build closure, Nix derivation, recursive closure, and Nix closure graph digests. The generated artifact seam exposes much of that as flattened `tigerstyle-check-config.toml` arrays plus the full policy JSON, but there is no small versioned proof-bundle artifact that downstream consumers can validate independently.

## Goals / Non-Goals

**Goals:**
- Define a stable proof-bundle schema version and generated file name.
- Keep Nickel as the authoring contract and Rust as the emitted/validated runtime representation.
- Reject missing or unsupported proof-bundle schemas before trusting artifact-backed proof receipts.
- Preserve registry-backed proof checks and existing receipt-line compatibility.

**Non-Goals:**
- Replace the existing line-oriented proof receipt format.
- Execute external proof tools from generated artifacts beyond the already-supported `--execute-proof-rails` path.
- Add cryptographic signing; this slice is schema/version enforcement plus deterministic content binding.

## Decisions

### 1. Add a JSON proof-bundle artifact

**Choice:** Export `tigerstyle-proof-bundles.json` with `schema_version = "tigerstyle-proof-bundle/v1"`, `generated_from`, `registry_schema_version`, and one deterministic entry per verified proof rail.

**Rationale:** JSON preserves nested proof-rail relationships and avoids lossy parallel TOML arrays. Keeping a separate file gives CI/downstream scripts a narrow schema gate without parsing the full registry.

**Alternative:** Only add more TOML arrays. Rejected because arrays cannot reliably express per-rail grouping and versioned bundle validation.

**Implementation:** Add Rust `ProofBundleArtifact`/`ProofBundleEntry` models in `artifacts.rs`, export the file from the artifact writer, and load/validate it when `--policy-artifacts-dir` is used.

### 2. Validate schema before proof diagnostics

**Choice:** Artifact-backed checks must fail with a standards diagnostic when `tigerstyle-proof-bundles.json` is missing, has an unsupported schema version, or omits a verified rail declared by the policy artifact.

**Rationale:** A stale or partial proof bundle should not silently fall back to weaker registry data. The failure needs to look like a standards gate failure, not a serde panic.

**Alternative:** Let JSON deserialization errors bubble up as CLI integration errors. Rejected because consumers need a policy-level remediation path.

**Implementation:** Add artifact-loader validation and fixture tests for missing/unsupported schema version while preserving existing hard failure exit behavior.

### 3. Keep receipt validation line-compatible

**Choice:** The proof-bundle artifact records expected receipt lines and binding inputs; the existing checker still accepts proof receipts as text files.

**Rationale:** The current receipt evidence corpus and closure graph digest tests remain valid. The bundle hardens the expected-contract side, not the receipt serialization itself.

**Alternative:** Migrate receipts to JSON in the same change. Rejected as too broad and likely to invalidate current archived evidence.

## Risks / Trade-offs

- **Duplicate proof metadata:** The full policy JSON and proof-bundle JSON both carry proof-rail data. Mitigation: generate both from one Rust model and add a freshness/round-trip test.
- **Schema churn:** Future proof fields need explicit versioning. Mitigation: reject unsupported schema versions and keep v1 fields additive where possible.
- **Downstream breakage:** Existing consumers may not expect the extra file. Mitigation: additive export file; artifact-backed checker validates it only when using policy-artifacts-dir and the bundle is part of this repo's generated contract.

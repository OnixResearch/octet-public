## Context

`cargo-octet` currently discovers files, extracts function spans, computes `fingerprint::normalize_tokens(function_source)`, computes dependency facts, and then passes those facts into `valence_core::function_object::build_function_object`. The core owns the declared `NORMALIZATION_ALGORITHM` and canonical object hash construction, but the API accepts normalized source as an untyped string and returns `Result<_, String>`. The hash payload is assembled inline with `serde_json::json!`.

## Decisions

### 1. Put normalization invariants at the core boundary

**Choice:** `valence-core` MUST either compute the v1 normalized function source from raw source itself or accept a typed `NormalizedFunctionSource` value that carries the normalization algorithm/version and cannot be paired with a different algorithm silently.

**Rationale:** The crate declaring the normalization algorithm should also guard the invariant that normalized bytes match that algorithm.

### 2. Preserve serialized v1 compatibility unless explicitly migrated

**Choice:** Internal Rust API types MAY become enums/newtypes, but the serialized Function Object v1 DTO, schema name, and object hash material MUST remain compatible unless the change includes fixture refreshes and explicit migration notes.

**Rationale:** Function Object v1 is already consumed by corpus receipts, bundles, supplements, and evidence gates. Tightening the seam should not churn identities accidentally.

### 3. Make hash material reviewable

**Choice:** Object hashing MUST use a dedicated `FunctionObjectHashPayload` struct or equivalent typed helper instead of an ad hoc inline JSON object.

**Rationale:** Reviewers need to see exactly which fields are included in the BLAKE3 identity and which display/source-local fields are excluded.

### 4. Type fail-closed conditions

**Choice:** Core construction errors, dependency identity statuses, effect caveat classifications, macro caveat classifications, and pure-cache blockers SHOULD be represented by typed enums/newtypes at the Rust boundary while serializing to the existing stable strings.

**Rationale:** Typed variants make negative tests exact and reduce accidental status spelling drift.

### 5. Keep the shell thin

**Choice:** `cargo-octet` continues to own filesystem reads, source discovery, marker mutation, corpus orchestration, supplement orchestration, stdout/stderr, and process exits. Its Function Object path should prepare scanner facts and delegate validation/hash construction to `valence-core`.

**Rationale:** This preserves the functional core / imperative shell split that the extraction seam is meant to establish.

### 6. Canonical hash payload is byte-stable by construction

**Choice:** `FunctionObjectHashPayload` is a `#[derive(Serialize)]` struct whose fields are declared in the historical inline `serde_json::json!` insertion order. Canonical JSON is produced by `serde_json::to_value(&payload)` followed by `serde_json::to_string(&value)`, not by direct `serde_json::to_string(&payload)`.

**Rationale:** `serde_json` is built without the `preserve_order` feature in this workspace (the only `indexmap` consumer is `toml`), so `serde_json::Map` is a `BTreeMap` and `to_value` serializes object keys in sorted order at every nesting level — exactly what the prior inline `json!` payload produced. Routing the typed struct through `to_value` reproduces that sorted-key byte stream, so existing object hashes are unchanged. Declaring fields in the historical insertion order also keeps the payload byte-identical if `preserve_order` is ever enabled later. The `legacy_inline_json_payload_matches_new_canonical_json` test reconstructs the prior inline payload and asserts byte-identical canonical JSON as a permanent guard.

### 7. Typed enums at the boundary, stable strings at the DTO edge

**Choice:** Typed enums (`NormalizedFunctionSource`, `ConstructionError`, `DependencyIdentityStatus`, `EffectCaveatKind`, `EffectCaveatStatus`, `MacroCaveatStatus`, `PureCacheBlocker`) live at the `FunctionObjectInput` boundary and in internal construction functions. The serialized DTO structs (`DependencyCaveat`, `EffectCaveat`, `MacroCaveat`, `CaveatSummary`, `PureCacheEligibility`) keep `&'static str` fields, populated via the typed enums' `as_str()`.

**Rationale:** This gives a typed, fail-closed construction boundary (mismatched normalization algorithm, unknown dependency status, and serialization failures all return `ConstructionError` before any object hash is emitted) while keeping the serialized Function Object v1 DTO byte-identical and avoiding churn at downstream string-comparison call sites (`corpus_caveat_counts`, the supplement boundary helpers, and the existing `unknown_effects_do_not_claim_purity` assertion).

## Risks / Trade-offs

- If normalization moves into `valence-core`, `cargo-octet` must stop duplicating the same normalization path or tests must prove both routes agree during transition.
- More typed DTOs add code, but they make the v1 compatibility contract cheaper to review.
- Any accidental object-hash churn is high impact; compatibility tests must run before accepting implementation.

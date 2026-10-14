## Context

Octet already has multiple enforcement surfaces: Dylint lints in `src/`, `cargo-octet` as the consumer runner, `dylint.toml` configuration, and `octet-standards` policy/source-shape checks. Mantle should not link to Octet's rustc-private lint crate or duplicate Octet's policy logic. Instead, Octet should publish a provider manifest that downstream build systems can treat as a compiler-policy input.

## Approach

1. Define a closed provider manifest schema. The manifest records schema version, provider name, profile name, compatible rustc identity, adapter kind, driver identity, lint library identity, config identity, standards policy identity when selected, invocation environment, waiver contract, and non-claim text.
2. Export the manifest through an Octet-owned command or Nix app. The export should be deterministic and suitable for BLAKE3 hashing by downstream receipts.
3. Define at least one artifact-acceptance profile. The first profile should support strict downstream builds by promoting configured hard lints to deny/error behavior, disallowing hidden `disabled_lints` for required lints, and requiring structured waivers for accepted exceptions.
4. Keep compiler-visible and source-shape checks distinct. The provider manifest may reference both the Dylint adapter material and an `octet-standards` policy artifact, but it should not imply that standards checks are rustc lints.
5. Make waivers reviewable. Octet owns the syntax and validation of `octet-allow` / legacy-compatible suppression comments, required fields, narrow scope, and reportable waiver summaries.
6. Publish bounded claim text. The manifest should state that downstream systems may claim compliance with the named Octet profile for the selected source scope, while correctness, complete FCIS architecture, and general safety remain non-claims unless separate evidence exists.

## Manifest Sketch

```json
{
  "schema": "octet-compiler-policy-provider/v1",
  "provider": "octet",
  "profile": "artifact-required",
  "adapter_kind": "dylint-rustc-driver",
  "compatible_rustc": {
    "release": "1.xx.x-nightly",
    "commit_hash": "..."
  },
  "driver": {
    "path": "/nix/store/.../bin/octet-dylint-driver",
    "blake3": "..."
  },
  "lint_library": {
    "path": "/nix/store/.../liboctet@toolchain.so",
    "blake3": "..."
  },
  "config": {
    "path": "dylint.toml",
    "blake3": "..."
  },
  "standards_policy": {
    "path": "octet-policies.json",
    "blake3": "..."
  },
  "required_environment": {
    "DYLINT_NO_DEPS": "1"
  },
  "waiver_contract": {
    "syntax": "octet-allow",
    "required_fields": ["policy", "owner", "reason", "review-trigger", "safer-default"]
  },
  "claims": ["configured-profile-compliance"],
  "non_claims": ["program-correctness", "complete-fcis-proof", "general-safety"]
}
```

The final representation may be JSON generated from Nickel-owned contracts. The important boundary is that Octet owns the manifest content and Mantle only consumes and receipts it.

## Risks

- Toolchain skew can make Dylint artifacts unusable. The manifest must carry enough rustc identity for downstream fail-closed checks.
- A profile that allows arbitrary `disabled_lints` would be too weak for artifact acceptance. Required profiles should forbid disabling required lints except through structured waivers.
- Standards checks are not compiler lints. The manifest and docs must preserve that distinction.
- Waiver syntax must be strict enough for receipts but compatible enough to migrate existing Octet/Slotcar suppressions.

## Validation

- Contract tests accept a complete provider manifest and reject missing driver, missing lint library, missing toolchain identity, malformed BLAKE3, hidden required-lint disablement, and missing waiver metadata.
- Runner tests prove the manifest points at runnable Dylint adapter material for the selected toolchain.
- Standards tests prove FCIS source-shape policy artifacts can be referenced and verified separately from Dylint lint execution.
- Downstream fixture tests prove a Mantle-style consumer can hash the provider manifest and distinguish compliant, waived, and noncompliant code.

let Upstream = import "upstream-default.ncl" in
let OctetTraceabilityProfile = {
  id = "octet-default",
  requirement_sources = [
    { root = ".cairn/specs/evidence-provenance/spec.md", source_kind = "accepted_specs", extensions = ["md"] },
    { root = ".cairn/specs/effect-closure-admission/spec.md", source_kind = "accepted_specs", extensions = ["md"] },
    { root = ".cairn/specs/cob-collaboration-adoption/spec.md", source_kind = "accepted_specs", extensions = ["md"] },
    { root = ".cairn/specs/cob-collaboration-conflict-recovery/spec.md", source_kind = "accepted_specs", extensions = ["md"] },
    { root = ".cairn/specs/runner-self-check/spec.md", source_kind = "accepted_specs", extensions = ["md"] },
  ],
  evidence_sources = [
    { root = "cargo-octet/src/baseline.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/engine.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/lib.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/main.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/admit.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/artifact.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/cli.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/effect_closure.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/evidence.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/evidence_chain.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/function_address.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/formal_proof_valence_export_fixture.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "cargo-octet/src/preserves_receipt_export.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "crates/octet-standards/src/runner.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "crates/valence-core/src/evidence_codebase_index.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "src/safety/no_todo.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "src/safety/public_unsafe_api.rs", source_kind = "rust_source", extensions = ["rs"] },
    { root = "tests", source_kind = "rust_source", extensions = ["rs"] },
    { root = "README.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/effect-closure-admission.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/formal-proof-valence-export-fixture.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/preserves-receipt-export.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/self-check-warning-budget.md", source_kind = "tool_source", extensions = ["md"] },
    { root = "docs/trust-boundary-evidence.md", source_kind = "tool_source", extensions = ["md"] },
  ],
  marker_verbs = ["impl", "verify", "depends", "related"],
} in
let OctetReceiptSchema = {
  command = "cargo octet artifact verify --emit-valence-provenance",
  top_level_keys = ["artifact_dir", "artifacts", "diagnostics", "non_claims", "proof_cards", "provenance_records", "schema_version", "typed_edges", "verification_status"],
  artifact_keys = ["artifact", "blake3", "checked_scope", "checker_id", "edge_kind", "evidence_tier", "failure_reason", "input_artifacts", "non_claims", "normalized_invocation", "output_artifacts", "parent_artifacts", "policy_or_checker_version", "producer_id", "schema_version", "source_artifact", "source_id", "status", "target_artifact", "tool_identity"],
  issue_keys = ["code", "message"],
  hash_fields = [],
} in
Upstream & {
  name | force = "octet-default",
  policy_schema_compatibility | force = Upstream.policy_schema_compatibility & {
    refresh_command | force = "nix run path:$CAIRN_SOURCE#cairn -- policy export --source cairn-policy/default.ncl --output cairn-policy/generated/cairn-policy.json",
  },
  traceability_policy | force = {
    default_profile = "octet-default",
    profiles = [
      OctetTraceabilityProfile,
      OctetTraceabilityProfile & { id | force = "cairn-default" },
    ],
  },
  receipt_schemas | force = Upstream.receipt_schemas @ [OctetReceiptSchema],
}

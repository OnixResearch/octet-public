# Structured receipt contracts for Rust-emitted acceptance evidence.

let NonEmptyString = std.contract.from_predicate (fun value => std.string.length value > 0) in
let Status = std.contract.from_predicate (fun value => value == "passed" || value == "failed" || value == "blocked") in
let DigestAlgorithm = std.contract.from_predicate (fun value => value == "blake3") in
let Digest = {
  algorithm | DigestAlgorithm,
  value | NonEmptyString,
} in
let Stage = {
  name | NonEmptyString,
  status | Status,
} in
let Input = {
  path | NonEmptyString,
  blake3 | NonEmptyString,
} in
let Artifact = {
  path | NonEmptyString,
  blake3 | NonEmptyString,
  bytes | Number,
} in
let VerificationCommand = {
  command | NonEmptyString,
  status | Status,
} in
let ReceiptEvidence = {
  path | NonEmptyString,
  blake3 | NonEmptyString,
} in
let EvidenceTier = std.contract.from_predicate (fun value =>
  value == "static_lint"
  || value == "policy_check"
  || value == "ui_fixture"
  || value == "autofix_verified"
  || value == "transcript_assertion"
  || value == "formal_proof"
  || value == "manual_review_required"
) in
let ProvenanceRecord = {
  schema_version | std.contract.from_predicate (fun value => value == "octet-provenance-record/v1"),
  source_id | NonEmptyString,
  checker_id | NonEmptyString,
  evidence_tier | EvidenceTier,
  artifact_locator | NonEmptyString,
  blake3 | NonEmptyString,
  checked_scope | NonEmptyString,
  verifier_identity | String | default = "",
  proof_artifact_identity | String | default = "",
  human_action | String | default = "",
  parent_artifacts | Array String | default = [],
} in
{
  AcceptanceReceipt = {
    schema_version | std.contract.from_predicate (fun value => value == "octet-acceptance-receipt/v1"),
    run_id | NonEmptyString,
    started_at | NonEmptyString,
    finished_at | NonEmptyString,
    tool | NonEmptyString,
    tool_version | NonEmptyString,
    git_revision | String,
    config_digest | NonEmptyString,
    config_digest_detail | Digest,
    inputs | Array Input,
    stages | Array Stage,
    artifacts | Array Artifact,
    status | Status,
    warnings | Array String,
    errors | Array String,
    verification_commands | Array VerificationCommand,
    verification_evidence | Array ReceiptEvidence,
    provenance_records | Array ProvenanceRecord | default = [],
    raw_log_path | String,
  },
}

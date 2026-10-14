let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let NonNegativeNumber = std.contract.from_predicate (fun value => std.is_number value && value >= 0) in
let StringList = Array NonEmptyString in
let ArtifactIdentity = {
  path | NonEmptyString,
  blake3 | NonEmptyString,
  bytes | NonNegativeNumber,
} in
let ImportedProofBundle = {
  bundle_id | NonEmptyString,
  repository_label | NonEmptyString,
  repository_revision | NonEmptyString,
  source_path | NonEmptyString,
  proof_path | NonEmptyString,
  verifier_command_argv | StringList,
  verifier_identity | NonEmptyString,
  hash_algorithm | NonEmptyString,
  source_artifact | ArtifactIdentity,
  proof_artifact | ArtifactIdentity,
  stdout_artifacts | Array ArtifactIdentity,
  stderr_artifacts | Array ArtifactIdentity,
  receipt_artifacts | Array ArtifactIdentity,
  evidence_origin | NonEmptyString,
  intended_evidence_tier | NonEmptyString,
  trust_boundary_notes | StringList,
  promotion_command_argv | StringList,
} in
{
  ImportedProofBundleManifest = {
    schema_version | std.contract.from_predicate (fun value => value == "octet-imported-proof-bundles/v1"),
    generated_from | NonEmptyString,
    bundles | Array ImportedProofBundle,
  },
}

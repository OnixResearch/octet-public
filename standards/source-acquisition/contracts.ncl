# Source-acquisition receipt contracts for Octet stack-owned source evidence.
let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let SourceKind = std.contract.from_predicate (fun value => std.is_string value && std.array.elem value [
  "registry",
  "public_git",
  "private_ssh_git",
  "vendored_snapshot",
  "nix_flake",
  "local_override",
]) in
let EvidenceTier = std.contract.from_predicate (fun value => std.is_string value && std.array.elem value [
  "policy_source_validation",
  "manual_review",
]) in
let SourceRowV1 = {
  label | NonEmptyString,
  kind | SourceKind,
  locator | NonEmptyString,
  revision | String | default = "",
  lock_identity | String | default = "",
  observed_lock_identity | String | default = "",
  credential_boundary | String | default = "",
  release_eligible | Bool | default = false,
  local_override_policy | String | default = "",
  non_claims | Array String,
} in
{
  SourceAcquisitionReceiptV1 = {
    schema_version | std.contract.from_predicate (fun value => value == "octet-source-acquisition-receipt/v1"),
    evidence_tier | EvidenceTier,
    rows | Array SourceRowV1,
    parent_artifacts | Array String | default = [],
  },
}

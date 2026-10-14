let NonEmptyString = std.contract.from_predicate (fun value => std.string.length value > 0) in
let Algorithm = std.contract.from_predicate (fun value => value == "blake3") in
let SecretRef = {
  id | NonEmptyString,
  "ref" | NonEmptyString,
} in
{
  RunProfile = {
    schema_version | std.contract.from_predicate (fun value => value == "octet-run-profile/v1"),
    profile_id | NonEmptyString,
    source_paths | Array NonEmptyString,
    artifact_out_dir | NonEmptyString,
    receipt_out_dir | NonEmptyString,
    config_digest_algorithm | Algorithm | default = "blake3",
    enabled_policy_families | Array NonEmptyString,
    secret_refs | Array SecretRef | default = [],
  },
}

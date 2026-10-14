let contracts = import "../contracts.ncl" in
{
  schema_version = "octet-run-profile/v1",
  profile_id = "bad-profile",
  source_paths = ["docs/standards/source"],
  artifact_out_dir = "build/octet-artifacts",
  receipt_out_dir = "build/octet-receipts",
  config_digest_algorithm = "sha256",
  enabled_policy_families = ["architecture"],
} | contracts.RunProfile

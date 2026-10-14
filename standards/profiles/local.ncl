let contracts = import "./contracts.ncl" in
{
  schema_version = "octet-run-profile/v1",
  profile_id = "local-standards-check",
  source_paths = ["docs/standards/source", "standards/policies/registry.ncl"],
  artifact_out_dir = "build/octet-artifacts",
  receipt_out_dir = "build/octet-receipts",
  config_digest_algorithm = "blake3",
  enabled_policy_families = ["architecture", "safety", "correctness"],
  secret_refs = [{ id = "github-token", "ref" = "env:GITHUB_TOKEN" }],
} | contracts.RunProfile

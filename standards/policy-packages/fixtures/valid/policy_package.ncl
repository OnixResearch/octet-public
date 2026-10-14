let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-policy-package-manifest/v1",
  package_id = "octet.policy-registry-package",
  kind = "policy",
  owner = "standards",
  status = "draft",
  source_paths = ["standards/policies/registry.ncl"],
  provenance = "repo-owned policy registry",
  license_notes = "repo-owned",
  non_claims = ["customs metadata does not prove policy completeness"],
  manual_review_actions = ["review policy source and enforcement mapping before promotion"],
} | contracts.PackageManifest | contracts.PackageRules

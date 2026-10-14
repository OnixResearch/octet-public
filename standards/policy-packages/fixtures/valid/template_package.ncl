let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-policy-package-manifest/v1",
  package_id = "octet.dylint-template-package",
  kind = "template",
  owner = "standards",
  status = "draft",
  source_paths = ["templates/dylint.toml"],
  provenance = "repo-owned consumer rollout template",
  license_notes = "repo-owned",
  non_claims = ["template acceptance does not prove consumer rollout readiness"],
  manual_review_actions = ["review consumer-specific lint scope before promotion"],
} | contracts.PackageManifest | contracts.PackageRules

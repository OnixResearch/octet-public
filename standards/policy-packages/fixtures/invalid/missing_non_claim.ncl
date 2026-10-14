let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-policy-package-manifest/v1",
  package_id = "octet.package-without-non-claim",
  kind = "profile",
  owner = "standards",
  status = "draft",
  source_paths = ["templates/dynamic-evidence-profiles.ncl"],
  provenance = "checked-in draft",
  license_notes = "repo-owned",
  non_claims = [],
  manual_review_actions = ["review before promotion"],
} | contracts.PackageManifest | contracts.PackageRules

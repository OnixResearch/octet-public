let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-policy-package-manifest/v1",
  package_id = "octet.dynamic-evidence-profile-package",
  kind = "profile",
  owner = "standards",
  status = "promotable",
  source_paths = ["templates/dynamic-evidence-profiles.ncl"],
  registry_family_ids = ["octet.dynamic-evidence-profile-config"],
  generation_commands = ["nickel export --format json templates/dynamic-evidence-profiles.ncl"],
  validation_commands = ["nix develop -c cargo test -p cargo-octet --lib evidence_rails::tests"],
  positive_fixtures = ["templates/dynamic-evidence-profiles.ncl"],
  negative_fixtures = ["cargo-octet/src/evidence_rails.rs::dynamic_evidence_accepts_configured_profile_and_rejects_profile_errors"],
  provenance = "repo-owned dynamic evidence profile template",
  license_notes = "repo-owned",
  non_claims = ["profile fixture coverage does not prove every workload is covered"],
  manual_review_actions = ["review workload-specific profile thresholds before promotion"],
} | contracts.PackageManifest | contracts.PackageRules

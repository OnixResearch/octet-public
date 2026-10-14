let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-policy-package-manifest/v1",
  package_id = "octet.package-without-positive-fixture",
  kind = "template",
  owner = "standards",
  status = "promotable",
  source_paths = ["templates/dynamic-evidence-profiles.ncl"],
  registry_family_ids = ["octet.dynamic-evidence-profile-config"],
  generation_commands = ["nickel export --format json templates/dynamic-evidence-profiles.ncl"],
  validation_commands = ["nix develop -c cargo test -p cargo-octet evidence_rails"],
  positive_fixtures = [],
  negative_fixtures = ["cargo-octet/src/evidence_rails.rs::dynamic_evidence_accepts_configured_profile_and_rejects_profile_errors"],
  provenance = "checked-in template",
  license_notes = "repo-owned",
  non_claims = ["template acceptance does not prove profile workload relevance"],
  manual_review_actions = ["review profile workload relevance"],
} | contracts.PackageManifest | contracts.PackageRules

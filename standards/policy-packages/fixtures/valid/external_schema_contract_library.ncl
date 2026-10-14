let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-policy-package-manifest/v1",
  package_id = "octet.external-schema-contract-library",
  kind = "generated-contract",
  owner = "standards",
  status = "promotable",
  source_paths = ["standards/external-schemas/contracts/generated.ncl"],
  generated_artifacts = [{
    path = "standards/external-schemas/contracts/generated.ncl",
    generation_command = "nix develop -c cargo test -p cargo-octet external_schema_contracts",
    validation_command = "nickel export --format json standards/external-schemas/fixtures/valid/dependency_audit_report.ncl",
  }],
  registry_family_ids = ["octet.external-schema-contract-library"],
  generation_commands = ["nix develop -c cargo test -p cargo-octet external_schema_contracts"],
  validation_commands = ["nix run .#octet-policy-validate"],
  positive_fixtures = ["standards/external-schemas/fixtures/valid/dependency_audit_report.ncl"],
  negative_fixtures = ["standards/external-schemas/fixtures/invalid/dependency_audit_wrong_type.ncl"],
  provenance = "checked-in pinned schemas only",
  license_notes = "upstream schema license reviewed before refresh",
  non_claims = ["passing customs does not trust the external tool result"],
  manual_review_actions = ["review upstream schema license and generator caveats"],
} | contracts.PackageManifest | contracts.PackageRules

let contracts = import "../../contracts.ncl" in
{
  config_receipt_families = [
    {
      family_id = "example.run-profile-config",
      ownership = "nickel-authored",
      owner_source_path = "standards/policies/registry.ncl",
      artifact_path = "example-run-profile.json",
      generation_command = "nickel export --format json standards/policies/registry.ncl",
      validation_command = "nickel export --format json standards/policies/fixtures/valid/config_receipt_families.ncl",
      status = "existing",
      config_digest_required = true,
      positive_fixtures = ["standards/policies/fixtures/valid/config_receipt_families.ncl"],
      negative_fixtures = ["standards/policies/fixtures/invalid/config_receipt_family_missing_digest.ncl"],
    },
    {
      family_id = "example.runtime-receipt",
      ownership = "rust-derived",
      owner_source_path = "crates/octet-standards/src/artifacts.rs",
      artifact_path = "example-runtime-receipt.ncl",
      generation_command = "cargo test -p octet-standards receipt_contract_freshness_detects_drift",
      validation_command = "cargo test -p octet-standards receipt_fixtures_validate_shape_not_behavior",
      status = "planned",
    },
    {
      family_id = "example.raw-secret",
      ownership = "non-candidate",
      owner_source_path = "docs/standards/source/what-safety-critical-means.adoc",
      generation_command = "review exclusion rationale",
      validation_command = "nickel export --format json standards/policies/fixtures/valid/config_receipt_families.ncl",
      status = "non-candidate",
      exclusion_rationale = "raw secret values are never Nickel-owned",
    },
  ],
} | {
  config_receipt_families | Array contracts.ConfigReceiptFamily,
}

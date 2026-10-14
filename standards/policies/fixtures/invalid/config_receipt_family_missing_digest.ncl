let contracts = import "../../contracts.ncl" in
let Rules = std.contract.from_predicate (fun value =>
  std.array.all (fun family =>
    if family.ownership == "nickel-authored" then family.config_digest_required else true
  ) value.config_receipt_families
) in
{
  config_receipt_families = [{
    family_id = "example.run-profile-config",
    ownership = "nickel-authored",
    owner_source_path = "standards/policies/registry.ncl",
    artifact_path = "example-run-profile.json",
    generation_command = "nickel export --format json standards/policies/registry.ncl",
    validation_command = "nickel export --format json standards/policies/fixtures/invalid/config_receipt_family_missing_digest.ncl",
    status = "existing",
    config_digest_required = false,
  }],
} | { config_receipt_families | Array contracts.ConfigReceiptFamily } | Rules

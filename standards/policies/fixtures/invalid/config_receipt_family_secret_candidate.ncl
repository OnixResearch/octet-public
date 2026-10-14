let contracts = import "../../contracts.ncl" in
let Rules = std.contract.from_predicate (fun value =>
  std.array.all (fun family =>
    family.ownership != "non-candidate" || std.string.length family.artifact_path == 0
  ) value.config_receipt_families
) in
{
  config_receipt_families = [{
    family_id = "example.raw-secret",
    ownership = "non-candidate",
    owner_source_path = "docs/standards/source/what-safety-critical-means.adoc",
    artifact_path = "raw-secret.json",
    generation_command = "review exclusion rationale",
    validation_command = "nickel export --format json standards/policies/fixtures/invalid/config_receipt_family_secret_candidate.ncl",
    status = "non-candidate",
    exclusion_rationale = "raw secret values are never Nickel-owned",
  }],
} | { config_receipt_families | Array contracts.ConfigReceiptFamily } | Rules

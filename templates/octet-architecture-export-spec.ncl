# Export this file to a temporary JSON request for `octet-standards nickel-export-manifest`.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let ExportSpec = {
  schema_version | std.contract.from_predicate (fun value => value == "octet-nickel-export-spec/v1"),
  family_id | std.contract.from_predicate (fun value => value == "octet.project-architecture-policy"),
  source | NonEmptyString,
  dependencies | Array NonEmptyString,
  selector | String,
  contract | NonEmptyString,
  output_format | std.contract.from_predicate (fun value => value == "json"),
  output_path | NonEmptyString,
  allow_secret_material | Bool,
} in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.project-architecture-policy",
  source = "octet-architecture.ncl",
  dependencies = ["octet-architecture.contracts.ncl"],
  selector = "",
  contract = "ProjectArchitecturePolicy and ProjectArchitecturePolicyRules",
  output_format = "json",
  output_path = "octet-architecture.json",
  allow_secret_material = false,
} | ExportSpec

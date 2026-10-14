let contracts = import "../../nickel-exports/contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.assurance-profile.onixos-lifecycle-proof",
  source = "standards/policies/assurance-profiles/onixos-lifecycle-proof.ncl",
  dependencies = ["standards/policies/assurance-profiles/contracts.ncl"],
  selector = "",
  contract = "AssuranceProfile and AssuranceProfileRules",
  output_format = "json",
  output_path = "standards/policies/assurance-profiles/generated/onixos-lifecycle-proof.json",
  allow_secret_material = false,
} | contracts.ExportSpec

let contracts = import "../nickel-exports/contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.durability-claims-profile",
  source = "standards/durability-claims/profile.ncl",
  dependencies = ["standards/durability-claims/contracts.ncl"],
  selector = "",
  contract = "Profile and ProfileRules",
  output_format = "json",
  output_path = "standards/durability-claims/generated/profile.json",
  allow_secret_material = false,
} | contracts.ExportSpec

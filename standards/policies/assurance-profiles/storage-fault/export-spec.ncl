let contracts = import "../../../nickel-exports/contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.storage-fault-assurance-profile",
  source = "standards/policies/assurance-profiles/storage-fault/profile.ncl",
  dependencies = ["standards/policies/assurance-profiles/storage-fault/contracts.ncl"],
  selector = "",
  contract = "Profile and ProfileRules",
  output_format = "json",
  output_path = "standards/policies/assurance-profiles/storage-fault/generated/profile.json",
  allow_secret_material = false,
} | contracts.ExportSpec

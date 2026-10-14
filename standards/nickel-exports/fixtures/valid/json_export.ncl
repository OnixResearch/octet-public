let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.dynamic-evidence-profile-config",
  source = "templates/dynamic-evidence-profiles.ncl",
  dependencies = ["standards/policies/contracts.ncl"],
  selector = "profiles",
  contract = "DynamicEvidenceProfiles",
  output_format = "json",
  output_path = "dynamic-evidence-profiles.json",
} | contracts.ExportSpec

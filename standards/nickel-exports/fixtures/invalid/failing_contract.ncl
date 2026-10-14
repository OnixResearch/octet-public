let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v0",
  family_id = "octet.bad-contract",
  source = "standards/policies/registry.ncl",
  output_format = "json",
  output_path = "bad-contract.json",
} | contracts.ExportSpec

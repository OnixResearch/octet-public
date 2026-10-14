let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.example-raw-config",
  source = "standards/policies/registry.ncl",
  output_format = "raw",
  output_path = "example.txt",
} | contracts.ExportSpec

let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.example-yaml-config",
  source = "standards/policies/registry.ncl",
  output_format = "yaml",
  output_path = "example.yaml",
} | contracts.ExportSpec

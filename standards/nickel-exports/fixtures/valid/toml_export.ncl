let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.example-toml-config",
  source = "standards/policies/registry.ncl",
  output_format = "toml",
  output_path = "example.toml",
} | contracts.ExportSpec

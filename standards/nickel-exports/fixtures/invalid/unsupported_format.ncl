let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.bad-format",
  source = "standards/policies/registry.ncl",
  output_format = "xml",
  output_path = "example.xml",
} | contracts.ExportSpec

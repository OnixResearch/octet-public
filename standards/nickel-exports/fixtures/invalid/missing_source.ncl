let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.missing-source",
  source = "",
  output_format = "json",
  output_path = "example.json",
} | contracts.ExportSpec

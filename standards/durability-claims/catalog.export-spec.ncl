let contracts = import "../nickel-exports/contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.durability-claims-catalog",
  source = "standards/durability-claims/catalog.ncl",
  dependencies = ["standards/durability-claims/contracts.ncl"],
  selector = "",
  contract = "Catalog and CatalogRules",
  output_format = "json",
  output_path = "standards/durability-claims/generated/catalog.json",
  allow_secret_material = false,
} | contracts.ExportSpec

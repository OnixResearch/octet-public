let contracts = import "../nickel-exports/contracts.ncl" in
{
  schema_version = "octet-nickel-export-spec/v1",
  family_id = "octet.project-architecture-policy",
  source = "standards/project-architecture/octet-self-architecture.ncl",
  dependencies = ["standards/project-architecture/octet-self-architecture.contracts.ncl"],
  selector = "",
  contract = "ProjectArchitecturePolicy and ProjectArchitecturePolicyRules",
  output_format = "json",
  output_path = "standards/project-architecture/octet-self-architecture.json",
  allow_secret_material = false,
} | contracts.ExportSpec

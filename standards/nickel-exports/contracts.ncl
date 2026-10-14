# Deterministic Nickel export helper specs.
let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let OneOf = fun allowed => std.contract.from_predicate (fun value => std.is_string value && std.array.elem value allowed) in
let SchemaVersion = std.contract.from_predicate (fun value => value == "octet-nickel-export-spec/v1") in
let ExportFormatContract = OneOf ["json", "toml", "yaml", "raw"] in
let ExportSpecContract = {
  schema_version | SchemaVersion,
  family_id | NonEmptyString,
  source | NonEmptyString,
  dependencies | Array NonEmptyString | default = [],
  selector | String | default = "",
  contract | String | default = "",
  output_format | ExportFormatContract,
  output_path | NonEmptyString,
  allow_secret_material | Bool | default = false,
} in
{
  ExportSpec = ExportSpecContract,
  ExportFormat = ExportFormatContract,
}

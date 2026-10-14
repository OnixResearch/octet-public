let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let SchemaVersion = OneOf ["octet-nominal-domain-policy/v1"] in
let EnforcementMode = OneOf ["inventory", "warn", "deny"] in
let RawRepresentation = OneOf ["string", "u64", "usize", "path"] in
let WrapperKind = OneOf ["opaque-newtype", "refined-newtype", "marker-generic", "unit-newtype"] in
let ConstructorPolicy = OneOf ["private-infallible", "private-fallible", "marker-only"] in
let Domain = {
  domain_id | NonEmptyString,
  symbols | NonEmptyArray | Array NonEmptyString,
  raw_representation | RawRepresentation,
  core_scopes | NonEmptyArray | Array NonEmptyString,
  compatibility_scopes | Array NonEmptyString,
  compatibility_required | Bool,
  wrapper_kind | WrapperKind,
  constructor_policy | ConstructorPolicy,
} in
let has_duplicates = fun values =>
  std.array.length values != std.array.length (std.array.dedup values)
in
let scopes_overlap = fun domain =>
  std.array.any
    (fun scope => std.array.elem scope domain.compatibility_scopes)
    domain.core_scopes
in
let constructor_matches_wrapper = fun domain =>
  if domain.wrapper_kind == "refined-newtype" then
    domain.constructor_policy == "private-fallible"
  else if domain.wrapper_kind == "marker-generic" then
    domain.constructor_policy == "marker-only"
  else
    domain.constructor_policy != "marker-only"
in
let domain_cross_fields_valid = fun domain =>
  !has_duplicates domain.symbols
  && !scopes_overlap domain
  && ((!domain.compatibility_required) || std.array.length domain.compatibility_scopes > 0)
  && constructor_matches_wrapper domain
in
let catalog_cross_fields_valid = fun catalog =>
  let domain_ids = catalog.domains |> std.array.map (fun domain => domain.domain_id) in
  !has_duplicates domain_ids
  && !has_duplicates catalog.heuristic_suffixes
  && std.array.all domain_cross_fields_valid catalog.domains
in
let CatalogRulesContract = std.contract.from_predicate catalog_cross_fields_valid in
{
  Catalog = {
    schema_version | SchemaVersion,
    default_mode | EnforcementMode,
    heuristic_suffixes | NonEmptyArray | Array NonEmptyString,
    domains | NonEmptyArray | Array Domain,
    non_claims | NonEmptyArray | Array NonEmptyString,
  },
  CatalogRules = CatalogRulesContract,
}

let contracts = import "../../contracts.ncl" in
let domain = {
  domain_id = "fixture.duplicate",
  symbols = ["UnitId"],
  raw_representation = "string",
  core_scopes = ["src/core.rs"],
  compatibility_scopes = [],
  compatibility_required = false,
  wrapper_kind = "opaque-newtype",
  constructor_policy = "private-infallible",
} in
{
  schema_version = "octet-nominal-domain-policy/v1",
  default_mode = "inventory",
  heuristic_suffixes = ["Id"],
  domains = [domain, domain],
  non_claims = ["The fixture must fail."],
} | contracts.Catalog | contracts.CatalogRules

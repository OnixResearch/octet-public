let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nominal-domain-policy/v1",
  default_mode = "inventory",
  heuristic_suffixes = ["Id"],
  domains = [{
    domain_id = "fixture.unit-id",
    symbols = ["UnitId"],
    raw_representation = "string",
    core_scopes = ["src/core.rs"],
    compatibility_scopes = [],
    compatibility_required = false,
    wrapper_kind = "refined-newtype",
    constructor_policy = "private-infallible",
  }],
  non_claims = ["The fixture must fail."],
} | contracts.Catalog | contracts.CatalogRules

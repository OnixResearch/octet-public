let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-nominal-domain-policy/v1",
  default_mode = "inventory",
  heuristic_suffixes = ["Id"],
  domains = [{
    domain_id = "fixture.unit-id",
    symbols = ["UnitId", "DynamicUnit::id"],
    raw_representation = "string",
    core_scopes = ["src/core.rs"],
    compatibility_scopes = ["src/wire.rs"],
    compatibility_required = true,
    wrapper_kind = "refined-newtype",
    constructor_policy = "private-fallible",
  }],
  non_claims = ["The fixture checks policy shape only."],
} | contracts.Catalog | contracts.CatalogRules

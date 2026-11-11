let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-strict-rust-profiles/v1",
  boolean_api_policy = (import "../../profiles.ncl").boolean_api_policy,
  profiles = [{
    profile_id = "fixture-profile",
    summary = "Valid strict-profile fixture.",
    required_lints = ["no_unwrap"],
    effect_families = ["failure"],
    requires_core_scopes = false,
    minimum_function_address_mode = "observe",
    function_address_scope = "workspace",
    claim_scope = "direct-source-shape-policy",
    non_claims = ["Fixture validation does not prove whole-program correctness."],
  }],
} | contracts.Catalog

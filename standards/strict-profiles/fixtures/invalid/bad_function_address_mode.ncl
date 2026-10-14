let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-strict-rust-profiles/v1",
  profiles = [{
    profile_id = "bad-mode",
    summary = "Invalid function-address mode fixture.",
    required_lints = ["no_unwrap"],
    effect_families = ["failure"],
    requires_core_scopes = false,
    minimum_function_address_mode = "strict",
    function_address_scope = "workspace",
    claim_scope = "direct-source-shape-policy",
    non_claims = ["Invalid fixture does not prove whole-program correctness."],
  }],
} | contracts.Catalog

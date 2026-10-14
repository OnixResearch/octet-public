let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-strict-rust-profiles/v1",
  profiles = [{
    profile_id = "empty-non-claims",
    summary = "Invalid empty non-claim fixture.",
    required_lints = ["no_unwrap"],
    effect_families = ["failure"],
    requires_core_scopes = false,
    minimum_function_address_mode = "observe",
    function_address_scope = "workspace",
    claim_scope = "direct-source-shape-policy",
    non_claims = [],
  }],
} | contracts.Catalog

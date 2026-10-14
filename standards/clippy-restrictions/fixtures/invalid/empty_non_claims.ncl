let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-clippy-restriction-profiles/v1",
  profiles = [{
    profile_id = "empty-non-claims",
    summary = "Invalid empty non-claims fixture.",
    lint_level = "deny",
    lints = ["dbg_macro"],
    claim_scope = "clippy-restriction-diagnostics",
    non_claims = [],
  }],
} | contracts.Catalog

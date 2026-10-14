let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-clippy-restriction-profiles/v1",
  profiles = [{
    profile_id = "fixture-profile",
    summary = "Valid Clippy restriction profile fixture.",
    lint_level = "warn",
    lints = ["dbg_macro"],
    claim_scope = "clippy-restriction-diagnostics",
    non_claims = ["Fixture diagnostics do not prove behavioral correctness."],
  }],
} | contracts.Catalog

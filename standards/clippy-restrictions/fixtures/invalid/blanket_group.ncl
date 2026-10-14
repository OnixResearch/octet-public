let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-clippy-restriction-profiles/v1",
  profiles = [{
    profile_id = "blanket-group",
    summary = "Invalid blanket restriction fixture.",
    lint_level = "deny",
    lints = ["restriction"],
    claim_scope = "clippy-restriction-diagnostics",
    non_claims = ["Invalid fixture."],
  }],
} | contracts.Catalog

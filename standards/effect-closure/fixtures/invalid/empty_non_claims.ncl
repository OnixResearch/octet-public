let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-effect-closure-workspace-policy/v1",
  profile_id = "deterministic-core",
  function_address_baseline = "evidence/octet/function-addresses.json",
  module_roles = [{ path = "src/lib.rs", role = "core" }],
  call_summaries = [],
  non_claims = [],
} | contracts.WorkspacePolicy

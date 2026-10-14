let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-effect-closure-workspace-policy/v1",
  profile_id = "deterministic-core",
  function_address_baseline = "evidence/octet/function-addresses.json",
  module_roles = [{ path = "src/lib.rs", role = "core" }],
  call_summaries = [{
    call_site_id = "b3:0000000000000000000000000000000000000000000000000000000000000000",
    review_label = "invalid unknown summary",
    callee = "opaque",
    provider = {
      kind = "toolchain",
      name = "core",
      version = "rustc",
      source = "rustc-toolchain",
      target = "x86_64-unknown-linux-gnu",
      features = [],
    },
    effects = ["ambient-mystery"],
    evidence = {
      kind = "reviewed-source-summary-v1",
      locator = "review.md",
      blake3 = "b3:1111111111111111111111111111111111111111111111111111111111111111",
    },
  }],
  non_claims = ["invalid fixture"],
} | contracts.WorkspacePolicy

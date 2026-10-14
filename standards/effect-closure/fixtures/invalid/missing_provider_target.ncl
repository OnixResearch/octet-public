let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-effect-closure-workspace-policy/v1",
  profile_id = "deterministic-core",
  function_address_baseline = "evidence/octet/function-addresses.json",
  module_roles = [{ path = "src/lib.rs", role = "core" }],
  call_summaries = [{
    call_site_id = "b3:0000000000000000000000000000000000000000000000000000000000000000",
    review_label = "missing exact target",
    callee = "blake3::hash",
    provider = {
      kind = "cargo-package",
      name = "blake3",
      version = "1.8.2",
      source = "registry+https://github.com/rust-lang/crates.io-index",
      features = [],
    },
    effects = [],
    evidence = {
      kind = "reviewed-source-summary-v1",
      locator = "Cargo.lock#blake3",
      blake3 = "b3:1111111111111111111111111111111111111111111111111111111111111111",
    },
  }],
  non_claims = ["invalid fixture"],
} | contracts.WorkspacePolicy

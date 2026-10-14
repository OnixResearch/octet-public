# Copy this file with standards/effect-closure/contracts.ncl, update the import,
# review every exact call site, then export with:
#   nickel export --format json effect-closure-policy.ncl > octet-effect-closure.json
let contracts = import "../standards/effect-closure/contracts.ncl" in
{
  schema_version = "octet-effect-closure-workspace-policy/v1",
  profile_id = "deterministic-core",
  function_address_baseline = "artifacts/octet-observe/function-addresses.json",
  module_roles = [
    { path = "crates/domain-core/src/lib.rs", role = "core" },
  ],
  call_summaries = [],
  non_claims = [
    "This workspace policy supports conservative selected source-token closure only; it does not establish complete Rust name resolution, MIR call-graph closure, or runtime reachability.",
    "Reviewed call summaries do not verify dependency implementations, semantic determinism, whole-program correctness, or release eligibility.",
  ],
} | contracts.WorkspacePolicy

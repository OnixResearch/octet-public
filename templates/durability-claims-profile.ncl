# Copy this file with standards/durability-claims/contracts.ncl, update the
# import, declare every maintained durability boundary, then export with:
#   nickel export --format json durability-claims-profile.ncl > durability-claims.json
# Point `durability_profile_path` in `dylint.toml` at the generated file.
let contracts = import "../standards/durability-claims/contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "workspace-durability-claims",
  boundaries = [
    {
      boundary_id = "store-persist",
      source_scope = "crates/store/src/",
      boundary_functions = ["persist"],
      required_observation = "synchronization",
      uncertain_outcome_types = ["DurabilityOutcome"],
      commit_surfaces = [],
      adapter_observations = [],
    },
  ],
  non_claims = [
    "This profile declares analyzed source scopes and observation classes only; it does not prove any filesystem, device, or hardware durability property.",
  ],
} | contracts.Profile | contracts.ProfileRules

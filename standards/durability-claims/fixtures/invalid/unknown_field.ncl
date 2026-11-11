let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "unknown-field",
  boundaries = [
    {
      boundary_id = "persist",
      source_scope = "crates/store/src/",
      boundary_functions = ["crate::store::persist"],
      required_observation = "synchronization",
      uncertain_outcome_types = ["DurabilityOutcome"],
      best_effort = true,
    },
  ],
  non_claims = ["This fixture must not export: the profile shape is closed."],
} | contracts.Profile | contracts.ProfileRules

let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "duplicate-boundary",
  boundaries = [
    {
      boundary_id = "persist",
      source_scope = "crates/store/src/",
      boundary_functions = ["crate::store::persist"],
      required_observation = "synchronization",
      uncertain_outcome_types = ["DurabilityOutcome"],
    },
    {
      boundary_id = "persist",
      source_scope = "crates/other/src/",
      boundary_functions = ["crate::other::persist"],
      required_observation = "synchronization",
      uncertain_outcome_types = ["DurabilityOutcome"],
    },
  ],
  non_claims = ["This fixture must not export: boundary ids are unique."],
} | contracts.Profile | contracts.ProfileRules

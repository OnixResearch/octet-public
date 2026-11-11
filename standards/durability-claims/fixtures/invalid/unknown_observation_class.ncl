let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "unknown-observation",
  boundaries = [
    {
      boundary_id = "persist",
      source_scope = "crates/store/src/",
      boundary_functions = ["crate::store::persist"],
      required_observation = "best-effort",
      uncertain_outcome_types = ["DurabilityOutcome"],
    },
  ],
  non_claims = ["This fixture must not export: the observation class is unknown."],
} | contracts.Profile | contracts.ProfileRules

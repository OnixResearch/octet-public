let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "missing-uncertainty",
  boundaries = [
    {
      boundary_id = "persist",
      source_scope = "crates/store/src/",
      boundary_functions = ["crate::store::persist"],
      required_observation = "synchronization",
      uncertain_outcome_types = [],
    },
  ],
  non_claims = ["This fixture must not export: every boundary names its uncertain-outcome type."],
} | contracts.Profile | contracts.ProfileRules

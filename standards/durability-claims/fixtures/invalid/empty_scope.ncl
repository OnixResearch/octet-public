let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "empty-scope",
  boundaries = [
    {
      boundary_id = "persist",
      source_scope = "",
      boundary_functions = ["crate::store::persist"],
      required_observation = "synchronization",
      uncertain_outcome_types = ["DurabilityOutcome"],
    },
  ],
  non_claims = ["This fixture must not export: every boundary names an analyzed source scope."],
} | contracts.Profile | contracts.ProfileRules

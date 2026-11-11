let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "minimum",
  boundaries = [
    {
      boundary_id = "persist",
      source_scope = "crates/store/src/",
      boundary_functions = ["crate::store::persist"],
      required_observation = "synchronization",
      uncertain_outcome_types = ["DurabilityOutcome"],
    },
  ],
  non_claims = ["This fixture declares one analyzed boundary only."],
} | contracts.Profile | contracts.ProfileRules

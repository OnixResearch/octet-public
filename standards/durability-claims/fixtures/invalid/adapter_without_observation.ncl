let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "adapter-without-observation",
  boundaries = [
    {
      boundary_id = "publish",
      source_scope = "crates/publication/src/",
      boundary_functions = ["crate::publication::publish"],
      required_observation = "adapter-observation",
      uncertain_outcome_types = ["PublicationOutcome"],
    },
  ],
  non_claims = ["This fixture must not export: adapter observation requires a configured adapter surface."],
} | contracts.Profile | contracts.ProfileRules

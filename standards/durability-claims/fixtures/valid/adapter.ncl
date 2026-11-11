let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "adapter",
  boundaries = [
    {
      boundary_id = "publish",
      source_scope = "crates/publication/src/",
      boundary_functions = ["crate::publication::publish"],
      required_observation = "adapter-observation",
      uncertain_outcome_types = ["PublicationOutcome"],
      adapter_observations = ["durable_file_publication::Publication::commit"],
    },
  ],
  non_claims = ["This fixture delegates durability observation to one configured adapter."],
} | contracts.Profile | contracts.ProfileRules

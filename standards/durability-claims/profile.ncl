let contracts = import "contracts.ncl" in
{
  schema_version = "octet-durability-claims-profile/v1",
  profile_id = "octet-durability-claims-example",
  boundaries = [
    {
      boundary_id = "wal-checkpoint",
      source_scope = "crates/store/src/",
      boundary_functions = [
        "crate::wal::checkpoint",
        "crate::wal::checkpoint_batch",
      ],
      required_observation = "synchronization",
      uncertain_outcome_types = [
        "DurabilityOutcome",
        "StoreError",
      ],
      commit_surfaces = [
        "WriteTransaction::commit",
      ],
    },
    {
      boundary_id = "published-artifact",
      source_scope = "crates/publication/src/",
      boundary_functions = [
        "crate::publication::publish",
      ],
      required_observation = "adapter-observation",
      uncertain_outcome_types = [
        "PublicationOutcome",
      ],
      adapter_observations = [
        "durable_file_publication::Publication::commit",
      ],
    },
  ],
  non_claims = [
    "This profile declares analyzed source scopes and observation classes only; it does not prove any filesystem, device, or hardware durability property.",
    "Passing the source checks does not prove that a synchronization call, configured commit, or adapter observation succeeded.",
  ],
} | contracts.Profile | contracts.ProfileRules

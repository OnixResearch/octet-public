# r[impl octet.aeneas_consumer_contract.profile]
# r[impl octet.aeneas_consumer_contract.theorem]
let contracts = import "consumer-contracts.ncl" in
let reviewed_provider_profile_identity = "b3:cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625" in
let reviewed_compatibility_matrix_identity = "b3:403656d75de41b955cd2493ef289ba17308da65a3d214babc1ea606aeae248f5" in
let selected_source_identity = "b3:691e8de8b8f92665dd00b938b48b398b451c71a06c99dae10b38b38184d4dddd" in
let selected_theorem_identity = "b3:776e9f85261080fbcfd75cb481c4f563cff50ffefaf8bdaae3d5fad010d8c77f" in
let configured_timeout_ms = 900000 in
let configured_log_max_bytes = 1048576 in
{
  schema_version = "octet-aeneas-consumer-profile/v1",
  profile_id = "octet-external-aeneas-consumer-v1",
  consumer_id = "octet.external-consumer-fixture",
  provider = {
    backend = "aeneas-lean",
    profile_id = "octet-aeneas-lean-0.1.0-bc2eb6f",
    profile_identity = reviewed_provider_profile_identity,
    compatibility_matrix_id = "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16",
    compatibility_matrix_identity = reviewed_compatibility_matrix_identity,
    network_allowed = false,
  },
  source = {
    root = ".",
    crate_name = "external_consumer",
    paths = [
      { path = "src/lib.rs", identity = selected_source_identity },
    ],
    selected_items = [
      {
        qualified_name = "external_consumer::increment",
        kind = "function",
        source_file = "src/lib.rs",
        source_identity = selected_source_identity,
        begin_line = 1,
        end_line = 3,
        generated_name = "external_consumer.increment",
      },
    ],
    broad_selection_allowed = false,
  },
  theorems = [
    {
      path = "Theorem.lean",
      theorem_id = "aeneas.consumer.increment-matches-model",
      declaration_name = "increment_matches_model",
      requirement_id = "octet.aeneas_consumer_contract.compatibility",
      claim_class = "named-theorem-over-generated-model",
      approved_statement_blake3 = selected_theorem_identity,
      source_disposition = "maintained",
    },
  ],
  external_models = [],
  trust_inputs = [],
  resource_policy = {
    timeout_ms = configured_timeout_ms,
    stdout_max_bytes = configured_log_max_bytes,
    stderr_max_bytes = configured_log_max_bytes,
    environment_allowlist = ["PATH", "CHARON_TOOLCHAIN_IS_IN_PATH", "LD_LIBRARY_PATH", "HOME", "MATHLIB_NO_CACHE_ON_UPDATE"],
  },
  outcomes = ["supported", "blocked", "failed"],
  trust_closure_complete = true,
  non_claims = [
    "does not prove Aeneas or Charon translation soundness",
    "does not prove Lean or verifier soundness",
    "does not prove whole-program correctness",
    "does not prove semantic equivalence",
    "does not establish release eligibility",
    "does not prove rustc compiler correctness",
    "does not transfer theorem intent or requirement ownership from the consumer to Octet",
  ],
} | contracts.AeneasConsumerProfile | contracts.AeneasConsumerProfileRules

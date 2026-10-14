let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-chaoscontrol-proof-style-input-contract/v1",
  consumer_repo = "github:OnixResearch/chaoscontrol",
  selected_profile = "proof-style-focused",
  policy_registry_digest = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  trellis_primitive_manifest_digest = "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  input_revision = "chaoscontrol-proof-style-fixture-revision",
  expected_command = ["cargo", "octet", "check", "--profile", "proof-style-focused"],
  non_claims = ["Octet validates profile and input identity only; it does not prove ChaosControl behavioral correctness, runtime determinism, or VMM correctness"],
} | contracts.ChaosControlProofStyleInputContractV1

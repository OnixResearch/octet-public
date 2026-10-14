let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-proof-bundle/v1",
  generated_from = "standards/policies/registry.ncl",
  registry_schema_version = "fixture",
  proof_bundles = [{
    policy_id = "tigerstyle.verification.fixture",
    rail_index = 0,
    proof_rail = {
      backend = "local-command-receipt",
      command = "bash proof-command.sh",
      source_paths = ["src/lib.rs"],
      proof_paths = ["proof-receipt.txt"],
      pass_signal = "proof-result: pass",
      trust_boundaries = ["local-source-root"],
      receipt_binding = "blake3-command-and-source",
      receipt_provenance = {
        schema_version = "tigerstyle-proof-receipt/v1",
        runner = "tigerstyle-local-command",
        environment = "local-devshell",
        requires_backend = true,
        requires_trust_boundary = true,
      },
      receipt_verifier = {
        verifier_id = "octet-standards@proof-receipt/v1",
        verifier_digest = "fixture",
      },
    },
  }],
} | contracts.ProofBundleArtifact

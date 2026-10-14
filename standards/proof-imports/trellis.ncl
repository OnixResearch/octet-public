let contracts = import "./contracts.ncl" in
{
  schema_version = "octet-imported-proof-bundles/v1",
  generated_from = "standards/proof-imports/trellis.ncl",
  bundles = [
    {
      bundle_id = "trellis.ucan-proof-reference.digest-admission",
      repository_label = "../trellis",
      repository_revision = "fb232e0e213d3e96878bbf86a10b26e53245a72c",
      source_path = "src/ucan_proof_reference.rs",
      proof_path = "src/ucan_proof_reference.rs",
      verifier_command_argv = ["./scripts/verus.sh", "src/ucan_proof_reference.rs"],
      verifier_identity = "trellis/scripts/verus.sh external reference",
      hash_algorithm = "blake3",
      source_artifact = {
        path = "standards/proof-imports/fixtures/trellis/ucan_proof_reference_source.rs",
        blake3 = "c39b9d0e85f778b5b52ddf00ee7c422ea2de782551ff4874314c009be687f85a",
        bytes = 388,
      },
      proof_artifact = {
        path = "standards/proof-imports/fixtures/trellis/ucan_proof_reference_proof.rs",
        blake3 = "434529911312c2f98def943df2479b2fc7b7a23f6cf8c911d283151352399f19",
        bytes = 310,
      },
      stdout_artifacts = [
        {
          path = "standards/proof-imports/fixtures/trellis/ucan_proof_reference.stdout",
          blake3 = "ac6c76d70c441cc470eae63849d6587274b6d4122abb9fc55049d6941f9720c5",
          bytes = 84,
        },
      ],
      stderr_artifacts = [
        {
          path = "standards/proof-imports/fixtures/trellis/ucan_proof_reference.stderr",
          blake3 = "af1349b9f5f9a1a6a0404dea36dcc9499bcb25c9adc112b7cc9a93cae41f3262",
          bytes = 0,
        },
      ],
      receipt_artifacts = [
        {
          path = "standards/proof-imports/fixtures/trellis/ucan_proof_reference.receipt.json",
          blake3 = "1d9c5738c4f151cd0b72e2a5a86a1ba8f7f65bca227249fd67d4a4b78bac9d3b",
          bytes = 223,
        },
      ],
      evidence_origin = "external_trellis_import",
      intended_evidence_tier = "manual_review_required",
      trust_boundary_notes = [
        "BLAKE3 identifies exact artifact bytes and does not by itself prove proof correctness, solver soundness, model adequacy, or Octet receipt validity",
        "Octet formal_proof requires rerunning with an Octet pinned verifier and validating an Octet-owned receipt",
      ],
      promotion_command_argv = ["cargo", "octet", "prove", "--source", "src/verified", "--artifact-dir", "artifacts/octet"],
    },
  ],
} | contracts.ImportedProofBundleManifest

let contracts = import "contracts.ncl" in
{
  schema_version = "octet-proof-claim-catalog/v1",
  claims = [
    {
      claim_id = "octet.verus_proof_rails.addition-bound#rail-0",
      source_requirement_or_policy_id = "octet.verus_proof_rails.addition-bound",
      checked_scope = "formal-proof:fixture-only:octet.verus_proof_rails.addition-bound#rail-0",
      expected_evidence_tier = "formal_proof",
      allowed_proof_or_property_ids = ["octet.verus_proof_rails.addition-bound#rail-0"],
      required_artifacts = [
        "src/verified/addition.rs",
        "verus/proofs/addition_bound.rs",
        "proofs/octet.verus_proof_rails.addition-bound-rail-0.receipt",
      ],
      documentation_anchors = ["docs/trust-boundary-evidence.md#verus-proof-path-lane"],
      owner_surface = "cargo-octet prove",
      status = "active",
      non_claims = [
        "not whole-program correctness",
        "not certification",
        "not business-domain validation",
        "not semantic equivalence",
        "not full safety",
        "not verified runtime",
      ],
    },
  ],
  aliases = [
    {
      alias_id = "octet.proof_claims.addition-bound-fixture",
      target_claim_id = "octet.verus_proof_rails.addition-bound#rail-0",
      status = "alias",
      rationale = "short reviewer-facing alias for the fixture addition-bound proof claim",
    },
  ],
} | contracts.ProofClaimCatalog

# Frozen local Kamacite effect-summary compatibility fixture.
#
# This fixture stands in for the external Kamacite semantic-operation cohort.
# It is not a live Kamacite run and it does not make Octet a canonicalization
# owner. Operation, family, and summary identities below are supplied by the
# compatibility projection; Octet only compares them.
let contracts = import "../../kamacite-contracts.ncl" in
{
  schema_version = "octet-kamacite-effect-summary-projection/v1",
  kamacite_schema_version = "kamacite.effect-summary.v1",
  preserves_profile_schema = "kamacite.preserves-profile.v1",
  preserves_profile_identity = {
    domain = "projection",
    algorithm = "blake3",
    hex = "1ae88fe233cb3df52e2e9438ae6b910c2d83b2e35025cfb99b3eda4f94370ece",
  },
  effect_summary_identity = {
    domain = "algebraic-effect",
    algorithm = "blake3",
    hex = "64471a04261cbb5f5d9e9b3df5ba9f2454145535355601760e779d2f6b0a46ac",
  },
  effect_summary_closed = true,
  operations = [
    {
      operation_identity = {
        domain = "effect-operation",
        algorithm = "blake3",
        hex = "7fa2709c205ccfa8c4b8773ab97396193a5130a2e42b2f94916802b04e07f5f8",
      },
      effect_family_identity = {
        domain = "algebraic-effect",
        algorithm = "blake3",
        hex = "1111111111111111111111111111111111111111111111111111111111111111",
      },
      display_name = "record-audit-event",
      non_claims = ["Operation identity is supplied by the frozen compatibility projection; it does not prove handler behavior."],
    },
    {
      operation_identity = {
        domain = "effect-operation",
        algorithm = "blake3",
        hex = "c598d03de53f54b01e8bc368840e6394e5cc3334b4e1be118f563ddfb8cfba2a",
      },
      effect_family_identity = {
        domain = "algebraic-effect",
        algorithm = "blake3",
        hex = "2222222222222222222222222222222222222222222222222222222222222222",
      },
      display_name = "emit-telemetry-batch",
      non_claims = ["Operation identity is supplied by the frozen compatibility projection; it does not prove replay truth."],
    },
  ],
  non_claims = [
    "This frozen local projection is a compatibility fixture, not a live Kamacite cohort, canonicalization result, or semantic-equivalence claim.",
    "A matching operation identity does not prove runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility.",
  ],
} | contracts.KamaciteEffectSummaryProjection

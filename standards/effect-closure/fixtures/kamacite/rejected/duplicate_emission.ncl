# Rejected fixture: one emission site may not bind two operations.
let contracts = import "../../../kamacite-contracts.ncl" in
let rule = {
  function_address = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  call_site_id = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  provider = { kind = "workspace", name = "octet", version = "0.1.0", source = "workspace", target = "x86_64-unknown-linux-gnu", features = [] },
  evidence = { kind = "reviewed-semantic-emission-v1", locator = "src/lib.rs", blake3 = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc" },
  non_claims = ["invalid fixture"],
} in
{
  schema_version = "octet-effect-closure-kamacite-profile/v1",
  profile_id = "deterministic-core",
  mapping_id = "octet-kamacite-effect-mapping-v1",
  kamacite = {
    preserves_profile_schema = "kamacite.preserves-profile.v1",
    preserves_profile_identity = { domain = "projection", algorithm = "blake3", hex = "1ae88fe233cb3df52e2e9438ae6b910c2d83b2e35025cfb99b3eda4f94370ece" },
    effect_summary_schema = "kamacite.effect-summary.v1",
    effect_summary_identity = { domain = "algebraic-effect", algorithm = "blake3", hex = "64471a04261cbb5f5d9e9b3df5ba9f2454145535355601760e779d2f6b0a46ac" },
    projection = "standards/effect-closure/fixtures/kamacite/effect-summary.json",
    projection_blake3 = "b3:7cdf5a386137b8145c39a55ff3a851e306f88efd2771d1ff0be8b0248e528291",
  },
  ambient_dispositions = [{ effect = "alloc", disposition = "profile-local" }],
  semantic_emissions = [
    rule & { operation_identity = { domain = "effect-operation", algorithm = "blake3", hex = "7fa2709c205ccfa8c4b8773ab97396193a5130a2e42b2f94916802b04e07f5f8" } },
    rule & { operation_identity = { domain = "effect-operation", algorithm = "blake3", hex = "c598d03de53f54b01e8bc368840e6394e5cc3334b4e1be118f563ddfb8cfba2a" } },
  ],
  non_claims = ["invalid fixture"],
} | contracts.KamaciteEffectProfile

# Positive fixture: minimal mapping profile with no reviewed emissions yet.
let contracts = import "../../../kamacite-contracts.ncl" in
{
  schema_version = "octet-effect-closure-kamacite-profile/v1",
  profile_id = "deterministic-core",
  mapping_id = "octet-kamacite-effect-mapping-v1",
  kamacite = {
    preserves_profile_schema = "kamacite.preserves-profile.v1",
    preserves_profile_identity = {
      domain = "projection",
      algorithm = "blake3",
      hex = "1ae88fe233cb3df52e2e9438ae6b910c2d83b2e35025cfb99b3eda4f94370ece",
    },
    effect_summary_schema = "kamacite.effect-summary.v1",
    effect_summary_identity = {
      domain = "algebraic-effect",
      algorithm = "blake3",
      hex = "64471a04261cbb5f5d9e9b3df5ba9f2454145535355601760e779d2f6b0a46ac",
    },
    projection = "standards/effect-closure/fixtures/kamacite/effect-summary.json",
    projection_blake3 = "b3:7cdf5a386137b8145c39a55ff3a851e306f88efd2771d1ff0be8b0248e528291",
  },
  ambient_dispositions = [
    { effect = "alloc", disposition = "profile-local" },
    { effect = "panic", disposition = "denied" },
    { effect = "unsafe", disposition = "denied" },
    { effect = "clock", disposition = "denied" },
    { effect = "random", disposition = "denied" },
    { effect = "fs", disposition = "denied" },
    { effect = "net", disposition = "denied" },
    { effect = "spawn", disposition = "denied" },
    { effect = "lock", disposition = "denied" },
    { effect = "ffi", disposition = "denied" },
    { effect = "unknown", disposition = "denied" },
  ],
  semantic_emissions = [],
  non_claims = [
    "This mapping profile binds one exact frozen Kamacite compatibility projection; it does not prove complete Rust effects, semantic determinism, runtime reachability, handler correctness, replay truth, property satisfaction, or release eligibility.",
    "Octet does not canonicalize Kamacite data and this profile grants no consumer, publication, or runtime authority.",
  ],
} | contracts.KamaciteEffectProfile

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let Digest = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^b3:[0-9a-f]{64}$" value
) in
let TrueOnly = std.contract.from_predicate (fun value => value == true) in
let RequiredStageKinds = [
  "rust_source",
  "charon_extraction",
  "aeneas_translation",
  "generated_lean",
  "theorem",
  "lean_check",
] in
let CompleteStageInventory = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value == std.array.length RequiredStageKinds
  && std.array.all (fun kind =>
    std.array.any (fun stage => stage.stage_kind == kind) value
  ) RequiredStageKinds
) in
let Stage = {
  stage_kind | NonEmptyString,
  artifact | NonEmptyString,
  blake3 | Digest,
  evidence_role | OneOf ["recorded_only", "boundary", "property_candidate"],
} in
let Fixture = {
  schema_version | OneOf ["octet-aeneas-lean-valence-adapter/v1"],
  adapter_target | OneOf ["valence.stack-provenance.adapter-input"],
  provider_id | OneOf ["aeneas-lean"],
  octet_export | {
    artifact_directory | NonEmptyString,
    artifact_verification_status | OneOf ["valid"],
    artifact_verification_receipt | NonEmptyString,
    proof_receipt_id | NonEmptyString,
    proof_receipt_status | OneOf ["passed"],
    exported_after_verification | TrueOnly,
  },
  provider_profile | NonEmptyString,
  package_lock | NonEmptyString,
  stages | CompleteStageInventory | Array Stage,
  property_candidate | {
    theorem_id | NonEmptyString,
    requirement_id | NonEmptyString,
    receipt_artifact | NonEmptyString,
    evidence_tier | OneOf ["formal_proof"],
    owner | OneOf ["octet"],
  },
  non_claims | Array NonEmptyString,
} in
{
  schema_version = "octet-aeneas-lean-valence-adapter/v1",
  adapter_target = "valence.stack-provenance.adapter-input",
  provider_id = "aeneas-lean",
  octet_export = {
    artifact_directory = "standards/proof-rails/aeneas-lean/fixtures/valid/artifacts",
    artifact_verification_status = "valid",
    artifact_verification_receipt = "standards/proof-rails/aeneas-lean/fixtures/valid/verification-report.json",
    proof_receipt_id = "octet.aeneas_lean_proof_rails.fixture#rail-0",
    proof_receipt_status = "passed",
    exported_after_verification = true,
  },
  provider_profile = "standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/provider-profile.json",
  package_lock = "standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/package-lock.json",
  stages = [
    {
      stage_kind = "rust_source",
      artifact = "proofs/aeneas/source/src/lib.rs",
      blake3 = "b3:fc70d39d863627bf99ab6a2c13a0b798aac059dc2520d9d2e48f7ac599fb4502",
      evidence_role = "recorded_only",
    },
    {
      stage_kind = "charon_extraction",
      artifact = "proofs/aeneas/charon-options.json",
      blake3 = "b3:628826f12a1951a45e105272e54328586bbc1591b25d74bf8a5afc9453b5ff95",
      evidence_role = "boundary",
    },
    {
      stage_kind = "aeneas_translation",
      artifact = "proofs/aeneas/translation.json",
      blake3 = "b3:0e8c4e0040003a4488fc572c04e2a66509834857a4b2786ad443cfc90fb100c8",
      evidence_role = "recorded_only",
    },
    {
      stage_kind = "generated_lean",
      artifact = "proofs/aeneas/Llbc.lean",
      blake3 = "b3:7b6726a5a5a9271ffed325f25aae9c11871f3dc681a3a9d90f5f70ba4d2ad48f",
      evidence_role = "recorded_only",
    },
    {
      stage_kind = "theorem",
      artifact = "proofs/aeneas/theorem-statement.lean",
      blake3 = "b3:08e7c2d1fb1e21b2e7a174340a9f5b87c3acc634c591c6102624cd3a231f1125",
      evidence_role = "boundary",
    },
    {
      stage_kind = "lean_check",
      artifact = "proofs/octet.aeneas_lean_proof_rails.fixture-rail-0.receipt",
      blake3 = "b3:b0ce7c7d5673e67b0dc5fff7f54d351f4f58f2b70c986023a3933dee15bcf489",
      evidence_role = "property_candidate",
    },
  ],
  property_candidate = {
    theorem_id = "aeneas.fixture.safe_add",
    requirement_id = "octet.aeneas_lean_proof_rail.theorem_adequacy",
    receipt_artifact = "proofs/octet.aeneas_lean_proof_rails.fixture-rail-0.receipt",
    evidence_tier = "formal_proof",
    owner = "octet",
  },
  non_claims = [
    "Valence consumes this adapter input for identity and linkage only",
    "does not prove Aeneas or Charon translation soundness",
    "does not prove Lean or verifier soundness",
    "does not prove whole-program correctness",
    "does not establish release eligibility",
  ],
} | Fixture

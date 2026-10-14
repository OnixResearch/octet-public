# Typed contract for the opt-in Aeneas-to-Lean provider profile.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let PositiveNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value > 0
) in
let GitRevision = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match "^[0-9a-f]{40}$" value
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let FalseOnly = std.contract.from_predicate (fun value => value == false) in
let TrueOnly = std.contract.from_predicate (fun value => value == true) in
let AeneasRevision = OneOf ["bc2eb6f941369fa3c695e30598b5758b11e4f0de"] in
let CharonRevision = OneOf ["cb50ff16b9f1066b8a97dc06da704de2da2fa41c"] in
let MechanizedLlbcRevision = OneOf ["8a9b8bb3823ceceb47026837d23baea0934dfb27"] in
let MechanizedLlbcLicense = OneOf ["Apache-2.0"] in
let MechanizedLlbcSentinel = OneOf ["README.md:Mechanized LLBC"] in
let NonClaimArray = std.contract.from_predicate (fun value =>
  std.is_array value
  && std.array.length value > 0
  && std.array.all (fun item => std.is_string item && std.string.length item > 0) value
  && std.array.elem "does not prove Aeneas or Charon translation soundness" value
  && std.array.elem "does not prove Lean or verifier soundness" value
  && std.array.elem "does not prove whole-program correctness" value
) in
let CommandStage = {
  charon_argv | NonEmptyArray | Array NonEmptyString,
  aeneas_argv | NonEmptyArray | Array NonEmptyString,
  lean_argv | NonEmptyArray | Array NonEmptyString,
} in
let ExecutionLimits = {
  timeout_ms | PositiveNumber,
  stdout_max_bytes | PositiveNumber,
  stderr_max_bytes | PositiveNumber,
} in
let ExtractionPolicy = {
  has_errors | FalseOnly,
  skip_borrowck | FalseOnly,
  target_triple | NonEmptyString,
  pointer_width_bits | PositiveNumber,
  endianness | OneOf ["little", "big"],
  mir_start_stage | NonEmptyString,
  drop_policy | NonEmptyString,
  start_from | NonEmptyString,
  include_rules | Array NonEmptyString,
  exclude_rules | Array NonEmptyString,
  opaque_rules | Array NonEmptyString,
  transformation_settings | NonEmptyArray | Array NonEmptyString,
  serialization_format | NonEmptyString,
} in
let AssumptionPolicy = {
  external_models | Array NonEmptyString,
  axioms | Array NonEmptyString,
  requires_complete_declaration | TrueOnly,
} in
let SemanticReference = {
  repository | NonEmptyString,
  revision | MechanizedLlbcRevision,
  license | MechanizedLlbcLicense,
  source_sentinel | MechanizedLlbcSentinel,
  recorded_only | TrueOnly,
} in
{
  AeneasLeanProviderProfile = {
    schema_version | OneOf ["octet-aeneas-lean-provider-profile/v1"],
    profile_id | NonEmptyString,
    provider_id | OneOf ["aeneas-lean"],
    claim_scope | OneOf ["named-theorem-over-generated-model"],
    aeneas_release | NonEmptyString,
    aeneas_revision | AeneasRevision,
    aeneas_repository | NonEmptyString,
    charon_revision | CharonRevision,
    charon_repository | NonEmptyString,
    ocaml_toolchain | NonEmptyString,
    lean_toolchain | NonEmptyString,
    aeneas_lean_library | NonEmptyString,
    checked_source_surface | NonEmptyArray | Array NonEmptyString,
    command_argv | NonEmptyArray | Array NonEmptyString,
    stage_commands | CommandStage,
    limits | ExecutionLimits,
    environment_allowlist | NonEmptyArray | Array NonEmptyString,
    extraction_policy | ExtractionPolicy,
    assumption_policy | AssumptionPolicy,
    semantic_reference | SemanticReference,
    non_claims | NonClaimArray,
  },
}

# Contracts for the opt-in Kamacite effect-closure handoff.
#
# The mapping profile is Octet-authored policy. The effect-summary projection is
# a frozen compatibility fixture: Octet never reimplements Kamacite
# canonicalization and never owns the canonical Preserves bytes.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let Blake3Identity = NonEmptyString in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let EffectContract = OneOf ["alloc", "panic", "unsafe", "clock", "random", "fs", "net", "spawn", "lock", "ffi", "unknown"] in
let ProviderKindContract = OneOf ["cargo-package", "toolchain", "workspace"] in
let ProviderIdentityContract = {
  kind | ProviderKindContract,
  name | NonEmptyString,
  version | NonEmptyString,
  source | NonEmptyString,
  target | NonEmptyString,
  features | Array NonEmptyString,
} in
let EvidenceReferenceContract = {
  kind | NonEmptyString,
  locator | NonEmptyString,
  blake3 | Blake3Identity,
} in
let KamaciteIdentityContract = {
  domain | NonEmptyString,
  algorithm | OneOf ["blake3"],
  hex | NonEmptyString,
} in
let AmbientDispositionContract = {
  effect | EffectContract,
  disposition | OneOf ["profile-local", "denied"],
} in
let SemanticEmissionContract = {
  function_address | Blake3Identity,
  call_site_id | Blake3Identity | optional,
  provider | ProviderIdentityContract,
  operation_identity | KamaciteIdentityContract,
  evidence | EvidenceReferenceContract,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
let KamaciteBindingContract = {
  preserves_profile_schema | OneOf ["kamacite.preserves-profile.v1"],
  preserves_profile_identity | KamaciteIdentityContract,
  effect_summary_schema | OneOf ["kamacite.effect-summary.v1"],
  effect_summary_identity | KamaciteIdentityContract,
  projection | NonEmptyString,
  projection_blake3 | Blake3Identity,
} in
let KamaciteEffectProfileContract = {
  schema_version | OneOf ["octet-effect-closure-kamacite-profile/v1"],
  profile_id | OneOf ["deterministic-core"],
  mapping_id | NonEmptyString,
  kamacite | KamaciteBindingContract,
  ambient_dispositions | NonEmptyArray | Array AmbientDispositionContract,
  semantic_emissions | Array SemanticEmissionContract,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
let KamaciteOperationContract = {
  operation_identity | KamaciteIdentityContract,
  effect_family_identity | KamaciteIdentityContract,
  display_name | NonEmptyString,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
let KamaciteEffectSummaryProjectionContract = {
  schema_version | OneOf ["octet-kamacite-effect-summary-projection/v1"],
  kamacite_schema_version | OneOf ["kamacite.effect-summary.v1"],
  preserves_profile_schema | OneOf ["kamacite.preserves-profile.v1"],
  preserves_profile_identity | KamaciteIdentityContract,
  effect_summary_identity | KamaciteIdentityContract,
  effect_summary_closed | Bool,
  operations | Array KamaciteOperationContract,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
{
  KamaciteIdentity = KamaciteIdentityContract,
  AmbientDisposition = AmbientDispositionContract,
  SemanticEmission = SemanticEmissionContract,
  KamaciteEffectProfile = KamaciteEffectProfileContract,
  KamaciteOperation = KamaciteOperationContract,
  KamaciteEffectSummaryProjection = KamaciteEffectSummaryProjectionContract,
}

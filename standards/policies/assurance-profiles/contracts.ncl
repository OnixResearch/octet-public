# Closed contracts for machine-readable assurance profiles.
let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > 0
) in
let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > 0
) in
let OneOf = fun allowed => std.contract.from_predicate (fun value =>
  std.is_string value && std.array.elem value allowed
) in
let SchemaVersion = OneOf ["octet-assurance-profile/v1"] in
let AssurancePlaneContract = OneOf [
  "construction",
  "semantic-design",
  "verification",
  "effect-control",
  "provenance-readiness",
] in
let VerificationRoleContract = OneOf [
  "static-lint",
  "policy-check",
  "test",
  "canonical-identity",
  "formal-proof",
  "model-check",
  "evidence-linkage",
  "lifecycle-gate",
] in
let SubjectScopeContract = {
  repository | NonEmptyString,
  component | NonEmptyString,
  source_paths | NonEmptyArray | Array NonEmptyString,
} in
let AssumptionContract = {
  assumption_id | NonEmptyString,
  owner | NonEmptyString,
  statement | NonEmptyString,
  validation | NonEmptyString,
} in
let TrustedBaseElementContract = {
  component_id | NonEmptyString,
  owner | NonEmptyString,
  responsibility | NonEmptyString,
  failure_impact | NonEmptyString,
} in
let EvidenceRequirementContract = {
  evidence_id | NonEmptyString,
  producer | NonEmptyString,
  artifact_path | NonEmptyString,
  artifact_kind | NonEmptyString,
  verification_role | VerificationRoleContract,
  verification_scope | NonEmptyString,
  validation_command | NonEmptyString,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
let AssuranceMechanismContract = {
  mechanism_id | NonEmptyString,
  owner | NonEmptyString,
  assurance_plane | AssurancePlaneContract,
  protected_property | NonEmptyString,
  assumption_ids | NonEmptyArray | Array NonEmptyString,
  trusted_base_ids | NonEmptyArray | Array NonEmptyString,
  evidence_ids | NonEmptyArray | Array NonEmptyString,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
let AssuranceProfileContract = {
  schema_version | SchemaVersion,
  profile_id | NonEmptyString,
  summary | NonEmptyString,
  scope | SubjectScopeContract,
  assumptions | NonEmptyArray | Array AssumptionContract,
  trusted_computing_base | NonEmptyArray | Array TrustedBaseElementContract,
  mechanisms | NonEmptyArray | Array AssuranceMechanismContract,
  required_evidence | NonEmptyArray | Array EvidenceRequirementContract,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in
let has_duplicates = fun values =>
  std.array.length values != std.array.length (std.array.dedup values)
in
let all_resolve = fun references known_ids =>
  std.array.all (fun reference => std.array.elem reference known_ids) references
in
let any_mechanism_references = fun field reference mechanisms =>
  std.array.any (fun mechanism => std.array.elem reference (field mechanism)) mechanisms
in
let profile_cross_fields_valid = fun profile =>
  let assumption_ids = profile.assumptions |> std.array.map (fun assumption => assumption.assumption_id) in
  let trusted_base_ids = profile.trusted_computing_base |> std.array.map (fun component => component.component_id) in
  let mechanism_ids = profile.mechanisms |> std.array.map (fun mechanism => mechanism.mechanism_id) in
  let evidence_ids = profile.required_evidence |> std.array.map (fun evidence => evidence.evidence_id) in
  !has_duplicates assumption_ids
  && !has_duplicates trusted_base_ids
  && !has_duplicates mechanism_ids
  && !has_duplicates evidence_ids
  && profile.mechanisms
    |> std.array.all (fun mechanism =>
      all_resolve mechanism.assumption_ids assumption_ids
      && all_resolve mechanism.trusted_base_ids trusted_base_ids
      && all_resolve mechanism.evidence_ids evidence_ids
    )
  && profile.assumptions
    |> std.array.all (fun assumption =>
      any_mechanism_references (fun mechanism => mechanism.assumption_ids) assumption.assumption_id profile.mechanisms
    )
  && profile.trusted_computing_base
    |> std.array.all (fun component =>
      any_mechanism_references (fun mechanism => mechanism.trusted_base_ids) component.component_id profile.mechanisms
    )
  && profile.required_evidence
    |> std.array.all (fun evidence =>
      any_mechanism_references (fun mechanism => mechanism.evidence_ids) evidence.evidence_id profile.mechanisms
    )
in
let AssuranceProfileRulesContract = std.contract.from_predicate profile_cross_fields_valid in
{
  AssurancePlane = AssurancePlaneContract,
  VerificationRole = VerificationRoleContract,
  SubjectScope = SubjectScopeContract,
  Assumption = AssumptionContract,
  TrustedBaseElement = TrustedBaseElementContract,
  EvidenceRequirement = EvidenceRequirementContract,
  AssuranceMechanism = AssuranceMechanismContract,
  AssuranceProfile = AssuranceProfileContract,
  AssuranceProfileRules = AssuranceProfileRulesContract,
}

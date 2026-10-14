# Typed proof-claim catalog contracts.
let NonEmptyString = std.contract.from_predicate (fun value => std.is_string value && std.string.length value > 0) in
let OneOf = fun options => std.contract.from_predicate (fun value => std.is_string value && std.array.elem value options) in
let EvidenceTier = OneOf [
  "formal_proof",
  "manual_review_required",
  "policy_check",
  "static_lint",
  "transcript_assertion",
] in
let ClaimStatus = OneOf [
  "active",
  "deprecated",
] in
let AliasStatus = OneOf [
  "alias",
  "deprecated",
] in
let ProofClaimContract = {
  claim_id | NonEmptyString,
  source_requirement_or_policy_id | NonEmptyString,
  checked_scope | NonEmptyString,
  expected_evidence_tier | EvidenceTier,
  allowed_proof_or_property_ids | Array NonEmptyString,
  required_artifacts | Array NonEmptyString,
  documentation_anchors | Array NonEmptyString,
  owner_surface | NonEmptyString,
  status | ClaimStatus,
  non_claims | Array NonEmptyString,
} in
let ProofClaimAliasContract = {
  alias_id | NonEmptyString,
  target_claim_id | NonEmptyString,
  status | AliasStatus,
  rationale | NonEmptyString,
} in
{
  ProofClaimCatalog = {
    schema_version | std.contract.from_predicate (fun value => value == "octet-proof-claim-catalog/v1"),
    claims | Array ProofClaimContract,
    aliases | Array ProofClaimAliasContract | default = [],
  },
  ProofClaim = ProofClaimContract,
  ProofClaimAlias = ProofClaimAliasContract,
}

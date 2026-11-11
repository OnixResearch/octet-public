# Nickel contracts for Valence stack provenance metadata.
#
# These contracts harden static metadata only. Rust remains responsible for
# reading binary bytes, computing BLAKE3, and comparing measured identity.

let EmptyLength = 0 in
let BLAKE3_HEX_DIGITS = 64 in
let BLAKE3_PATTERN = "^(b3:)?[0-9a-fA-F]{64}$" in
let Constants = {
  Blake3HexDigits = BLAKE3_HEX_DIGITS,
  OctetExportSchema = "octet-valence-provenance-export/v1",
  StackSidecarSchema = "valence.stack-provenance-sidecar.v1",
  ProvenanceChainSchema = "valence.provenance-chain.v1",
  StackEvidenceRole = "stack-provenance-trace",
  ClaimScope = "identity-linkage-sidecar",
  ValidStatus = "valid",
  FormalProofTier = "formal_proof",
  ManualReviewTier = "manual_review_required",
  ExternalTrellisOrigin = "external_trellis_import",
  StackNonClaim = "stack provenance proves identity and linkage only; it does not prove behavioral correctness, semantic equivalence, whole-program safety, or requirement satisfaction",
  BuiltBinaryThreadNonClaim = "built-binary thread verification proves measured-byte identity and linkage only; it does not prove behavioral correctness, semantic equivalence, requirement satisfaction, release eligibility, verifier soundness, or whole-program safety",
  ProvenanceBoundary = "provenance chain proves identity and linkage only; it does not prove behavioral correctness, semantic equivalence, or that the binary satisfies the requirement",
} in

let NonEmptyString = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.length value > EmptyLength
) in

let EmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value == EmptyLength
) in

let NonEmptyArray = std.contract.from_predicate (fun value =>
  std.is_array value && std.array.length value > EmptyLength
) in

let NonNegativeNumber = std.contract.from_predicate (fun value =>
  std.is_number value && value >= EmptyLength
) in

let Blake3Hex = std.contract.from_predicate (fun value =>
  std.is_string value && std.string.is_match BLAKE3_PATTERN value
) in

let OptionalBlake3Hex = std.contract.from_predicate (fun value =>
  value == null || (std.is_string value && std.string.is_match BLAKE3_PATTERN value)
) in

let OptionalString = std.contract.from_predicate (fun value =>
  value == null || std.is_string value
) in

let OctetExportSchemaContract = std.contract.from_predicate (fun value => value == Constants.OctetExportSchema) in
let StackSidecarSchemaContract = std.contract.from_predicate (fun value => value == Constants.StackSidecarSchema) in
let ProvenanceChainSchemaContract = std.contract.from_predicate (fun value => value == Constants.ProvenanceChainSchema) in
let ClaimScopeContract = std.contract.from_predicate (fun value => value == Constants.ClaimScope) in
let ValidStatusContract = std.contract.from_predicate (fun value => value == Constants.ValidStatus) in
let ProvenanceBoundaryContract = std.contract.from_predicate (fun value => value == Constants.ProvenanceBoundary) in

let contains_string = fun values expected =>
  std.array.any (fun value => value == expected) values
in

let has_required_stack_non_claim = fun value =>
  contains_string value.non_claims Constants.StackNonClaim
in

let has_required_built_binary_non_claim = fun value =>
  contains_string value.non_claims Constants.BuiltBinaryThreadNonClaim
in

let has_no_overclaim_text = fun claims =>
  std.array.all
    (fun claim =>
      !(std.string.is_match ".*proves behavioral correctness.*" claim)
      && !(std.string.is_match ".*proves semantic equivalence.*" claim)
      && !(std.string.is_match ".*proves requirement satisfaction.*" claim)
      && !(std.string.is_match ".*proves release eligibility.*" claim)
      && !(std.string.is_match ".*proves whole-program safety.*" claim)
      && !(std.string.is_match ".*certifies release eligibility.*" claim)
      && !(std.string.is_match ".*guarantees behavioral correctness.*" claim)
      && !(std.string.is_match ".*guarantees whole-program safety.*" claim))
    claims
in

let artifact_name_present = fun artifacts name =>
  std.array.any (fun artifact => artifact.artifact == name) artifacts
in

let record_parents_known = fun export record =>
  artifact_name_present export.artifacts record.artifact
  && std.array.all (fun parent => artifact_name_present export.artifacts parent) record.parent_artifacts
in

let parent_links_known = fun export =>
  std.array.all (fun record => record_parents_known export record) export.provenance_records
in

let proof_card_not_promoted = fun card =>
  !(card.evidence_origin == Constants.ExternalTrellisOrigin && card.evidence_tier == Constants.FormalProofTier)
in

let provenance_record_not_promoted = fun record =>
  !(record.human_action != null && record.evidence_tier == Constants.FormalProofTier)
in

let manual_review_has_action = fun record =>
  !(record.evidence_tier == Constants.ManualReviewTier && record.human_action == null)
in

let export_tiers_bounded = fun export =>
  std.array.all proof_card_not_promoted export.proof_cards
  && std.array.all provenance_record_not_promoted export.provenance_records
  && std.array.all manual_review_has_action export.provenance_records
in

let sidecar_links_match_binary = fun sidecar =>
  std.array.all (fun link => link.binary_hash == sidecar.binary_hash) sidecar.chain.links
in

let sidecar_external_evidence_record_valid = fun record =>
  if record.role == Constants.StackEvidenceRole then
    record.schema == Constants.StackSidecarSchema
    && record.claim_scope == Constants.ClaimScope
    && contains_string record.non_claims Constants.StackNonClaim
  else
    true
in

let has_stack_external_evidence_role = fun manifest =>
  std.array.any (fun record => record.role == Constants.StackEvidenceRole) manifest.external_evidence
in

let sidecar_external_evidence_records_valid = fun manifest =>
  std.array.all sidecar_external_evidence_record_valid manifest.external_evidence
in

let thread_static_metadata_aligned = fun input =>
  input.binary_locator.expected_digest_blake3 == input.sidecar.binary_hash
  && input.sidecar.octet_export_artifact_dir == input.octet_export.artifact_dir
  && input.sidecar.parent_artifacts == input.octet_export.artifacts
in

let ArtifactIdentityContract = {
  artifact | NonEmptyString,
  blake3 | Blake3Hex,
} in

let ProvenanceLinkContract = {
  source_id | NonEmptyString,
  function_object_id | NonEmptyString,
  requirement_ids | NonEmptyArray | Array NonEmptyString,
  artifact_hash | OptionalBlake3Hex | default = null,
  binary_hash | OptionalBlake3Hex | default = null,
} in

let ProvenanceChainContract = {
  schema | ProvenanceChainSchemaContract,
  links | NonEmptyArray | Array ProvenanceLinkContract,
  boundary | ProvenanceBoundaryContract,
} in

let OctetProvenanceRecordContract = {
  source_id | NonEmptyString,
  checker_id | NonEmptyString,
  evidence_tier | NonEmptyString,
  artifact | NonEmptyString,
  blake3 | Blake3Hex,
  checked_scope | NonEmptyString,
  verifier_identity | OptionalString | default = null,
  proof_artifact_identity | OptionalBlake3Hex | default = null,
  human_action | OptionalString | default = null,
  parent_artifacts | Array NonEmptyString | default = [],
} in

let OctetProofCardContract = {
  proof_or_property_id | NonEmptyString,
  source_requirement_or_policy_id | NonEmptyString,
  policy_id | NonEmptyString,
  rail_index | NonNegativeNumber,
  receipt_path | NonEmptyString,
  artifact_blake3_identities | NonEmptyArray | Array ArtifactIdentityContract,
  evidence_tier | NonEmptyString,
  evidence_origin | NonEmptyString,
  status | ValidStatusContract,
  non_claims | NonEmptyArray | Array NonEmptyString,
  diagnostics | EmptyArray | Array NonEmptyString | default = [],
} in

let OctetValenceProvenanceExportShapeContract = {
  schema_version | OctetExportSchemaContract,
  artifact_dir | NonEmptyString,
  verification_status | ValidStatusContract,
  artifacts | NonEmptyArray | Array ArtifactIdentityContract,
  provenance_records | NonEmptyArray | Array OctetProvenanceRecordContract,
  proof_cards | Array OctetProofCardContract | default = [],
  diagnostics | EmptyArray | Array NonEmptyString | default = [],
  non_claims | NonEmptyArray | Array NonEmptyString,
} in

let OctetValenceProvenanceExportContract = std.contract.from_predicate (fun export =>
  parent_links_known export
  && export_tiers_bounded export
  && has_required_stack_non_claim export
  && has_no_overclaim_text export.non_claims
  && std.array.all (fun card => has_no_overclaim_text card.non_claims) export.proof_cards
) in

let StackProvenanceSidecarShapeContract = {
  schema | StackSidecarSchemaContract,
  claim_scope | ClaimScopeContract,
  binary_hash | Blake3Hex,
  octet_export_artifact_dir | NonEmptyString,
  chain | ProvenanceChainContract,
  parent_artifacts | NonEmptyArray | Array ArtifactIdentityContract,
  diagnostics | EmptyArray | Array NonEmptyString | default = [],
  non_claims | NonEmptyArray | Array NonEmptyString,
} in

let StackProvenanceSidecarContract = std.contract.from_predicate (fun sidecar =>
  has_required_stack_non_claim sidecar
  && has_no_overclaim_text sidecar.non_claims
  && sidecar_links_match_binary sidecar
) in

let MantleExternalEvidenceRecordContract = {
  role | NonEmptyString,
  schema | NonEmptyString,
  relative_path | NonEmptyString,
  digest_blake3 | Blake3Hex,
  claim_scope | NonEmptyString,
  non_claims | Array NonEmptyString | default = [],
} in

let MantleBinaryIdentityContract = {
  relative_path | NonEmptyString,
  digest_blake3 | Blake3Hex,
} in

let MantleProvenanceCoverageViewContract = {
  binary_hash | Blake3Hex,
} in

let MantleReleaseManifestViewShapeContract = {
  external_evidence | NonEmptyArray | Array MantleExternalEvidenceRecordContract,
  binaries | NonEmptyArray | Array MantleBinaryIdentityContract,
  provenance_coverage | MantleProvenanceCoverageViewContract,
} in

let MantleReleaseManifestViewContract = std.contract.from_predicate (fun manifest =>
  has_stack_external_evidence_role manifest && sidecar_external_evidence_records_valid manifest
) in

let BinaryArtifactLocatorContract = {
  path | NonEmptyString,
  expected_digest_blake3 | Blake3Hex,
} in

let BuiltBinaryThreadInputShapeContract = {
  binary_locator | BinaryArtifactLocatorContract,
  octet_export | OctetValenceProvenanceExportShapeContract | OctetValenceProvenanceExportContract,
  sidecar | StackProvenanceSidecarShapeContract | StackProvenanceSidecarContract,
  mantle_manifest | MantleReleaseManifestViewShapeContract | MantleReleaseManifestViewContract,
} in

let BuiltBinaryThreadInputContract = std.contract.from_predicate thread_static_metadata_aligned in

let BuiltBinaryThreadReportShapeContract = {
  valid | Bool,
  issue_codes | Array NonEmptyString,
  binary_hash | Blake3Hex,
  sidecar_digest_blake3 | Blake3Hex,
  source_ids | Array NonEmptyString,
  requirement_ids | Array NonEmptyString,
  artifact_identities | Array ArtifactIdentityContract,
  non_claims | NonEmptyArray | Array NonEmptyString,
} in

let BuiltBinaryThreadReportContract = std.contract.from_predicate (fun report =>
  has_required_built_binary_non_claim report && has_no_overclaim_text report.non_claims
) in

{
  constants = Constants,
  ArtifactIdentity = ArtifactIdentityContract,
  ProvenanceLink = ProvenanceLinkContract,
  ProvenanceChain = ProvenanceChainContract,
  OctetProvenanceRecord = OctetProvenanceRecordContract,
  OctetProofCard = OctetProofCardContract,
  OctetValenceProvenanceExportShape = OctetValenceProvenanceExportShapeContract,
  OctetValenceProvenanceExport = OctetValenceProvenanceExportContract,
  StackProvenanceSidecarShape = StackProvenanceSidecarShapeContract,
  StackProvenanceSidecar = StackProvenanceSidecarContract,
  MantleExternalEvidenceRecord = MantleExternalEvidenceRecordContract,
  MantleBinaryIdentity = MantleBinaryIdentityContract,
  MantleProvenanceCoverageView = MantleProvenanceCoverageViewContract,
  MantleReleaseManifestViewShape = MantleReleaseManifestViewShapeContract,
  MantleReleaseManifestView = MantleReleaseManifestViewContract,
  BinaryArtifactLocator = BinaryArtifactLocatorContract,
  BuiltBinaryThreadInputShape = BuiltBinaryThreadInputShapeContract,
  BuiltBinaryThreadInput = BuiltBinaryThreadInputContract,
  BuiltBinaryThreadReportShape = BuiltBinaryThreadReportShapeContract,
  BuiltBinaryThreadReport = BuiltBinaryThreadReportContract,
}

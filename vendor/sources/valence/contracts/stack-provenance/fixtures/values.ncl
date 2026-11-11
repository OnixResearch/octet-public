let Contracts = import "../contracts.ncl" in
let C = Contracts.constants in
let BinaryHashValue = "1111111111111111111111111111111111111111111111111111111111111111" in
let StatusHashValue = "2222222222222222222222222222222222222222222222222222222222222222" in
let ResultsHashValue = "3333333333333333333333333333333333333333333333333333333333333333" in
let SidecarDigestValue = "4444444444444444444444444444444444444444444444444444444444444444" in
let StatusArtifactValue = {
  artifact = "status.json",
  blake3 = StatusHashValue,
} in
let ResultsArtifactValue = {
  artifact = "results.jsonl",
  blake3 = ResultsHashValue,
} in
let ProvenanceRecordValue = {
  source_id = "octet.standards.formal-proof",
  checker_id = "octet-artifact-verifier",
  evidence_tier = C.FormalProofTier,
  artifact = "status.json",
  blake3 = StatusHashValue,
  checked_scope = "standards/proof-claims/catalog.ncl",
  verifier_identity = "octet-verus-rail",
  proof_artifact_identity = ResultsHashValue,
  parent_artifacts = ["results.jsonl"],
} in
let ProofCardValue = {
  proof_or_property_id = "octet.no-unwrap-proof",
  source_requirement_or_policy_id = "r[octet.policy.no_unwrap]",
  policy_id = "octet.standards.formal-proof",
  rail_index = 0,
  receipt_path = "status.json",
  artifact_blake3_identities = [StatusArtifactValue],
  evidence_tier = C.FormalProofTier,
  evidence_origin = "octet_verified_receipt",
  status = C.ValidStatus,
  non_claims = [C.StackNonClaim],
  diagnostics = [],
} in
let OctetExportValue = {
  schema_version = C.OctetExportSchema,
  artifact_dir = "target/octet-artifact",
  verification_status = C.ValidStatus,
  artifacts = [StatusArtifactValue, ResultsArtifactValue],
  provenance_records = [ProvenanceRecordValue],
  proof_cards = [ProofCardValue],
  diagnostics = [],
  non_claims = [C.StackNonClaim],
} in
let SidecarValue = {
  schema = C.StackSidecarSchema,
  claim_scope = C.ClaimScope,
  binary_hash = BinaryHashValue,
  octet_export_artifact_dir = "target/octet-artifact",
  chain = {
    schema = C.ProvenanceChainSchema,
    links = [{
      source_id = "octet.standards.formal-proof",
      function_object_id = "status.json",
      requirement_ids = ["r[octet.policy.no_unwrap]"],
      artifact_hash = StatusHashValue,
      binary_hash = BinaryHashValue,
    }],
    boundary = C.ProvenanceBoundary,
  },
  parent_artifacts = [StatusArtifactValue, ResultsArtifactValue],
  diagnostics = [],
  non_claims = [C.StackNonClaim, C.ProvenanceBoundary],
} in
let MantleManifestValue = {
  binaries = [{
    relative_path = "bin/valence-fixture",
    digest_blake3 = BinaryHashValue,
  }],
  external_evidence = [{
    role = C.StackEvidenceRole,
    schema = C.StackSidecarSchema,
    relative_path = "external-evidence/stack-provenance.json",
    digest_blake3 = SidecarDigestValue,
    claim_scope = C.ClaimScope,
    non_claims = [C.StackNonClaim],
  }],
  provenance_coverage = {
    binary_hash = BinaryHashValue,
  },
} in
let BinaryLocatorValue = {
  path = "bin/valence-fixture",
  expected_digest_blake3 = BinaryHashValue,
} in
let ThreadInputValue = {
  binary_locator = BinaryLocatorValue,
  octet_export = OctetExportValue,
  sidecar = SidecarValue,
  mantle_manifest = MantleManifestValue,
} in
let ReportValue = {
  valid = true,
  issue_codes = [],
  binary_hash = BinaryHashValue,
  sidecar_digest_blake3 = SidecarDigestValue,
  source_ids = ["octet.standards.formal-proof"],
  requirement_ids = ["r[octet.policy.no_unwrap]"],
  artifact_identities = [StatusArtifactValue, ResultsArtifactValue],
  non_claims = [C.BuiltBinaryThreadNonClaim, C.StackNonClaim],
} in
{
  BinaryHash = BinaryHashValue,
  StatusHash = StatusHashValue,
  ResultsHash = ResultsHashValue,
  SidecarDigest = SidecarDigestValue,
  StatusArtifact = StatusArtifactValue,
  ResultsArtifact = ResultsArtifactValue,
  ProvenanceRecord = ProvenanceRecordValue,
  ProofCard = ProofCardValue,
  OctetExport = OctetExportValue,
  Sidecar = SidecarValue,
  MantleManifest = MantleManifestValue,
  BinaryLocator = BinaryLocatorValue,
  ThreadInput = ThreadInputValue,
  Report = ReportValue,
}

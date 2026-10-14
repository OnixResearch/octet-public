let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-provenance-record/v1",
  source_id = "octet.runner.artifact-bundle",
  checker_id = "cargo-octet",
  evidence_tier = "manual_review_required",
  artifact_locator = "artifacts/octet/status.json",
  blake3 = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
  checked_scope = "profile:workspace-metadata",
  human_action = "review runner status before release",
} | contracts.ProvenanceRecordV1

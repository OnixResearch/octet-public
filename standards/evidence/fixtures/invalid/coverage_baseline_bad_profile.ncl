let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-coverage-receipt/v1",
  profile_id = "ambient-coverage",
  profile_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  command_argv = ["cargo", "llvm-cov"],
  current = {},
  baseline = {},
  current_envelope_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  baseline_envelope_identity = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  projection_identity = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  comparison = {},
  evidence_tier = "transcript_assertion",
  non_claims = ["bounded coverage observation"],
} | contracts.CoverageBaselineReceiptDetailsV1

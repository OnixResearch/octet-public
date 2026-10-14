let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-coverage-receipt/v1",
  profile_id = "cargo-llvm-cov-octet-lib",
  profile_identity = "b3:e221f5d35528e6979bb44ab58cc5e486109799cf99fb80fc4d85ab77a7082634",
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

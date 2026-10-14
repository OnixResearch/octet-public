let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-benchmark-receipt/v1",
  profile_id = "octet-metadata-benchmark",
  profile_identity = "b3:64e4b8163b3c89d8143d4f6ff07230d33b2b2a9b7bcab35bf45325027a20cd47",
  provider_identity = "cargo-octet@0.1.0:pinned",
  command_argv = ["cargo", "metadata"],
  baseline_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  candidate_identity = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  allocation_identity = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  copy_identity = "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  oracle_identity = "b3:eeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeeee",
  findings_identity = "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
  projection_identity = "b3:1111111111111111111111111111111111111111111111111111111111111111",
  bounded_log_identity = "b3:2222222222222222222222222222222222222222222222222222222222222222",
  projection = {}, public_claims = [], evidence_tier = "transcript_assertion",
  non_claims = ["bounded benchmark observation"],
} | contracts.BenchmarkEvidenceReceiptDetailsV1

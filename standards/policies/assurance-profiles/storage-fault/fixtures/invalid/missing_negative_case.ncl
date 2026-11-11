let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-storage-fault-assurance/v1",
  profile_id = "missing-negative-case",
  source_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  candidate_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  required_cases = [
    { case_id = "clean-write-sync", kind = "positive" },
  ],
  producer_receipt = {
    path = "fixtures/evidence/current/producer.json",
    blake3 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  },
  cairn_receipt = {
    path = "fixtures/evidence/current/cairn.json",
    blake3 = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  },
  non_claims = ["This fixture must not export: at least one negative case is required."],
} | contracts.Profile | contracts.ProfileRules

let contracts = import "../../contracts.ncl" in
{
  schema_version = "octet-storage-fault-assurance/v1",
  profile_id = "bad-identity",
  source_blake3 = "not-a-blake3-identity",
  candidate_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  required_cases = [
    { case_id = "flush-error", kind = "negative" },
  ],
  producer_receipt = {
    path = "fixtures/evidence/current/producer.json",
    blake3 = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  },
  cairn_receipt = {
    path = "fixtures/evidence/current/cairn.json",
    blake3 = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  },
  non_claims = ["This fixture must not export: identities are exact 64-character BLAKE3 values."],
} | contracts.Profile | contracts.ProfileRules

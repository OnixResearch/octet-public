let contracts = import "contracts.ncl" in
{
  schema_version = "octet-storage-fault-assurance/v1",
  profile_id = "octet-storage-fault-example",
  source_blake3 = "0000000000000000000000000000000000000000000000000000000000000000",
  candidate_blake3 = "1111111111111111111111111111111111111111111111111111111111111111",
  required_cases = [
    { case_id = "clean-write-sync", kind = "positive" },
    { case_id = "flush-error", kind = "negative" },
    { case_id = "uncertain-commit", kind = "negative" },
  ],
  producer_receipt = {
    path = "evidence/storage-fault/producer-receipt.json",
    blake3 = "2222222222222222222222222222222222222222222222222222222222222222",
  },
  cairn_receipt = {
    path = "evidence/storage-fault/cairn-receipt.json",
    blake3 = "3333333333333333333333333333333333333333333333333333333333333333",
  },
  non_claims = [
    "This profile declares required saved evidence identities; it does not report that any receipt exists, matches, or passes.",
    "Passing this validation links saved producer and Cairn receipts only; it does not prove filesystem, device, hardware, or release durability.",
  ],
} | contracts.Profile | contracts.ProfileRules

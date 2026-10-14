let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-cargo-mutants-receipt/v1",
  profile_id = "ambient-mutants",
  profile_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  provider_identity = "cargo-mutants@27.0.0:pinned",
  rust_toolchain = "nightly-2026-03-21",
  command_argv = ["cargo", "mutants"],
  source_manifest_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  native_outcomes_identity = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  native_inventory_identity = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  projection_identity = "b3:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
  baseline = {}, mutants = [], outcome_counts = {}, gate_status = "passed",
  evidence_tier = "transcript_assertion", non_claims = ["bounded mutation observation"],
} | contracts.CargoMutantsReceiptDetailsV1

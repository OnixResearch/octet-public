let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-cargo-mutants-receipt/v1",
  profile_id = "cargo-mutants-octet-metadata",
  profile_identity = "b3:a40fc0726aea38932f42d6096a84846778e832472767b3c3129de4f7e69d9176",
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

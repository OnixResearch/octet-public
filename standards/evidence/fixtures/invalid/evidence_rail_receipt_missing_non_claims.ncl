let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-evidence-rail-receipt/v1",
  rail_id = "kani-model-check",
  checker_id = "cargo-kani",
  mode = "import",
  command_argv = ["cargo", "kani"],
  workspace_scope = "fixture-workspace",
  checked_rule_families = ["harness-identity"],
  status = "passed",
  bounded_log_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  imported_input_identity = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
} | contracts.EvidenceRailReceiptV1

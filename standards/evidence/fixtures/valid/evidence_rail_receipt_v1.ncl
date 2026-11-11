let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-evidence-rail-receipt/v1",
  rail_id = "kani-model-check",
  checker_id = "cargo-kani",
  mode = "import",
  command_argv = ["cargo", "kani"],
  workspace_scope = "fixture-workspace",
  checked_rule_families = ["harness-identity", "toolchain-identity"],
  status = "passed",
  finding_counts = { warning = 1 },
  bounded_log_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  imported_input_identity = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  tool_exit_code = 0,
  non_claims = ["Kani evidence is scoped to named harnesses"],
  findings = [
    {
      family = "harness-identity",
      severity = "warning",
      message = "harness reached its unwind bound",
    },
  ],
  property_classes = ["encode-decode"],
} | contracts.EvidenceRailReceiptV1

let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "cargo-llvm-cov-summary/v1",
  summary = {
    line_percent = 92.5,
    branch_percent = 81.0,
    function_percent = 100,
  },
  regressions = [],
  threshold_failures = [],
} | contracts.CoverageBaselineSummaryV1

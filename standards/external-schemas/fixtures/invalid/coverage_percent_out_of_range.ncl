let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "cargo-llvm-cov-summary/v1",
  summary = {
    line_percent = 150,
  },
  regressions = [],
  threshold_failures = [],
} | contracts.CoverageBaselineSummaryV1

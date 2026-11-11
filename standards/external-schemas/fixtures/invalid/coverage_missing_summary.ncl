let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "cargo-llvm-cov-summary/v1",
  regressions = [],
} | contracts.CoverageBaselineSummaryV1

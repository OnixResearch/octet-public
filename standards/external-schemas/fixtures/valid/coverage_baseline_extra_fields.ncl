let contracts = import "../../contracts/generated.ncl" in
{
  summary = {
    line_percent = 75,
    region_percent = 70,
  },
  regressions = [
    {
      metric = "line_percent",
      delta = -1.5,
    },
  ],
  generated_by = "cargo-llvm-cov",
} | contracts.CoverageBaselineSummaryV1

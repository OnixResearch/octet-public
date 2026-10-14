# External schema contract library entry point.
let contracts = import "contracts/generated.ncl" in
{
  metadata = contracts.metadata,
  CargoDenyReportV1 = contracts.CargoDenyReportV1,
  CoverageBaselineSummaryV1 = contracts.CoverageBaselineSummaryV1,
}

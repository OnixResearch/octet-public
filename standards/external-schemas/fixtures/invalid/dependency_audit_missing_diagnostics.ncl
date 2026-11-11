let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "cargo-deny-diagnostics/v1",
} | contracts.CargoDenyReportV1

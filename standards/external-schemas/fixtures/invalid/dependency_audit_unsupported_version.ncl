let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "cargo-deny-diagnostics/v0",
  diagnostics = [],
} | contracts.CargoDenyReportV1

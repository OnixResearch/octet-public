let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "cargo-deny-diagnostics/v1",
  diagnostics = [
    {
      category = "licenses",
      severity = "error",
      message = "license denied",
    },
  ],
} | contracts.CargoDenyReportV1

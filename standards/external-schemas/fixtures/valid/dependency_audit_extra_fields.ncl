let contracts = import "../../contracts/generated.ncl" in
{
  diagnostics = [
    {
      category = "advisories",
      severity = "warning",
      message = "unmaintained crate",
      code = "unmaintained",
    },
  ],
  tool_version = "0.16.1",
} | contracts.CargoDenyReportV1

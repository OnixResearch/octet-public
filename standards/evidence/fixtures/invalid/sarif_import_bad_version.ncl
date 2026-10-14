let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-sarif-intake-receipt/v1",
  profile_id = "codeql-local",
  profile_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  raw_report_identity = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  projection_identity = "b3:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
  source_driver = "CodeQL",
  supported_version = "2.0.0",
  findings = [],
  non_claims = ["bounded external observations"],
} | contracts.SarifImportReceiptDetailsV1

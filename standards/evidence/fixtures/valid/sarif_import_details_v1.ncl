let contracts = import "../../contracts/generated.ncl" in
{
  schema_version = "octet-sarif-intake-receipt/v1",
  profile_id = "codeql-local",
  profile_identity = "b3:81a482fe3b807b973898bd7454e73857605f6c5e48e5092d1d6e47b3fe7a9ce7",
  raw_report_identity = "b3:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
  projection_identity = "b3:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
  source_driver = "CodeQL",
  supported_version = "2.1.0",
  findings = [],
  non_claims = ["bounded external observations"],
} | contracts.SarifImportReceiptDetailsV1

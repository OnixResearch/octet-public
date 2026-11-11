let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
(
  Values.OctetExport & {
    schema_version | force = "octet-valence-provenance-export/v0",
  }
)
| Contracts.OctetValenceProvenanceExportShape
| Contracts.OctetValenceProvenanceExport

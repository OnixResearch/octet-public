let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
let StaleRecord = Values.ProvenanceRecord & {
  parent_artifacts | force = ["missing-parent.json"],
} in
(
  Values.OctetExport & {
    provenance_records | force = [StaleRecord],
  }
)
| Contracts.OctetValenceProvenanceExportShape
| Contracts.OctetValenceProvenanceExport

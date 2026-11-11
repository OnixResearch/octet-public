let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
Values.OctetExport
| Contracts.OctetValenceProvenanceExportShape
| Contracts.OctetValenceProvenanceExport

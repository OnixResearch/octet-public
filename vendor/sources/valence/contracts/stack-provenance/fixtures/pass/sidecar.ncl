let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
Values.Sidecar
| Contracts.StackProvenanceSidecarShape
| Contracts.StackProvenanceSidecar

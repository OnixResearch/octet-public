let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
(
  Values.Sidecar & {
    non_claims | force = [Contracts.constants.ProvenanceBoundary],
  }
)
| Contracts.StackProvenanceSidecarShape
| Contracts.StackProvenanceSidecar

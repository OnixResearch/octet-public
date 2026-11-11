let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
(
  Values.Sidecar & {
    binary_hash | force = "not-a-blake3-digest",
  }
)
| Contracts.StackProvenanceSidecarShape
| Contracts.StackProvenanceSidecar

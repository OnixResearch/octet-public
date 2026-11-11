let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
(
  Values.Report & {
    non_claims | force = [
      Contracts.constants.BuiltBinaryThreadNonClaim,
      "proves semantic equivalence",
    ],
  }
)
| Contracts.BuiltBinaryThreadReportShape
| Contracts.BuiltBinaryThreadReport

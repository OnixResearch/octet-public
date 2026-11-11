let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
Values.Report
| Contracts.BuiltBinaryThreadReportShape
| Contracts.BuiltBinaryThreadReport

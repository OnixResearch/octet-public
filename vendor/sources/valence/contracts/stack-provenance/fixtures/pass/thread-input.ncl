let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
Values.ThreadInput
| Contracts.BuiltBinaryThreadInputShape
| Contracts.BuiltBinaryThreadInput

let Contracts = import "../../contracts.ncl" in
let Values = import "../values.ncl" in
Values.MantleManifest
| Contracts.MantleReleaseManifestViewShape
| Contracts.MantleReleaseManifestView

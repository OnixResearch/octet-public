let contracts = import "../../contracts.ncl" in
let profile = import "../valid/minimum.ncl" in
(profile & { scope.source_paths | force = [] }) | contracts.AssuranceProfile | contracts.AssuranceProfileRules

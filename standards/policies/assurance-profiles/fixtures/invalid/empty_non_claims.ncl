let contracts = import "../../contracts.ncl" in
let profile = import "../valid/minimum.ncl" in
(profile & { non_claims | force = [] }) | contracts.AssuranceProfile | contracts.AssuranceProfileRules

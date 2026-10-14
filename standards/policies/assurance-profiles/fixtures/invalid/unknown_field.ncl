let contracts = import "../../contracts.ncl" in
let profile = import "../valid/minimum.ncl" in
(profile & { universal_security_score = "unsupported" }) | contracts.AssuranceProfile | contracts.AssuranceProfileRules

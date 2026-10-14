let contracts = import "../../contracts.ncl" in
let profile = import "../valid/minimum.ncl" in
let mechanism = std.array.first profile.mechanisms in
(profile & { mechanisms | force = [mechanism, mechanism] }) | contracts.AssuranceProfile | contracts.AssuranceProfileRules

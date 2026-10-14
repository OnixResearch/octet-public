let contracts = import "../../contracts.ncl" in
let profile = import "../valid/minimum.ncl" in
let unused = {
  assumption_id = "unused-assumption",
  owner = "fixture",
  statement = "This declaration is intentionally disconnected.",
  validation = "No mechanism references it.",
} in
(profile & { assumptions | force = profile.assumptions @ [unused] }) | contracts.AssuranceProfile | contracts.AssuranceProfileRules

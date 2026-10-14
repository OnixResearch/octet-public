let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
(policy & {
  mode | force = "admission",
  admission | force = {
    strict_profile = "deterministic-core",
    effect_closure_policy = "octet-effect-closure.json",
  },
})
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

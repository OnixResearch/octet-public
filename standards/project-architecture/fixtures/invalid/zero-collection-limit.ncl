let contracts = import "../../../../templates/octet-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
(policy & {
  collection_limits.max_units | force = 0,
})
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

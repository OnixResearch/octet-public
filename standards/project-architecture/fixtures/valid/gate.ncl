let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
(policy & { mode | force = "gate" })
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

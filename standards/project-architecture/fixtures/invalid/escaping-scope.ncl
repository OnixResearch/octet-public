let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
(
  policy
  & {
    roles | force = policy.roles @ [{
      scope_kind = "source",
      scope = "../outside",
      role = "generated",
    }],
  }
)
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

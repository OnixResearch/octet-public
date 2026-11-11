let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
let effect = std.array.first policy.outbound_effects in
(
  policy
  & {
    outbound_effects | force = [effect & { executor_scope | force = "crates/host/src" }],
  }
)
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

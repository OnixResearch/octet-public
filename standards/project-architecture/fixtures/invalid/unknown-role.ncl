let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
let assignment = std.array.first policy.roles in
(
  policy
  & {
    roles | force = [assignment & { role | force = "core-ish" }]
      @ std.array.slice 1 (std.array.length policy.roles) policy.roles,
  }
)
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

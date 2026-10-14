let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
(
  policy
  & {
    roles | force = policy.roles
      |> std.array.filter (fun assignment => assignment.scope != "crates/domain/src"),
  }
)
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
let port = std.array.first policy.ports in
(
  policy
  & {
    ports | force = [port & {
      application_owner_scope | force = "crates/domain/src",
      core_owner_scope = "crates/domain/src",
    }],
  }
)
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
(
  policy
  & {
    waivers | force = [{
      finding_id = "architecture.example.stale",
      policy_blake3 = "stale",
      source_blake3 = "stale",
      owner = "architecture-team",
      reason = "unbound migration",
      review_condition = "unknown",
      affected_claim = "application-owned-port",
    }],
  }
)
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

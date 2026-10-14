let contracts = import "../../octet-self-architecture.contracts.ncl" in
let policy = import "../../../../templates/octet-architecture.ncl" in
(
  policy
  & {
    waivers | force = [{
      finding_id = "architecture.example.current",
      policy_blake3 = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
      source_blake3 = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
      owner = "architecture-team",
      reason = "bounded migration",
      review_condition = "remove after adapter migration",
      affected_claim = "application-owned-port",
    }],
  }
)
| contracts.ProjectArchitecturePolicy
| contracts.ProjectArchitecturePolicyRules

let contracts = import "../../contracts.ncl" in
let registry = import "../../registry.ncl" in

registry & {
  policies = std.array.map (fun policy =>
    if policy.policy_id == "tigerstyle.verification.deterministic-effect-handlers" then
      policy & {
        proof_rails = [{
          receipt_verifier = {
            verifier_id = "octet-standards@proof-receipt/v1",
            verifier_digest = true,
            toolchain_id = "rust-toolchain.toml",
            toolchain_digest = "50692525e10589e3a1060a0ed0adc77f695ad5617121b572e549ef9855ceb7a8",
          },
        }],
      }
    else policy
  ) registry.policies,
} | contracts.PolicyRegistry | contracts.PolicyRegistryRules

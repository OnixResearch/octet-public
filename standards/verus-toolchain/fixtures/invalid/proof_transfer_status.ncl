let contracts = import "../../contracts.ncl" in
let profile = import "../../profile.ncl" in
let invalid_compatibility = [{
  consumer = "downstream",
  verifier_release = "0.2026.04.03.21dfcd2",
  status = "proof-transfers",
  rationale = "invalid fixture attempts to transfer proof authority",
}] in
(profile & { compatibility | force = invalid_compatibility }) | contracts.VerusToolchainProfile

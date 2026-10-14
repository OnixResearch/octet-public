let contracts = import "../../contracts.ncl" in
let profile = import "../../profile.ncl" in
let invalid_compatibility = [{
  consumer = "unregistered-downstream",
  verifier_release = "0.2026.04.03.21dfcd2",
  status = "native",
  rationale = "unknown consumers must not inherit package authority",
}] in
(profile & { compatibility | force = invalid_compatibility }) | contracts.VerusToolchainProfile

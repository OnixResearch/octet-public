let contracts = import "../../contracts.ncl" in
let profile = import "../../profile.ncl" in
(profile & { non_claims | force = [] }) | contracts.VerusToolchainProfile

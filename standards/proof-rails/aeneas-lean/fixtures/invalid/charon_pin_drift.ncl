let contracts = import "../../contracts.ncl" in
let profile = import "../../profile.ncl" in
(profile & { charon_revision | force = "5a5017333554ec1e95cae3c4fe05f5bd90b44487" }) | contracts.AeneasLeanProviderProfile

# r[verify octet.aeneas_lean_proof_rail.semantic_reference]
let contracts = import "../../contracts.ncl" in
let profile = import "../../profile.ncl" in
(profile & { semantic_reference.revision | force = "c8eac455e8402a9ef78fccf75bafb215658f4906" }) | contracts.AeneasLeanProviderProfile

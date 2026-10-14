# r[verify octet.aeneas_lean_proof_rail.semantic_reference]
let contracts = import "../../contracts.ncl" in
let profile = import "../../profile.ncl" in
(profile & { semantic_reference.recorded_only | force = false }) | contracts.AeneasLeanProviderProfile

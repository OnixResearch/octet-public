let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails.addition_bound.trust_boundaries | force = [true],
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules

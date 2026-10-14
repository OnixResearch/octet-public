let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails.addition_bound.source_paths | force = [],
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules

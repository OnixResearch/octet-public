let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails | force = {},
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules

let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest | contracts.ProofRailManifest | contracts.ProofRailManifestRules

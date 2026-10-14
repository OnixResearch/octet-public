let contracts = import "../../contracts.ncl" in
let manifest = import "../../manifest.ncl" in

manifest & {
  rails.consumer_range_proof = manifest.rails.addition_bound & {
    command | force = "octet-verus verus/proofs/consumer_range.rs",
    command_argv | force = ["octet-verus", "verus/proofs/consumer_range.rs"],
    source_paths | force = ["src/verified/consumer_range.rs"],
    proof_paths | force = ["verus/proofs/consumer_range.rs"],
    receipt_paths | force = ["proofs/consumer-range-rail-0.receipt"],
    pass_signal | force = "proof-result: pass consumer-range-proof",
    trust_boundaries | force = ["consumer-scoped Verus rail proves only the declared range property; it does not prove whole-program correctness"],
    checked_scope | force = "formal-proof:consumer:consumer.range-proof#rail-0",
  },
} | contracts.ProofRailManifest | contracts.ProofRailManifestRules

# r[impl octet.vermilion_proof_rail.semantic_contract]
# Common semantic contract for the bounded-increment pilot.
#
# The statement identity is the BLAKE3 of `pilot/semantic-spec.txt`. Each
# route records the obligations it can actually discharge and the bridge from
# its representation to this u64 checked contract. A named bridge that is not
# checked remains an open obligation and prevents cross-provider agreement
# from being read as a merged proof.
let contracts = import "contracts.ncl" in
{
  schema_version = "octet-proof-semantic-contract/v1",
  contract_id = "octet.bounded_increment.checked/v1",
  revision = "2026-11-09",
  subject_path = "standards/proof-rails/vermilion-lean/pilot/src/lib.rs",
  subject_blake3 = "b3:dc2836fb70d645fb0d518a05f3c245756f58cb82989c33570b73283cd0ff1098",
  statement = "For value < u64::MAX, increment_checked(value) == Some(value + 1); at u64::MAX it returns None. No nat reinterpretation is implied.",
  statement_blake3 = "b3:af259da355a01524f6030f09ee196f20cb675991e4421204b63093fd017d38c4",
  width_bits = 64,
  overflow_behavior = "checked",
  precondition = "value < u64::MAX",
  postcondition = "result == Some(value + 1)",
  result_type = "Option<u64>",
  observable_failures = ["None at value == u64::MAX"],
  termination_scope = "single-call, no loops",
  routes = [
    {
      backend = "verus",
      claim_scope = "u64-checked-contract-model",
      required_obligations = [
        { obligation_id = "pilot.increment.overflow-check", kind = "overflow" },
        { obligation_id = "pilot.increment.value", kind = "logical-vc" },
      ],
      refinement_bridge_id = "pilot.verus.plain-rust-refinement",
      refinement_bridge = "open: the production Verus proof target mirrors the executable body and checks the u64 contract, but no formal plain-Rust-to-model refinement theorem is checked",
      modeled_library_contracts = [],
    },
    {
      backend = "vermilion-lean",
      claim_scope = "per-obligation-lean-kernel-theorem-over-pinned-verus-frontend",
      required_obligations = [
        { obligation_id = "pilot.increment.overflow-check", kind = "overflow" },
        { obligation_id = "pilot.increment.value", kind = "logical-vc" },
      ],
      refinement_bridge_id = "pilot.increment.refinement-to-spec",
      refinement_bridge = "open: the embedding maps u64 to Lean Int with range facts; the bridge from the generated obligation to this contract statement is not yet checked",
      modeled_library_contracts = [],
    },
    {
      backend = "aeneas-lean",
      claim_scope = "named-theorem-over-generated-model",
      required_obligations = [
        { obligation_id = "aeneas.fixture.increment-model", kind = "logical-vc" },
      ],
      refinement_bridge_id = "pilot.aeneas.u32-to-u64-checked",
      refinement_bridge = "open: the shipped Aeneas external fixture models a u32 increment, not this u64 checked contract",
      modeled_library_contracts = [],
    },
  ],
  non_claims = [
    "does not prove Vermilion, Verus, or Lean soundness",
    "does not prove that any route refines the executable Rust function",
    "does not prove whole-program correctness",
    "does not prove semantic equivalence",
    "does not establish release eligibility",
  ],
} | contracts.PilotSemanticContract

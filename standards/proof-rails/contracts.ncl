# Verus proof-rail manifest contracts.
let policy_contracts = import "../policies/contracts.ncl" in

let NonEmptyString =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.length value > 0
  )
in

let SchemaVersion =
  std.contract.from_predicate (fun value => value == "octet-proof-rail-manifest/v1")
in

let rail_has_verifier_identity = fun rail =>
  std.string.length rail.receipt_verifier.verifier_id > 0
  && std.string.length rail.receipt_verifier.verifier_binary_path > 0
  && std.string.length rail.receipt_verifier.verifier_binary_digest > 0
  && std.string.length rail.receipt_verifier.toolchain_id > 0
  && std.string.length rail.receipt_verifier.toolchain_digest > 0
in

let rail_has_production_identity = fun rail =>
  rail.verifier_mode != "production"
  || (
    rail_has_verifier_identity rail
    && std.string.length rail.receipt_verifier.solver_id > 0
    && std.string.length rail.receipt_verifier.solver_digest > 0
    && (
      std.string.length rail.receipt_verifier.verifier_nix_closure_digest > 0
      || std.string.length rail.receipt_verifier.verifier_nix_closure_graph_digest > 0
    )
  )
in

let rail_has_aeneas_theorem_approval = fun rail =>
  rail.backend != "aeneas-lean"
  || (
    std.string.length rail.theorem_id > 0
    && std.string.is_match "^[0-9a-f]{64}$" rail.approved_theorem_statement_blake3
  )
in

let rail_rules = fun rail =>
  std.array.elem rail.backend ["verus", "aeneas-lean"]
  && std.array.length rail.command_argv > 0
  && std.array.length rail.source_paths > 0
  && std.array.length rail.proof_paths > 0
  && std.array.length rail.receipt_paths > 0
  && std.array.length rail.trust_boundaries > 0
  && rail_has_verifier_identity rail
  && rail_has_production_identity rail
  && rail_has_aeneas_theorem_approval rail
in

let manifest_rails = fun manifest => std.record.values manifest.rails in

let ManifestRules =
  std.contract.from_predicate (fun manifest =>
    std.array.length (manifest_rails manifest) > 0
    && std.array.all rail_rules (manifest_rails manifest)
  )
in
{
  ProofRailManifest = {
    schema_version | SchemaVersion,
    rails | { _ | policy_contracts.ProofRail },
  },
  ProofRailManifestRules = ManifestRules,
}

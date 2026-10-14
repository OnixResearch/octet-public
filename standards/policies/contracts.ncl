# Tiger Style policy registry contracts.
# The registry is authored in Nickel so required fields and closed schemas fail
# before checker code consumes exported JSON.
let NonEmptyString =
  std.contract.from_predicate (fun value =>
    std.is_string value && std.string.length value > 0
  )
in

let OneOf = fun allowed =>
  std.contract.from_predicate (fun value =>
    std.is_string value && std.array.elem value allowed
  )
in

let ClassificationContract =
  OneOf [
    "hard",
    "advisory",
    "human-process-only",
    "deferred",
  ]
in

let EnforcementLevelContract =
  OneOf [
    "hard",
    "advisory",
    "human-process-only",
    "deferred",
  ]
in

let EnforcementModeContract =
  OneOf [
    "lint",
    "gate",
    "structural",
    "verified",
    "human-oracle",
  ]
in

let MechanismKindContract =
  OneOf [
    "crate-boundary",
    "capability-api",
    "bounded-primitive",
    "type-state",
    "port",
  ]
in

let DependencyEdgeKindContract =
  OneOf [
    "normal",
    "build",
    "dev",
  ]
in
let CapabilityScopeClassContract =
  OneOf [
    "core",
    "shell",
    "adapter",
    "test",
    "generated",
    "remote-execution",
  ]
in
let CapabilityFamilyContract =
  OneOf [
    "time",
    "randomness",
    "filesystem",
    "network",
    "process",
    "environment",
    "secrets",
    "remote-execution",
    "unsafe-code",
    "global-state",
    "concurrency",
    "observability",
    "unknown",
  ]
in
let CapabilityManifestEnforcementContract =
  OneOf [
    "advisory",
    "strict",
  ]
in
let SeedRoleContract =
  OneOf [
    "architecture-reference",
    "trajectory-fixture",
    "proof-seed",
    "core-shell-fixture",
  ]
in

let SeverityContract =
  OneOf [
    "error",
    "warning",
    "info",
    "note",
  ]
in

let FixtureStatusContract =
  OneOf [
    "planned",
    "covered",
    "not-applicable",
  ]
in

let CheckerIdContract =
  OneOf [
    "registry",
    "rustc",
    "clippy",
    "rustfmt",
    "tigerstyle",
    "cargo-deny",
    "cargo-vet",
    "cargo-semver-checks",
    "tracey",
    "verus",
    "octet-standards",
    "cargo-octet",
    "octet-policy-validate",
    "octet-policy-export",
    "octet-fingerprint",
    "octet-transcript",
    "octet-dynamic-evidence",
    "octet-release-supply-chain",
    "fitness-report",
    "safety-evidence",
    "human-process",
  ]
in

let ReceiptBindingContract =
  OneOf [
    "none",
    "blake3-command-and-source",
  ]
in

let ConfigReceiptOwnershipContract =
  OneOf [
    "nickel-authored",
    "rust-derived",
    "non-candidate",
  ]
in

let ConfigReceiptFamilyStatusContract =
  OneOf [
    "planned",
    "existing",
    "non-candidate",
  ]
in

let ReceiptProvenanceContract = {
  schema_version | String | default = "",
  runner | String | default = "",
  environment | String | default = "",
  requires_backend | Bool | default = false,
  requires_trust_boundary | Bool | default = false,
}
in

let ConfigReceiptFamilyContract = {
  family_id | NonEmptyString,
  ownership | ConfigReceiptOwnershipContract,
  owner_source_path | NonEmptyString,
  artifact_path | String | default = "",
  generation_command | NonEmptyString,
  validation_command | NonEmptyString,
  status | ConfigReceiptFamilyStatusContract,
  config_digest_required | Bool | default = false,
  positive_fixtures | Array NonEmptyString | default = [],
  negative_fixtures | Array NonEmptyString | default = [],
  exclusion_rationale | String | default = "",
}
in

let ReceiptVerifierContract = {
  verifier_id | String | default = "",
  verifier_digest | String | default = "",
  verifier_binary_path | String | default = "",
  verifier_binary_digest | String | default = "",
  verifier_build_closure | String | default = "",
  verifier_build_closure_digest | String | default = "",
  verifier_build_closure_paths | Array NonEmptyString | default = [],
  verifier_nix_derivation | String | default = "",
  verifier_nix_derivation_digest | String | default = "",
  verifier_nix_closure | String | default = "",
  verifier_nix_closure_digest | String | default = "",
  verifier_nix_closure_paths | Array NonEmptyString | default = [],
  verifier_nix_closure_graph_digest | String | default = "",
  verifier_nix_closure_graph_paths | Array NonEmptyString | default = [],
  toolchain_id | String | default = "",
  toolchain_digest | String | default = "",
  solver_id | String | default = "",
  solver_digest | String | default = "",
}
in

let SuppressionRequirements = {
  requires_policy_id | Bool,
  requires_source_section | Bool,
  requires_owner | Bool,
  requires_rationale | Bool,
  requires_review_trigger | Bool,
  requires_safer_default | Bool,
}
in

let FixtureCoverage = {
  status | FixtureStatusContract,
  compliant | Array String,
  noncompliant | Array String,
}
in

let EvidenceContract = {
  command | NonEmptyString,
  expected_signal | NonEmptyString,
}
in

let MechanismContract = {
  kind | MechanismKindContract,
  description | NonEmptyString,
  exported_obligations | Array NonEmptyString,
  crate_boundary | Array NonEmptyString | default = [],
  constructor_authority | Array NonEmptyString | default = [],
  api_obligations | Array NonEmptyString | default = [],
  forbidden_apis | Array NonEmptyString | default = [],
  approved_bounded_apis | Array NonEmptyString | default = [],
  approved_wrapper_scopes | Array NonEmptyString | default = [],
  capacity_witnesses | Array NonEmptyString | default = [],
  backpressure_behaviors | Array NonEmptyString | default = [],
  forbidden_ambient_apis | Array NonEmptyString | default = [],
  core_source_scopes | Array NonEmptyString | default = [],
  shell_constructor_scopes | Array NonEmptyString | default = [],
  approved_capability_names | Array NonEmptyString | default = [],
  capability_enforcement_mode | CapabilityManifestEnforcementContract | default = "advisory",
  capability_scope_classes | Array CapabilityScopeClassContract | default = [],
  capability_families | Array CapabilityFamilyContract | default = [],
  capability_injected_traits | Array NonEmptyString | default = [],
  capability_tokens | Array NonEmptyString | default = [],
  capability_shell_interpreters | Array NonEmptyString | default = [],
  capability_adapter_scopes | Array NonEmptyString | default = [],
  capability_test_scopes | Array NonEmptyString | default = [],
  capability_generated_scopes | Array NonEmptyString | default = [],
  capability_remote_execution_scopes | Array NonEmptyString | default = [],
  capability_deterministic_handlers | Array NonEmptyString | default = [],
  capability_handler_replay_commands | Array NonEmptyString | default = [],
  capability_handler_artifacts | Array NonEmptyString | default = [],
  type_state_source_scopes | Array NonEmptyString | default = [],
  raw_state_alias_patterns | Array NonEmptyString | default = [],
  raw_transition_patterns | Array NonEmptyString | default = [],
  required_state_markers | Array NonEmptyString | default = [],
  required_transition_methods | Array NonEmptyString | default = [],

  # Crate-boundary mechanism fields (policy-driven-crate-boundaries)
  core_crates | Array NonEmptyString | default = [],
  shell_crates | Array NonEmptyString | default = [],
  adapter_crates | Array NonEmptyString | default = [],
  forbidden_core_deps | Array NonEmptyString | default = [],
  external_domain_patterns | Array NonEmptyString | default = [],
  dependency_edge_kinds | Array DependencyEdgeKindContract | default = [],
  core_no_std | Bool | default = false,

  # Port mechanism fields (hexagonal-port-mechanism)
  port_traits | Array NonEmptyString | default = [],
  core_owner_scope | String | default = "",
  permitted_adapter_scopes | Array NonEmptyString | default = [],
  forbidden_concrete_types | Array NonEmptyString | default = [],
  reserved | Bool | default = false,
}
in

let ProofVerifierModeContract =
  OneOf [
    "",
    "fixture",
    "production",
  ]
in
let ProofRailContract = {
  backend | NonEmptyString,
  command | NonEmptyString,
  command_argv | Array NonEmptyString | default = [],
  source_paths | Array NonEmptyString,
  proof_paths | Array NonEmptyString,
  receipt_paths | Array NonEmptyString | default = [],
  pass_signal | NonEmptyString,
  trust_boundaries | Array NonEmptyString,
  receipt_binding | ReceiptBindingContract | default = "none",
  receipt_provenance | ReceiptProvenanceContract | default = {},
  receipt_verifier | ReceiptVerifierContract | default = {},
  verifier_mode | ProofVerifierModeContract | default = "",
  checked_scope | String | default = "",
  timeout_ms | Number | default = 0,
  stdout_max_bytes | Number | default = 0,
  stderr_max_bytes | Number | default = 0,
  environment_allowlist | Array NonEmptyString | default = [],
}
in

let SeedManifestContract = {
  name | NonEmptyString,
  source_path | NonEmptyString,
  source_commit | NonEmptyString,
  role | SeedRoleContract,
  paths | Array NonEmptyString,
  provenance | NonEmptyString,
  license | NonEmptyString,
  content_hash | NonEmptyString,
  proof_relevant | Bool,
  local_acceptance | Array NonEmptyString,
}
in

let ProofBundleSchemaVersionContract = OneOf [
  "octet-proof-bundle/v1",
]
in

let ArtifactManifestSchemaVersionContract = OneOf [
  "octet-artifact-manifest/v1",
]
in

let ArtifactManifestEntryContract = {
  path | NonEmptyString,
  blake3 | NonEmptyString,
  bytes | Number,
}
in

let ArtifactManifestContract = {
  schema_version | ArtifactManifestSchemaVersionContract,
  generated_from | NonEmptyString,
  registry_schema_version | NonEmptyString,
  artifacts | Array ArtifactManifestEntryContract,
}
in

let ProofBundleEntryContract = {
  policy_id | NonEmptyString,
  rail_index | Number,
  proof_rail | ProofRailContract,
}
in

let ProofBundleArtifactContract = {
  schema_version | ProofBundleSchemaVersionContract,
  generated_from | NonEmptyString,
  registry_schema_version | NonEmptyString,
  proof_bundles | Array ProofBundleEntryContract,
}
in

let mode_rank = fun mode =>
  if mode == "lint" then 0
  else if mode == "gate" then 1
  else if mode == "structural" then 2
  else if mode == "verified" then 3
  else 4
in

let has_mechanism = fun policy kind =>
  std.array.any (fun mechanism => mechanism.kind == kind) policy.mechanisms
in

let crate_boundary_mechanism_valid = fun mechanism =>
  mechanism.kind != "crate-boundary"
  || std.array.length mechanism.core_crates > 0
in

let port_mechanism_valid = fun mechanism =>
  mechanism.kind != "port"
  || std.array.length mechanism.port_traits == 0
  || (
    std.string.length mechanism.core_owner_scope > 0
    && (mechanism.reserved || std.array.length mechanism.permitted_adapter_scopes > 0)
  )
in

let capability_mechanism_valid = fun mechanism =>
  mechanism.kind != "capability-api"
  || mechanism.capability_enforcement_mode != "strict"
  || (
    std.array.length mechanism.capability_scope_classes > 0
    && std.array.length mechanism.capability_families > 0
    && std.array.length mechanism.core_source_scopes > 0
    && (
      std.array.length mechanism.capability_deterministic_handlers == 0
      || (
        std.array.length mechanism.capability_handler_artifacts > 0
        && std.array.length mechanism.capability_handler_replay_commands > 0
      )
    )
  )
in
let mechanism_cross_fields_valid = fun mechanism =>
  crate_boundary_mechanism_valid mechanism
  && port_mechanism_valid mechanism
  && capability_mechanism_valid mechanism
in

let policy_cross_fields_valid = fun policy =>
  (mode_rank policy.target_enforcement_mode >= mode_rank policy.current_enforcement_mode)
  && (policy.target_enforcement_mode == policy.current_enforcement_mode || std.string.length policy.escalation_criteria > 0)
  && (policy.target_enforcement_mode != "structural" || std.array.length policy.mechanisms > 0)
  && (policy.target_enforcement_mode != "verified" || std.array.length policy.proof_rails > 0)
  && (policy.default_severity != "error" || std.array.length policy.verification_evidence_requirements > 0)
  && (policy.default_severity != "error" || std.string.length policy.deterministic_checker.command > 0)
  && std.array.all mechanism_cross_fields_valid policy.mechanisms
in

let seed_cross_fields_valid = fun seed =>
  (!seed.proof_relevant) || std.array.length seed.local_acceptance > 0
in

let config_receipt_family_valid = fun family =>
  if family.ownership == "nickel-authored" then
    std.string.length family.artifact_path > 0
    && std.string.length family.exclusion_rationale == 0
    && family.config_digest_required
  else if family.ownership == "rust-derived" then
    std.string.length family.artifact_path > 0
    && std.string.length family.exclusion_rationale == 0
  else
    family.status == "non-candidate"
    && std.string.length family.artifact_path == 0
    && std.string.length family.exclusion_rationale > 0
in

let RegistryCrossFieldContract =
  std.contract.from_predicate (fun registry =>
    std.array.all policy_cross_fields_valid registry.policies
    && std.array.all seed_cross_fields_valid registry.seed_manifests
    && std.array.all config_receipt_family_valid registry.config_receipt_families
  )
in

let PolicyEntryContract = {
  policy_id | NonEmptyString,
  pillar | NonEmptyString,
  source_path | NonEmptyString,
  source_heading | NonEmptyString,
  source_anchor | NonEmptyString,
  summary | NonEmptyString,
  classification | ClassificationContract,
  enforcement_level | EnforcementLevelContract,
  current_enforcement_mode | EnforcementModeContract | default = "lint",
  target_enforcement_mode | EnforcementModeContract | default = "lint",
  escalation_criteria | String | default = "no escalation planned",
  mechanisms | Array MechanismContract | default = [],
  deterministic_checker | EvidenceContract | default = { command = "registry validation", expected_signal = "export succeeds" },
  proof_rails | Array ProofRailContract | default = [],
  checker_owner | NonEmptyString,
  checker_id | CheckerIdContract,
  default_severity | SeverityContract,
  suppression_requirements | SuppressionRequirements,
  fixture_coverage | FixtureCoverage,
  verification_evidence_requirements | Array NonEmptyString,
  classification_rationale | NonEmptyString,
  false_positive_risk | NonEmptyString,
  existing_enforcement | Array NonEmptyString,
  custom_checker_justification | NonEmptyString,
}
in

{
  PolicyRegistry = {
    schema_version | NonEmptyString,
    checker_ids | Array CheckerIdContract,
    policies | Array PolicyEntryContract,
    seed_manifests | Array SeedManifestContract | default = [],
    config_receipt_families | Array ConfigReceiptFamilyContract | default = [],
  },

  PolicyEntry = PolicyEntryContract,
  CheckerId = CheckerIdContract,
  Classification = ClassificationContract,
  EnforcementLevel = EnforcementLevelContract,
  Severity = SeverityContract,
  EnforcementMode = EnforcementModeContract,
  Mechanism = MechanismContract,
  CapabilityScopeClass = CapabilityScopeClassContract,
  CapabilityFamily = CapabilityFamilyContract,
  CapabilityManifestEnforcement = CapabilityManifestEnforcementContract,
  ProofRail = ProofRailContract,
  SeedManifest = SeedManifestContract,
  ConfigReceiptFamily = ConfigReceiptFamilyContract,
  ConfigReceiptOwnership = ConfigReceiptOwnershipContract,
  ProofBundleArtifact = ProofBundleArtifactContract,
  ProofBundleSchemaVersion = ProofBundleSchemaVersionContract,
  ArtifactManifest = ArtifactManifestContract,
  ArtifactManifestSchemaVersion = ArtifactManifestSchemaVersionContract,
  PolicyRegistryRules = RegistryCrossFieldContract,
}

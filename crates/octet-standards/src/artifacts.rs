//! Generated policy artifacts consumed by standards tooling.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde::Serialize;

use crate::model::ConfigReceiptFamily;
use crate::model::EnforcementLevel;
use crate::model::EnforcementMode;
use crate::model::PolicyEntry;
use crate::model::PolicyRegistry;
use crate::model::ProofRail;
use crate::model::SeedManifest;
use crate::proof_artifact::artifact_text_identity;

/// Canonical registry artifact file name.
pub const POLICIES_JSON_FILE: &str = "octet-policies.json";
/// Checker configuration artifact file name.
pub const CHECK_CONFIG_TOML_FILE: &str = "octet-check-config.toml";
/// Seed manifest artifact file name.
pub const SEED_MANIFESTS_JSON_FILE: &str = "octet-seed-manifests.json";
/// Proof-bundle artifact file name.
pub const PROOF_BUNDLES_JSON_FILE: &str = "octet-proof-bundles.json";
/// Config/receipt source-of-truth family artifact file name.
pub const CONFIG_RECEIPT_FAMILIES_JSON_FILE: &str = "octet-config-receipt-families.json";
/// Generated artifact integrity manifest file name.
pub const ARTIFACT_MANIFEST_JSON_FILE: &str = "octet-artifact-manifest.json";
/// Supported proof-bundle artifact schema version.
pub const PROOF_BUNDLE_SCHEMA_VERSION: &str = "octet-proof-bundle/v1";
/// Supported generated artifact manifest schema version.
pub const ARTIFACT_MANIFEST_SCHEMA_VERSION: &str = "octet-artifact-manifest/v1";

const GENERATED_ARTIFACT_SCHEMA_VERSION: &str = "1";
const GENERATED_FROM: &str = "standards/policies/registry.ncl";

/// Deterministic generated artifact integrity manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtifactManifest {
    /// Artifact manifest schema version.
    pub schema_version: String,
    /// Source policy registry path.
    pub generated_from: String,
    /// Registry schema version this manifest was generated from.
    pub registry_schema_version: String,
    /// Generated artifacts covered by this manifest.
    pub artifacts: Vec<ArtifactManifestEntry>,
}

/// One generated artifact digest entry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtifactManifestEntry {
    /// Generated artifact file name.
    pub path: String,
    /// BLAKE3 digest over the exact written artifact bytes.
    pub blake3: String,
    /// Exact written artifact byte length.
    pub bytes: u64,
}

/// Deterministic seed-manifest export artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SeedManifestArtifact {
    /// Artifact schema version.
    pub schema_version: String,
    /// Source policy registry path.
    pub generated_from: String,
    /// Registry schema version this artifact was generated from.
    pub registry_schema_version: String,
    /// Seed manifests copied from the policy registry.
    pub seed_manifests: Vec<SeedManifest>,
}

/// Deterministic config/receipt source-of-truth family export artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ConfigReceiptFamilyArtifact {
    /// Artifact schema version.
    pub schema_version: String,
    /// Source policy registry path.
    pub generated_from: String,
    /// Registry schema version this artifact was generated from.
    pub registry_schema_version: String,
    /// Config and receipt families copied from the policy registry.
    pub config_receipt_families: Vec<ConfigReceiptFamily>,
}

/// Deterministic proof-bundle export artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProofBundleArtifact {
    /// Artifact schema version.
    pub schema_version: String,
    /// Source policy registry path.
    pub generated_from: String,
    /// Registry schema version this artifact was generated from.
    pub registry_schema_version: String,
    /// Verified proof rails exported from the policy registry.
    pub proof_bundles: Vec<ProofBundleEntry>,
}

/// One verified proof-rail bundle entry.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProofBundleEntry {
    /// Policy ID that owns the proof rail.
    pub policy_id: String,
    /// Zero-based proof rail index within the owning policy.
    pub rail_index: u32,
    /// Verified proof rail metadata and receipt obligations.
    pub proof_rail: ProofRail,
}

/// Build the generated artifact integrity manifest from exact artifact texts.
///
/// # Panics
///
/// Panics only if an in-memory artifact text length does not fit in `u64`.
#[must_use]
pub fn artifact_manifest(
    registry: &PolicyRegistry,
    artifacts: &[(impl AsRef<str>, impl AsRef<str>)],
) -> ArtifactManifest {
    let mut entries = artifacts
        .iter()
        .map(|(path, text)| {
            let identity = artifact_text_identity(path.as_ref(), text.as_ref());
            ArtifactManifestEntry {
                path: identity.path,
                blake3: identity.blake3,
                bytes: identity.bytes,
            }
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| left.path.cmp(&right.path));
    ArtifactManifest {
        schema_version: ARTIFACT_MANIFEST_SCHEMA_VERSION.to_owned(),
        generated_from: GENERATED_FROM.to_owned(),
        registry_schema_version: registry.schema_version.clone(),
        artifacts: entries,
    }
}

/// Validate generated artifact digests against a manifest without parsing policy metadata.
///
/// # Panics
///
/// Panics only if an in-memory artifact text length does not fit in `u64`.
#[must_use]
pub fn validate_artifact_manifest_texts(
    manifest: &ArtifactManifest,
    artifacts: &[(impl AsRef<str>, impl AsRef<str>)],
) -> Vec<String> {
    let mut errors = Vec::new();
    if manifest.schema_version != ARTIFACT_MANIFEST_SCHEMA_VERSION {
        errors.push(format!(
            "unsupported artifact manifest schema version `{}`; expected `{}`",
            manifest.schema_version, ARTIFACT_MANIFEST_SCHEMA_VERSION
        ));
    }

    let mut expected_entries = artifacts
        .iter()
        .map(|(path, text)| {
            let identity = artifact_text_identity(path.as_ref(), text.as_ref());
            ArtifactManifestEntry {
                path: identity.path,
                blake3: identity.blake3,
                bytes: identity.bytes,
            }
        })
        .collect::<Vec<_>>();
    expected_entries.sort_by(|left, right| left.path.cmp(&right.path));
    for expected_entry in &expected_entries {
        match manifest.artifacts.iter().find(|entry| entry.path == expected_entry.path) {
            Some(actual) if actual == expected_entry => {}
            Some(actual) => errors.push(format!(
                "artifact integrity mismatch for `{}`; expected blake3 {} bytes {}, got blake3 {} bytes {}",
                expected_entry.path, expected_entry.blake3, expected_entry.bytes, actual.blake3, actual.bytes
            )),
            None => errors.push(format!("artifact manifest is missing generated artifact `{}`", expected_entry.path)),
        }
    }
    for actual in &manifest.artifacts {
        if !expected_entries.iter().any(|entry| entry.path == actual.path) {
            errors.push(format!("artifact manifest contains stale or unknown generated artifact `{}`", actual.path));
        }
    }

    errors
}

/// Validate generated artifact digests and registry schema against a manifest.
#[must_use]
pub fn validate_artifact_manifest(
    registry: &PolicyRegistry,
    manifest: &ArtifactManifest,
    artifacts: &[(impl AsRef<str>, impl AsRef<str>)],
) -> Vec<String> {
    let mut errors = validate_artifact_manifest_texts(manifest, artifacts);
    if manifest.registry_schema_version != registry.schema_version {
        errors.push(format!(
            "artifact manifest registry schema version `{}` does not match policy registry `{}`",
            manifest.registry_schema_version, registry.schema_version
        ));
    }
    errors
}

/// Build the proof-bundle artifact from a registry.
#[must_use]
pub fn proof_bundle_artifact(registry: &PolicyRegistry) -> ProofBundleArtifact {
    ProofBundleArtifact {
        schema_version: PROOF_BUNDLE_SCHEMA_VERSION.to_owned(),
        generated_from: GENERATED_FROM.to_owned(),
        registry_schema_version: registry.schema_version.clone(),
        proof_bundles: proof_bundle_entries(registry),
    }
}

/// Validate that a proof-bundle artifact matches the registry's verified proof rails.
#[must_use]
pub fn validate_proof_bundle_artifact(registry: &PolicyRegistry, artifact: &ProofBundleArtifact) -> Vec<String> {
    let mut errors = Vec::new();
    if artifact.schema_version != PROOF_BUNDLE_SCHEMA_VERSION {
        errors.push(format!(
            "unsupported proof-bundle schema version `{}`; expected `{}`",
            artifact.schema_version, PROOF_BUNDLE_SCHEMA_VERSION
        ));
    }
    if artifact.registry_schema_version != registry.schema_version {
        errors.push(format!(
            "proof-bundle registry schema version `{}` does not match policy registry `{}`",
            artifact.registry_schema_version, registry.schema_version
        ));
    }

    let expected_entries = proof_bundle_entries(registry);
    for expected in &expected_entries {
        let covered = artifact.proof_bundles.iter().any(|entry| {
            entry.policy_id == expected.policy_id
                && entry.rail_index == expected.rail_index
                && entry.proof_rail == expected.proof_rail
        });
        if !covered {
            errors.push(format!(
                "proof bundle is missing verified rail `{}` index {}",
                expected.policy_id, expected.rail_index
            ));
        }
    }
    for actual in &artifact.proof_bundles {
        let expected = expected_entries.iter().any(|entry| {
            entry.policy_id == actual.policy_id
                && entry.rail_index == actual.rail_index
                && entry.proof_rail == actual.proof_rail
        });
        if !expected {
            errors.push(format!(
                "proof bundle contains stale or unknown verified rail `{}` index {}",
                actual.policy_id, actual.rail_index
            ));
        }
    }

    errors
}

/// Returns true when a registry declares at least one verified proof rail.
#[must_use]
pub fn registry_has_verified_proof_rails(registry: &PolicyRegistry) -> bool {
    registry.policies.iter().any(|policy| policy_is_verified(policy) && !policy.proof_rails.is_empty())
}

fn proof_bundle_entries(registry: &PolicyRegistry) -> Vec<ProofBundleEntry> {
    let mut entries = registry
        .policies
        .iter()
        .filter(|policy| policy_is_verified(policy))
        .flat_map(|policy| {
            policy.proof_rails.iter().enumerate().map(move |(rail_index, rail)| ProofBundleEntry {
                policy_id: policy.policy_id.clone(),
                rail_index: u32::try_from(rail_index).expect("proof rail index should fit u32"),
                proof_rail: rail.clone(),
            })
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.policy_id
            .cmp(&right.policy_id)
            .then_with(|| left.rail_index.cmp(&right.rail_index))
            .then_with(|| left.proof_rail.backend.cmp(&right.proof_rail.backend))
    });
    entries
}

fn policy_is_verified(policy: &PolicyEntry) -> bool {
    policy.current_enforcement_mode == EnforcementMode::Verified
        || policy.target_enforcement_mode == EnforcementMode::Verified
}

/// Build the seed-manifest artifact from a registry.
#[must_use]
pub fn seed_manifest_artifact(registry: &PolicyRegistry) -> SeedManifestArtifact {
    SeedManifestArtifact {
        schema_version: GENERATED_ARTIFACT_SCHEMA_VERSION.to_owned(),
        generated_from: GENERATED_FROM.to_owned(),
        registry_schema_version: registry.schema_version.clone(),
        seed_manifests: registry.seed_manifests.clone(),
    }
}

/// Build the config/receipt source-of-truth family artifact from a registry.
#[must_use]
pub fn config_receipt_family_artifact(registry: &PolicyRegistry) -> ConfigReceiptFamilyArtifact {
    ConfigReceiptFamilyArtifact {
        schema_version: GENERATED_ARTIFACT_SCHEMA_VERSION.to_owned(),
        generated_from: GENERATED_FROM.to_owned(),
        registry_schema_version: registry.schema_version.clone(),
        config_receipt_families: registry.config_receipt_families.clone(),
    }
}

/// Render deterministic checker configuration TOML from a registry.
#[must_use]
pub fn render_check_config_toml(registry: &PolicyRegistry) -> String {
    let hard_policy_ids = policy_ids_by_level(registry, EnforcementLevel::Hard);
    let structural_policy_ids = policy_ids_by_target_mode(registry, EnforcementMode::Structural);
    let verified_policy_ids = policy_ids_by_target_mode(registry, EnforcementMode::Verified);
    let families = policy_families(registry);
    let bounded_queue = bounded_queue_policy(registry);
    let bounded_queue_forbidden_apis =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.forbidden_apis);
    let bounded_queue_approved_apis =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.approved_bounded_apis);
    let bounded_queue_wrapper_scopes =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.approved_wrapper_scopes);
    let bounded_queue_capacity_witnesses =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.capacity_witnesses);
    let bounded_queue_backpressure_behaviors =
        bounded_queue_mechanism_field(bounded_queue, |mechanism| &mechanism.backpressure_behaviors);
    let type_state_policy = nominal_type_boundary_policy(registry);
    let type_state_source_scopes =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.type_state_source_scopes);
    let type_state_raw_alias_patterns =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.raw_state_alias_patterns);
    let type_state_raw_transition_patterns =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.raw_transition_patterns);
    let type_state_required_state_markers =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.required_state_markers);
    let type_state_required_transition_methods =
        type_state_mechanism_field(type_state_policy, |mechanism| &mechanism.required_transition_methods);

    // Crate-boundary config fields (policy-driven-crate-boundaries)
    let crate_boundary_policy = crate_boundary_config_policy(registry);
    let crate_core_crates = crate_boundary_mechanism_field(crate_boundary_policy, |mechanism| &mechanism.core_crates);
    let crate_shell_crates = crate_boundary_mechanism_field(crate_boundary_policy, |mechanism| &mechanism.shell_crates);
    let crate_adapter_crates =
        crate_boundary_mechanism_field(crate_boundary_policy, |mechanism| &mechanism.adapter_crates);
    let crate_forbidden_core_deps =
        crate_boundary_mechanism_field(crate_boundary_policy, |mechanism| &mechanism.forbidden_core_deps);
    let crate_external_domain_patterns =
        crate_boundary_mechanism_field(crate_boundary_policy, |mechanism| &mechanism.external_domain_patterns);
    let crate_dependency_edge_kinds =
        crate_boundary_mechanism_field(crate_boundary_policy, |mechanism| &mechanism.dependency_edge_kinds);

    let mut toml = String::new();
    toml.push_str("schema_version = \"");
    toml.push_str(GENERATED_ARTIFACT_SCHEMA_VERSION);
    toml.push_str("\"\n");
    toml.push_str("generated_from = \"");
    toml.push_str(GENERATED_FROM);
    toml.push_str("\"\n");
    toml.push_str("registry_schema_version = \"");
    toml.push_str(&escape_toml_string(&registry.schema_version));
    toml.push_str("\"\n");
    append_toml_array(&mut toml, "checker_ids", &registry.checker_ids);
    append_toml_array(&mut toml, "families", &families);
    append_toml_array(&mut toml, "hard_policy_ids", &hard_policy_ids);
    append_toml_array(&mut toml, "structural_policy_ids", &structural_policy_ids);
    append_toml_array(&mut toml, "verified_policy_ids", &verified_policy_ids);
    append_toml_array(&mut toml, "bounded_queue_forbidden_apis", &bounded_queue_forbidden_apis);
    append_toml_array(&mut toml, "bounded_queue_approved_apis", &bounded_queue_approved_apis);
    append_toml_array(&mut toml, "bounded_queue_wrapper_scopes", &bounded_queue_wrapper_scopes);
    append_toml_array(&mut toml, "bounded_queue_capacity_witnesses", &bounded_queue_capacity_witnesses);
    append_toml_array(&mut toml, "bounded_queue_backpressure_behaviors", &bounded_queue_backpressure_behaviors);
    append_capability_config(&mut toml, registry);
    append_toml_array(&mut toml, "type_state_source_scopes", &type_state_source_scopes);
    append_toml_array(&mut toml, "type_state_raw_alias_patterns", &type_state_raw_alias_patterns);
    append_toml_array(&mut toml, "type_state_raw_transition_patterns", &type_state_raw_transition_patterns);
    append_toml_array(&mut toml, "type_state_required_state_markers", &type_state_required_state_markers);
    append_toml_array(&mut toml, "type_state_required_transition_methods", &type_state_required_transition_methods);
    append_toml_array(&mut toml, "crate_core_crates", &crate_core_crates);
    append_toml_array(&mut toml, "crate_shell_crates", &crate_shell_crates);
    append_toml_array(&mut toml, "crate_adapter_crates", &crate_adapter_crates);
    append_toml_array(&mut toml, "crate_forbidden_core_deps", &crate_forbidden_core_deps);
    append_toml_array(&mut toml, "crate_external_domain_patterns", &crate_external_domain_patterns);
    append_toml_array(&mut toml, "crate_dependency_edge_kinds", &crate_dependency_edge_kinds);
    append_proof_rail_config(&mut toml, registry);
    toml
}

fn append_capability_config(toml: &mut String, registry: &PolicyRegistry) {
    let capability_policy = effect_capability_policy(registry);
    append_toml_array(
        toml,
        "capability_forbidden_ambient_apis",
        &capability_mechanism_field(capability_policy, |mechanism| &mechanism.forbidden_ambient_apis),
    );
    append_toml_array(
        toml,
        "capability_core_source_scopes",
        &capability_mechanism_field(capability_policy, |mechanism| &mechanism.core_source_scopes),
    );
    append_toml_array(
        toml,
        "capability_shell_constructor_scopes",
        &capability_mechanism_field(capability_policy, |mechanism| &mechanism.shell_constructor_scopes),
    );
    append_toml_array(
        toml,
        "capability_approved_names",
        &capability_mechanism_field(capability_policy, |mechanism| &mechanism.approved_capability_names),
    );
    append_capability_manifest_config(toml, capability_policy);
}

fn append_capability_manifest_config(toml: &mut String, policy: Option<&PolicyEntry>) {
    append_toml_array(
        toml,
        "capability_enforcement_modes",
        &capability_mechanism_string_field(policy, |mechanism| &mechanism.capability_enforcement_mode),
    );
    append_toml_array(
        toml,
        "capability_scope_classes",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_scope_classes),
    );
    append_toml_array(
        toml,
        "capability_families",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_families),
    );
    append_toml_array(
        toml,
        "capability_injected_traits",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_injected_traits),
    );
    append_toml_array(
        toml,
        "capability_tokens",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_tokens),
    );
    append_capability_manifest_scope_config(toml, policy);
    append_capability_manifest_handler_config(toml, policy);
}

fn append_capability_manifest_scope_config(toml: &mut String, policy: Option<&PolicyEntry>) {
    append_toml_array(
        toml,
        "capability_shell_interpreters",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_shell_interpreters),
    );
    append_toml_array(
        toml,
        "capability_adapter_scopes",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_adapter_scopes),
    );
    append_toml_array(
        toml,
        "capability_test_scopes",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_test_scopes),
    );
    append_toml_array(
        toml,
        "capability_generated_scopes",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_generated_scopes),
    );
    append_toml_array(
        toml,
        "capability_remote_execution_scopes",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_remote_execution_scopes),
    );
}

fn append_capability_manifest_handler_config(toml: &mut String, policy: Option<&PolicyEntry>) {
    append_toml_array(
        toml,
        "capability_deterministic_handlers",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_deterministic_handlers),
    );
    append_toml_array(
        toml,
        "capability_handler_replay_commands",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_handler_replay_commands),
    );
    append_toml_array(
        toml,
        "capability_handler_artifacts",
        &capability_mechanism_field(policy, |mechanism| &mechanism.capability_handler_artifacts),
    );
}

fn append_proof_rail_config(toml: &mut String, registry: &PolicyRegistry) {
    append_proof_rail_command_config(toml, registry);
    append_proof_rail_receipt_config(toml, registry);
    append_proof_rail_verifier_config(toml, registry);
    append_proof_rail_runtime_config(toml, registry);
}

fn append_proof_rail_command_config(toml: &mut String, registry: &PolicyRegistry) {
    append_proof_rail_field(toml, "proof_rail_backends", registry, |rail| &rail.backend);
    append_proof_rail_field(toml, "proof_rail_commands", registry, |rail| &rail.command);
    append_proof_rail_fields(toml, "proof_rail_command_argv", registry, |rail| &rail.command_argv);
    append_proof_rail_fields(toml, "proof_rail_source_paths", registry, |rail| &rail.source_paths);
    append_proof_rail_fields(toml, "proof_rail_proof_paths", registry, |rail| &rail.proof_paths);
    append_proof_rail_fields(toml, "proof_rail_receipt_paths", registry, |rail| &rail.receipt_paths);
    append_proof_rail_field(toml, "proof_rail_pass_signals", registry, |rail| &rail.pass_signal);
    append_proof_rail_fields(toml, "proof_rail_trust_boundaries", registry, |rail| &rail.trust_boundaries);
    append_proof_rail_field(toml, "proof_rail_receipt_bindings", registry, |rail| &rail.receipt_binding);
}

fn append_proof_rail_receipt_config(toml: &mut String, registry: &PolicyRegistry) {
    append_proof_rail_field(toml, "proof_rail_receipt_schemas", registry, |rail| {
        &rail.receipt_provenance.schema_version
    });
    append_proof_rail_field(toml, "proof_rail_receipt_runners", registry, |rail| &rail.receipt_provenance.runner);
    append_proof_rail_field(toml, "proof_rail_receipt_environments", registry, |rail| {
        &rail.receipt_provenance.environment
    });
    let requirements = proof_rail_receipt_requirements(registry);
    append_toml_array(toml, "proof_rail_receipt_requires", &requirements);
}

fn append_proof_rail_verifier_config(toml: &mut String, registry: &PolicyRegistry) {
    append_proof_rail_field(toml, "proof_rail_receipt_verifiers", registry, |rail| &rail.receipt_verifier.verifier_id);
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_digests", registry, |rail| {
        &rail.receipt_verifier.verifier_digest
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_binaries", registry, |rail| {
        &rail.receipt_verifier.verifier_binary_path
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_binary_digests", registry, |rail| {
        &rail.receipt_verifier.verifier_binary_digest
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_build_closures", registry, |rail| {
        &rail.receipt_verifier.verifier_build_closure
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_build_closure_digests", registry, |rail| {
        &rail.receipt_verifier.verifier_build_closure_digest
    });
    append_proof_rail_fields(toml, "proof_rail_receipt_verifier_build_closure_paths", registry, |rail| {
        &rail.receipt_verifier.verifier_build_closure_paths
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_nix_derivations", registry, |rail| {
        &rail.receipt_verifier.verifier_nix_derivation
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_nix_derivation_digests", registry, |rail| {
        &rail.receipt_verifier.verifier_nix_derivation_digest
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_nix_closures", registry, |rail| {
        &rail.receipt_verifier.verifier_nix_closure
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_nix_closure_digests", registry, |rail| {
        &rail.receipt_verifier.verifier_nix_closure_digest
    });
    append_proof_rail_fields(toml, "proof_rail_receipt_verifier_nix_closure_paths", registry, |rail| {
        &rail.receipt_verifier.verifier_nix_closure_paths
    });
    append_proof_rail_field(toml, "proof_rail_receipt_verifier_nix_closure_graph_digests", registry, |rail| {
        &rail.receipt_verifier.verifier_nix_closure_graph_digest
    });
    append_proof_rail_fields(toml, "proof_rail_receipt_verifier_nix_closure_graph_paths", registry, |rail| {
        &rail.receipt_verifier.verifier_nix_closure_graph_paths
    });
    append_proof_rail_field(toml, "proof_rail_receipt_toolchains", registry, |rail| {
        &rail.receipt_verifier.toolchain_id
    });
    append_proof_rail_field(toml, "proof_rail_receipt_toolchain_digests", registry, |rail| {
        &rail.receipt_verifier.toolchain_digest
    });
    append_proof_rail_field(toml, "proof_rail_receipt_solvers", registry, |rail| &rail.receipt_verifier.solver_id);
    append_proof_rail_field(toml, "proof_rail_receipt_solver_digests", registry, |rail| {
        &rail.receipt_verifier.solver_digest
    });
}

fn append_proof_rail_runtime_config(toml: &mut String, registry: &PolicyRegistry) {
    append_proof_rail_field(toml, "proof_rail_verifier_modes", registry, |rail| &rail.verifier_mode);
    append_proof_rail_field(toml, "proof_rail_checked_scopes", registry, |rail| &rail.checked_scope);
    append_proof_rail_fields(toml, "proof_rail_environment_allowlist", registry, |rail| &rail.environment_allowlist);
}

fn append_proof_rail_field(
    toml: &mut String,
    name: &str,
    registry: &PolicyRegistry,
    field: impl Fn(&crate::model::ProofRail) -> &String,
) {
    let values = proof_rail_field(registry, field);
    append_toml_array(toml, name, &values);
}

fn append_proof_rail_fields(
    toml: &mut String,
    name: &str,
    registry: &PolicyRegistry,
    field: impl Fn(&crate::model::ProofRail) -> &Vec<String>,
) {
    let values = proof_rail_fields(registry, field);
    append_toml_array(toml, name, &values);
}

fn proof_rail_field(registry: &PolicyRegistry, field: impl Fn(&crate::model::ProofRail) -> &String) -> Vec<String> {
    let mut values = registry
        .policies
        .iter()
        .flat_map(|policy| policy.proof_rails.iter())
        .map(|rail| field(rail).clone())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn proof_rail_receipt_requirements(registry: &PolicyRegistry) -> Vec<String> {
    let mut values = registry
        .policies
        .iter()
        .flat_map(|policy| policy.proof_rails.iter())
        .flat_map(|rail| {
            [
                rail.receipt_provenance.requires_backend.then_some("backend".to_owned()),
                rail.receipt_provenance.requires_trust_boundary.then_some("trust-boundary".to_owned()),
            ]
        })
        .flatten()
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn proof_rail_fields(
    registry: &PolicyRegistry,
    field: impl Fn(&crate::model::ProofRail) -> &Vec<String>,
) -> Vec<String> {
    let mut values = registry
        .policies
        .iter()
        .flat_map(|policy| policy.proof_rails.iter())
        .flat_map(|rail| field(rail).clone())
        .collect::<Vec<_>>();
    values.sort();
    values.dedup();
    values
}

fn bounded_queue_policy(registry: &PolicyRegistry) -> Option<&PolicyEntry> {
    registry.policy_by_id("tigerstyle.embedded.bounded-queues")
}

fn effect_capability_policy(registry: &PolicyRegistry) -> Option<&PolicyEntry> {
    registry.policy_by_id("tigerstyle.architecture.effect-capability-boundaries")
}

fn crate_boundary_config_policy(registry: &PolicyRegistry) -> Option<&PolicyEntry> {
    registry.policy_by_id("tigerstyle.architecture.fcis-crate-boundary")
}

fn nominal_type_boundary_policy(registry: &PolicyRegistry) -> Option<&PolicyEntry> {
    registry.policy_by_id("tigerstyle.idioms.nominal-type-boundaries")
}

fn crate_boundary_mechanism_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &Vec<String>,
) -> Vec<String> {
    policy
        .and_then(|policy| {
            policy
                .mechanisms
                .iter()
                .find(|mechanism| mechanism.kind == crate::model::MechanismKind::CrateBoundary)
        })
        .map_or_else(Vec::new, |mechanism| field(mechanism).clone())
}

fn type_state_mechanism_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &Vec<String>,
) -> Vec<String> {
    policy
        .and_then(|policy| {
            policy.mechanisms.iter().find(|mechanism| mechanism.kind == crate::model::MechanismKind::TypeState)
        })
        .map_or_else(Vec::new, |mechanism| field(mechanism).clone())
}

fn capability_mechanism_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &Vec<String>,
) -> Vec<String> {
    match capability_mechanism(policy) {
        Some(mechanism) => field(mechanism).clone(),
        None => Vec::new(),
    }
}

fn capability_mechanism_string_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &String,
) -> Vec<String> {
    let Some(mechanism) = capability_mechanism(policy) else {
        return Vec::new();
    };
    let value = field(mechanism);
    if value.is_empty() {
        Vec::new()
    } else {
        vec![value.clone()]
    }
}

fn capability_mechanism(policy: Option<&PolicyEntry>) -> Option<&crate::model::Mechanism> {
    policy.and_then(|policy| {
        policy
            .mechanisms
            .iter()
            .find(|mechanism| mechanism.kind == crate::model::MechanismKind::CapabilityApi)
    })
}

fn bounded_queue_mechanism_field(
    policy: Option<&PolicyEntry>,
    field: impl Fn(&crate::model::Mechanism) -> &Vec<String>,
) -> Vec<String> {
    policy
        .and_then(|policy| {
            policy
                .mechanisms
                .iter()
                .find(|mechanism| mechanism.kind == crate::model::MechanismKind::BoundedPrimitive)
        })
        .map_or_else(Vec::new, |mechanism| field(mechanism).clone())
}

fn policy_ids_by_level(registry: &PolicyRegistry, level: EnforcementLevel) -> Vec<String> {
    sorted_policy_ids(registry.policies.iter().filter(|policy| policy.enforcement_level == level))
}

fn policy_ids_by_target_mode(registry: &PolicyRegistry, mode: EnforcementMode) -> Vec<String> {
    sorted_policy_ids(registry.policies.iter().filter(|policy| policy.target_enforcement_mode == mode))
}

fn sorted_policy_ids<'a>(policies: impl Iterator<Item = &'a PolicyEntry>) -> Vec<String> {
    let mut ids = policies.map(|policy| policy.policy_id.clone()).collect::<Vec<_>>();
    ids.sort();
    ids
}

fn policy_families(registry: &PolicyRegistry) -> Vec<String> {
    registry
        .policies
        .iter()
        .map(|policy| policy.pillar.clone())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}

fn append_toml_array(toml: &mut String, name: &str, values: &[String]) {
    toml.push_str(name);
    toml.push_str(" = [");
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            toml.push_str(", ");
        }
        toml.push('"');
        toml.push_str(&escape_toml_string(value));
        toml.push('"');
    }
    toml.push_str("]\n");
}

fn escape_toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use crate::artifacts::render_check_config_toml;
    use crate::artifacts::seed_manifest_artifact;
    use crate::model::PolicyRegistry;

    #[test]
    fn seed_manifest_artifact_preserves_registry_schema() {
        let registry = PolicyRegistry {
            schema_version: "test-schema".to_owned(),
            checker_ids: Vec::new(),
            policies: Vec::new(),
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        };

        let artifact = seed_manifest_artifact(&registry);

        assert_eq!(artifact.schema_version, "1");
        assert_eq!(artifact.registry_schema_version, "test-schema");
        assert!(artifact.seed_manifests.is_empty());
    }

    #[test]
    fn check_config_output_is_stable() {
        let registry = PolicyRegistry {
            schema_version: "test".to_owned(),
            checker_ids: vec!["checker-b".to_owned(), "checker-a".to_owned()],
            policies: Vec::new(),
            seed_manifests: Vec::new(),
            config_receipt_families: Vec::new(),
        };

        let output = render_check_config_toml(&registry);

        assert!(output.starts_with("schema_version = \"1\"\n"));
        assert!(output.contains("checker_ids = [\"checker-b\", \"checker-a\"]\n"));
        assert!(output.contains("hard_policy_ids = []\n"));
    }
}

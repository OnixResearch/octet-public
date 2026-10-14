use std::collections::BTreeSet;
#[cfg(test)]
use std::fs;
#[cfg(test)]
use std::path::Path;

use serde::Deserialize;
use serde::Serialize;

pub const IMPORTED_PROOF_BUNDLE_SCHEMA_VERSION: &str = "octet-imported-proof-bundles/v1";
pub const HASH_ALGORITHM_BLAKE3: &str = "blake3";
pub const EVIDENCE_ORIGIN_EXTERNAL_TRELLIS: &str = "external_trellis_import";
pub const EVIDENCE_TIER_MANUAL_REVIEW: &str = "manual_review_required";
pub const EVIDENCE_TIER_FORMAL_PROOF: &str = "formal_proof";
pub const TRELLIS_PROOF_IR_INTAKE_SCHEMA_VERSION: &str = "octet-trellis-proof-ir-intake/v1";
pub const TRELLIS_PROOF_IR_PROFILE: &str = "trellis-proof-evidence-ir-v1";
pub const TRELLIS_INTAKE_ROLE_RECORDED_ONLY: &str = "recorded_only_reference";
pub const TRELLIS_INTAKE_ROLE_PROMOTION_CANDIDATE: &str = "promotion_candidate";
pub const TRELLIS_INTAKE_ROLE_FORMAL_PROOF: &str = "formal_proof";
const BLAKE3_HEX_LEN: usize = 64;
const TRELLIS_RESULT_PASSED: &str = "passed";
const REQUIRED_TRELLIS_NON_CLAIMS: &[&str] = &[
    "recorded-only reference evidence by default",
    "not verifier soundness",
    "not Rust semantic correctness",
    "Octet formal_proof requires rerun",
];
const FORBIDDEN_TRELLIS_OVERCLAIMS: &[&str] = &[
    "proves whole-program correctness",
    "proves rust semantic correctness",
    "proves verifier soundness",
    "proves downstream correctness",
    "proves release eligibility",
];

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedProofBundleManifest {
    pub bundles: Vec<ImportedProofBundle>,
    pub generated_from: String,
    pub schema_version: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedProofBundle {
    pub bundle_id: String,
    pub evidence_origin: String,
    pub hash_algorithm: String,
    pub intended_evidence_tier: String,
    pub promotion_command_argv: Vec<String>,
    pub proof_artifact: ImportedArtifactIdentity,
    pub proof_path: String,
    pub receipt_artifacts: Vec<ImportedArtifactIdentity>,
    pub repository_label: String,
    pub repository_revision: String,
    pub source_artifact: ImportedArtifactIdentity,
    pub source_path: String,
    pub stderr_artifacts: Vec<ImportedArtifactIdentity>,
    pub stdout_artifacts: Vec<ImportedArtifactIdentity>,
    pub trust_boundary_notes: Vec<String>,
    pub verifier_command_argv: Vec<String>,
    pub verifier_identity: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ImportedArtifactIdentity {
    pub blake3: String,
    pub bytes: u64,
    pub path: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrellisProofIrIntake {
    pub schema_version: String,
    pub evidence_origin: String,
    pub intake_role: String,
    pub source_artifact: ImportedArtifactIdentity,
    pub source_path: String,
    pub proof_ir_artifact: ImportedArtifactIdentity,
    pub proof_ir_profile: String,
    pub verifier_receipt_artifact: ImportedArtifactIdentity,
    pub verifier_identity: String,
    pub toolchain_identity: String,
    pub solver_identity: String,
    pub verifier_command_argv: Vec<String>,
    pub assumptions: Vec<String>,
    pub property_id: String,
    pub property_scope: String,
    pub result_status: String,
    pub non_claims: Vec<String>,
    pub octet_rerun_receipt: Option<TrellisProofIrRerunReceipt>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct TrellisProofIrRerunReceipt {
    pub source_blake3: String,
    pub proof_ir_blake3: String,
    pub verifier_identity: String,
    pub toolchain_identity: String,
    pub solver_identity: String,
    pub command_argv: Vec<String>,
    pub assumptions: Vec<String>,
    pub property_id: String,
    pub property_scope: String,
    pub result_status: String,
    pub receipt_blake3: String,
}

#[cfg(test)]
pub struct ImportedProofArtifactBytes<'a> {
    pub source: &'a [u8],
    pub proof: &'a [u8],
    pub stdout: Vec<&'a [u8]>,
    pub stderr: Vec<&'a [u8]>,
    pub receipts: Vec<&'a [u8]>,
}

#[cfg(test)]
pub struct OwnedImportedProofArtifactBytes {
    pub source: Vec<u8>,
    pub proof: Vec<u8>,
    pub stdout: Vec<Vec<u8>>,
    pub stderr: Vec<Vec<u8>>,
    pub receipts: Vec<Vec<u8>>,
}

#[cfg(test)]
impl OwnedImportedProofArtifactBytes {
    #[must_use]
    pub fn as_borrowed(&self) -> ImportedProofArtifactBytes<'_> {
        ImportedProofArtifactBytes {
            source: &self.source,
            proof: &self.proof,
            stdout: self.stdout.iter().map(Vec::as_slice).collect(),
            stderr: self.stderr.iter().map(Vec::as_slice).collect(),
            receipts: self.receipts.iter().map(Vec::as_slice).collect(),
        }
    }
}

#[must_use]
pub fn validate_imported_proof_manifest_shape(manifest: &ImportedProofBundleManifest) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if manifest.schema_version != IMPORTED_PROOF_BUNDLE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported imported proof bundle schema_version `{}`; expected `{IMPORTED_PROOF_BUNDLE_SCHEMA_VERSION}`",
            manifest.schema_version
        ));
    }
    if manifest.generated_from.trim().is_empty() {
        diagnostics.push("imported proof bundle manifest missing generated_from".to_owned());
    }
    diagnostics.extend(duplicate_bundle_diagnostics(&manifest.bundles));
    for bundle in &manifest.bundles {
        diagnostics.extend(validate_imported_proof_bundle_shape(bundle));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

#[must_use]
pub fn validate_imported_proof_bundle_shape(bundle: &ImportedProofBundle) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_required(&mut diagnostics, "bundle_id", &bundle.bundle_id);
    push_required(&mut diagnostics, "repository_label", &bundle.repository_label);
    push_required(&mut diagnostics, "repository_revision", &bundle.repository_revision);
    push_required(&mut diagnostics, "source_path", &bundle.source_path);
    push_required(&mut diagnostics, "proof_path", &bundle.proof_path);
    push_required(&mut diagnostics, "verifier_identity", &bundle.verifier_identity);
    push_required_collection(&mut diagnostics, "verifier_command_argv", &bundle.verifier_command_argv);
    push_required_collection(&mut diagnostics, "promotion_command_argv", &bundle.promotion_command_argv);
    push_required_collection(&mut diagnostics, "trust_boundary_notes", &bundle.trust_boundary_notes);

    if bundle.hash_algorithm != HASH_ALGORITHM_BLAKE3 {
        diagnostics.push(format!(
            "imported proof bundle `{}` uses unsupported hash_algorithm `{}`",
            bundle.bundle_id, bundle.hash_algorithm
        ));
    }
    if bundle.evidence_origin != EVIDENCE_ORIGIN_EXTERNAL_TRELLIS {
        diagnostics.push(format!(
            "imported proof bundle `{}` uses unsupported evidence_origin `{}`",
            bundle.bundle_id, bundle.evidence_origin
        ));
    }
    if bundle.intended_evidence_tier != EVIDENCE_TIER_MANUAL_REVIEW {
        diagnostics.push(format!(
            "imported proof bundle `{}` must stay `{EVIDENCE_TIER_MANUAL_REVIEW}`, got `{}`",
            bundle.bundle_id, bundle.intended_evidence_tier
        ));
    }
    if bundle.intended_evidence_tier == EVIDENCE_TIER_FORMAL_PROOF {
        diagnostics.push(format!(
            "imported proof bundle `{}` cannot claim formal_proof without Octet rerun receipt",
            bundle.bundle_id
        ));
    }
    diagnostics.extend(validate_imported_artifact_identity("source_artifact", &bundle.source_artifact));
    diagnostics.extend(validate_imported_artifact_identity("proof_artifact", &bundle.proof_artifact));
    for artifact in &bundle.stdout_artifacts {
        diagnostics.extend(validate_imported_artifact_identity("stdout_artifacts", artifact));
    }
    for artifact in &bundle.stderr_artifacts {
        diagnostics.extend(validate_imported_artifact_identity("stderr_artifacts", artifact));
    }
    for artifact in &bundle.receipt_artifacts {
        diagnostics.extend(validate_imported_artifact_identity("receipt_artifacts", artifact));
    }
    diagnostics.extend(validate_imported_non_claims(bundle));
    diagnostics.extend(validate_imported_promotion_command(bundle));
    diagnostics
}

#[must_use]
pub fn validate_trellis_proof_ir_intake(intake: &TrellisProofIrIntake) -> Vec<String> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(validate_trellis_intake_shape(intake));
    diagnostics.extend(validate_trellis_intake_non_claims(intake));
    diagnostics.extend(validate_trellis_intake_promotion(intake));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

fn validate_trellis_intake_shape(intake: &TrellisProofIrIntake) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if intake.schema_version != TRELLIS_PROOF_IR_INTAKE_SCHEMA_VERSION {
        diagnostics.push(format!(
            "Trellis proof IR intake schema_version `{}` is unsupported; expected `{TRELLIS_PROOF_IR_INTAKE_SCHEMA_VERSION}`",
            intake.schema_version
        ));
    }
    if intake.evidence_origin != EVIDENCE_ORIGIN_EXTERNAL_TRELLIS {
        diagnostics
            .push(format!("Trellis proof IR intake evidence_origin `{}` is unsupported", intake.evidence_origin));
    }
    if !matches!(
        intake.intake_role.as_str(),
        TRELLIS_INTAKE_ROLE_RECORDED_ONLY | TRELLIS_INTAKE_ROLE_PROMOTION_CANDIDATE | TRELLIS_INTAKE_ROLE_FORMAL_PROOF
    ) {
        diagnostics.push(format!("Trellis proof IR intake role `{}` is unsupported", intake.intake_role));
    }
    if intake.proof_ir_profile != TRELLIS_PROOF_IR_PROFILE {
        diagnostics.push(format!(
            "Trellis proof IR profile `{}` is unsupported; expected `{TRELLIS_PROOF_IR_PROFILE}`",
            intake.proof_ir_profile
        ));
    }
    push_required(&mut diagnostics, "source_path", &intake.source_path);
    push_required(&mut diagnostics, "verifier_identity", &intake.verifier_identity);
    push_required(&mut diagnostics, "toolchain_identity", &intake.toolchain_identity);
    push_required(&mut diagnostics, "solver_identity", &intake.solver_identity);
    push_required(&mut diagnostics, "property_id", &intake.property_id);
    push_required(&mut diagnostics, "property_scope", &intake.property_scope);
    push_required_collection(&mut diagnostics, "verifier_command_argv", &intake.verifier_command_argv);
    push_required_collection(&mut diagnostics, "assumptions", &intake.assumptions);
    diagnostics.extend(validate_imported_artifact_identity("source_artifact", &intake.source_artifact));
    diagnostics.extend(validate_imported_artifact_identity("proof_ir_artifact", &intake.proof_ir_artifact));
    diagnostics
        .extend(validate_imported_artifact_identity("verifier_receipt_artifact", &intake.verifier_receipt_artifact));
    if intake.source_artifact.path != intake.source_path {
        diagnostics.push(format!(
            "Trellis proof IR source linkage is stale: source_path `{}` does not match source artifact path `{}`",
            intake.source_path, intake.source_artifact.path
        ));
    }
    if intake.result_status != TRELLIS_RESULT_PASSED {
        diagnostics.push(format!(
            "Trellis proof IR verifier result_status `{}` is not `{TRELLIS_RESULT_PASSED}`",
            intake.result_status
        ));
    }
    diagnostics
}

fn validate_trellis_intake_non_claims(intake: &TrellisProofIrIntake) -> Vec<String> {
    let mut diagnostics = Vec::new();
    for required in REQUIRED_TRELLIS_NON_CLAIMS {
        if !intake.non_claims.iter().any(|claim| claim == required) {
            diagnostics.push(format!("Trellis proof IR intake missing non-claim `{required}`"));
        }
    }
    let joined = intake.non_claims.join(" ").to_lowercase();
    for forbidden in FORBIDDEN_TRELLIS_OVERCLAIMS {
        if joined.contains(forbidden) {
            diagnostics.push(format!("Trellis proof IR intake contains forbidden overclaim `{forbidden}`"));
        }
    }
    diagnostics
}

fn validate_trellis_intake_promotion(intake: &TrellisProofIrIntake) -> Vec<String> {
    if intake.intake_role == TRELLIS_INTAKE_ROLE_RECORDED_ONLY {
        return Vec::new();
    }
    let Some(rerun) = intake.octet_rerun_receipt.as_ref() else {
        return vec![format!(
            "Trellis proof IR intake role `{}` requires an Octet-owned rerun receipt",
            intake.intake_role
        )];
    };

    let mut diagnostics = Vec::new();
    if !is_blake3_hex(&rerun.receipt_blake3) {
        diagnostics.push("Trellis proof IR rerun receipt has malformed BLAKE3 digest".to_owned());
    }
    compare_promotion_field(&mut diagnostics, "source_blake3", &intake.source_artifact.blake3, &rerun.source_blake3);
    compare_promotion_field(
        &mut diagnostics,
        "proof_ir_blake3",
        &intake.proof_ir_artifact.blake3,
        &rerun.proof_ir_blake3,
    );
    compare_promotion_field(&mut diagnostics, "verifier_identity", &intake.verifier_identity, &rerun.verifier_identity);
    compare_promotion_field(
        &mut diagnostics,
        "toolchain_identity",
        &intake.toolchain_identity,
        &rerun.toolchain_identity,
    );
    compare_promotion_field(&mut diagnostics, "solver_identity", &intake.solver_identity, &rerun.solver_identity);
    compare_promotion_field(&mut diagnostics, "property_id", &intake.property_id, &rerun.property_id);
    compare_promotion_field(&mut diagnostics, "property_scope", &intake.property_scope, &rerun.property_scope);
    compare_promotion_field(&mut diagnostics, "result_status", TRELLIS_RESULT_PASSED, &rerun.result_status);
    if intake.verifier_command_argv != rerun.command_argv {
        diagnostics.push("Trellis proof IR promotion command identity mismatch".to_owned());
    }
    if intake.assumptions != rerun.assumptions {
        diagnostics.push("Trellis proof IR promotion assumptions mismatch".to_owned());
    }
    diagnostics
}

fn compare_promotion_field(diagnostics: &mut Vec<String>, field: &str, expected: &str, actual: &str) {
    if expected != actual {
        diagnostics.push(format!("Trellis proof IR promotion {field} mismatch: expected `{expected}`, got `{actual}`"));
    }
}

#[must_use]
#[cfg(test)]
pub fn validate_imported_proof_bundle_artifacts(
    bundle: &ImportedProofBundle,
    artifacts: &ImportedProofArtifactBytes<'_>,
) -> Vec<String> {
    let mut diagnostics = validate_imported_proof_bundle_shape(bundle);
    diagnostics.extend(validate_artifact_bytes("source_artifact", &bundle.source_artifact, artifacts.source));
    diagnostics.extend(validate_artifact_bytes("proof_artifact", &bundle.proof_artifact, artifacts.proof));
    diagnostics.extend(validate_artifact_byte_list("stdout_artifacts", &bundle.stdout_artifacts, &artifacts.stdout));
    diagnostics.extend(validate_artifact_byte_list("stderr_artifacts", &bundle.stderr_artifacts, &artifacts.stderr));
    diagnostics.extend(validate_artifact_byte_list(
        "receipt_artifacts",
        &bundle.receipt_artifacts,
        &artifacts.receipts,
    ));
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

#[cfg(test)]
pub fn validate_imported_proof_manifest_files(
    root: &Path,
    manifest: &ImportedProofBundleManifest,
) -> Result<Vec<String>, String> {
    let shape_diagnostics = validate_imported_proof_manifest_shape(manifest);
    if !shape_diagnostics.is_empty() {
        return Ok(shape_diagnostics);
    }
    let owned_artifacts = manifest
        .bundles
        .iter()
        .map(|bundle| read_imported_proof_bundle_artifacts(root, bundle))
        .collect::<Result<Vec<_>, _>>()?;
    let borrowed_artifacts =
        owned_artifacts.iter().map(OwnedImportedProofArtifactBytes::as_borrowed).collect::<Vec<_>>();
    Ok(validate_imported_proof_manifest_artifacts(manifest, &borrowed_artifacts))
}

#[must_use]
#[cfg(test)]
pub fn validate_imported_proof_manifest_artifacts(
    manifest: &ImportedProofBundleManifest,
    artifacts: &[ImportedProofArtifactBytes<'_>],
) -> Vec<String> {
    let mut diagnostics = validate_imported_proof_manifest_shape(manifest);
    if manifest.bundles.len() != artifacts.len() {
        diagnostics.push(format!(
            "imported proof artifact byte set count {} does not match bundle count {}",
            artifacts.len(),
            manifest.bundles.len()
        ));
    }
    for (bundle, bytes) in manifest.bundles.iter().zip(artifacts.iter()) {
        diagnostics.extend(validate_imported_proof_bundle_artifacts(bundle, bytes));
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

#[cfg(test)]
fn read_imported_proof_bundle_artifacts(
    root: &Path,
    bundle: &ImportedProofBundle,
) -> Result<OwnedImportedProofArtifactBytes, String> {
    Ok(OwnedImportedProofArtifactBytes {
        source: read_artifact_bytes(root, &bundle.source_artifact)?,
        proof: read_artifact_bytes(root, &bundle.proof_artifact)?,
        stdout: read_artifact_bytes_list(root, &bundle.stdout_artifacts)?,
        stderr: read_artifact_bytes_list(root, &bundle.stderr_artifacts)?,
        receipts: read_artifact_bytes_list(root, &bundle.receipt_artifacts)?,
    })
}

#[cfg(test)]
fn read_artifact_bytes_list(root: &Path, artifacts: &[ImportedArtifactIdentity]) -> Result<Vec<Vec<u8>>, String> {
    artifacts.iter().map(|artifact| read_artifact_bytes(root, artifact)).collect::<Result<Vec<_>, _>>()
}

#[cfg(test)]
fn read_artifact_bytes(root: &Path, artifact: &ImportedArtifactIdentity) -> Result<Vec<u8>, String> {
    let path = root.join(&artifact.path);
    fs::read(&path).map_err(|error| format!("read imported proof artifact `{}`: {error}", artifact.path))
}

#[must_use]
pub fn bundle_artifacts(bundle: &ImportedProofBundle) -> Vec<ImportedArtifactIdentity> {
    let mut artifacts = vec![bundle.source_artifact.clone(), bundle.proof_artifact.clone()];
    artifacts.extend(bundle.stdout_artifacts.clone());
    artifacts.extend(bundle.stderr_artifacts.clone());
    artifacts.extend(bundle.receipt_artifacts.clone());
    artifacts
}

fn duplicate_bundle_diagnostics(bundles: &[ImportedProofBundle]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let mut bundle_ids = BTreeSet::new();
    let mut identities = BTreeSet::new();
    for bundle in bundles {
        if !bundle_ids.insert(bundle.bundle_id.clone()) {
            diagnostics.push(format!("duplicate imported proof bundle_id `{}`", bundle.bundle_id));
        }
        let local_identities = bundle_artifacts(bundle)
            .into_iter()
            .map(|artifact| format!("{}:{}", artifact.path, artifact.blake3))
            .collect::<BTreeSet<_>>();
        for identity in local_identities {
            if !identities.insert(identity.clone()) {
                diagnostics.push(format!("duplicate imported proof artifact identity `{identity}`"));
            }
        }
    }
    diagnostics
}

#[cfg(test)]
fn validate_artifact_byte_list(
    label: &str,
    identities: &[ImportedArtifactIdentity],
    byte_sets: &[&[u8]],
) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if identities.len() != byte_sets.len() {
        diagnostics.push(format!(
            "imported proof `{label}` byte set count {} does not match artifact count {}",
            byte_sets.len(),
            identities.len()
        ));
    }
    for (identity, bytes) in identities.iter().zip(byte_sets.iter()) {
        diagnostics.extend(validate_artifact_bytes(label, identity, bytes));
    }
    diagnostics
}

#[cfg(test)]
fn validate_artifact_bytes(label: &str, identity: &ImportedArtifactIdentity, bytes: &[u8]) -> Vec<String> {
    let mut diagnostics = validate_imported_artifact_identity(label, identity);
    let actual_bytes = u64::try_from(bytes.len()).expect("artifact bytes length should fit u64");
    if identity.bytes != actual_bytes {
        diagnostics.push(format!(
            "imported proof artifact `{label}` byte length mismatch for `{}`; expected {}, got {actual_bytes}",
            identity.path, identity.bytes
        ));
    }
    let actual_blake3 = blake3::hash(bytes).to_hex().to_string();
    if is_blake3_hex(&identity.blake3) && identity.blake3 != actual_blake3 {
        diagnostics.push(format!(
            "imported proof artifact `{label}` hash mismatch for `{}`; expected `{}`, got `{actual_blake3}`",
            identity.path, identity.blake3
        ));
    }
    diagnostics
}

fn validate_imported_artifact_identity(label: &str, identity: &ImportedArtifactIdentity) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_required(&mut diagnostics, &format!("{label}.path"), &identity.path);
    push_required(&mut diagnostics, &format!("{label}.blake3"), &identity.blake3);
    if !is_normalized_relative_path(&identity.path) {
        diagnostics
            .push(format!("imported proof artifact `{label}` path `{}` is not normalized relative", identity.path));
    }
    if !is_blake3_hex(&identity.blake3) {
        diagnostics
            .push(format!("imported proof artifact `{label}` has malformed BLAKE3 digest `{}`", identity.blake3));
    }
    diagnostics
}

fn validate_imported_non_claims(bundle: &ImportedProofBundle) -> Vec<String> {
    let mut diagnostics = Vec::new();
    let joined = bundle.trust_boundary_notes.join(" ").to_lowercase();
    for required in [
        "blake3 identifies exact artifact bytes",
        "does not by itself prove proof correctness",
        "octet formal_proof requires rerunning",
    ] {
        if !joined.contains(required) {
            diagnostics
                .push(format!("imported proof bundle `{}` missing trust-boundary note `{required}`", bundle.bundle_id));
        }
    }
    for forbidden in [
        "proves whole-program correctness",
        "provides certification",
        "provides business-domain validation",
        "proves semantic equivalence",
        "guarantees full safety",
    ] {
        if joined.contains(forbidden) {
            diagnostics
                .push(format!("imported proof bundle `{}` contains unscoped claim `{forbidden}`", bundle.bundle_id));
        }
    }
    diagnostics
}

fn validate_imported_promotion_command(bundle: &ImportedProofBundle) -> Vec<String> {
    if bundle.promotion_command_argv.iter().any(|part| part == "octet")
        && bundle.promotion_command_argv.iter().any(|part| part == "prove")
    {
        Vec::new()
    } else {
        vec![format!(
            "imported proof bundle `{}` missing Octet prove promotion command",
            bundle.bundle_id
        )]
    }
}

fn push_required(diagnostics: &mut Vec<String>, field: &str, value: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("imported proof bundle missing required field `{field}`"));
    }
}

fn push_required_collection(diagnostics: &mut Vec<String>, field: &str, values: &[String]) {
    if values.is_empty() || values.iter().any(|value| value.trim().is_empty()) {
        diagnostics.push(format!("imported proof bundle missing required field `{field}`"));
    }
}

fn is_normalized_relative_path(path: &str) -> bool {
    !path.trim().is_empty()
        && !path.starts_with('/')
        && !path.contains('\\')
        && !path.split('/').any(|component| component == ".." || component.is_empty())
}

fn is_blake3_hex(value: &str) -> bool {
    value.len() == BLAKE3_HEX_LEN && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    const GENERATED_IMPORTS: &str = include_str!("../../standards/proof-imports/generated/trellis-proof-imports.json");
    const SOURCE_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference_source.rs");
    const PROOF_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference_proof.rs");
    const STDOUT_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference.stdout");
    const STDERR_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference.stderr");
    const RECEIPT_BYTES: &[u8] =
        include_bytes!("../../standards/proof-imports/fixtures/trellis/ucan_proof_reference.receipt.json");

    #[test]
    fn generated_trellis_import_fixture_validates() {
        let manifest = generated_manifest();
        let artifacts = [fixture_bytes()];

        let diagnostics = validate_imported_proof_manifest_artifacts(&manifest, &artifacts);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        let bundle = manifest.bundles.first().expect("bundle fixture");
        assert_eq!(bundle.repository_label, "../trellis");
        assert_eq!(bundle.hash_algorithm, HASH_ALGORITHM_BLAKE3);
        assert_eq!(bundle.intended_evidence_tier, EVIDENCE_TIER_MANUAL_REVIEW);
        assert_ne!(bundle.intended_evidence_tier, EVIDENCE_TIER_FORMAL_PROOF);
        assert!(bundle.promotion_command_argv.iter().any(|part| part == "prove"));
    }

    #[test]
    fn generated_trellis_import_json_is_deterministic() {
        let manifest = generated_manifest();
        let rendered = serde_json::to_value(&manifest).expect("render manifest value");
        let generated = serde_json::from_str::<serde_json::Value>(GENERATED_IMPORTS).expect("parse generated value");

        assert_eq!(rendered, generated);
        assert!(GENERATED_IMPORTS.contains("\"schema_version\": \"octet-imported-proof-bundles/v1\""));
    }

    #[test]
    fn trellis_import_file_shell_reads_fixture_artifacts() {
        let manifest = generated_manifest();

        let diagnostics =
            validate_imported_proof_manifest_files(&repo_root(), &manifest).expect("read fixture artifacts");

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn trellis_import_validation_rejects_stale_hash_and_formal_overclaim() {
        let mut manifest = generated_manifest();
        let bundle = manifest.bundles.first_mut().expect("bundle fixture");
        bundle.source_artifact.blake3 = "0".repeat(BLAKE3_HEX_LEN);
        bundle.intended_evidence_tier = EVIDENCE_TIER_FORMAL_PROOF.to_owned();
        bundle.trust_boundary_notes = vec!["provides certification".to_owned()];
        let artifacts = [fixture_bytes()];

        let diagnostics = validate_imported_proof_manifest_artifacts(&manifest, &artifacts);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("hash mismatch")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("cannot claim formal_proof")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("provides certification")));
    }

    #[test]
    fn trellis_import_validation_rejects_missing_malformed_and_duplicate_fields() {
        let mut manifest = generated_manifest();
        let duplicate = manifest.bundles.first().expect("bundle fixture").clone();
        let bundle = manifest.bundles.first_mut().expect("bundle fixture");
        bundle.repository_revision.clear();
        bundle.hash_algorithm = "sha256".to_owned();
        bundle.proof_artifact.path = "../bad".to_owned();
        bundle.proof_artifact.blake3 = "bad-digest".to_owned();
        manifest.bundles.push(duplicate);
        let artifacts = [fixture_bytes(), fixture_bytes()];

        let diagnostics = validate_imported_proof_manifest_artifacts(&manifest, &artifacts);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("repository_revision")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported hash_algorithm")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed BLAKE3")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("not normalized relative")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("duplicate imported proof bundle_id")));
    }

    #[test]
    fn trellis_proof_ir_reference_intake_stays_recorded_only_without_rerun() {
        let intake = valid_trellis_proof_ir_intake(TRELLIS_INTAKE_ROLE_RECORDED_ONLY, None);

        let diagnostics = validate_trellis_proof_ir_intake(&intake);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
        assert_eq!(intake.intake_role, TRELLIS_INTAKE_ROLE_RECORDED_ONLY);
        assert!(intake.octet_rerun_receipt.is_none());
    }

    #[test]
    fn trellis_proof_ir_promotion_candidate_accepts_matching_octet_rerun() {
        let base = valid_trellis_proof_ir_intake(TRELLIS_INTAKE_ROLE_RECORDED_ONLY, None);
        let rerun = matching_trellis_rerun(&base);
        let intake = valid_trellis_proof_ir_intake(TRELLIS_INTAKE_ROLE_FORMAL_PROOF, Some(rerun));

        let diagnostics = validate_trellis_proof_ir_intake(&intake);

        assert!(diagnostics.is_empty(), "unexpected diagnostics: {diagnostics:?}");
    }

    #[test]
    fn trellis_proof_ir_intake_rejects_stale_incomplete_and_overclaiming_reference() {
        let mut intake = valid_trellis_proof_ir_intake(TRELLIS_INTAKE_ROLE_RECORDED_ONLY, None);
        intake.source_path = "src/stale.rs".to_owned();
        intake.verifier_identity.clear();
        intake.assumptions.clear();
        intake.property_id.clear();
        intake.result_status = "failed".to_owned();
        intake.non_claims = vec!["proves release eligibility".to_owned()];

        let diagnostics = validate_trellis_proof_ir_intake(&intake);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("source linkage is stale")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("verifier_identity")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("assumptions")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("property_id")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("result_status")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("forbidden overclaim")));
    }

    #[test]
    fn trellis_proof_ir_promotion_rejects_missing_or_mismatched_octet_rerun() {
        let unauthorized = valid_trellis_proof_ir_intake(TRELLIS_INTAKE_ROLE_FORMAL_PROOF, None);
        let mut candidate_base = valid_trellis_proof_ir_intake(TRELLIS_INTAKE_ROLE_RECORDED_ONLY, None);
        let mut rerun = matching_trellis_rerun(&candidate_base);
        rerun.source_blake3 = "0".repeat(BLAKE3_HEX_LEN);
        rerun.verifier_identity = "different-verifier".to_owned();
        rerun.property_scope = "different-scope".to_owned();
        rerun.command_argv = vec!["different-command".to_owned()];
        rerun.assumptions = vec!["different-assumption".to_owned()];
        rerun.result_status = "failed".to_owned();
        rerun.receipt_blake3 = "not-a-digest".to_owned();
        candidate_base.intake_role = TRELLIS_INTAKE_ROLE_PROMOTION_CANDIDATE.to_owned();
        candidate_base.octet_rerun_receipt = Some(rerun);

        let unauthorized_diagnostics = validate_trellis_proof_ir_intake(&unauthorized);
        let mismatch_diagnostics = validate_trellis_proof_ir_intake(&candidate_base);

        assert!(
            unauthorized_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("requires an Octet-owned rerun"))
        );
        assert!(mismatch_diagnostics.iter().any(|diagnostic| diagnostic.contains("source_blake3 mismatch")));
        assert!(mismatch_diagnostics.iter().any(|diagnostic| diagnostic.contains("verifier_identity mismatch")));
        assert!(mismatch_diagnostics.iter().any(|diagnostic| diagnostic.contains("property_scope mismatch")));
        assert!(mismatch_diagnostics.iter().any(|diagnostic| diagnostic.contains("command identity mismatch")));
        assert!(mismatch_diagnostics.iter().any(|diagnostic| diagnostic.contains("assumptions mismatch")));
        assert!(mismatch_diagnostics.iter().any(|diagnostic| diagnostic.contains("malformed BLAKE3")));
    }

    fn valid_trellis_proof_ir_intake(
        intake_role: &str,
        octet_rerun_receipt: Option<TrellisProofIrRerunReceipt>,
    ) -> TrellisProofIrIntake {
        TrellisProofIrIntake {
            schema_version: TRELLIS_PROOF_IR_INTAKE_SCHEMA_VERSION.to_owned(),
            evidence_origin: EVIDENCE_ORIGIN_EXTERNAL_TRELLIS.to_owned(),
            intake_role: intake_role.to_owned(),
            source_artifact: fixture_artifact("src/ucan_proof_reference.rs", SOURCE_BYTES),
            source_path: "src/ucan_proof_reference.rs".to_owned(),
            proof_ir_artifact: fixture_artifact("proof-ir/ucan_proof_reference.preserves", b"trellis proof ir"),
            proof_ir_profile: TRELLIS_PROOF_IR_PROFILE.to_owned(),
            verifier_receipt_artifact: fixture_artifact("receipts/ucan_proof_reference.receipt.json", RECEIPT_BYTES),
            verifier_identity: "verus:trellis-fixture".to_owned(),
            toolchain_identity: "rust-toolchain:trellis-fixture".to_owned(),
            solver_identity: "z3:trellis-fixture".to_owned(),
            verifier_command_argv: vec!["verus".to_owned(), "src/ucan_proof_reference.rs".to_owned()],
            assumptions: vec!["trusted-boundary:verus-soundness-reviewed".to_owned()],
            property_id: "trellis.ucan.digest_admission".to_owned(),
            property_scope: "ucan digest admission reference proof".to_owned(),
            result_status: TRELLIS_RESULT_PASSED.to_owned(),
            non_claims: REQUIRED_TRELLIS_NON_CLAIMS.iter().map(|non_claim| (*non_claim).to_owned()).collect(),
            octet_rerun_receipt,
        }
    }

    fn matching_trellis_rerun(intake: &TrellisProofIrIntake) -> TrellisProofIrRerunReceipt {
        TrellisProofIrRerunReceipt {
            source_blake3: intake.source_artifact.blake3.clone(),
            proof_ir_blake3: intake.proof_ir_artifact.blake3.clone(),
            verifier_identity: intake.verifier_identity.clone(),
            toolchain_identity: intake.toolchain_identity.clone(),
            solver_identity: intake.solver_identity.clone(),
            command_argv: intake.verifier_command_argv.clone(),
            assumptions: intake.assumptions.clone(),
            property_id: intake.property_id.clone(),
            property_scope: intake.property_scope.clone(),
            result_status: TRELLIS_RESULT_PASSED.to_owned(),
            receipt_blake3: intake.verifier_receipt_artifact.blake3.clone(),
        }
    }

    fn fixture_artifact(path: &str, bytes: &[u8]) -> ImportedArtifactIdentity {
        ImportedArtifactIdentity {
            blake3: blake3::hash(bytes).to_hex().to_string(),
            bytes: u64::try_from(bytes.len()).expect("fixture bytes length should fit u64"),
            path: path.to_owned(),
        }
    }

    fn generated_manifest() -> ImportedProofBundleManifest {
        serde_json::from_str(GENERATED_IMPORTS).expect("generated imports JSON")
    }

    fn fixture_bytes() -> ImportedProofArtifactBytes<'static> {
        ImportedProofArtifactBytes {
            source: SOURCE_BYTES,
            proof: PROOF_BYTES,
            stdout: vec![STDOUT_BYTES],
            stderr: vec![STDERR_BYTES],
            receipts: vec![RECEIPT_BYTES],
        }
    }

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .expect("cargo-octet has workspace parent")
            .to_path_buf()
    }
}

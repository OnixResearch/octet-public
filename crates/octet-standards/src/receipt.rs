//! Runtime-emitted structured acceptance receipts.

use serde::Deserialize;
use serde::Serialize;

/// Acceptance receipt schema emitted by standards CLI commands.
pub const ACCEPTANCE_RECEIPT_SCHEMA_VERSION: &str = "octet-acceptance-receipt/v1";

/// Runtime status recorded in acceptance receipts.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum AcceptanceStatus {
    /// Stage or command passed.
    Passed,
    /// Stage or command failed.
    Failed,
    /// Stage or command was blocked before execution.
    Blocked,
}

/// Digest of a config, input, artifact, or evidence item.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptDigest {
    /// Digest algorithm.
    pub algorithm: String,
    /// Hex-encoded digest value.
    pub value: String,
}

/// Input file consumed by the receipt-emitting command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptInput {
    /// Source-root-relative or absolute path consumed by the command.
    pub path: String,
    /// BLAKE3 digest of the exact bytes consumed.
    pub blake3: String,
}

/// Stage outcome recorded by a receipt-emitting command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptStage {
    /// Stable stage name.
    pub name: String,
    /// Stage status.
    pub status: AcceptanceStatus,
}

/// Artifact produced by a receipt-emitting command.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptArtifact {
    /// Artifact path.
    pub path: String,
    /// BLAKE3 digest of artifact bytes.
    pub blake3: String,
    /// Artifact byte length.
    pub bytes: u64,
}

/// Verification command or replay command bound into the receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptVerificationCommand {
    /// Command line or reproducible command description.
    pub command: String,
    /// Command status.
    pub status: AcceptanceStatus,
}

/// Verification evidence artifact referenced by the receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptEvidence {
    /// Evidence path.
    pub path: String,
    /// BLAKE3 digest of evidence bytes.
    pub blake3: String,
}

/// Cross-gate provenance record for a standards receipt artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ReceiptProvenanceRecord {
    /// Provenance schema version.
    pub schema_version: String,
    /// Source family identifier.
    pub source_id: String,
    /// Checker or producer identifier.
    pub checker_id: String,
    /// Evidence tier.
    pub evidence_tier: String,
    /// Artifact path or locator.
    pub artifact_locator: String,
    /// BLAKE3 hash of the exact artifact bytes.
    pub blake3: String,
    /// Checked scope.
    pub checked_scope: String,
    /// Optional verifier identity for formal proofs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub verifier_identity: Option<String>,
    /// Optional proof artifact identity for formal proofs.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub proof_artifact_identity: Option<String>,
    /// Optional required human action.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub human_action: Option<String>,
    /// Parent artifacts summarized by this artifact.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parent_artifacts: Vec<String>,
}

/// Structured acceptance receipt emitted after a successful standards run.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct AcceptanceReceipt {
    /// Receipt schema version.
    pub schema_version: String,
    /// Stable run identifier.
    pub run_id: String,
    /// Command start timestamp.
    pub started_at: String,
    /// Command finish timestamp.
    pub finished_at: String,
    /// Tool name.
    pub tool: String,
    /// Tool version.
    pub tool_version: String,
    /// Git revision for the source checkout when available.
    pub git_revision: String,
    /// Digest of the Nickel-exported profile config consumed by the command.
    pub config_digest: String,
    /// Structured config digest with algorithm metadata.
    pub config_digest_detail: ReceiptDigest,
    /// Inputs consumed by the command.
    pub inputs: Vec<ReceiptInput>,
    /// Stage outcomes.
    pub stages: Vec<ReceiptStage>,
    /// Artifacts produced by the command.
    pub artifacts: Vec<ReceiptArtifact>,
    /// Overall command status.
    pub status: AcceptanceStatus,
    /// Structured warnings. Raw logs do not belong here.
    pub warnings: Vec<String>,
    /// Structured errors. Raw logs do not belong here.
    pub errors: Vec<String>,
    /// Verification or replay commands.
    pub verification_commands: Vec<ReceiptVerificationCommand>,
    /// Evidence references.
    pub verification_evidence: Vec<ReceiptEvidence>,
    /// Cross-gate provenance records for generated policy artifacts.
    pub provenance_records: Vec<ReceiptProvenanceRecord>,
    /// Optional raw-log path reference; empty means no raw log is an acceptance record.
    pub raw_log_path: String,
}

/// Validated exported Nickel run profile consumed by receipt-emitting commands.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RunProfile {
    /// Profile schema version.
    pub schema_version: String,
    /// Profile identifier.
    pub profile_id: String,
    /// Source paths selected by this profile.
    pub source_paths: Vec<String>,
    /// Artifact output directory selected by this profile.
    pub artifact_out_dir: String,
    /// Receipt output directory selected by this profile.
    pub receipt_out_dir: String,
    /// Config digest algorithm; currently only `blake3` is supported.
    pub config_digest_algorithm: String,
    /// Enabled policy families.
    pub enabled_policy_families: Vec<String>,
    /// Secret reference handles. Values are references, not raw secrets.
    #[serde(default)]
    pub secret_refs: Vec<SecretReference>,
}

/// Secret reference handle carried by exported run profiles.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct SecretReference {
    /// Stable reference identifier.
    pub id: String,
    /// External secret reference handle.
    #[serde(rename = "ref")]
    pub reference: String,
}

impl AcceptanceReceipt {
    /// Builds a passing acceptance receipt for a generated artifact bundle.
    #[must_use]
    pub fn passed_export_artifacts(input: ExportReceiptInput) -> Self {
        let config_digest = input.config_digest.value.clone();
        Self {
            schema_version: ACCEPTANCE_RECEIPT_SCHEMA_VERSION.to_owned(),
            run_id: input.run_id,
            started_at: input.started_at,
            finished_at: input.finished_at,
            tool: input.tool,
            tool_version: input.tool_version,
            git_revision: input.git_revision,
            config_digest,
            config_digest_detail: input.config_digest,
            inputs: input.inputs,
            stages: passed_export_stages(),
            artifacts: input.artifacts,
            status: AcceptanceStatus::Passed,
            warnings: Vec::new(),
            errors: Vec::new(),
            verification_commands: input.verification_commands,
            verification_evidence: input.verification_evidence,
            provenance_records: input.provenance_records,
            raw_log_path: String::new(),
        }
    }
}

/// Constructor input for an export-artifacts acceptance receipt.
pub struct ExportReceiptInput {
    /// Stable run identifier.
    pub run_id: String,
    /// Command start timestamp.
    pub started_at: String,
    /// Command finish timestamp.
    pub finished_at: String,
    /// Tool name.
    pub tool: String,
    /// Tool version.
    pub tool_version: String,
    /// Git revision for the source checkout when available.
    pub git_revision: String,
    /// Digest of the exported Nickel profile.
    pub config_digest: ReceiptDigest,
    /// Command inputs.
    pub inputs: Vec<ReceiptInput>,
    /// Generated artifacts.
    pub artifacts: Vec<ReceiptArtifact>,
    /// Verification commands.
    pub verification_commands: Vec<ReceiptVerificationCommand>,
    /// Evidence references.
    pub verification_evidence: Vec<ReceiptEvidence>,
    /// Cross-gate provenance records.
    pub provenance_records: Vec<ReceiptProvenanceRecord>,
}

fn passed_export_stages() -> Vec<ReceiptStage> {
    ["registry-load", "profile-load", "artifact-export", "receipt-write"]
        .into_iter()
        .map(|name| ReceiptStage {
            name: name.to_owned(),
            status: AcceptanceStatus::Passed,
        })
        .collect()
}

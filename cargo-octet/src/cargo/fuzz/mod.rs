// r[impl octet.dynamic_evidence.cargo_fuzz_profile]
// r[impl octet.dynamic_evidence.cargo_fuzz_receipt]
pub const PROFILE_ID: &str = "cargo-fuzz-libfuzzer";
pub const EXPECTED_PROFILE_IDENTITY: &str = "b3:7ad698ddebaaac2640a51e5f58871ac026b4b32653fb95bf971ec491560b48d6";
pub const RECEIPT_SCHEMA_VERSION: &str = "octet-cargo-fuzz-receipt/v1";
pub const EVIDENCE_TIER: &str = "transcript_assertion";
pub const ENGINE: &str = "libfuzzer";
pub const TARGET: &str = "octet-profile-intake";
pub const TOOLCHAIN: &str = "nightly-2026-03-21";
pub const SANITIZER: &str = "address";
pub const MAX_TOTAL_TIME_SECONDS: u64 = 60;
pub const MAX_RUNS: u64 = 100_000;
pub const FIXED_SEED: u64 = 1;
pub const MAX_ARTIFACT_COUNT: usize = 128;
pub const MAX_ARTIFACT_BYTES: u64 = 16 * 1_024 * 1_024;
pub const MAX_PATH_BYTES: usize = 240;
pub const MAX_RECEIPT_BYTES: u64 = 1_048_576;
pub const CORPUS_DOMAIN: &[u8] = b"octet-cargo-fuzz-corpus-v1\0";
pub const REQUIRED_NON_CLAIM: &str = "Cargo-fuzz evidence covers one target, seed, corpus, sanitizer, and bounded run; it is not exhaustive coverage, memory-safety proof, panic-freedom proof, or release eligibility.";

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Budget {
    pub max_total_time_seconds: u64,
    pub max_runs: u64,
    pub seed: u64,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, serde::Serialize, serde::Deserialize)]
pub struct ArtifactIdentity {
    pub path: String,
    pub length: u64,
    pub blake3: String,
}

#[derive(Debug, Clone, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub target: String,
    pub engine: String,
    pub toolchain: String,
    pub compiler_identity: String,
    pub sanitizer: String,
    pub command_argv: Vec<String>,
    pub budget: Budget,
    pub initial_corpus: Vec<ArtifactIdentity>,
    pub initial_corpus_root: String,
    pub final_corpus: Vec<ArtifactIdentity>,
    pub final_corpus_root: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dictionary: Option<ArtifactIdentity>,
    pub outcome: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crash_input: Option<ArtifactIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimized_reproducer: Option<ArtifactIdentity>,
    pub replay_argv: Vec<String>,
    pub bounded_log_identity: String,
    pub evidence_tier: String,
    pub claimed_properties: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Evaluation {
    pub details: Option<ReceiptDetails>,
    pub diagnostics: Vec<String>,
    pub is_crash_observed: bool,
}

pub fn command_argv() -> Vec<String> {
    vec![
        "cargo".to_owned(),
        "fuzz".to_owned(),
        "run".to_owned(),
        TARGET.to_owned(),
        "--".to_owned(),
        format!("-max_total_time={MAX_TOTAL_TIME_SECONDS}"),
        format!("-runs={MAX_RUNS}"),
        format!("-seed={FIXED_SEED}"),
    ]
}

// r[impl octet.dynamic_evidence.cargo_fuzz_boundary]
pub fn evaluate_envelope(text: &str) -> Evaluation {
    let parsed = serde_json::from_str::<ReceiptDetails>(text);
    let Ok(details) = parsed else {
        return Evaluation {
            details: None,
            diagnostics: vec![format!(
                "cargo-fuzz receipt is not valid JSON: {}",
                parsed.expect_err("parse failed")
            )],
            is_crash_observed: false,
        };
    };
    let mut diagnostics = validate(&details);
    diagnostics.sort();
    diagnostics.dedup();
    let is_crash_observed = details.outcome == "crash";
    Evaluation {
        details: Some(details),
        diagnostics,
        is_crash_observed,
    }
}

pub(crate) mod shell;
mod validation;
pub use validation::corpus_root;
pub use validation::validate;

#[cfg(test)]
mod tests;

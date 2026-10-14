// r[impl octet.mirai_semantic_pilot.provider]
// r[impl octet.mirai_semantic_pilot.profile]
// r[impl octet.mirai_semantic_pilot.corpus]
// r[impl octet.mirai_semantic_pilot.evidence]
// r[impl octet.mirai_semantic_pilot.comparison]
// r[impl octet.mirai_semantic_pilot.decision]
// r[impl octet.mirai_semantic_pilot.boundary]
pub const POLICY_SCHEMA: &str = "octet-mirai-semantic-pilot-policy/v1";
pub const CORPUS_SCHEMA: &str = "octet-mirai-semantic-pilot-corpus/v1";
pub const DECISION_SCHEMA: &str = "octet-mirai-semantic-pilot-decision/v1";
pub const PROFILE_ID: &str = "octet-mirai-blocked-output-contract-v1";
pub const POLICY_IDENTITY: &str = "b3:cd4120f1df6d9715a615b87bbce4cbf41cdb3e0abd0a40dfd3ae175dc085b1ca";
pub const CORPUS_IDENTITY: &str = "b3:1dfb3e7ddeb1bc470d0e17e0a0b1c6b2895895643fa7c98a7f529a20ee99b015";
pub const EXPECTED_REPOSITORY: &str = "https://github.com/endorlabs/MIRAI";
pub const EXPECTED_REVISION: &str = "aae0a56f91d9a0a62073bb86e7b2c9793cefc24b";
pub const EXPECTED_SOURCE_HASH: &str = "sha256-+H0JIbrwxMi3uMuxBqgVBzNkDYom8X616rH9y7YqIXg=";
pub const EXPECTED_VERSION: &str = "1.1.12";
pub const EXPECTED_TOOLCHAIN: &str = "nightly-2025-01-10";
pub const EXPECTED_BLOCKER: &str = "stable-machine-output-unavailable";
pub const EXPECTED_CHARON_PARENT: &str = "b3:9c37280796d04408389e7efc4fd8413b6c3bb3594fc66bed79ea01492b9cdf67";
pub const EXPECTED_EFFECT_PARENT: &str = "b3:3be8c6c519ad668d3485c841c87f8ebb04ff3b94ee18490ecd3ec76c2e8ece99";
pub const EXPECTED_FEATURES: [&str; 1] = ["bundled"];
pub const EXPECTED_ENVIRONMENT: [&str; 1] = ["MIRAI_FLAGS=--diag=verify"];
pub const EXPECTED_COMMAND: [&str; 5] = [
    "cargo",
    "mirai",
    "--manifest-path",
    "cargo-octet/tests/fixtures/mirai-pilot/Cargo.toml",
    "--lib",
];

const POLICY_JSON: &str = include_str!("../../../standards/mirai/pilot.json");
const CORPUS_JSON: &str = include_str!("../../../standards/mirai/corpus.json");

pub use decision::blocked_receipt;
pub use decision::receipt_bytes;
pub use decision::validate_receipt;
pub use model::AnalyzerFact;
pub use model::ComparisonRow;
pub use model::CorpusFixture;
pub use model::DecisionReceipt;
pub use model::PilotProfile;

pub fn selected_policy() -> Result<PilotProfile, String> {
    if prefixed_identity(POLICY_JSON.as_bytes()) != POLICY_IDENTITY {
        return Err("embedded MIRAI pilot policy identity drifted".to_owned());
    }
    if prefixed_identity(CORPUS_JSON.as_bytes()) != CORPUS_IDENTITY {
        return Err("embedded MIRAI pilot corpus identity drifted".to_owned());
    }
    let registry: model::PolicyRegistry =
        serde_json::from_str(POLICY_JSON).map_err(|error| format!("parse embedded MIRAI pilot policy: {error}"))?;
    if registry.schema_version != POLICY_SCHEMA {
        return Err("embedded MIRAI pilot policy schema is unsupported".to_owned());
    }
    let corpus: model::CorpusRegistry =
        serde_json::from_str(CORPUS_JSON).map_err(|error| format!("parse embedded MIRAI pilot corpus: {error}"))?;
    if corpus.schema_version != CORPUS_SCHEMA || corpus.fixtures != registry.profile.corpus {
        return Err("embedded MIRAI pilot corpus schema or fixture projection drifted".to_owned());
    }
    let diagnostics = policy::validate(&registry.profile);
    if diagnostics.is_empty() {
        Ok(registry.profile)
    } else {
        Err(format!("invalid embedded MIRAI pilot policy: {}", diagnostics.join("; ")))
    }
}

pub fn parse_decision(text: &str) -> Result<DecisionReceipt, Vec<String>> {
    let receipt: DecisionReceipt =
        serde_json::from_str(text).map_err(|error| vec![format!("parse MIRAI pilot decision receipt: {error}")])?;
    let profile = selected_policy().map_err(|error| vec![error])?;
    let diagnostics = validate_receipt(&receipt, &profile);
    if diagnostics.is_empty() {
        Ok(receipt)
    } else {
        Err(diagnostics)
    }
}

pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

pub fn is_b3_identity(value: &str) -> bool {
    value.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == blake3::OUT_LEN * 2
            && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

pub(crate) fn pretty_bytes<T: serde::Serialize + ?Sized>(value: &T, label: &str) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| format!("serialize {label}: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub mod comparison;
mod decision;
mod model;
mod policy;
pub mod shell;

#[cfg(test)]
mod tests;

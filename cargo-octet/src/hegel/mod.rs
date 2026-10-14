// r[impl octet.dynamic_evidence.hegel_profiled_run]
// r[impl octet.dynamic_evidence.hegel_output_contract]
// r[impl octet.dynamic_evidence.hegel_measured_identity]
// r[impl octet.law_tested_deterministic_cores.hegel_catalog]
pub const PROFILE_SCHEMA: &str = "octet-hegel-producer-profiles/v1";
pub const PROFILE_ID: &str = "octet-pure-core-laws-v1";
pub const PROFILE_IDENTITY: &str = "b3:8739e90ca0707af37eae8553c3b792ec747b0668cad8c4e7e6b0b7c7a5e00e17";
pub const LAW_CATALOG_SCHEMA: &str = "octet-hegel-law-catalog/v1";
pub const LAW_CATALOG_IDENTITY: &str = "b3:33710ac60efb4a7edca25de746d9740d75233b372d0016f54a2e138ec1a038eb";
pub const SUMMARY_SCHEMA: &str = "octet-hegel-producer-summary/v2";
pub const REQUEST_SCHEMA: &str = "octet-hegel-evaluation-input/v1";
pub const RECEIPT_SCHEMA: &str = "octet-hegel-profiled-receipt/v1";
pub const SUMMARY_ARTIFACT: &str = "hegel-producer-summary.json";
pub const LOG_ARTIFACT: &str = "hegel-bounded-log.txt";
pub const SOURCE_MANIFEST_ARTIFACT: &str = "hegel-source-manifest.json";
pub const GENERATOR_MANIFEST_ARTIFACT: &str = "hegel-generator-manifest.json";
pub const COUNTEREXAMPLE_ARTIFACT: &str = "hegel-counterexample.json";
pub const PROJECTION_ARTIFACT: &str = "hegel-projection.json";
pub const REQUIRED_NON_CLAIM: &str = "Hegel checks bounded generated inputs only and does not prove correctness, semantic equivalence, verifier soundness, requirement satisfaction, release eligibility, or whole-program safety";

const PROFILE_JSON: &str = include_str!("../../../standards/hegel/profiles.json");
const LAW_CATALOG_JSON: &str = include_str!("../../../standards/hegel/laws.json");

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Producer {
    pub package: String,
    pub package_version: String,
    pub binary_target: String,
    pub hegel_version: String,
    pub backend: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Settings {
    pub case_budget: u64,
    pub phases: Vec<String>,
    pub seed_posture: String,
    pub derandomized: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Limits {
    pub max_summary_bytes: u64,
    pub max_counterexample_bytes: u64,
    pub max_log_bytes: usize,
    pub max_source_files: usize,
    pub max_generator_files: usize,
    pub max_issue_codes: usize,
    pub timeout_ms: u64,
    pub poll_interval_ms: u64,
    pub teardown_timeout_ms: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Retention {
    pub retain_summary: bool,
    pub retain_logs: bool,
    pub retain_counterexample: bool,
    pub redaction: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Profile {
    pub profile_id: String,
    pub producer: Producer,
    pub command_argv: Vec<String>,
    pub property_catalog: String,
    pub output_root: String,
    pub summary_path: String,
    pub counterexample_path: String,
    pub source_scope: Vec<String>,
    pub generator_inputs: Vec<String>,
    pub settings: Settings,
    pub limits: Limits,
    pub retention: Retention,
    pub evidence_tier: String,
    pub verification_role: String,
    pub non_claims: Vec<String>,
}

#[derive(serde::Deserialize)]
struct ProfileRegistry {
    schema_version: String,
    profiles: Vec<Profile>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Law {
    pub law_id: String,
    pub pure_core: String,
    pub generator: String,
    pub positive_property: String,
    pub negative_property: String,
    pub generated_input_benefit: String,
    pub case_budget: u64,
    pub replay_method: String,
    pub fixture_complement: String,
    pub effect_free: bool,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct LawCatalog {
    pub schema_version: String,
    pub laws: Vec<Law>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct GeneratorSummary {
    pub family: String,
    pub content_hash: String,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ProducerSummary {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub package: String,
    pub package_version: String,
    pub hegel_version: String,
    pub backend: String,
    pub property_catalog: String,
    pub property_id: String,
    pub linked_ids: Vec<String>,
    pub source_scope: String,
    pub settings: Settings,
    pub generator: GeneratorSummary,
    pub source_manifest_identity: String,
    pub generator_manifest_identity: String,
    pub cargo_lock_identity: String,
    pub package_manifest_identity: String,
    pub result: String,
    pub cases_completed: u64,
    pub issue_codes: Vec<String>,
    pub counterexample_identity: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct FileIdentity {
    pub locator: String,
    pub identity: String,
    pub bytes: u64,
}

#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ReceiptDetails {
    pub schema_version: String,
    pub profile_id: String,
    pub profile_identity: String,
    pub law_catalog_identity: String,
    pub property_id: String,
    pub linked_ids: Vec<String>,
    pub source_scope: String,
    pub producer: Producer,
    pub settings: Settings,
    pub generator: GeneratorSummary,
    pub command_argv: Vec<String>,
    pub cargo_lock_identity: String,
    pub package_manifest_identity: String,
    pub source_manifest_identity: String,
    pub generator_manifest_identity: String,
    pub summary_identity: String,
    pub bounded_log_identity: String,
    pub counterexample_identity: Option<String>,
    pub result: String,
    pub cases_completed: u64,
    pub issue_codes: Vec<String>,
    pub projection_identity: String,
    pub evidence_tier: String,
    pub verification_role: String,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Evaluation {
    pub details: Option<ReceiptDetails>,
    pub diagnostics: Vec<String>,
}

pub fn selected_profile(profile_id: &str) -> Result<Profile, String> {
    let registry: ProfileRegistry = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse embedded Hegel profile registry: {error}"))?;
    if registry.schema_version != PROFILE_SCHEMA {
        return Err("embedded Hegel profile registry schema is unsupported".to_owned());
    }
    registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown Hegel profile `{profile_id}`"))
}

pub fn law_catalog() -> Result<LawCatalog, String> {
    let catalog: LawCatalog =
        serde_json::from_str(LAW_CATALOG_JSON).map_err(|error| format!("parse embedded Hegel law catalog: {error}"))?;
    if catalog.schema_version != LAW_CATALOG_SCHEMA {
        return Err("embedded Hegel law catalog schema is unsupported".to_owned());
    }
    Ok(catalog)
}

pub fn normalize_issue_codes(values: &[String]) -> Vec<String> {
    let mut normalized = values
        .iter()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    normalized.sort();
    normalized.dedup();
    normalized
}

pub fn projection_bytes(details: &ReceiptDetails) -> Result<Vec<u8>, String> {
    let mut projection = details.clone();
    projection.projection_identity.clear();
    pretty_bytes(&projection, "Hegel hardened projection")
}

pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

pub(crate) fn pretty_bytes<T: serde::Serialize + ?Sized>(value: &T, label: &str) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| format!("serialize {label}: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

pub fn validate(details: &ReceiptDetails) -> Vec<String> {
    validation::validate_details(details)
}

mod profile_validation;
pub(crate) mod request;
pub(crate) mod safe_read;
pub mod shell;
mod validation;

#[cfg(test)]
mod tests;

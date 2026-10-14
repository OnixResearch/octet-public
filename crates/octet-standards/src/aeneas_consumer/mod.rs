//! Pure downstream-consumer contract for the pinned Aeneas provider.
// r[impl octet.aeneas_consumer_contract.profile]
// r[impl octet.aeneas_consumer_contract.source_selection]
// r[impl octet.aeneas_consumer_contract.execution]
// r[impl octet.aeneas_consumer_contract.outcomes]
// r[impl octet.aeneas_consumer_contract.theorem]
// r[impl octet.aeneas_consumer_contract.artifacts]
// r[impl octet.aeneas_consumer_contract.compatibility]
/// Consumer-profile schema version.
pub const PROFILE_SCHEMA: &str = "octet-aeneas-consumer-profile/v1";
/// Consumer receipt-binding schema version.
pub const RECEIPT_SCHEMA: &str = "octet-aeneas-consumer-receipt-binding/v1";
/// Selected external-consumer fixture profile.
pub const EXPECTED_PROFILE_ID: &str = "octet-external-aeneas-consumer-v1";
/// BLAKE3 identity of the generated consumer profile.
pub const PROFILE_IDENTITY: &str = "b3:62bb3c01a2037eef1410e35d3902d73fb5e7439e8856d721d0823e14c0af4879";
/// Reviewed provider backend.
pub const EXPECTED_BACKEND: &str = "aeneas-lean";
/// Reviewed provider profile identifier.
pub const EXPECTED_PROVIDER_ID: &str = "octet-aeneas-lean-0.1.0-bc2eb6f";
/// BLAKE3 identity of the reviewed provider profile.
pub const EXPECTED_PROVIDER_IDENTITY: &str = "b3:cd11b49d1acede2c6a91fa26f0b7a6b4b28db4007b4601a65b3269706a0ce625";
/// Reviewed compatibility-matrix identifier.
pub const EXPECTED_COMPATIBILITY_ID: &str = "octet-aeneas-lean-compatibility-0.1.0-bc2eb6f-cb50ff16";
/// BLAKE3 identity of the reviewed compatibility matrix.
pub const EXPECTED_COMPATIBILITY_IDENTITY: &str = "b3:403656d75de41b955cd2493ef289ba17308da65a3d214babc1ea606aeae248f5";
/// Claim class available to a supported consumer theorem.
pub const EXPECTED_CLAIM_CLASS: &str = "named-theorem-over-generated-model";
/// Evidence role available after full consumer validation.
pub const EXPECTED_VERIFICATION_ROLE: &str = "property";
/// Environment allowlist inherited from the provider profile.
pub const EXPECTED_ENVIRONMENT: [&str; 5] = [
    "PATH",
    "CHARON_TOOLCHAIN_IS_IN_PATH",
    "LD_LIBRARY_PATH",
    "HOME",
    "MATHLIB_NO_CACHE_ON_UPDATE",
];
/// Claim boundaries required on every consumer profile and receipt.
pub const REQUIRED_NON_CLAIMS: [&str; 5] = [
    "does not prove Aeneas or Charon translation soundness",
    "does not prove Lean or verifier soundness",
    "does not prove whole-program correctness",
    "does not prove semantic equivalence",
    "does not establish release eligibility",
];

const PROFILE_JSON: &str =
    include_str!("../../../../standards/proof-rails/aeneas-lean/generated/consumer-profile.json");
const BLAKE3_HEX_LENGTH: usize = 64;

pub use model::ConsumerArtifactIdentity;
pub use model::ConsumerOutcome;
pub use model::ConsumerProfile;
pub use model::ConsumerReceiptBinding;
pub use model::OutcomeDecision;
pub use model::StageObservation;
pub use model::StageStatus;
pub use observation::ObservationInput;
pub use observation::build_binding;
pub use outcome::classify;

/// Load and validate the checked consumer profile.
///
/// # Errors
///
/// Returns an error when the embedded profile is malformed or violates its closed contract.
pub fn selected_profile() -> Result<ConsumerProfile, String> {
    let (profile, identity) = load_profile(PROFILE_JSON.as_bytes())?;
    if identity == PROFILE_IDENTITY {
        Ok(profile)
    } else {
        Err("embedded Aeneas consumer profile identity drifted".to_owned())
    }
}

/// Parse, measure, and validate an exact external consumer profile.
///
/// # Errors
///
/// Returns an error when the supplied bytes are malformed or violate the closed contract.
pub fn load_profile(bytes: &[u8]) -> Result<(ConsumerProfile, String), String> {
    let profile: ConsumerProfile =
        serde_json::from_slice(bytes).map_err(|error| format!("parse Aeneas consumer profile: {error}"))?;
    let diagnostics = validation::profile(&profile);
    if diagnostics.is_empty() {
        Ok((profile, prefixed_identity(bytes)))
    } else {
        Err(format!("invalid Aeneas consumer profile: {}", diagnostics.join("; ")))
    }
}

/// Validate a loaded consumer profile without external effects.
#[must_use]
pub fn validate_profile(profile: &ConsumerProfile) -> Vec<String> {
    validation::profile(profile)
}

/// Validate a consumer receipt against the checked profile.
#[must_use]
pub fn validate_receipt(receipt: &ConsumerReceiptBinding) -> Vec<String> {
    match selected_profile() {
        Ok(profile) => validation::receipt(receipt, &profile, PROFILE_IDENTITY),
        Err(error) => vec![error],
    }
}

/// Validate a consumer receipt against an explicit measured profile.
#[must_use]
pub fn validate_receipt_against(
    receipt: &ConsumerReceiptBinding,
    profile: &ConsumerProfile,
    profile_identity: &str,
) -> Vec<String> {
    let mut diagnostics = validate_profile(profile);
    if is_b3(profile_identity) {
        diagnostics.extend(validation::receipt(receipt, profile, profile_identity));
    } else {
        diagnostics.push("Aeneas consumer profile identity is malformed".to_owned());
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// Compute the BLAKE3 identity of a receipt with its identity field cleared.
///
/// # Errors
///
/// Returns an error when the typed receipt cannot be serialized.
pub fn receipt_identity(receipt: &ConsumerReceiptBinding) -> Result<String, String> {
    let mut projection = receipt.clone();
    projection.receipt_identity.clear();
    pretty_bytes(&projection, "Aeneas consumer receipt").map(|bytes| prefixed_identity(&bytes))
}

/// Return a canonical `b3:` identity for exact bytes.
#[must_use]
pub fn prefixed_identity(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

/// Return whether a string is one canonical lowercase `b3:` identity.
#[must_use]
pub fn is_b3(value: &str) -> bool {
    value.strip_prefix("b3:").is_some_and(|hex| {
        hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    })
}

fn pretty_bytes<T: serde::Serialize + ?Sized>(value: &T, label: &str) -> Result<Vec<u8>, String> {
    let mut bytes = serde_json::to_vec_pretty(value).map_err(|error| format!("serialize {label}: {error}"))?;
    bytes.push(b'\n');
    Ok(bytes)
}

mod model;
mod observation;
mod outcome;
mod validation;

#[cfg(test)]
mod envelope_tests;
#[cfg(test)]
mod tests;

//! Recorded-only validation for the measured Vermilion live canary receipt.
//!
//! This module validates the exact canary evidence produced by the pinned
//! Vermilion u64 pilot. It does not execute Vermilion, promote the canary to a
//! formal proof, or claim the typed receipt obligations. It exists so
//! `cargo octet prove` can bind the measured canary as recorded-only evidence
//! with a manual-review proof card.
// r[impl octet.vermilion_canary_rail.recorded_only]

use serde::Deserialize;
use serde::Serialize;

use crate::vermilion::prefixed_identity;
use crate::vermilion::selected_profile;

/// Canary schema version.
pub const CANARY_SCHEMA: &str = "octet-vermilion-live-canary-receipt/v1";
/// Canary provider identity.
pub const CANARY_PROVIDER: &str = "vermilion-lean";
/// Canary claim scope.
pub const CANARY_CLAIM_SCOPE: &str = "per-obligation-lean-kernel-theorem-over-pinned-verus-frontend";
/// Exact measured source path.
pub const CANARY_SOURCE_PATH: &str = "standards/proof-rails/vermilion-lean/evidence/2026-11-09-u64-pilot/checked.rs";
/// Exact measured source bytes.
pub const CANARY_SOURCE: &str =
    include_str!("../../../standards/proof-rails/vermilion-lean/evidence/2026-11-09-u64-pilot/checked.rs");
/// Exact required obligation identifiers.
pub const CANARY_REQUIRED_OBLIGATIONS: [&str; 2] = ["pilot.increment.overflow-check", "pilot.increment.value"];
/// Required canary non-claim fragments.
pub const CANARY_REQUIRED_NON_CLAIMS: [&str; 4] = [
    "the Verus-syntax source is a parallel specification, not the plain Rust executable",
    "does not prove the generated obligations refine the executable Rust function",
    "does not prove provider soundness, whole-program correctness, semantic equivalence, or release eligibility",
    "does not add cargo octet prove, proof-card, artifact-verification, or Valence integration",
];

/// Measured Vermilion live canary receipt.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct VermilionCanaryReceipt {
    /// Canary schema.
    pub schema_version: String,
    /// Provider identity.
    pub provider_id: String,
    /// Provider claim scope.
    pub claim_scope: String,
    /// Vermilion revision.
    pub vermilion_revision: String,
    /// Verus fork revision.
    pub verus_fork_revision: String,
    /// Lean toolchain.
    pub lean_toolchain: String,
    /// Mathlib revision.
    pub mathlib_revision: String,
    /// cvc5 revision.
    pub cvc5_revision: String,
    /// Measured source path.
    pub source_path: String,
    /// Measured source BLAKE3.
    pub source_blake3: String,
    /// Measured statement.
    pub statement: String,
    /// Required obligation identifiers.
    pub required_obligations: Vec<String>,
    /// Verified obligation count.
    pub obligations_verified: usize,
    /// Failed obligation count.
    pub obligations_failed: usize,
    /// Sorry count.
    pub sorry_count: usize,
    /// Proof twin classification.
    pub proof_twin: String,
    /// Measured outcome.
    pub outcome: String,
    /// Measured non-claims.
    pub non_claims: Vec<String>,
}

/// Validate the measured canary against the reviewed provider profile.
#[must_use]
pub fn validate(receipt: &VermilionCanaryReceipt) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if receipt.schema_version != CANARY_SCHEMA
        || receipt.provider_id != CANARY_PROVIDER
        || receipt.claim_scope != CANARY_CLAIM_SCOPE
    {
        diagnostics.push("vermilion canary schema, provider, or claim scope drifted".to_owned());
    }
    match selected_profile() {
        Ok(profile) => {
            let cvc5 = profile
                .lean
                .packages
                .iter()
                .find(|package| package.name == "cvc5")
                .map_or("", |package| package.revision.as_str());
            if receipt.vermilion_revision != profile.vermilion_revision
                || receipt.verus_fork_revision != profile.verus_fork.revision
                || receipt.lean_toolchain != profile.lean.toolchain
                || receipt.mathlib_revision != profile.lean.mathlib_revision
                || receipt.cvc5_revision != cvc5
            {
                diagnostics.push("vermilion canary toolchain pins drifted from the reviewed profile".to_owned());
            }
        }
        Err(error) => diagnostics.push(format!("load reviewed Vermilion profile: {error}")),
    }
    let observed_source = prefixed_identity(CANARY_SOURCE.as_bytes());
    let observed_hex = observed_source.strip_prefix("b3:").unwrap_or(&observed_source);
    if receipt.source_path != CANARY_SOURCE_PATH || receipt.source_blake3 != observed_hex {
        diagnostics.push("vermilion canary source path or BLAKE3 drifted".to_owned());
    }
    if receipt.required_obligations != CANARY_REQUIRED_OBLIGATIONS.iter().map(|id| (*id).to_owned()).collect::<Vec<_>>()
    {
        diagnostics.push("vermilion canary required obligations drifted".to_owned());
    }
    if receipt.obligations_verified != 2
        || receipt.obligations_failed != 0
        || receipt.sorry_count != 0
        || receipt.proof_twin != "kernel-checked"
        || receipt.outcome != "verified"
    {
        diagnostics.push("vermilion canary outcome or obligation count is not verified".to_owned());
    }
    if receipt.statement.trim().is_empty() {
        diagnostics.push("vermilion canary statement is empty".to_owned());
    }
    for required in CANARY_REQUIRED_NON_CLAIMS {
        if !receipt.non_claims.iter().any(|claim| claim == required) {
            diagnostics.push(format!("vermilion canary missing non-claim `{required}`"));
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

/// BLAKE3 identity of the exact measured receipt bytes.
#[must_use]
pub fn receipt_identity(bytes: &[u8]) -> String {
    prefixed_identity(bytes)
}

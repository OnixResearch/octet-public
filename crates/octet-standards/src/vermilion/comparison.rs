use serde::Deserialize;
use serde::Serialize;

use super::COMPARISON_SCHEMA;

/// Closed cross-provider comparison category.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparisonCategory {
    /// Every route checked the same declared property and bridge.
    ComparableAgreement,
    /// Two routes claim the same property identity but disagree on semantics.
    SemanticMismatch,
    /// Routes claim different properties, widths, or representations.
    IncomparableClaims,
    /// At least one executed provider rejected its proof obligation.
    ProofFailure,
    /// A provider was unsupported or blocked by an unavailable toolchain.
    UnsupportedOrBlocked,
    /// A provider exhausted its time or resource budget.
    TimeoutOrExhaustion,
    /// A provider or surrounding tool failed to execute.
    ToolError,
}

impl ComparisonCategory {
    /// Return the stable serialized spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::ComparableAgreement => "comparable_agreement",
            Self::SemanticMismatch => "semantic_mismatch",
            Self::IncomparableClaims => "incomparable_claims",
            Self::ProofFailure => "proof_failure",
            Self::UnsupportedOrBlocked => "unsupported_or_blocked",
            Self::TimeoutOrExhaustion => "timeout_or_exhaustion",
            Self::ToolError => "tool_error",
        }
    }

    /// Map this comparison category onto the existing Aeneas consumer outcome vocabulary.
    ///
    /// Returns `None` for categories that make no Aeneas-style proof claim.
    #[must_use]
    pub const fn aeneas_outcome(self) -> Option<&'static str> {
        match self {
            Self::ProofFailure | Self::SemanticMismatch | Self::TimeoutOrExhaustion | Self::ToolError => Some("failed"),
            Self::UnsupportedOrBlocked => Some("blocked"),
            Self::ComparableAgreement | Self::IncomparableClaims => None,
        }
    }

    /// Map this comparison category onto the Vermilion outcome vocabulary.
    #[must_use]
    pub const fn vermilion_outcome(self) -> Option<&'static str> {
        match self {
            Self::ComparableAgreement | Self::IncomparableClaims => None,
            Self::UnsupportedOrBlocked => Some("blocked"),
            Self::ProofFailure | Self::SemanticMismatch | Self::TimeoutOrExhaustion | Self::ToolError => Some("failed"),
        }
    }
}

/// One route's declared semantic claim.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RouteClaim {
    /// Stable property identity.
    pub property_id: String,
    /// BLAKE3 identity of the route statement.
    pub statement_blake3: String,
    /// Declared arithmetic width.
    pub width_bits: u16,
    /// Declared overflow behavior.
    pub overflow_behavior: String,
    /// Declared precondition.
    pub precondition: String,
    /// Declared result type.
    pub result_type: String,
}

/// One route's observed outcome.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RouteOutcome {
    /// The route checked its declared claim.
    Supported,
    /// A reviewed language or toolchain boundary blocked the route.
    Blocked,
    /// The route executed and rejected a proof obligation.
    Failed,
    /// The route exhausted its budget.
    TimedOut,
    /// The route did not execute because a tool failed.
    ToolError,
}

/// One backend route and its refinement-bridge status.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RouteEvidence {
    /// Backend identifier.
    pub backend: String,
    /// Scoped checked statement.
    pub scope: String,
    /// Declared semantic claim.
    pub claim: RouteClaim,
    /// Observed outcome.
    pub outcome: RouteOutcome,
    /// Refinement-bridge identifier.
    pub refinement_bridge_id: String,
    /// Whether the refinement bridge has its own checked result.
    pub refinement_bridge_checked: bool,
}

/// One comparison row.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonRow {
    /// Backend identifier.
    pub backend: String,
    /// Scoped checked statement.
    pub scope: String,
    /// Observed outcome.
    pub outcome: RouteOutcome,
    /// Claim identity.
    pub property_id: String,
    /// Whether this route can participate in agreement.
    pub comparable: bool,
    /// Reason a route is not comparable.
    pub non_comparable_reason: Option<String>,
}

/// Deterministic cross-provider report.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonReport {
    /// Report schema version.
    pub schema_version: String,
    /// Closed comparison category.
    pub category: ComparisonCategory,
    /// One row per route.
    pub rows: Vec<ComparisonRow>,
    /// Property identity shared by all comparable routes, when any.
    pub agreed_property_id: Option<String>,
    /// Whether provider evidence authority was merged.
    pub evidence_merge: bool,
    /// Refinement bridges that remain open obligations.
    pub open_obligations: Vec<String>,
    /// Explicit claim boundaries.
    pub non_claims: Vec<String>,
    /// Aeneas outcome spelling this category maps to, when any.
    pub aeneas_outcome: Option<String>,
    /// Vermilion outcome spelling this category maps to, when any.
    pub vermilion_outcome: Option<String>,
}

/// Compare route evidence without executing or merging provider authority.
#[must_use]
pub fn compare(routes: &[RouteEvidence]) -> ComparisonReport {
    let category = classify(routes);
    let rows = routes.iter().map(|route| row(route, routes)).collect::<Vec<_>>();
    let agreed_property_id = (category == ComparisonCategory::ComparableAgreement)
        .then(|| routes.first().map(|route| route.claim.property_id.clone()))
        .flatten();
    let mut open_obligations = routes
        .iter()
        .filter(|route| !route.refinement_bridge_checked)
        .map(|route| route.refinement_bridge_id.clone())
        .collect::<Vec<_>>();
    open_obligations.sort();
    open_obligations.dedup();
    ComparisonReport {
        schema_version: COMPARISON_SCHEMA.to_owned(),
        category,
        rows,
        agreed_property_id,
        evidence_merge: false,
        open_obligations,
        non_claims: vec![
            "agreement does not merge provider evidence authority".to_owned(),
            "agreement does not prove either provider's soundness".to_owned(),
            "hashes bind identity, not semantic correctness".to_owned(),
            "no majority-vote success or release eligibility is implied".to_owned(),
        ],
        aeneas_outcome: category.aeneas_outcome().map(str::to_owned),
        vermilion_outcome: category.vermilion_outcome().map(str::to_owned),
    }
}

fn classify(routes: &[RouteEvidence]) -> ComparisonCategory {
    if routes.len() < 2 {
        return ComparisonCategory::IncomparableClaims;
    }
    if routes.iter().any(|route| route.outcome == RouteOutcome::ToolError) {
        return ComparisonCategory::ToolError;
    }
    if routes.iter().any(|route| route.outcome == RouteOutcome::TimedOut) {
        return ComparisonCategory::TimeoutOrExhaustion;
    }
    let mut property_ids = routes.iter().map(|route| route.claim.property_id.as_str());
    let first = property_ids.next().unwrap_or_default();
    if property_ids.any(|property| property != first) {
        return ComparisonCategory::IncomparableClaims;
    }
    let first_claim = &routes[0].claim;
    if routes.iter().any(|route| {
        route.claim.statement_blake3 != first_claim.statement_blake3
            || route.claim.width_bits != first_claim.width_bits
            || route.claim.overflow_behavior != first_claim.overflow_behavior
            || route.claim.precondition != first_claim.precondition
            || route.claim.result_type != first_claim.result_type
    }) {
        return ComparisonCategory::SemanticMismatch;
    }
    if routes.iter().any(|route| route.outcome == RouteOutcome::Failed) {
        return ComparisonCategory::ProofFailure;
    }
    if routes.iter().any(|route| !route.refinement_bridge_checked) {
        return ComparisonCategory::IncomparableClaims;
    }
    if routes.iter().any(|route| route.outcome == RouteOutcome::Blocked) {
        return ComparisonCategory::UnsupportedOrBlocked;
    }
    if routes.iter().all(|route| route.outcome == RouteOutcome::Supported) {
        return ComparisonCategory::ComparableAgreement;
    }
    ComparisonCategory::IncomparableClaims
}

fn row(route: &RouteEvidence, routes: &[RouteEvidence]) -> ComparisonRow {
    let first = routes.first();
    let reason = if first.is_some_and(|first| first.claim.property_id != route.claim.property_id) {
        Some("other routes claim another property".to_owned())
    } else if first.is_some_and(|first| {
        route.claim.statement_blake3 != first.claim.statement_blake3
            || route.claim.width_bits != first.claim.width_bits
            || route.claim.overflow_behavior != first.claim.overflow_behavior
            || route.claim.precondition != first.claim.precondition
            || route.claim.result_type != first.claim.result_type
    }) {
        Some("declared semantic fields differ from the first route".to_owned())
    } else if !route.refinement_bridge_checked {
        Some("refinement bridge is an open obligation".to_owned())
    } else {
        None
    };
    ComparisonRow {
        backend: route.backend.clone(),
        scope: route.scope.clone(),
        outcome: route.outcome,
        property_id: route.claim.property_id.clone(),
        comparable: reason.is_none(),
        non_comparable_reason: reason,
    }
}

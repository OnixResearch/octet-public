pub const NOMINAL_DOMAIN_SCHEMA_VERSION: &str = "octet-nominal-domain-policy/v1";
pub const NOMINAL_DOMAIN_INVENTORY_SCHEMA_VERSION: &str = "octet-nominal-domain-inventory/v1";

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum EnforcementMode {
    Inventory,
    Warn,
    Deny,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RawRepresentation {
    String,
    U64,
    Usize,
    Path,
}

impl RawRepresentation {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "string",
            Self::U64 => "u64",
            Self::Usize => "usize",
            Self::Path => "path",
        }
    }
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum WrapperKind {
    OpaqueNewtype,
    RefinedNewtype,
    MarkerGeneric,
    UnitNewtype,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConstructorPolicy {
    PrivateInfallible,
    PrivateFallible,
    MarkerOnly,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct NominalDomainPolicy {
    pub domain_id: String,
    pub symbols: Vec<String>,
    pub raw_representation: RawRepresentation,
    pub core_scopes: Vec<String>,
    pub compatibility_scopes: Vec<String>,
    pub compatibility_required: bool,
    pub wrapper_kind: WrapperKind,
    pub constructor_policy: ConstructorPolicy,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct NominalDomainCatalog {
    pub schema_version: String,
    pub default_mode: EnforcementMode,
    pub heuristic_suffixes: Vec<String>,
    pub domains: Vec<NominalDomainPolicy>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DescriptorKind {
    TypeAlias,
    Field,
    Parameter,
    Result,
    EnumPayload,
    MapKey,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DomainDescriptor<'a> {
    pub symbol: &'a str,
    pub raw_representation: RawRepresentation,
    pub source_path: &'a str,
    pub kind: DescriptorKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DomainDisposition {
    ExactCore { domain_id: String },
    Compatibility { domain_id: String },
    ExactOutsideScope { domain_id: String },
    HeuristicCandidate { suffix: String },
    Unclassified,
}

/// Independent observed bypass facts in the public v1 compatibility API.
#[allow(clippy::struct_excessive_bools)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WrapperFacts {
    pub inner_field_public: bool,
    pub unchecked_from_raw: bool,
    pub has_fallible_constructor: bool,
    pub boundary_erasing_deref: bool,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct InventorySource {
    pub repository: String,
    pub revision: String,
    pub path: String,
    pub blake3: String,
}

#[derive(Clone, Copy, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum InventoryClassification {
    ExactCore,
    HeuristicCandidate,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct InventoryFinding {
    pub lint_name: String,
    pub classification: InventoryClassification,
    pub domain_id: Option<String>,
    pub symbol: String,
    pub source_path: String,
    pub line: usize,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct InventorySummary {
    pub exact_findings: usize,
    pub heuristic_candidates: usize,
    pub compatibility_skips: usize,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct InventoryReview {
    pub reviewed: bool,
    pub true_positives: usize,
    pub false_positives: usize,
    pub coverage_gaps: Vec<String>,
}

#[derive(Clone, Debug, serde::Deserialize, Eq, PartialEq, serde::Serialize)]
pub struct InventoryReceipt {
    pub schema_version: String,
    pub pilot_id: String,
    pub mode: EnforcementMode,
    pub source: InventorySource,
    pub policy_blake3: String,
    pub findings: Vec<InventoryFinding>,
    pub summary: InventorySummary,
    pub review: InventoryReview,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WrapperViolation {
    PublicInnerField,
    UncheckedPrimitiveConversion,
    MissingFallibleConstructor,
    BoundaryErasingDeref,
}

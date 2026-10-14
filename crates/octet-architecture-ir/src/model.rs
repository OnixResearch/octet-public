use serde::Deserialize;
use serde::Serialize;

pub const COMPILER_ARCHITECTURE_SHARD_SCHEMA_VERSION: &str = "octet-compiler-architecture-shard/v1";
pub const COMPILER_ARCHITECTURE_IR_SCHEMA_VERSION: &str = "octet-compiler-architecture-ir/v1";
pub const COMPILER_ARCHITECTURE_COVERAGE_SCHEMA_VERSION: &str = "octet-compiler-architecture-coverage/v1";
pub const COMPILER_ARCHITECTURE_ISSUE_SCHEMA_VERSION: &str = "octet-compiler-architecture-issue/v1";
pub const COMPILER_ARCHITECTURE_RECEIPT_SCHEMA_VERSION: &str = "octet-compiler-architecture-receipt/v1";
pub const COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_SCHEMA_VERSION: &str =
    "octet-compiler-architecture-collector-context/v1";
pub const COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV: &str = "OCTET_COMPILER_ARCHITECTURE_CONTEXT";
pub const COMPILER_ARCHITECTURE_CONTRACT_PATH: &str =
    "standards/compiler-architecture-ir/contracts/compiler-architecture-ir.generated.ncl";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ArchitectureCollectionLimits {
    pub max_units: usize,
    pub max_shards: usize,
    pub max_modules: usize,
    pub max_items: usize,
    pub max_types: usize,
    pub max_traits: usize,
    pub max_implementations: usize,
    pub max_signatures: usize,
    pub max_calls: usize,
    pub max_constructors: usize,
    pub max_effects: usize,
    pub max_expansions: usize,
    pub max_configurations: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_type_shapes: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_function_safety_facts: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_provenance_facts: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_flow_bodies: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_flow_steps: usize,
    #[serde(default = "default_authority_fact_limit")]
    pub max_value_flow_rounds: usize,
    pub max_unknowns: usize,
    pub max_spans: usize,
    pub max_span_bytes: usize,
    pub max_shard_bytes: usize,
    pub max_artifact_bytes: usize,
}

const fn default_authority_fact_limit() -> usize {
    1
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CompilerArchitectureRole {
    DomainCore,
    ApplicationShell,
    InboundAdapter,
    OutboundAdapter,
    CompositionRoot,
    Test,
    Generated,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilationUnit {
    pub unit_id: String,
    pub package_name: String,
    pub package_version: String,
    pub target_name: String,
    pub target_kind: String,
    pub crate_types: Vec<String>,
    pub target_triple: String,
    pub features: Vec<String>,
    pub default_features: bool,
    pub role: CompilerArchitectureRole,
    pub toolchain_blake3: String,
    pub collector_blake3: String,
    pub policy_blake3: String,
    pub source_blake3: String,
    pub invocation_blake3: String,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum SourceOrigin {
    Workspace { source_path: String },
    ExternalGenerated { provenance_blake3: String },
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpan {
    pub span_id: String,
    pub origin: SourceOrigin,
    pub byte_start: u64,
    pub byte_end: u64,
    pub line_start: u32,
    pub column_start: u32,
    pub from_expansion: bool,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ItemKind {
    Module,
    Function,
    Struct,
    Enum,
    Union,
    Trait,
    TypeAlias,
    Implementation,
    Constant,
    Static,
    Foreign,
    Other,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum TypeKind {
    Primitive,
    Adt,
    Tuple,
    Reference,
    Pointer,
    Function,
    TraitObject,
    Opaque,
    Generic,
    Other,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum DispatchKind {
    Static,
    Trait,
    Dynamic,
    Foreign,
    Intrinsic,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CapabilityFamily {
    Time,
    Randomness,
    Filesystem,
    Network,
    Process,
    Environment,
    Secrets,
    UnsafeCode,
    GlobalState,
    Concurrency,
    Observability,
    Ffi,
    Allocation,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValueProvenanceStatus {
    Exact,
    Merged,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerValueFlowSelection {
    pub tracked_callee_definitions: Vec<String>,
    pub tracked_type_definitions: Vec<String>,
    #[serde(default)]
    pub root_origin_definitions: Vec<String>,
    #[serde(default)]
    pub propagation_definitions: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum CompilerArchitectureFact {
    Module {
        fact_id: String,
        module_path: String,
        span_id: String,
    },
    Item {
        fact_id: String,
        item_id: String,
        qualified_path: String,
        item_kind: ItemKind,
        origin_crate: String,
        visibility: String,
        signature_id: Option<String>,
        span_id: String,
    },
    Type {
        fact_id: String,
        type_id: String,
        display: String,
        definition_path: Option<String>,
        origin_crate: String,
        type_kind: TypeKind,
    },
    Trait {
        fact_id: String,
        trait_id: String,
        qualified_path: String,
        origin_crate: String,
        span_id: String,
    },
    Implementation {
        fact_id: String,
        implementation_id: String,
        trait_id: Option<String>,
        trait_definition: Option<String>,
        self_type_id: String,
        span_id: String,
    },
    Signature {
        fact_id: String,
        signature_id: String,
        owner_item_id: String,
        input_type_ids: Vec<String>,
        output_type_id: String,
        is_public: bool,
    },
    Call {
        fact_id: String,
        caller_item_id: String,
        callee_item_id: Option<String>,
        resolved_definition: Option<String>,
        dispatch: DispatchKind,
        span_id: String,
    },
    Constructor {
        fact_id: String,
        caller_item_id: String,
        constructed_type_id: String,
        constructor_definition: Option<String>,
        span_id: String,
    },
    Effect {
        fact_id: String,
        caller_item_id: String,
        capability_family: CapabilityFamily,
        primitive_definition: String,
        span_id: String,
    },
    Expansion {
        fact_id: String,
        generated_span_id: String,
        callsite_span_id: String,
        macro_definition: Option<String>,
    },
    Configuration {
        fact_id: String,
        predicate: String,
        active: bool,
        span_id: Option<String>,
    },
    TypeShape {
        fact_id: String,
        type_id: String,
        definition_path: String,
        field_visibilities: Vec<String>,
        #[serde(default)]
        field_type_definitions: Vec<String>,
        implemented_traits: Vec<String>,
        span_id: String,
    },
    FunctionSafety {
        fact_id: String,
        item_id: String,
        qualified_path: String,
        output_type_id: String,
        is_unsafe: bool,
        span_id: String,
    },
    CallArgumentProvenance {
        fact_id: String,
        caller_item_id: String,
        callee_definition: String,
        argument_position: usize,
        argument_type_id: String,
        origin_definitions: Vec<String>,
        status: ValueProvenanceStatus,
        span_id: String,
    },
}

impl CompilerArchitectureFact {
    #[must_use]
    pub fn fact_id(&self) -> &str {
        match self {
            Self::Module { fact_id, .. }
            | Self::Item { fact_id, .. }
            | Self::Type { fact_id, .. }
            | Self::Trait { fact_id, .. }
            | Self::Implementation { fact_id, .. }
            | Self::Signature { fact_id, .. }
            | Self::Call { fact_id, .. }
            | Self::Constructor { fact_id, .. }
            | Self::Effect { fact_id, .. }
            | Self::Expansion { fact_id, .. }
            | Self::Configuration { fact_id, .. }
            | Self::TypeShape { fact_id, .. }
            | Self::FunctionSafety { fact_id, .. }
            | Self::CallArgumentProvenance { fact_id, .. } => fact_id,
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UnknownFactKind {
    AmbiguousTraitDispatch,
    DynamicDispatch,
    Ffi,
    UnsupportedExpansion,
    OpaqueGeneratedCode,
    MissingBody,
    UnresolvedDefinition,
    UnsupportedCompilerShape,
    AmbiguousValueProvenance,
    MissingValueProvenance,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct UnknownCompilerFact {
    pub unknown_id: String,
    pub kind: UnknownFactKind,
    pub owner_item_id: Option<String>,
    pub target_identity: Option<String>,
    pub span_id: Option<String>,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CollectionStatus {
    Complete,
    Incomplete,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureShard {
    pub schema_version: String,
    pub shard_id: String,
    pub unit: CompilationUnit,
    pub collection_status: CollectionStatus,
    pub spans: Vec<SourceSpan>,
    pub facts: Vec<CompilerArchitectureFact>,
    pub unknowns: Vec<UnknownCompilerFact>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedCompilationUnit {
    pub unit: CompilationUnit,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceRoleScope {
    pub source_scope: String,
    pub role: CompilerArchitectureRole,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CollectorCompilationUnit {
    pub rustc_crate_name: String,
    pub manifest_dir: String,
    pub source_roles: Vec<SourceRoleScope>,
    pub unit: CompilationUnit,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureCollectorContext {
    pub schema_version: String,
    pub output_directory: String,
    pub limits: ArchitectureCollectionLimits,
    pub selected_units: Vec<CollectorCompilationUnit>,
    #[serde(default)]
    pub value_flow_selections: Vec<CompilerValueFlowSelection>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureIr {
    pub schema_version: String,
    pub architecture_ir_id: String,
    pub policy_blake3: String,
    pub units: Vec<CompilationUnit>,
    pub spans: Vec<SourceSpan>,
    pub facts: Vec<CompilerArchitectureFact>,
    pub unknowns: Vec<UnknownCompilerFact>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoverageStatus {
    Complete,
    Incomplete,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureCoverage {
    pub schema_version: String,
    pub coverage_id: String,
    pub status: CoverageStatus,
    pub expected_unit_ids: Vec<String>,
    pub observed_unit_ids: Vec<String>,
    pub missing_unit_ids: Vec<String>,
    pub unexpected_unit_ids: Vec<String>,
    pub duplicate_unit_ids: Vec<String>,
    pub invalid_shard_ids: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureIssue {
    pub schema_version: String,
    pub issue_id: String,
    pub code: String,
    pub unit_id: Option<String>,
    pub fact_id: Option<String>,
    pub message: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CompilerArchitectureReceipt {
    pub schema_version: String,
    pub receipt_id: String,
    pub status: CollectionStatus,
    pub policy_blake3: String,
    pub coverage_id: String,
    pub architecture_ir_id: Option<String>,
    pub issue_ids: Vec<String>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvalidCompilerArchitectureShard {
    pub shard_id: String,
    pub code: String,
    pub message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizationInput {
    pub policy_blake3: String,
    pub limits: ArchitectureCollectionLimits,
    pub expected_units: Vec<ExpectedCompilationUnit>,
    pub shards: Vec<CompilerArchitectureShard>,
    pub invalid_shards: Vec<InvalidCompilerArchitectureShard>,
    pub non_claims: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NormalizationOutcome {
    pub architecture_ir: Option<CompilerArchitectureIr>,
    pub coverage: CompilerArchitectureCoverage,
    pub issues: Vec<CompilerArchitectureIssue>,
    pub receipt: CompilerArchitectureReceipt,
}

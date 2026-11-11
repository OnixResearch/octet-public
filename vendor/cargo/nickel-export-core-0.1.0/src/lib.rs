#![no_std]
#![doc = "Evaluator-neutral deterministic Nickel export identities and receipts."]

// r[impl nickel_export.core.evaluator_neutral]

extern crate alloc;

#[cfg(feature = "serde")]
use alloc::collections::BTreeSet;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use core::fmt;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// Canonical request schema.
pub const REQUEST_SCHEMA: &str = "onix-nickel-export-request/v1";
/// Canonical receipt schema.
pub const RECEIPT_SCHEMA: &str = "onix-nickel-export-receipt/v1";
/// Canonical manifest schema.
pub const MANIFEST_SCHEMA: &str = "onix-nickel-export-manifest/v1";
/// Canonical diagnostic schema.
pub const DIAGNOSTIC_SCHEMA: &str = "onix-nickel-export-diagnostic/v1";
/// Octet compatibility manifest schema.
pub const OCTET_MANIFEST_SCHEMA: &str = "octet-nickel-export-manifest/v1";
/// Mantle compatibility receipt schema.
pub const MANTLE_RECEIPT_SCHEMA: &str = "mantle-nickel-export-receipt-v1";
/// Project generator identity.
pub const GENERATOR_ID: &str = "nickel-export-core/v1";
/// Generator identity retained by the Octet v1 projection.
pub const OCTET_GENERATOR_ID: &str = "octet-standards.nickel-export-helper/v1";
/// Non-claim retained by the Mantle v1 projection.
pub const MANTLE_NON_CLAIM: &str = "Nickel export success proves only the declared evaluation output digest under the recorded evaluator descriptor; it does not prove deployability, frontend correctness, or build success";
/// Bound on user-provided lists before allocation-heavy processing.
pub const MAX_ARTIFACTS: usize = 4_096;
/// Non-claim carried by canonical and compatibility receipts.
pub const NON_CLAIM: &str = "Nickel export success proves only exact declared input and output identities under the recorded evaluator descriptor; it does not prove deployability, product-policy conformance, evaluator equivalence, build success, or release eligibility";

const BLAKE3_PREFIX: &str = "b3:";
const HEX_RADIX: u8 = 16;
const HEX_DIGITS_PER_BYTE: usize = 2;
const HALF_BYTE_BITS: u32 = 4;
const LOW_HALF_BYTE_MASK: u8 = 0x0f;
const HEX_DIGITS: &[u8; HEX_RADIX as usize] = b"0123456789abcdef";
const SECRET_MARKERS: &[&[u8]] = &[
    b"begin private key",
    b"password=",
    b"password =",
    b"token=",
    b"token =",
    b"secret=",
    b"secret =",
    b"api_key=",
    b"api_key =",
];

/// Output formats natively supported by Nickel.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(rename_all = "lowercase")
)]
pub enum ExportFormat {
    /// JSON output.
    Json,
    /// TOML output.
    Toml,
    /// YAML output.
    Yaml,
    /// Raw text output.
    Raw,
}

impl ExportFormat {
    /// Return the Nickel CLI spelling.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Toml => "toml",
            Self::Yaml => "yaml",
            Self::Raw => "raw",
        }
    }
}

/// How dependency closure was established.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(rename_all = "snake_case")
)]
pub enum ImportPathPolicy {
    /// The shell supplied declared paths; the evaluator did not report a closure.
    DeclaredOnly,
    /// The evaluator reported the complete observed dependency closure.
    EvaluatorObservedClosure,
}

/// Severity of one structured evaluator or contract diagnostic.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(
    feature = "serde",
    derive(Deserialize, Serialize),
    serde(rename_all = "lowercase")
)]
pub enum DiagnosticSeverity {
    /// Informational evidence.
    Note,
    /// Non-fatal warning.
    Warning,
    /// Fatal evaluation or contract failure.
    Error,
}

/// Structured diagnostic at the evaluator-neutral boundary.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct Diagnostic {
    /// Diagnostic schema.
    pub schema: String,
    /// Stable machine-readable class.
    pub class: String,
    /// Related source, selector, contract, or output.
    pub subject: String,
    /// Human-readable detail.
    pub message: String,
    /// Severity.
    pub severity: DiagnosticSeverity,
}

impl Diagnostic {
    /// Construct a canonical diagnostic.
    #[must_use]
    pub fn new(class: &str, subject: &str, message: &str, severity: DiagnosticSeverity) -> Self {
        Self {
            schema: DIAGNOSTIC_SCHEMA.to_string(),
            class: class.to_string(),
            subject: subject.to_string(),
            message: message.to_string(),
            severity,
        }
    }
}

/// One normalized export request.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExportRequest {
    /// Request schema.
    pub schema: String,
    /// Consumer-defined family identifier.
    pub family_id: String,
    /// Repository-root-relative source path.
    pub source: String,
    /// Complete declared dependency paths.
    #[cfg_attr(feature = "serde", serde(default))]
    pub dependencies: Vec<String>,
    /// Evaluator import roots, recorded but not interpreted by the core.
    #[cfg_attr(feature = "serde", serde(default))]
    pub import_paths: Vec<String>,
    /// Optional field selector.
    #[cfg_attr(feature = "serde", serde(default))]
    pub selector: String,
    /// Optional consumer-owned contract label or source locator metadata.
    #[cfg_attr(feature = "serde", serde(default))]
    pub contract: String,
    /// Native output format.
    pub format: ExportFormat,
    /// Repository-root-relative destination path.
    pub destination: String,
    /// Explicit opt-in for material matching conservative secret markers.
    #[cfg_attr(feature = "serde", serde(default))]
    pub allow_secret_material: bool,
}

/// Exact material for one source or output artifact.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactMaterial<'a> {
    /// Normalized path.
    pub path: &'a str,
    /// Exact bytes.
    pub bytes: &'a [u8],
}

/// Exact-byte identity for one artifact.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ArtifactIdentity {
    /// Normalized path.
    pub path: String,
    /// BLAKE3 identity with the `b3:` algorithm tag.
    pub identity: String,
    /// Exact byte length.
    pub bytes: u64,
}

/// Evaluator implementation and option descriptor.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct EvaluatorDescriptor {
    /// Evaluator implementation or derivation identity.
    pub identity: String,
    /// Exact evaluator version or package identity.
    pub version: String,
    /// Stable evaluator options.
    #[cfg_attr(feature = "serde", serde(default))]
    pub options: Vec<String>,
    /// Dependency-observation policy.
    pub import_path_policy: ImportPathPolicy,
}

/// Pure post-evaluation input used to admit a receipt.
pub struct EvaluationObservation<'a> {
    /// Validated export request.
    pub request: &'a ExportRequest,
    /// Exact root source bytes.
    pub source: ArtifactMaterial<'a>,
    /// Exact declared dependency bytes.
    pub dependencies: Vec<ArtifactMaterial<'a>>,
    /// Exact generated output bytes.
    pub output: ArtifactMaterial<'a>,
    /// Evaluator descriptor.
    pub evaluator: &'a EvaluatorDescriptor,
    /// Paths observed by an evaluator able to report dependency closure.
    pub observed_dependencies: Vec<&'a str>,
    /// Structured evaluator and contract diagnostics.
    pub diagnostics: Vec<Diagnostic>,
}

/// Accepted deterministic export receipt.
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExportReceipt {
    /// Receipt schema.
    pub schema: String,
    /// Consumer-defined family identifier.
    pub family_id: String,
    /// Root source identity.
    pub source: ArtifactIdentity,
    /// Sorted dependency identities.
    pub dependencies: Vec<ArtifactIdentity>,
    /// Recorded import paths.
    pub import_paths: Vec<String>,
    /// Optional field selector.
    pub selector: String,
    /// Optional consumer-owned contract label.
    pub contract: String,
    /// Native output format.
    pub format: ExportFormat,
    /// Generated output identity.
    pub output: ArtifactIdentity,
    /// Evaluator descriptor.
    pub evaluator: EvaluatorDescriptor,
    /// Sorted non-fatal diagnostics.
    pub diagnostics: Vec<Diagnostic>,
    /// Explicit claim boundary.
    pub non_claim: String,
}

/// Multi-export freshness and mixed-evaluator manifest.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct ExportManifest {
    /// Manifest schema.
    pub schema: String,
    /// Generator identity.
    pub generator: String,
    /// Evaluator shared by every receipt.
    pub evaluator: EvaluatorDescriptor,
    /// Receipts sorted by output path.
    pub exports: Vec<ExportReceipt>,
    /// BLAKE3 identity of the canonical manifest payload.
    pub manifest_identity: String,
}

/// Deterministic core rejection.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CoreError {
    /// One or more request fields are invalid.
    InvalidRequest(Vec<Diagnostic>),
    /// Material does not match the declared request.
    MaterialMismatch(Vec<Diagnostic>),
    /// Evaluation or contract diagnostics include an error.
    EvaluationFailed(Vec<Diagnostic>),
    /// An observed dependency was not declared.
    UndeclaredDependency(String),
    /// Declared and evaluator-observed dependency closures differ.
    DependencyClosureMismatch,
    /// Secret-like material requires explicit opt-in.
    SecretMaterial(String),
    /// A manifest mixed evaluator identities.
    MixedEvaluators,
    /// A manifest contains duplicate output paths.
    DuplicateOutput(String),
    /// Canonical serialization failed.
    Serialization,
    /// A checked-in manifest differs from current exact inputs or outputs.
    StaleManifest,
}

impl fmt::Display for CoreError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidRequest(_) => formatter.write_str("invalid export request"),
            Self::MaterialMismatch(_) => {
                formatter.write_str("export material does not match its request")
            }
            Self::EvaluationFailed(_) => {
                formatter.write_str("evaluation or contract diagnostics contain an error")
            }
            Self::UndeclaredDependency(path) => {
                write!(formatter, "undeclared evaluator dependency `{path}`")
            }
            Self::DependencyClosureMismatch => {
                formatter.write_str("declared and evaluator-observed dependency closures differ")
            }
            Self::SecretMaterial(path) => {
                write!(formatter, "secret-like material rejected in `{path}`")
            }
            Self::MixedEvaluators => {
                formatter.write_str("manifest cannot mix evaluator descriptors")
            }
            Self::DuplicateOutput(path) => write!(formatter, "duplicate manifest output `{path}`"),
            Self::Serialization => formatter.write_str("canonical manifest serialization failed"),
            Self::StaleManifest => formatter.write_str("checked-in export manifest is stale"),
        }
    }
}

/// Validate and normalize one request without I/O.
///
/// # Errors
///
/// Returns structured diagnostics when schemas, paths, identities, or list
/// bounds are invalid.
pub fn normalize_request(request: &ExportRequest) -> Result<ExportRequest, CoreError> {
    let mut diagnostics = Vec::new();
    if request.schema != REQUEST_SCHEMA {
        diagnostics.push(error(
            "unsupported-schema",
            "schema",
            "request schema is not supported",
        ));
    }
    require_nonempty(&request.family_id, "family_id", &mut diagnostics);
    require_nonempty(&request.destination, "destination", &mut diagnostics);
    if request.dependencies.len() > MAX_ARTIFACTS || request.import_paths.len() > MAX_ARTIFACTS {
        diagnostics.push(error(
            "artifact-bound",
            "dependencies",
            "request exceeds the bounded artifact count",
        ));
    }

    let source = normalize_path_field(&request.source, "source", &mut diagnostics);
    let destination = normalize_path_field(&request.destination, "destination", &mut diagnostics);
    let dependencies = normalize_path_list(&request.dependencies, "dependency", &mut diagnostics);
    let import_paths = normalize_path_list(&request.import_paths, "import-path", &mut diagnostics);
    if source == destination && !source.is_empty() {
        diagnostics.push(error(
            "overlapping-output",
            &destination,
            "destination must not overwrite the source",
        ));
    }
    if dependencies.iter().any(|path| path == &destination) && !destination.is_empty() {
        diagnostics.push(error(
            "overlapping-output",
            &destination,
            "destination must not overwrite a dependency",
        ));
    }
    if !diagnostics.is_empty() {
        diagnostics.sort();
        return Err(CoreError::InvalidRequest(diagnostics));
    }

    Ok(ExportRequest {
        schema: REQUEST_SCHEMA.to_string(),
        family_id: request.family_id.trim().to_string(),
        source,
        dependencies,
        import_paths,
        selector: request.selector.trim().to_string(),
        contract: request.contract.trim().to_string(),
        format: request.format,
        destination,
        allow_secret_material: request.allow_secret_material,
    })
}

/// Build an accepted receipt from exact post-evaluation material.
///
/// # Errors
///
/// Fails closed for malformed requests, material mismatches, secret-like
/// material, undeclared observed dependencies, or error diagnostics.
// r[impl nickel_export.core.identity]
// r[impl nickel_export.core.fail_closed]
pub fn build_receipt(observation: &EvaluationObservation<'_>) -> Result<ExportReceipt, CoreError> {
    let request = normalize_request(observation.request)?;
    let mut mismatches = Vec::new();
    if observation.source.path != request.source {
        mismatches.push(error(
            "source-path-mismatch",
            observation.source.path,
            "source material path differs from request",
        ));
    }
    if observation.output.path != request.destination {
        mismatches.push(error(
            "output-path-mismatch",
            observation.output.path,
            "output material path differs from request",
        ));
    }
    let mut dependencies = observation
        .dependencies
        .iter()
        .map(artifact_identity)
        .collect::<Vec<_>>();
    dependencies.sort();
    let actual_paths = dependencies
        .iter()
        .map(|artifact| artifact.path.clone())
        .collect::<Vec<_>>();
    if actual_paths != request.dependencies {
        mismatches.push(error(
            "dependency-set-mismatch",
            &request.source,
            "dependency material paths differ from the request",
        ));
    }
    if !mismatches.is_empty() {
        mismatches.sort();
        return Err(CoreError::MaterialMismatch(mismatches));
    }

    let evaluator = validate_evaluator(observation.evaluator)?;
    reject_secret_material(&request, observation)?;
    validate_dependency_closure(&request, observation)?;
    let mut diagnostics = observation.diagnostics.clone();
    diagnostics.sort();
    if diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == DiagnosticSeverity::Error)
    {
        return Err(CoreError::EvaluationFailed(diagnostics));
    }

    Ok(ExportReceipt {
        schema: RECEIPT_SCHEMA.to_string(),
        family_id: request.family_id,
        source: artifact_identity(&observation.source),
        dependencies,
        import_paths: request.import_paths,
        selector: request.selector,
        contract: request.contract,
        format: request.format,
        output: artifact_identity(&observation.output),
        evaluator,
        diagnostics,
        non_claim: NON_CLAIM.to_string(),
    })
}

/// Build a deterministic manifest while prohibiting mixed evaluators.
///
/// # Errors
///
/// Returns an error for empty input, duplicate outputs, mixed evaluator
/// descriptors, or unavailable canonical serialization.
#[cfg(feature = "serde")]
pub fn build_manifest(receipts: &[ExportReceipt]) -> Result<ExportManifest, CoreError> {
    let Some(first) = receipts.first() else {
        return Err(CoreError::InvalidRequest(vec![error(
            "empty-manifest",
            "exports",
            "manifest requires at least one receipt",
        )]));
    };
    let evaluator = first.evaluator.clone();
    let mut exports = receipts.to_vec();
    exports.sort_by(|left, right| left.output.path.cmp(&right.output.path));
    let mut outputs = BTreeSet::new();
    for receipt in &exports {
        if receipt.evaluator != evaluator {
            return Err(CoreError::MixedEvaluators);
        }
        if !outputs.insert(receipt.output.path.clone()) {
            return Err(CoreError::DuplicateOutput(receipt.output.path.clone()));
        }
    }
    let payload = ManifestPayload {
        schema: MANIFEST_SCHEMA,
        generator: GENERATOR_ID,
        evaluator: &evaluator,
        exports: &exports,
    };
    let bytes = serde_json::to_vec(&payload).map_err(|_| CoreError::Serialization)?;
    Ok(ExportManifest {
        schema: MANIFEST_SCHEMA.to_string(),
        generator: GENERATOR_ID.to_string(),
        evaluator,
        exports,
        manifest_identity: blake3_identity(&bytes),
    })
}

/// Compare a checked-in manifest with a freshly derived manifest.
///
/// # Errors
///
/// Returns [`CoreError::StaleManifest`] when any exact identity or metadata
/// differs.
pub fn verify_manifest_fresh(
    expected: &ExportManifest,
    actual: &ExportManifest,
) -> Result<(), CoreError> {
    if expected == actual {
        Ok(())
    } else {
        Err(CoreError::StaleManifest)
    }
}

#[cfg(feature = "serde")]
#[derive(Serialize)]
struct ManifestPayload<'a> {
    schema: &'static str,
    generator: &'static str,
    evaluator: &'a EvaluatorDescriptor,
    exports: &'a [ExportReceipt],
}

/// Octet v1 compatibility projection.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct OctetManifest {
    /// Legacy schema version.
    pub schema_version: String,
    /// Generator identity.
    pub generator: String,
    /// Legacy evaluator identity string.
    pub nickel_identity: String,
    /// Legacy exports.
    pub exports: Vec<OctetManifestEntry>,
}

/// One Octet v1 compatibility entry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct OctetManifestEntry {
    /// Family identifier.
    pub family_id: String,
    /// Output format.
    pub output_format: String,
    /// Root source identity.
    pub source: OctetIdentity,
    /// Dependency identities.
    pub dependencies: Vec<OctetIdentity>,
    /// Output identity.
    pub output: OctetIdentity,
    /// Selector.
    pub selector: String,
    /// Contract label.
    pub contract: String,
    /// Shell-friendly replay note.
    pub generation_command: String,
    /// Shell-free replay shape.
    pub command_argv: Vec<String>,
}

/// Octet v1 exact identity shape.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct OctetIdentity {
    /// Artifact path.
    pub path: String,
    /// Bare BLAKE3 hex digest retained for compatibility.
    pub blake3: String,
    /// Exact byte length.
    pub bytes: u64,
}

/// Project one canonical manifest into the Octet v1 shape.
#[must_use]
// r[impl nickel_export.compat.projections]
pub fn project_octet_manifest(manifest: &ExportManifest) -> OctetManifest {
    OctetManifest {
        schema_version: OCTET_MANIFEST_SCHEMA.to_string(),
        generator: OCTET_GENERATOR_ID.to_string(),
        nickel_identity: manifest.evaluator.identity.clone(),
        exports: manifest.exports.iter().map(project_octet_entry).collect(),
    }
}

/// Mantle v1 compatibility receipt.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MantleReceipt {
    /// Legacy schema.
    pub schema: String,
    /// Root source.
    pub root_source: MantleSourceRef,
    /// Dependencies.
    pub deps: Vec<MantleSourceRef>,
    /// Import paths.
    pub import_paths: Vec<String>,
    /// Format.
    pub format: String,
    /// Output destination.
    pub output_target: String,
    /// Bare output BLAKE3 digest.
    pub output_digest_blake3: String,
    /// Evaluator descriptor.
    pub evaluator: MantleEvaluatorDescriptor,
    /// Legacy claim boundary.
    pub non_claim: String,
}

/// Mantle v1 source reference.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MantleSourceRef {
    /// Source path.
    pub path: String,
    /// Bare BLAKE3 digest.
    pub digest_blake3: String,
}

/// Mantle v1 evaluator descriptor.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(Deserialize, Serialize))]
pub struct MantleEvaluatorDescriptor {
    /// Evaluator identity.
    pub identity: String,
    /// Evaluator version.
    pub version: String,
    /// Stable options.
    pub options: Vec<String>,
}

/// Project one canonical receipt into the Mantle v1 shape.
#[must_use]
pub fn project_mantle_receipt(receipt: &ExportReceipt) -> MantleReceipt {
    MantleReceipt {
        schema: MANTLE_RECEIPT_SCHEMA.to_string(),
        root_source: mantle_source(&receipt.source),
        deps: receipt.dependencies.iter().map(mantle_source).collect(),
        import_paths: receipt.import_paths.clone(),
        format: receipt.format.as_str().to_string(),
        output_target: receipt.output.path.clone(),
        output_digest_blake3: bare_blake3(&receipt.output.identity).to_string(),
        evaluator: MantleEvaluatorDescriptor {
            identity: receipt.evaluator.identity.clone(),
            version: receipt.evaluator.version.clone(),
            options: receipt.evaluator.options.clone(),
        },
        non_claim: MANTLE_NON_CLAIM.to_string(),
    }
}

/// Return a BLAKE3 identity for exact bytes.
#[must_use]
pub fn blake3_identity(bytes: &[u8]) -> String {
    let digest = blake3::hash(bytes);
    let mut output =
        String::with_capacity(BLAKE3_PREFIX.len() + digest.as_bytes().len() * HEX_DIGITS_PER_BYTE);
    output.push_str(BLAKE3_PREFIX);
    for byte in digest.as_bytes() {
        output.push(char::from(HEX_DIGITS[usize::from(byte >> HALF_BYTE_BITS)]));
        output.push(char::from(
            HEX_DIGITS[usize::from(byte & LOW_HALF_BYTE_MASK)],
        ));
    }
    output
}

fn artifact_identity(material: &ArtifactMaterial<'_>) -> ArtifactIdentity {
    ArtifactIdentity {
        path: material.path.to_string(),
        identity: blake3_identity(material.bytes),
        bytes: u64::try_from(material.bytes.len()).unwrap_or(u64::MAX),
    }
}

fn validate_evaluator(evaluator: &EvaluatorDescriptor) -> Result<EvaluatorDescriptor, CoreError> {
    let mut diagnostics = Vec::new();
    require_nonempty(&evaluator.identity, "evaluator.identity", &mut diagnostics);
    require_nonempty(&evaluator.version, "evaluator.version", &mut diagnostics);
    if evaluator.options.len() > MAX_ARTIFACTS {
        diagnostics.push(error(
            "option-bound",
            "evaluator.options",
            "evaluator exceeds the bounded option count",
        ));
    }
    if !diagnostics.is_empty() {
        return Err(CoreError::InvalidRequest(diagnostics));
    }
    let mut normalized = evaluator.clone();
    normalized.options.sort();
    normalized.options.dedup();
    Ok(normalized)
}

fn normalize_path_list(
    paths: &[String],
    field: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<String> {
    let mut normalized = paths
        .iter()
        .map(|path| normalize_path_field(path, field, diagnostics))
        .filter(|path| !path.is_empty())
        .collect::<Vec<_>>();
    normalized.sort();
    let original_len = normalized.len();
    normalized.dedup();
    if normalized.len() != original_len {
        diagnostics.push(error(
            "duplicate-path",
            field,
            "path list contains duplicates",
        ));
    }
    normalized
}

fn normalize_path_field(path: &str, field: &str, diagnostics: &mut Vec<Diagnostic>) -> String {
    match normalize_relative_path(path) {
        Ok(normalized) => normalized,
        Err(message) => {
            diagnostics.push(error("unsafe-path", field, message));
            String::new()
        }
    }
}

fn normalize_relative_path(path: &str) -> Result<String, &'static str> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("path must not be empty");
    }
    if trimmed.starts_with('/') || trimmed.contains('\\') {
        return Err("path must be portable and repository-root-relative");
    }
    let mut normalized = Vec::new();
    for component in trimmed.split('/') {
        match component {
            "" | "." => {}
            ".." => return Err("path must not traverse a parent directory"),
            safe => normalized.push(safe),
        }
    }
    if normalized.is_empty() {
        return Err("path must name an artifact");
    }
    Ok(normalized.join("/"))
}

fn reject_secret_material(
    request: &ExportRequest,
    observation: &EvaluationObservation<'_>,
) -> Result<(), CoreError> {
    if request.allow_secret_material {
        return Ok(());
    }
    let materials = core::iter::once(&observation.source).chain(observation.dependencies.iter());
    for material in materials {
        let lowercase = material
            .bytes
            .iter()
            .map(u8::to_ascii_lowercase)
            .collect::<Vec<_>>();
        if SECRET_MARKERS
            .iter()
            .any(|marker| contains_bytes(&lowercase, marker))
        {
            return Err(CoreError::SecretMaterial(material.path.to_string()));
        }
    }
    Ok(())
}

fn validate_dependency_closure(
    request: &ExportRequest,
    observation: &EvaluationObservation<'_>,
) -> Result<(), CoreError> {
    if observation.evaluator.import_path_policy == ImportPathPolicy::DeclaredOnly {
        if observation.observed_dependencies.is_empty() {
            return Ok(());
        }
        return Err(CoreError::DependencyClosureMismatch);
    }
    let mut observed = observation
        .observed_dependencies
        .iter()
        .map(|path| {
            normalize_relative_path(path)
                .map_err(|_| CoreError::UndeclaredDependency((*path).to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    observed.sort();
    observed.dedup();
    for path in &observed {
        if !request.dependencies.contains(path) {
            return Err(CoreError::UndeclaredDependency(path.clone()));
        }
    }
    if observed == request.dependencies {
        Ok(())
    } else {
        Err(CoreError::DependencyClosureMismatch)
    }
}

fn contains_bytes(haystack: &[u8], needle: &[u8]) -> bool {
    haystack
        .windows(needle.len())
        .any(|window| window == needle)
}

fn project_octet_entry(receipt: &ExportReceipt) -> OctetManifestEntry {
    let mut command_argv = vec![
        "nickel".to_string(),
        "export".to_string(),
        "--format".to_string(),
        receipt.format.as_str().to_string(),
        receipt.source.path.clone(),
    ];
    if !receipt.selector.is_empty() {
        command_argv.push("--field".to_string());
        command_argv.push(receipt.selector.clone());
    }
    OctetManifestEntry {
        family_id: receipt.family_id.clone(),
        output_format: receipt.format.as_str().to_string(),
        source: octet_identity(&receipt.source),
        dependencies: receipt.dependencies.iter().map(octet_identity).collect(),
        output: octet_identity(&receipt.output),
        selector: receipt.selector.clone(),
        contract: receipt.contract.clone(),
        generation_command: command_argv.join(" "),
        command_argv,
    }
}

fn octet_identity(identity: &ArtifactIdentity) -> OctetIdentity {
    OctetIdentity {
        path: identity.path.clone(),
        blake3: bare_blake3(&identity.identity).to_string(),
        bytes: identity.bytes,
    }
}

fn mantle_source(identity: &ArtifactIdentity) -> MantleSourceRef {
    MantleSourceRef {
        path: identity.path.clone(),
        digest_blake3: bare_blake3(&identity.identity).to_string(),
    }
}

fn bare_blake3(identity: &str) -> &str {
    identity.strip_prefix(BLAKE3_PREFIX).unwrap_or(identity)
}

fn require_nonempty(value: &str, field: &str, diagnostics: &mut Vec<Diagnostic>) {
    if value.trim().is_empty() {
        diagnostics.push(error(
            "empty-field",
            field,
            "required field must not be empty",
        ));
    }
}

fn error(class: &str, subject: &str, message: &str) -> Diagnostic {
    Diagnostic::new(class, subject, message, DiagnosticSeverity::Error)
}

#[cfg(test)]
extern crate std;

#[cfg(test)]
#[allow(clippy::panic)]
mod tests {
    use super::*;
    use alloc::vec;

    const SOURCE: &[u8] = b"{ value = 1 }\n";
    const OUTPUT: &[u8] = b"{\"value\":1}\n";
    const DEPENDENCY: &[u8] = b"{ enabled = true }\n";

    fn request(destination: &str) -> ExportRequest {
        ExportRequest {
            schema: REQUEST_SCHEMA.to_string(),
            family_id: "tests.config".to_string(),
            source: "config/source.ncl".to_string(),
            dependencies: vec!["config/dependency.ncl".to_string()],
            import_paths: vec!["config".to_string()],
            selector: "value".to_string(),
            contract: "config/dependency.ncl".to_string(),
            format: ExportFormat::Json,
            destination: destination.to_string(),
            allow_secret_material: false,
        }
    }

    fn evaluator(identity: &str) -> EvaluatorDescriptor {
        EvaluatorDescriptor {
            identity: identity.to_string(),
            version: "nickel-1.13.0".to_string(),
            options: vec!["format=json".to_string()],
            import_path_policy: ImportPathPolicy::EvaluatorObservedClosure,
        }
    }

    fn receipt_for(destination: &str, evaluator: &EvaluatorDescriptor) -> ExportReceipt {
        let request = request(destination);
        let observation = EvaluationObservation {
            request: &request,
            source: ArtifactMaterial {
                path: "config/source.ncl",
                bytes: SOURCE,
            },
            dependencies: vec![ArtifactMaterial {
                path: "config/dependency.ncl",
                bytes: DEPENDENCY,
            }],
            output: ArtifactMaterial {
                path: destination,
                bytes: OUTPUT,
            },
            evaluator,
            observed_dependencies: vec!["config/dependency.ncl"],
            diagnostics: vec![Diagnostic::new(
                "contract-ok",
                "config/dependency.ncl",
                "contract admitted output",
                DiagnosticSeverity::Note,
            )],
        };
        match build_receipt(&observation) {
            Ok(receipt) => receipt,
            Err(error) => panic_for_test(error),
        }
    }

    #[test]
    fn exact_inputs_produce_stable_receipts_and_manifests() {
        let evaluator = evaluator("nickel-cli");
        let receipt = receipt_for("generated/config.json", &evaluator);
        let first = build_manifest(core::slice::from_ref(&receipt));
        let second = build_manifest(core::slice::from_ref(&receipt));
        assert_eq!(first, second);
        let manifest = first.unwrap_or_else(panic_for_test);
        assert!(manifest.manifest_identity.starts_with(BLAKE3_PREFIX));
        assert_eq!(verify_manifest_fresh(&manifest, &manifest), Ok(()));
    }

    #[test]
    fn normalizes_dependency_order_before_identity() {
        let mut request = request("generated/config.json");
        request.dependencies = vec![
            "z.ncl".to_string(),
            "config/dependency.ncl".to_string(),
            "a.ncl".to_string(),
        ];
        let normalized = normalize_request(&request).unwrap_or_else(panic_for_test);
        assert_eq!(
            normalized.dependencies,
            vec![
                "a.ncl".to_string(),
                "config/dependency.ncl".to_string(),
                "z.ncl".to_string(),
            ]
        );
    }

    #[test]
    fn embedded_observations_cover_all_formats_selector_contract_and_multiple_dependencies() {
        let formats = [
            ExportFormat::Json,
            ExportFormat::Toml,
            ExportFormat::Yaml,
            ExportFormat::Raw,
        ];
        let evaluator = EvaluatorDescriptor {
            identity: "mantle-embedded-crunch-eval".to_string(),
            version: "mantle-evaluator-fixture-v1".to_string(),
            options: vec!["deterministic=true".to_string()],
            import_path_policy: ImportPathPolicy::EvaluatorObservedClosure,
        };
        for format in formats {
            let mut request = request("generated/config.out");
            request.format = format;
            request.dependencies.push("config/second.ncl".to_string());
            let observation = EvaluationObservation {
                request: &request,
                source: ArtifactMaterial {
                    path: "config/source.ncl",
                    bytes: SOURCE,
                },
                dependencies: vec![
                    ArtifactMaterial {
                        path: "config/dependency.ncl",
                        bytes: DEPENDENCY,
                    },
                    ArtifactMaterial {
                        path: "config/second.ncl",
                        bytes: b"{ mode = \"embedded\" }\n",
                    },
                ],
                output: ArtifactMaterial {
                    path: "generated/config.out",
                    bytes: OUTPUT,
                },
                evaluator: &evaluator,
                observed_dependencies: vec!["config/second.ncl", "config/dependency.ncl"],
                diagnostics: Vec::new(),
            };
            let receipt = build_receipt(&observation).unwrap_or_else(panic_for_test);
            assert_eq!(receipt.format, format);
            assert_eq!(receipt.selector, "value");
            assert_eq!(receipt.contract, "config/dependency.ncl");
            assert_eq!(receipt.dependencies.len(), request.dependencies.len());
            assert_eq!(receipt.evaluator.identity, evaluator.identity);
        }
    }

    #[test]
    fn rejects_unsafe_paths_and_duplicate_dependencies() {
        let mut request = request("../generated/config.json");
        request
            .dependencies
            .push("config/dependency.ncl".to_string());
        let result = normalize_request(&request);
        assert!(matches!(result, Err(CoreError::InvalidRequest(_))));
    }

    #[test]
    fn rejects_undeclared_observed_dependency() {
        let evaluator = evaluator("nickel-cli");
        let request = request("generated/config.json");
        let observation = EvaluationObservation {
            request: &request,
            source: ArtifactMaterial {
                path: "config/source.ncl",
                bytes: SOURCE,
            },
            dependencies: vec![ArtifactMaterial {
                path: "config/dependency.ncl",
                bytes: DEPENDENCY,
            }],
            output: ArtifactMaterial {
                path: "generated/config.json",
                bytes: OUTPUT,
            },
            evaluator: &evaluator,
            observed_dependencies: vec!["config/hidden.ncl"],
            diagnostics: Vec::new(),
        };
        assert_eq!(
            build_receipt(&observation),
            Err(CoreError::UndeclaredDependency(
                "config/hidden.ncl".to_string()
            ))
        );
    }

    #[test]
    fn rejects_secret_like_material_without_opt_in() {
        let evaluator = evaluator("nickel-cli");
        let request = request("generated/config.json");
        let observation = EvaluationObservation {
            request: &request,
            source: ArtifactMaterial {
                path: "config/source.ncl",
                bytes: b"{ token = \"raw-value\" }",
            },
            dependencies: vec![ArtifactMaterial {
                path: "config/dependency.ncl",
                bytes: DEPENDENCY,
            }],
            output: ArtifactMaterial {
                path: "generated/config.json",
                bytes: OUTPUT,
            },
            evaluator: &evaluator,
            observed_dependencies: vec!["config/dependency.ncl"],
            diagnostics: Vec::new(),
        };
        assert!(matches!(
            build_receipt(&observation),
            Err(CoreError::SecretMaterial(_))
        ));
    }

    #[test]
    fn rejects_contract_error_diagnostics() {
        let evaluator = evaluator("nickel-cli");
        let request = request("generated/config.json");
        let observation = EvaluationObservation {
            request: &request,
            source: ArtifactMaterial {
                path: "config/source.ncl",
                bytes: SOURCE,
            },
            dependencies: vec![ArtifactMaterial {
                path: "config/dependency.ncl",
                bytes: DEPENDENCY,
            }],
            output: ArtifactMaterial {
                path: "generated/config.json",
                bytes: OUTPUT,
            },
            evaluator: &evaluator,
            observed_dependencies: vec!["config/dependency.ncl"],
            diagnostics: vec![error("contract", "Config", "contract rejected output")],
        };
        assert!(matches!(
            build_receipt(&observation),
            Err(CoreError::EvaluationFailed(_))
        ));
    }

    #[test]
    fn rejects_mixed_evaluator_and_duplicate_output_manifests() {
        let first_evaluator = evaluator("nickel-cli");
        let second_evaluator = evaluator("embedded-nickel");
        let first = receipt_for("generated/config.json", &first_evaluator);
        let second = receipt_for("generated/other.json", &second_evaluator);
        assert_eq!(
            build_manifest(&[first.clone(), second]),
            Err(CoreError::MixedEvaluators)
        );
        assert_eq!(
            build_manifest(&[first.clone(), first]),
            Err(CoreError::DuplicateOutput(
                "generated/config.json".to_string()
            ))
        );
    }

    #[test]
    fn compatibility_projections_preserve_exact_identities() {
        let evaluator = evaluator("nickel-cli");
        let receipt = receipt_for("generated/config.json", &evaluator);
        let manifest =
            build_manifest(core::slice::from_ref(&receipt)).unwrap_or_else(panic_for_test);
        let octet = project_octet_manifest(&manifest);
        let mantle = project_mantle_receipt(&receipt);
        assert_eq!(octet.schema_version, OCTET_MANIFEST_SCHEMA);
        assert_eq!(octet.generator, OCTET_GENERATOR_ID);
        assert_eq!(octet.nickel_identity, receipt.evaluator.identity);
        assert_eq!(
            octet.exports[0].output.blake3,
            bare_blake3(&receipt.output.identity)
        );
        assert_eq!(mantle.schema, MANTLE_RECEIPT_SCHEMA);
        assert_eq!(
            mantle.output_digest_blake3,
            bare_blake3(&receipt.output.identity)
        );
        assert_eq!(mantle.non_claim, MANTLE_NON_CLAIM);
    }

    #[test]
    fn changing_source_dependency_or_output_bytes_changes_receipt_identity() {
        let baseline = custom_receipt(SOURCE, DEPENDENCY, OUTPUT);
        let source_changed = custom_receipt(b"{ value = 2 }\n", DEPENDENCY, OUTPUT);
        let dependency_changed = custom_receipt(SOURCE, b"{ enabled = false }\n", OUTPUT);
        let output_changed = custom_receipt(SOURCE, DEPENDENCY, b"{\"value\":2}\n");
        assert_ne!(baseline.source.identity, source_changed.source.identity);
        assert_ne!(baseline.dependencies, dependency_changed.dependencies);
        assert_ne!(baseline.output.identity, output_changed.output.identity);
    }

    #[test]
    fn rejects_incomplete_observed_closure_and_empty_evaluator_identity() {
        let request = request("generated/config.json");
        let valid_evaluator = evaluator("nickel-cli");
        let incomplete = EvaluationObservation {
            request: &request,
            source: ArtifactMaterial {
                path: "config/source.ncl",
                bytes: SOURCE,
            },
            dependencies: vec![ArtifactMaterial {
                path: "config/dependency.ncl",
                bytes: DEPENDENCY,
            }],
            output: ArtifactMaterial {
                path: "generated/config.json",
                bytes: OUTPUT,
            },
            evaluator: &valid_evaluator,
            observed_dependencies: Vec::new(),
            diagnostics: Vec::new(),
        };
        assert_eq!(
            build_receipt(&incomplete),
            Err(CoreError::DependencyClosureMismatch)
        );

        let invalid_evaluator = evaluator("");
        let invalid = EvaluationObservation {
            evaluator: &invalid_evaluator,
            observed_dependencies: vec!["config/dependency.ncl"],
            ..incomplete
        };
        assert!(matches!(
            build_receipt(&invalid),
            Err(CoreError::InvalidRequest(_))
        ));
    }

    #[test]
    fn secret_opt_in_and_consumer_owned_contract_metadata_are_explicit() {
        let mut secret_request = request("generated/config.json");
        secret_request.allow_secret_material = true;
        let evaluator = evaluator("nickel-cli");
        let observation = EvaluationObservation {
            request: &secret_request,
            source: ArtifactMaterial {
                path: "config/source.ncl",
                bytes: b"{ token = \"fixture-only\" }",
            },
            dependencies: vec![ArtifactMaterial {
                path: "config/dependency.ncl",
                bytes: DEPENDENCY,
            }],
            output: ArtifactMaterial {
                path: "generated/config.json",
                bytes: OUTPUT,
            },
            evaluator: &evaluator,
            observed_dependencies: vec!["config/dependency.ncl"],
            diagnostics: Vec::new(),
        };
        assert!(build_receipt(&observation).is_ok());

        let mut labeled_contract = request("generated/config.json");
        labeled_contract.contract = "DynamicEvidenceProfiles".to_string();
        let normalized = normalize_request(&labeled_contract).unwrap_or_else(panic_for_test);
        assert_eq!(normalized.contract, "DynamicEvidenceProfiles");
    }

    #[test]
    fn rejects_unsupported_serialized_format_malformed_manifest_and_weakened_nonclaim() {
        let unsupported_format = br#"{
            "schema":"onix-nickel-export-request/v1",
            "family_id":"tests.invalid",
            "source":"config/source.ncl",
            "dependencies":[],
            "import_paths":[],
            "selector":"",
            "contract":"",
            "format":"xml",
            "destination":"generated/config.xml",
            "allow_secret_material":false
        }"#;
        assert!(serde_json::from_slice::<ExportRequest>(unsupported_format).is_err());
        assert!(serde_json::from_str::<ExportManifest>("{not-json}").is_err());

        let evaluator = evaluator("nickel-cli");
        let receipt = receipt_for("generated/config.json", &evaluator);
        let expected =
            build_manifest(core::slice::from_ref(&receipt)).unwrap_or_else(panic_for_test);
        let mut weakened = expected.clone();
        weakened.exports[0].non_claim = "export succeeded".to_string();
        assert_eq!(
            verify_manifest_fresh(&expected, &weakened),
            Err(CoreError::StaleManifest)
        );
    }

    #[test]
    fn stale_manifest_is_rejected() {
        let evaluator = evaluator("nickel-cli");
        let receipt = receipt_for("generated/config.json", &evaluator);
        let expected =
            build_manifest(core::slice::from_ref(&receipt)).unwrap_or_else(panic_for_test);
        let mut actual = expected.clone();
        actual.exports[0].output.identity = blake3_identity(b"tampered");
        assert_eq!(
            verify_manifest_fresh(&expected, &actual),
            Err(CoreError::StaleManifest)
        );
    }

    fn custom_receipt(source: &[u8], dependency: &[u8], output: &[u8]) -> ExportReceipt {
        let request = request("generated/config.json");
        let evaluator = evaluator("nickel-cli");
        let observation = EvaluationObservation {
            request: &request,
            source: ArtifactMaterial {
                path: "config/source.ncl",
                bytes: source,
            },
            dependencies: vec![ArtifactMaterial {
                path: "config/dependency.ncl",
                bytes: dependency,
            }],
            output: ArtifactMaterial {
                path: "generated/config.json",
                bytes: output,
            },
            evaluator: &evaluator,
            observed_dependencies: vec!["config/dependency.ncl"],
            diagnostics: Vec::new(),
        };
        build_receipt(&observation).unwrap_or_else(panic_for_test)
    }

    fn panic_for_test<T, E: fmt::Debug>(error: E) -> T {
        std::panic!("unexpected test error: {error:?}")
    }
}

//! Legacy Octet facade over the standalone Nickel export core.
//!
//! Canonical request admission, exact-byte identities, evaluator descriptors,
//! diagnostics, and receipt semantics are owned by the pinned
//! `nickel-export-core` dependency. This module retains only Octet's v1 spec and
//! manifest projection so existing generated artifacts and commands remain
//! compatible during migration.

use nickel_export_core as canonical;
use serde::Deserialize;
use serde::Serialize;

/// Export manifest schema version retained for Octet compatibility.
pub const NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION: &str = canonical::OCTET_MANIFEST_SCHEMA;
/// Export spec schema version retained for Octet compatibility.
pub const NICKEL_EXPORT_SPEC_SCHEMA_VERSION: &str = "octet-nickel-export-spec/v1";
/// Generator identity retained for Octet compatibility.
pub const NICKEL_EXPORT_GENERATOR_ID: &str = canonical::OCTET_GENERATOR_ID;
/// Exact evaluator identity admitted for checked-path export evidence.
pub const REVIEWED_NICKEL_EVALUATOR_IDENTITY: &str = "nickel-lang-cli nickel 1.17.0 (rev 1320a98)";

const FORMAT_JSON: &str = "json";
const FORMAT_TOML: &str = "toml";
const FORMAT_YAML: &str = "yaml";
const FORMAT_RAW: &str = "raw";
const OCTET_LEGACY_EVALUATOR_VERSION: &str = "octet-legacy-evaluator-descriptor/v1";
const SECRET_REMEDIATION: &str = "register this family as non-candidate or replace raw secret material with references";

static SUPPORTED_FORMATS: &[&str] = &[FORMAT_JSON, FORMAT_TOML, FORMAT_YAML, FORMAT_RAW];

/// A single deterministic Nickel export request in Octet's legacy shape.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NickelExportSpec {
    /// Spec schema version.
    pub schema_version: String,
    /// Config or receipt family ID this export belongs to.
    pub family_id: String,
    /// Nickel source path.
    pub source: String,
    /// Declared Nickel dependency/import paths.
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Optional field selector or projection expression.
    #[serde(default)]
    pub selector: String,
    /// Optional consumer-owned contract label.
    #[serde(default)]
    pub contract: String,
    /// Native output format: json, toml, yaml, or raw.
    pub output_format: String,
    /// Generated artifact output path.
    pub output_path: String,
    /// True when this export is allowed to carry raw secret material.
    #[serde(default)]
    pub allow_secret_material: bool,
}

/// Text associated with a path for manifest identity checks.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NamedText<'a> {
    /// Source-root-relative path or artifact locator.
    pub path: &'a str,
    /// Exact UTF-8 text bytes represented as a string.
    pub text: &'a str,
}

/// Pure input used after the shell has produced export bytes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NickelExportInput<'a> {
    /// Export specification.
    pub spec: &'a NickelExportSpec,
    /// Source file text.
    pub source_text: &'a str,
    /// Dependency file texts.
    pub dependencies: Vec<NamedText<'a>>,
    /// Generated output artifact text.
    pub output_text: &'a str,
    /// Nickel binary or derivation identity.
    pub nickel_identity: &'a str,
    /// Contract diagnostics captured by the shell before accepting output.
    pub contract_diagnostics: Vec<String>,
}

/// BLAKE3 identity in Octet's legacy manifest shape.
pub type ExportIdentity = canonical::OctetIdentity;
/// One accepted export in Octet's legacy manifest shape.
pub type NickelExportManifestEntry = canonical::OctetManifestEntry;
/// Deterministic Octet compatibility manifest.
pub type NickelExportManifest = canonical::OctetManifest;

/// Return true when an output format is accepted by the standalone core.
#[must_use]
pub fn is_supported_output_format(format: &str) -> bool {
    canonical_format(format).is_some()
}

/// Validate an Octet export spec before the shell evaluates Nickel.
#[must_use]
pub fn validate_export_spec(spec: &NickelExportSpec) -> Vec<String> {
    let mut diagnostics = Vec::new();
    push_if_blank(&mut diagnostics, &spec.family_id, "family_id");
    push_if_blank(&mut diagnostics, &spec.source, "source");
    push_if_blank(&mut diagnostics, &spec.output_path, "output_path");
    if spec.schema_version != NICKEL_EXPORT_SPEC_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Nickel export spec schema `{}`; expected `{NICKEL_EXPORT_SPEC_SCHEMA_VERSION}`",
            spec.schema_version
        ));
    }
    if !is_supported_output_format(&spec.output_format) {
        diagnostics.push(format!(
            "unsupported Nickel export format `{}`; expected one of {}",
            spec.output_format,
            SUPPORTED_FORMATS.join(", ")
        ));
    }
    diagnostics
}

/// Validate generated output through the canonical standalone admission core.
#[must_use]
pub fn validate_export_acceptance(input: &NickelExportInput<'_>) -> Vec<String> {
    let mut diagnostics = validate_export_spec(input.spec);
    push_evaluator_identity_diagnostic(&mut diagnostics, input.nickel_identity);
    if !diagnostics.is_empty() {
        diagnostics.extend(legacy_contract_diagnostics(input));
        return diagnostics;
    }

    match build_canonical_receipt(input) {
        Ok(_) => diagnostics,
        Err(canonical::CoreError::SecretMaterial(path)) => {
            diagnostics.push(format!(
                "Nickel export source `{path}` appears to contain raw secret material; {SECRET_REMEDIATION}"
            ));
            diagnostics
        }
        Err(canonical::CoreError::EvaluationFailed(_)) => {
            diagnostics.extend(legacy_contract_diagnostics(input));
            diagnostics
        }
        Err(error) => {
            diagnostics.push(format!("standalone Nickel export admission failed: {error}"));
            diagnostics
        }
    }
}

/// Build an admitted Octet v1 manifest through the standalone core and
/// compatibility projector.
///
/// # Errors
///
/// Returns deterministic diagnostics when any observation fails canonical
/// admission or the canonical manifest rejects empty, duplicate, or
/// mixed-evaluator inputs.
pub fn manifest_for_exports(inputs: &[NickelExportInput<'_>]) -> Result<NickelExportManifest, Vec<String>> {
    let mut evaluator_diagnostics = Vec::new();
    for input in inputs {
        push_evaluator_identity_diagnostic(&mut evaluator_diagnostics, input.nickel_identity);
    }
    if !evaluator_diagnostics.is_empty() {
        return Err(evaluator_diagnostics);
    }
    let receipts = inputs
        .iter()
        .map(build_canonical_receipt)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|error| canonical_manifest_diagnostics(&error))?;
    let canonical_manifest =
        canonical::build_manifest(&receipts).map_err(|error| canonical_manifest_diagnostics(&error))?;
    Ok(canonical::project_octet_manifest(&canonical_manifest))
}

/// Verify that an existing Octet v1 manifest matches current exact inputs.
#[must_use]
pub fn verify_manifest(manifest: &NickelExportManifest, inputs: &[NickelExportInput<'_>]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if manifest.schema_version != NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Nickel export manifest schema `{}`; expected `{NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION}`",
            manifest.schema_version
        ));
    }
    let expected = match manifest_for_exports(inputs) {
        Ok(expected) => expected,
        Err(mut manifest_diagnostics) => {
            diagnostics.append(&mut manifest_diagnostics);
            return diagnostics;
        }
    };
    for expected_entry in &expected.exports {
        match manifest.exports.iter().find(|entry| entry.output.path == expected_entry.output.path) {
            Some(actual) => compare_manifest_entry(expected_entry, actual, &mut diagnostics),
            None => diagnostics.push(format!("Nickel export manifest missing output `{}`", expected_entry.output.path)),
        }
    }
    for actual in &manifest.exports {
        if !expected.exports.iter().any(|entry| entry.output.path == actual.output.path) {
            diagnostics.push(format!("Nickel export manifest contains stale output `{}`", actual.output.path));
        }
    }
    diagnostics
}

/// Build the shell-friendly legacy generation command represented by a spec.
#[must_use]
pub fn generation_command(spec: &NickelExportSpec) -> String {
    export_command_argv(spec).join(" ")
}

/// Build the shell-free legacy command shape represented by a spec.
#[must_use]
pub fn export_command_argv(spec: &NickelExportSpec) -> Vec<String> {
    let mut argv = vec![
        "nickel".to_owned(),
        "export".to_owned(),
        "--format".to_owned(),
        spec.output_format.clone(),
        spec.source.clone(),
    ];
    if !spec.selector.trim().is_empty() {
        argv.push("--field".to_owned());
        argv.push(spec.selector.clone());
    }
    argv
}

fn build_canonical_receipt(input: &NickelExportInput<'_>) -> Result<canonical::ExportReceipt, canonical::CoreError> {
    let request = canonical_request(input.spec).ok_or_else(|| {
        canonical::CoreError::InvalidRequest(vec![canonical::Diagnostic::new(
            "unsupported-format",
            &input.spec.output_format,
            "legacy format could not be projected",
            canonical::DiagnosticSeverity::Error,
        )])
    })?;
    let evaluator = evaluator_for_input(input);
    let dependencies = input
        .dependencies
        .iter()
        .map(|dependency| canonical::ArtifactMaterial {
            path: dependency.path,
            bytes: dependency.text.as_bytes(),
        })
        .collect::<Vec<_>>();
    let observed_dependencies = input.dependencies.iter().map(|dependency| dependency.path).collect::<Vec<_>>();
    let contract_diagnostics = input
        .contract_diagnostics
        .iter()
        .map(|message| {
            canonical::Diagnostic::new(
                "octet-contract",
                contract_label(input.spec),
                message,
                canonical::DiagnosticSeverity::Error,
            )
        })
        .collect::<Vec<_>>();
    canonical::build_receipt(&canonical::EvaluationObservation {
        request: &request,
        source: canonical::ArtifactMaterial {
            path: &input.spec.source,
            bytes: input.source_text.as_bytes(),
        },
        dependencies,
        output: canonical::ArtifactMaterial {
            path: &input.spec.output_path,
            bytes: input.output_text.as_bytes(),
        },
        evaluator: &evaluator,
        observed_dependencies,
        diagnostics: contract_diagnostics,
    })
}

fn canonical_request(spec: &NickelExportSpec) -> Option<canonical::ExportRequest> {
    Some(canonical::ExportRequest {
        schema: canonical::REQUEST_SCHEMA.to_owned(),
        family_id: spec.family_id.clone(),
        source: spec.source.clone(),
        dependencies: spec.dependencies.clone(),
        import_paths: Vec::new(),
        selector: spec.selector.clone(),
        contract: spec.contract.clone(),
        format: canonical_format(&spec.output_format)?,
        destination: spec.output_path.clone(),
        allow_secret_material: spec.allow_secret_material,
    })
}

fn canonical_format(format: &str) -> Option<canonical::ExportFormat> {
    match format {
        FORMAT_JSON => Some(canonical::ExportFormat::Json),
        FORMAT_TOML => Some(canonical::ExportFormat::Toml),
        FORMAT_YAML => Some(canonical::ExportFormat::Yaml),
        FORMAT_RAW => Some(canonical::ExportFormat::Raw),
        _ => None,
    }
}

fn evaluator_for_input(input: &NickelExportInput<'_>) -> canonical::EvaluatorDescriptor {
    canonical::EvaluatorDescriptor {
        identity: input.nickel_identity.to_owned(),
        version: OCTET_LEGACY_EVALUATOR_VERSION.to_owned(),
        options: Vec::new(),
        import_path_policy: canonical::ImportPathPolicy::EvaluatorObservedClosure,
    }
}

fn canonical_manifest_diagnostics(error: &canonical::CoreError) -> Vec<String> {
    vec![format!("standalone Nickel export manifest failed: {error}")]
}

fn legacy_contract_diagnostics(input: &NickelExportInput<'_>) -> Vec<String> {
    input
        .contract_diagnostics
        .iter()
        .map(|diagnostic| {
            format!("Nickel contract `{}` failed for `{}`: {diagnostic}", contract_label(input.spec), input.spec.source)
        })
        .collect()
}

fn compare_manifest_entry(
    expected: &NickelExportManifestEntry,
    actual: &NickelExportManifestEntry,
    diagnostics: &mut Vec<String>,
) {
    push_identity_mismatch("source", &expected.source, &actual.source, diagnostics);
    compare_dependencies(expected, actual, diagnostics);
    push_identity_mismatch("output", &expected.output, &actual.output, diagnostics);
    if expected.output_format != actual.output_format {
        diagnostics.push(format!(
            "stale Nickel export metadata `{}`; expected format `{}`, got `{}`",
            expected.output.path, expected.output_format, actual.output_format
        ));
    }
    if expected.generation_command != actual.generation_command {
        diagnostics.push(format!(
            "stale Nickel export metadata `{}`; expected generation command `{}`, got `{}`",
            expected.output.path, expected.generation_command, actual.generation_command
        ));
    }
}

fn compare_dependencies(
    expected: &NickelExportManifestEntry,
    actual: &NickelExportManifestEntry,
    diagnostics: &mut Vec<String>,
) {
    for expected_dependency in &expected.dependencies {
        match actual.dependencies.iter().find(|dependency| dependency.path == expected_dependency.path) {
            Some(actual_dependency) => {
                push_identity_mismatch("dependency", expected_dependency, actual_dependency, diagnostics);
            }
            None => diagnostics.push(format!(
                "Nickel export manifest missing dependency `{}` for output `{}`",
                expected_dependency.path, expected.output.path
            )),
        }
    }
    for actual_dependency in &actual.dependencies {
        if !expected.dependencies.iter().any(|dependency| dependency.path == actual_dependency.path) {
            diagnostics.push(format!(
                "Nickel export manifest contains stale dependency `{}` for output `{}`",
                actual_dependency.path, expected.output.path
            ));
        }
    }
}

fn push_identity_mismatch(
    label: &str,
    expected: &ExportIdentity,
    actual: &ExportIdentity,
    diagnostics: &mut Vec<String>,
) {
    if expected.blake3 == actual.blake3 && expected.bytes == actual.bytes {
        return;
    }
    diagnostics.push(format!(
        "stale Nickel export {label} `{}`; expected blake3 {} bytes {}, got blake3 {} bytes {}",
        expected.path, expected.blake3, expected.bytes, actual.blake3, actual.bytes
    ));
}

fn push_if_blank(diagnostics: &mut Vec<String>, value: &str, field: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("Nickel export spec field `{field}` must not be empty"));
    }
}

fn push_evaluator_identity_diagnostic(diagnostics: &mut Vec<String>, identity: &str) {
    if identity != REVIEWED_NICKEL_EVALUATOR_IDENTITY {
        diagnostics
            .push(format!("unsupported Nickel evaluator identity; expected `{REVIEWED_NICKEL_EVALUATOR_IDENTITY}`"));
    }
}

fn contract_label(spec: &NickelExportSpec) -> &str {
    if spec.contract.trim().is_empty() {
        "<unspecified>"
    } else {
        spec.contract.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAMILY_ID: &str = "octet.dynamic-evidence-profile-config";
    const SOURCE_PATH: &str = "templates/dynamic-evidence-profiles.ncl";
    const OUTPUT_PATH: &str = "dynamic-evidence-profiles.json";
    const SOURCE_TEXT: &str = "{ schema_version = \"octet-dynamic-evidence-profiles/v1\" }\n";
    const OUTPUT_TEXT: &str = "{\"schema_version\":\"octet-dynamic-evidence-profiles/v1\"}\n";
    const NICKEL_IDENTITY: &str = REVIEWED_NICKEL_EVALUATOR_IDENTITY;

    #[test]
    fn stable_manifest_records_json_toml_yaml_and_raw_exports() {
        let specs = supported_format_specs();
        let inputs = specs.iter().map(|spec| export_input(spec, OUTPUT_TEXT)).collect::<Vec<_>>();

        let first = manifest_for_exports(&inputs).expect("supported observations should produce a manifest");
        let second = manifest_for_exports(&inputs).expect("repeated observations should produce a manifest");

        assert_eq!(first, second);
        assert_eq!(first.schema_version, NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION);
        assert_eq!(first.generator, canonical::OCTET_GENERATOR_ID);
        assert_eq!(first.nickel_identity, NICKEL_IDENTITY);
        assert_eq!(first.exports.len(), SUPPORTED_FORMATS.len());
        assert!(first.exports.iter().any(|entry| entry.output_format == FORMAT_JSON));
        assert!(first.exports.iter().any(|entry| entry.output_format == FORMAT_TOML));
        assert!(first.exports.iter().any(|entry| entry.output_format == FORMAT_YAML));
        assert!(first.exports.iter().any(|entry| entry.output_format == FORMAT_RAW));
    }

    #[test]
    fn export_acceptance_rejects_missing_source_contract_failure_and_unsupported_format() {
        let mut spec = json_spec();
        spec.source.clear();
        spec.output_format = "xml".to_owned();
        let input = NickelExportInput {
            spec: &spec,
            source_text: SOURCE_TEXT,
            dependencies: Vec::new(),
            output_text: OUTPUT_TEXT,
            nickel_identity: NICKEL_IDENTITY,
            contract_diagnostics: vec!["missing field `schema_version`".to_owned()],
        };

        let diagnostics = validate_export_acceptance(&input);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("field `source`")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported Nickel export format")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing field `schema_version`")));
    }

    #[test]
    fn standalone_admission_accepts_consumer_owned_contract_labels() {
        let spec = json_spec();
        let input = export_input(&spec, OUTPUT_TEXT);

        let diagnostics = validate_export_acceptance(&input);

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        let manifest = manifest_for_exports(&[input]).expect("consumer-owned contract label should remain admissible");
        assert_eq!(manifest.exports[0].contract, "DynamicEvidenceProfiles");
    }

    #[test]
    // r[verify octet.nickel_toolchain.boundary]
    // r[verify octet.nickel_toolchain.evidence]
    fn evaluator_identity_cannot_replace_policy_or_be_substituted() {
        let valid_spec = json_spec();
        let valid_input = export_input(&valid_spec, OUTPUT_TEXT);
        assert!(validate_export_acceptance(&valid_input).is_empty());

        let mut unsupported_spec = json_spec();
        unsupported_spec.output_format = "unsupported".to_owned();
        let unsupported_input = export_input(&unsupported_spec, OUTPUT_TEXT);
        let policy_diagnostics = validate_export_acceptance(&unsupported_input);
        assert!(policy_diagnostics.iter().any(|diagnostic| diagnostic.contains("unsupported Nickel export format")));

        let mut substituted_input = export_input(&valid_spec, OUTPUT_TEXT);
        substituted_input.nickel_identity = "nickel-substituted";
        let identity_diagnostics = validate_export_acceptance(&substituted_input);
        assert!(
            identity_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unsupported Nickel evaluator identity"))
        );
        let manifest_diagnostics = manifest_for_exports(&[substituted_input])
            .expect_err("substituted evaluator identity must not produce evidence");
        assert!(
            manifest_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unsupported Nickel evaluator identity"))
        );
    }

    #[test]
    fn standalone_admission_rejects_dependency_material_drift() {
        let spec = json_spec();
        let input = NickelExportInput {
            spec: &spec,
            source_text: SOURCE_TEXT,
            dependencies: Vec::new(),
            output_text: OUTPUT_TEXT,
            nickel_identity: NICKEL_IDENTITY,
            contract_diagnostics: Vec::new(),
        };

        let diagnostics = validate_export_acceptance(&input);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("standalone Nickel export admission failed"))
        );
    }

    #[test]
    fn manifest_construction_rejects_empty_error_and_mixed_evaluator_observations() {
        let empty_diagnostics = manifest_for_exports(&[]).expect_err("empty manifests must fail closed");
        assert!(empty_diagnostics.iter().any(|diagnostic| diagnostic.contains("invalid export request")));

        let spec = json_spec();
        let mut error_input = export_input(&spec, OUTPUT_TEXT);
        error_input.contract_diagnostics = vec!["contract rejected output".to_owned()];
        let error_diagnostics =
            manifest_for_exports(&[error_input]).expect_err("contract errors must not produce a success manifest");
        assert!(
            error_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("contract diagnostics contain an error"))
        );

        let first = export_input(&spec, OUTPUT_TEXT);
        let mut second_spec = spec.clone();
        second_spec.output_path = "generated/other.json".to_owned();
        let mut second = export_input(&second_spec, OUTPUT_TEXT);
        second.nickel_identity = "nickel-other";
        let mixed_diagnostics =
            manifest_for_exports(&[first, second]).expect_err("mixed evaluators must not produce a manifest");
        assert!(
            mixed_diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("unsupported Nickel evaluator identity"))
        );
    }

    #[test]
    fn manifest_verification_rejects_stale_source_dependency_output_bytes_and_extra_entries() {
        let spec = json_spec();
        let input = export_input(&spec, OUTPUT_TEXT);
        let fresh = manifest_for_exports(&[input]).expect("valid observation should produce a manifest");
        let mut stale_spec = spec.clone();
        stale_spec.dependencies = vec!["templates/shared.ncl".to_owned()];
        let stale_input = NickelExportInput {
            spec: &stale_spec,
            source_text: "{ schema_version = \"changed\" }\n",
            dependencies: vec![NamedText {
                path: "templates/shared.ncl",
                text: "{ inherited = true }\n",
            }],
            output_text: "{\"schema_version\":\"changed\"}\n",
            nickel_identity: NICKEL_IDENTITY,
            contract_diagnostics: Vec::new(),
        };
        let mut stale = fresh.clone();
        stale.exports.push(NickelExportManifestEntry {
            output: ExportIdentity {
                path: "stale.json".to_owned(),
                blake3: bare_blake3("stale"),
                bytes: u64::try_from("stale".len()).unwrap_or(u64::MAX),
            },
            ..fresh.exports[0].clone()
        });

        let diagnostics = verify_manifest(&stale, &[stale_input]);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("stale Nickel export source")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("missing dependency")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("stale Nickel export output")));
        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("contains stale output")));
    }

    #[test]
    fn secret_bearing_source_is_rejected_before_manifest_acceptance() {
        let spec = json_spec();
        let input = NickelExportInput {
            spec: &spec,
            source_text: "api_key = \"do-not-commit\"\n",
            dependencies: vec![NamedText {
                path: "standards/policies/contracts.ncl",
                text: "{ Contract = String }\n",
            }],
            output_text: OUTPUT_TEXT,
            nickel_identity: NICKEL_IDENTITY,
            contract_diagnostics: Vec::new(),
        };

        let diagnostics = validate_export_acceptance(&input);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("raw secret material")));
    }

    fn supported_format_specs() -> Vec<NickelExportSpec> {
        SUPPORTED_FORMATS
            .iter()
            .map(|format| {
                let mut spec = json_spec();
                spec.output_format = (*format).to_owned();
                spec.output_path = format!("generated/export.{format}");
                spec
            })
            .collect()
    }

    fn json_spec() -> NickelExportSpec {
        NickelExportSpec {
            schema_version: NICKEL_EXPORT_SPEC_SCHEMA_VERSION.to_owned(),
            family_id: FAMILY_ID.to_owned(),
            source: SOURCE_PATH.to_owned(),
            dependencies: vec!["standards/policies/contracts.ncl".to_owned()],
            selector: "profiles".to_owned(),
            contract: "DynamicEvidenceProfiles".to_owned(),
            output_format: FORMAT_JSON.to_owned(),
            output_path: OUTPUT_PATH.to_owned(),
            allow_secret_material: false,
        }
    }

    fn export_input<'a>(spec: &'a NickelExportSpec, output_text: &'a str) -> NickelExportInput<'a> {
        NickelExportInput {
            spec,
            source_text: SOURCE_TEXT,
            dependencies: vec![NamedText {
                path: "standards/policies/contracts.ncl",
                text: "{ Contract = String }\n",
            }],
            output_text,
            nickel_identity: NICKEL_IDENTITY,
            contract_diagnostics: Vec::new(),
        }
    }

    fn bare_blake3(text: &str) -> String {
        canonical::blake3_identity(text.as_bytes()).trim_start_matches("b3:").to_owned()
    }
}

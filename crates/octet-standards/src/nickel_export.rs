//! Deterministic Nickel export helper core.
//!
//! The shell can evaluate Nickel, but this module owns the pure export-plan,
//! manifest, freshness, and boundary checks used to decide whether the exported
//! bytes may be accepted as generated Octet evidence.

use serde::Deserialize;
use serde::Serialize;

use crate::proof_artifact::artifact_text_identity;

/// Export manifest schema version.
pub const NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION: &str = "octet-nickel-export-manifest/v1";
/// Export spec schema version.
pub const NICKEL_EXPORT_SPEC_SCHEMA_VERSION: &str = "octet-nickel-export-spec/v1";
/// Generator identity recorded in helper manifests.
pub const NICKEL_EXPORT_GENERATOR_ID: &str = "octet-standards.nickel-export-helper/v1";

const FORMAT_JSON: &str = "json";
const FORMAT_TOML: &str = "toml";
const FORMAT_YAML: &str = "yaml";
const FORMAT_RAW: &str = "raw";
const NICKEL_BINARY_PLACEHOLDER: &str = "nickel:unspecified";
const SECRET_REMEDIATION: &str = "register this family as non-candidate or replace raw secret material with references";

static SUPPORTED_FORMATS: &[&str] = &[FORMAT_JSON, FORMAT_TOML, FORMAT_YAML, FORMAT_RAW];
static SECRET_MARKERS: &[&str] = &["BEGIN PRIVATE KEY", "password =", "token =", "secret =", "api_key ="];

/// A single deterministic Nickel export request.
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
    /// Optional contract applied before export acceptance.
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

/// BLAKE3 identity for one source, dependency, or output artifact.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ExportIdentity {
    /// Path or artifact locator.
    pub path: String,
    /// BLAKE3 digest over exact bytes.
    pub blake3: String,
    /// Exact byte length.
    pub bytes: u64,
}

/// One accepted export entry inside a helper manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NickelExportManifestEntry {
    /// Config or receipt family ID.
    pub family_id: String,
    /// Native output format.
    pub output_format: String,
    /// Source Nickel file identity.
    pub source: ExportIdentity,
    /// Declared dependency identities.
    pub dependencies: Vec<ExportIdentity>,
    /// Generated output identity.
    pub output: ExportIdentity,
    /// Optional field selector or projection.
    pub selector: String,
    /// Optional contract applied before export acceptance.
    pub contract: String,
    /// Shell-friendly generation command used for replay notes.
    pub generation_command: String,
    /// Shell-free command shape used for replay.
    pub command_argv: Vec<String>,
}

/// Deterministic manifest for one or more helper-produced exports.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct NickelExportManifest {
    /// Manifest schema version.
    pub schema_version: String,
    /// Generator identity.
    pub generator: String,
    /// Nickel binary or derivation identity when known.
    pub nickel_identity: String,
    /// Export entries sorted by output path.
    pub exports: Vec<NickelExportManifestEntry>,
}

/// Return true when an output format is accepted by the helper.
#[must_use]
pub fn is_supported_output_format(format: &str) -> bool {
    SUPPORTED_FORMATS.contains(&format)
}

/// Validate an export spec before the shell evaluates Nickel.
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

/// Validate generated output acceptance after Nickel evaluation.
#[must_use]
pub fn validate_export_acceptance(input: &NickelExportInput<'_>) -> Vec<String> {
    let mut diagnostics = validate_export_spec(input.spec);
    diagnostics.extend(secret_diagnostics(input));
    diagnostics.extend(input.contract_diagnostics.iter().map(|diagnostic| {
        format!("Nickel contract `{}` failed for `{}`: {diagnostic}", contract_label(input.spec), input.spec.source)
    }));
    diagnostics
}

/// Build a deterministic manifest for accepted export outputs.
#[must_use]
pub fn manifest_for_exports(inputs: &[NickelExportInput<'_>]) -> NickelExportManifest {
    let nickel_identity = inputs.first().map_or(NICKEL_BINARY_PLACEHOLDER, |input| input.nickel_identity).to_owned();
    let mut exports = inputs.iter().map(manifest_entry).collect::<Vec<_>>();
    exports.sort_by(|left, right| left.output.path.cmp(&right.output.path));
    NickelExportManifest {
        schema_version: NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION.to_owned(),
        generator: NICKEL_EXPORT_GENERATOR_ID.to_owned(),
        nickel_identity,
        exports,
    }
}

/// Verify that an existing manifest matches exact current export bytes.
#[must_use]
pub fn verify_manifest(manifest: &NickelExportManifest, inputs: &[NickelExportInput<'_>]) -> Vec<String> {
    let mut diagnostics = Vec::new();
    if manifest.schema_version != NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION {
        diagnostics.push(format!(
            "unsupported Nickel export manifest schema `{}`; expected `{NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION}`",
            manifest.schema_version
        ));
    }
    let expected = manifest_for_exports(inputs);
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
                actual_dependency.path, actual.output.path
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

/// Build the shell-friendly Nickel generation command represented by a spec.
#[must_use]
pub fn generation_command(spec: &NickelExportSpec) -> String {
    export_command_argv(spec).join(" ")
}

/// Build the shell-free Nickel command shape represented by a spec.
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

fn manifest_entry(input: &NickelExportInput<'_>) -> NickelExportManifestEntry {
    let mut dependencies = input
        .dependencies
        .iter()
        .map(|dependency| text_identity(dependency.path, dependency.text))
        .collect::<Vec<_>>();
    dependencies.sort_by(|left, right| left.path.cmp(&right.path));
    NickelExportManifestEntry {
        family_id: input.spec.family_id.clone(),
        output_format: input.spec.output_format.clone(),
        source: text_identity(&input.spec.source, input.source_text),
        dependencies,
        output: text_identity(&input.spec.output_path, input.output_text),
        selector: input.spec.selector.clone(),
        contract: input.spec.contract.clone(),
        generation_command: generation_command(input.spec),
        command_argv: export_command_argv(input.spec),
    }
}

fn text_identity(path: &str, text: &str) -> ExportIdentity {
    let identity = artifact_text_identity(path, text);
    ExportIdentity {
        path: identity.path,
        blake3: identity.blake3,
        bytes: identity.bytes,
    }
}

fn push_if_blank(diagnostics: &mut Vec<String>, value: &str, field: &str) {
    if value.trim().is_empty() {
        diagnostics.push(format!("Nickel export spec field `{field}` must not be empty"));
    }
}

fn contract_label(spec: &NickelExportSpec) -> &str {
    if spec.contract.trim().is_empty() {
        "<unspecified>"
    } else {
        spec.contract.as_str()
    }
}

fn secret_diagnostics(input: &NickelExportInput<'_>) -> Vec<String> {
    if input.spec.allow_secret_material {
        return Vec::new();
    }
    let mut diagnostics = Vec::new();
    push_secret_diagnostic(&mut diagnostics, &input.spec.source, input.source_text);
    for dependency in &input.dependencies {
        push_secret_diagnostic(&mut diagnostics, dependency.path, dependency.text);
    }
    diagnostics
}

fn push_secret_diagnostic(diagnostics: &mut Vec<String>, path: &str, text: &str) {
    let lower_text = text.to_ascii_lowercase();
    for marker in SECRET_MARKERS {
        if lower_text.contains(&marker.to_ascii_lowercase()) {
            diagnostics.push(format!(
                "Nickel export source `{path}` appears to contain raw secret material marker `{marker}`; {SECRET_REMEDIATION}"
            ));
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_artifact::blake3_hex_text;

    const FAMILY_ID: &str = "octet.dynamic-evidence-profile-config";
    const SOURCE_PATH: &str = "templates/dynamic-evidence-profiles.ncl";
    const OUTPUT_PATH: &str = "dynamic-evidence-profiles.json";
    const SOURCE_TEXT: &str = "{ schema_version = \"octet-dynamic-evidence-profiles/v1\" }\n";
    const OUTPUT_TEXT: &str = "{\"schema_version\":\"octet-dynamic-evidence-profiles/v1\"}\n";
    const NICKEL_IDENTITY: &str = "nickel-1.11.0";

    #[test]
    fn stable_manifest_records_json_toml_yaml_and_raw_exports() {
        let specs = supported_format_specs();
        let inputs = specs.iter().map(|spec| export_input(spec, OUTPUT_TEXT)).collect::<Vec<_>>();

        let first = manifest_for_exports(&inputs);
        let second = manifest_for_exports(&inputs);

        assert_eq!(first, second);
        assert_eq!(first.schema_version, NICKEL_EXPORT_MANIFEST_SCHEMA_VERSION);
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
    fn manifest_verification_rejects_stale_source_dependency_output_bytes_and_extra_entries() {
        let spec = json_spec();
        let input = export_input(&spec, OUTPUT_TEXT);
        let fresh = manifest_for_exports(&[input]);
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
                blake3: blake3_hex_text("stale"),
                bytes: u64::try_from("stale".len()).expect("fixture length fits u64"),
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
            dependencies: Vec::new(),
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
}

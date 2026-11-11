//! Pure observations over measured Aeneas provider artifacts.
//!
//! The imperative shell reads the produced LLBC, `translation.json`, generated Lean, and the
//! maintained theorem source; these functions turn those bytes into the typed facts that the
//! receipt records and the validators compare against the reviewed provider profile.

use std::collections::BTreeSet;

use serde::Deserialize;
use serde_json::Value;

use crate::formal_proof::AENEAS_TRANSLATION_MANIFEST_SCHEMA_VERSION;
use crate::formal_proof::AeneasAssumptionPolicy;
use crate::formal_proof::AeneasGeneratedLeanFile;
use crate::formal_proof::AeneasLlbcDeclaration;
use crate::formal_proof::AeneasSourceSpan;
use crate::formal_proof::AeneasTranslationDeclaration;
use crate::formal_proof::AeneasTranslationManifest;
use crate::formal_proof::AeneasTrustClosure;
use crate::formal_proof::CharonExtractionPolicy;
use crate::formal_proof::validate_aeneas_translation_manifest;

/// Reviewed-vocabulary MIR stage for Charon's default (`mir = null`) borrow-checked MIR.
const MIR_STAGE_DEFAULT: &str = "borrowck";
/// Reviewed-vocabulary drop policy when neither precise nor desugared drops are requested.
const DROP_POLICY_DEFAULT: &str = "include-drops";
/// Reviewed-vocabulary `start_from` when no item selection narrows the translated crate.
const START_FROM_CRATE: &str = "crate";
/// Reviewed-vocabulary transformation setting observed when every declaration keeps its span.
const TRANSFORMATION_PRESERVE_SPANS: &str = "preserve-spans";
/// Reviewed-vocabulary serialization of JSON-encoded structured LLBC.
const SERIALIZATION_LLBC_JSON: &str = "llbc-json-v1";
const JSON_FORMAT: &str = "json";
const BITS_PER_BYTE: u64 = 8;
const DECLARATION_COLLECTIONS: [&str; 5] = ["fun_decls", "type_decls", "global_decls", "trait_decls", "trait_impls"];
const THEOREM_ORIGIN: &str = "theorem";
const GENERATED_ORIGIN: &str = "generated";
/// Prefix that resolves a Lean name from the root namespace (`_root_.sorryAx` is `sorryAx`).
const ROOT_NAMESPACE_PREFIX: &str = "_root_.";
/// Lean 4.31 syntax whose next string literal is parsed as `interpolatedStr`: the `s!`, `m!`, and
/// `f!` macros and the core macros and commands that take the string right after their keyword.
const INTERPOLATING_KEYWORDS: [&str; 9] = [
    "s!",
    "m!",
    "f!",
    "println!",
    "dbg_trace",
    "throwError",
    "reportIssue!",
    "reportDbgIssue!",
    "reportEMatchIssue!",
];
/// Lean 4.31 commands that take one term or name (`throwErrorAt ref`) before their interpolated
/// string.
const INTERPOLATING_AFTER_ONE_ARGUMENT: [&str; 4] =
    ["throwErrorAt", "throwNamedError", "logNamedError", "logNamedWarning"];
/// Lean 4.31 commands that take a term and a name before their interpolated string.
const INTERPOLATING_AFTER_TWO_ARGUMENTS: [&str; 3] = ["throwNamedErrorAt", "logNamedErrorAt", "logNamedWarningAt"];
/// Lean 4.31 `name[cls]` macros whose string after the closing `]` is interpolated.
const INTERPOLATING_TRACE_MACROS: [&str; 3] = ["trace", "trace_goal", "Macro.trace"];
/// Origin label of trust inputs that Lean reports for the approved theorem.
const LEAN_ORIGIN: &str = "lean";
/// Axioms of Lean's core logic that every classical proof may depend on.
const LEAN_CORE_AXIOMS: [&str; 3] = ["propext", "Classical.choice", "Quot.sound"];
/// The axiom `sorry`, `admit`, and the `stop` tactic elaborate to.
const LEAN_ADMITTED_AXIOM: &str = "sorryAx";
/// Axioms that make the kernel trust compiled code.
const LEAN_NATIVE_AXIOMS: [&str; 3] = ["Lean.ofReduceBool", "Lean.ofReduceNat", "Lean.trustCompiler"];
/// Name component of the auxiliary axioms `native_decide` declares
/// (`thm._native.native_decide.ax_1_1`).
const LEAN_NATIVE_NAME_COMPONENT: &str = "_native";

/// Schema version of the axiom report Octet's Lean check program prints.
pub const AENEAS_LEAN_AXIOM_REPORT_SCHEMA_VERSION: &str = "octet-aeneas-lean-axioms/v1";

/// Observe the effective Charon extraction policy from the produced LLBC bytes.
///
/// Charon's serialized options are mapped into the reviewed provider-profile vocabulary:
/// `mir = null` is the default borrow-checked MIR (`borrowck`); no precise or desugared drops is
/// `include-drops`; empty `start_from*` selections without `start_from_pub` is `crate`; every
/// declaration carrying a source span is `preserve-spans`; and JSON-encoded structured LLBC is
/// `llbc-json-v1`. Any other value is recorded in a form that cannot equal the reviewed
/// vocabulary, so the contract comparison fails closed.
///
/// # Errors
///
/// Returns diagnostics when the LLBC is not JSON or omits a field the mapping requires.
// r[impl octet.aeneas_lean_proof_rail.charon_contract]
pub fn observe_charon_extraction_policy(llbc_text: &str) -> Result<CharonExtractionPolicy, Vec<String>> {
    let value =
        serde_json::from_str::<Value>(llbc_text).map_err(|error| vec![format!("parse Charon LLBC: {error}")])?;
    let mut diagnostics = Vec::new();
    let has_errors = required_bool(&value, "/has_errors", &mut diagnostics);
    let translated = &value["translated"];
    let options = &translated["options"];
    if !options.is_object() {
        diagnostics.push("Charon LLBC is missing translated.options".to_owned());
        return Err(diagnostics);
    }
    let skip_borrowck = required_bool(options, "/skip_borrowck", &mut diagnostics);
    let (target_triple, pointer_width_bits, endianness) = observed_target(translated, &mut diagnostics);
    let mir_start_stage = match options.get("mir") {
        Some(Value::Null) => MIR_STAGE_DEFAULT.to_owned(),
        Some(Value::String(stage)) => format!("mir:{stage}"),
        _ => {
            diagnostics.push("Charon LLBC options.mir is missing or malformed".to_owned());
            String::new()
        }
    };
    let precise_drops = required_bool(options, "/precise_drops", &mut diagnostics);
    let desugar_drops = required_bool(options, "/desugar_drops", &mut diagnostics);
    let drop_policy = match (precise_drops, desugar_drops) {
        (false, false) => DROP_POLICY_DEFAULT.to_owned(),
        (precise, desugar) => format!("precise-drops={precise},desugar-drops={desugar}"),
    };
    let start_from = observed_start_from(options, &mut diagnostics);
    let include_rules = required_strings(options, "include", &mut diagnostics);
    let exclude_rules = required_strings(options, "exclude", &mut diagnostics);
    let opaque_rules = required_strings(options, "opaque", &mut diagnostics);
    let mut transformation_settings = Vec::new();
    if declarations_preserve_spans(translated, &mut diagnostics) {
        transformation_settings.push(TRANSFORMATION_PRESERVE_SPANS.to_owned());
    }
    let serialization_format = observed_serialization(options, &mut diagnostics);
    if diagnostics.is_empty() {
        Ok(CharonExtractionPolicy {
            has_errors,
            skip_borrowck,
            target_triple,
            pointer_width_bits,
            endianness,
            mir_start_stage,
            drop_policy,
            start_from,
            include_rules,
            exclude_rules,
            opaque_rules,
            transformation_settings,
            serialization_format,
        })
    } else {
        diagnostics.sort();
        diagnostics.dedup();
        Err(diagnostics)
    }
}

fn required_bool(value: &Value, pointer: &str, diagnostics: &mut Vec<String>) -> bool {
    if let Some(flag) = value.pointer(pointer).and_then(Value::as_bool) {
        flag
    } else {
        diagnostics.push(format!("Charon LLBC field `{pointer}` is missing or not a boolean"));
        // Fail toward the weakened value so a missing flag can never look reviewed.
        true
    }
}

fn required_strings(options: &Value, field: &str, diagnostics: &mut Vec<String>) -> Vec<String> {
    let Some(values) = options.get(field).and_then(Value::as_array) else {
        diagnostics.push(format!("Charon LLBC options.{field} is missing or not an array"));
        return Vec::new();
    };
    values
        .iter()
        .map(|value| {
            value.as_str().map_or_else(
                || {
                    diagnostics.push(format!("Charon LLBC options.{field} contains a non-string rule"));
                    String::new()
                },
                str::to_owned,
            )
        })
        .collect()
}

fn observed_target(translated: &Value, diagnostics: &mut Vec<String>) -> (String, u16, String) {
    let rows = translated.get("target_information").and_then(Value::as_array).map_or(&[][..], Vec::as_slice);
    let [row] = rows else {
        diagnostics.push(format!("Charon LLBC must record exactly one target, found {}", rows.len()));
        return (String::new(), 0, String::new());
    };
    let triple = row.get("key").and_then(Value::as_str).unwrap_or_default().to_owned();
    if triple.is_empty() {
        diagnostics.push("Charon LLBC target triple is missing".to_owned());
    }
    let pointer_width_bits = row
        .pointer("/value/target_pointer_size")
        .and_then(Value::as_u64)
        .and_then(|bytes| bytes.checked_mul(BITS_PER_BYTE))
        .and_then(|bits| u16::try_from(bits).ok())
        .unwrap_or_else(|| {
            diagnostics.push("Charon LLBC target pointer size is missing or malformed".to_owned());
            0
        });
    let endianness = match row.pointer("/value/is_little_endian").and_then(Value::as_bool) {
        Some(true) => "little".to_owned(),
        Some(false) => "big".to_owned(),
        None => {
            diagnostics.push("Charon LLBC target endianness is missing".to_owned());
            String::new()
        }
    };
    (triple, pointer_width_bits, endianness)
}

fn observed_start_from(options: &Value, diagnostics: &mut Vec<String>) -> String {
    let mut selections = Vec::new();
    for field in ["start_from", "start_from_if_exists", "start_from_attribute"] {
        for rule in required_strings(options, field, diagnostics) {
            selections.push(format!("{field}:{rule}"));
        }
    }
    let by_visibility = required_bool(options, "/start_from_pub", diagnostics);
    if by_visibility {
        selections.push("start_from_pub".to_owned());
    }
    if selections.is_empty() {
        START_FROM_CRATE.to_owned()
    } else {
        selections.join(",")
    }
}

fn declarations_preserve_spans(translated: &Value, diagnostics: &mut Vec<String>) -> bool {
    let mut preserved = true;
    for collection in DECLARATION_COLLECTIONS {
        let Some(declarations) = translated.get(collection).and_then(Value::as_array) else {
            diagnostics.push(format!("Charon LLBC translated.{collection} is missing"));
            return false;
        };
        preserved &= declarations
            .iter()
            .filter(|declaration| !declaration.is_null())
            .all(|declaration| declaration.pointer("/item_meta/span/data").is_some_and(Value::is_object));
    }
    preserved
}

fn observed_serialization(options: &Value, diagnostics: &mut Vec<String>) -> String {
    let unstructured = required_bool(options, "/ullbc", diagnostics);
    let not_serialized = required_bool(options, "/no_serialize", diagnostics);
    let format = match options.get("format") {
        Some(Value::Null) => JSON_FORMAT.to_owned(),
        Some(Value::String(format)) => format.to_ascii_lowercase(),
        _ => {
            diagnostics.push("Charon LLBC options.format is missing or malformed".to_owned());
            String::new()
        }
    };
    if !unstructured && !not_serialized && format == JSON_FORMAT {
        SERIALIZATION_LLBC_JSON.to_owned()
    } else {
        format!("ullbc={unstructured},no-serialize={not_serialized},format={format}")
    }
}

#[derive(Deserialize)]
struct TranslationJson {
    #[serde(rename = "crate")]
    crate_name: String,
    charon_version: String,
    functions: Vec<TranslationJsonFunction>,
}

// The flags mirror Aeneas `translation.json` keys; they are independent declaration facts.
#[allow(clippy::struct_excessive_bools)]
#[derive(Deserialize)]
struct TranslationJsonFunction {
    def_id: u64,
    lean_name: String,
    lean_file: String,
    rust_name: String,
    is_local: bool,
    source: TranslationJsonSource,
    is_opaque: bool,
    can_fail: bool,
    can_diverge: bool,
    is_rec: bool,
}

#[derive(Deserialize)]
struct TranslationJsonSource {
    file: String,
    begin_line: u64,
    end_line: u64,
}

/// A generated Lean file measured by the imperative shell.
pub struct MeasuredLeanFile<'a> {
    /// Locator named by `translation.json` `lean_file`.
    pub locator: &'a str,
    /// Exact measured bytes.
    pub contents: &'a [u8],
}

/// Normalize the produced LLBC and Aeneas `translation.json` into the typed manifest contract.
///
/// Local LLBC function declarations become the current-run join targets; local `translation.json`
/// functions become manifest links. Both sides express their Rust span as the byte envelope of the
/// covered source lines in the LLBC-embedded source file, so they compare equal exactly when the
/// manifest names the same lines Charon extracted.
///
/// # Errors
///
/// Returns diagnostics when either artifact is malformed or the two artifacts name different
/// crates or Charon versions.
// r[impl octet.aeneas_lean_proof_rail.translation_manifest]
pub fn observe_aeneas_translation(
    llbc_text: &str,
    translation_text: &str,
    generated_files: &[MeasuredLeanFile<'_>],
) -> Result<(AeneasTranslationManifest, Vec<AeneasLlbcDeclaration>), Vec<String>> {
    let llbc = serde_json::from_str::<Value>(llbc_text).map_err(|error| vec![format!("parse Charon LLBC: {error}")])?;
    let translation = serde_json::from_str::<TranslationJson>(translation_text)
        .map_err(|error| vec![format!("parse Aeneas translation.json: {error}")])?;
    let mut diagnostics = Vec::new();
    let translated = &llbc["translated"];
    if translated.get("crate_name").and_then(Value::as_str) != Some(translation.crate_name.as_str()) {
        diagnostics.push("translation.json crate does not match the LLBC crate".to_owned());
    }
    if llbc.get("charon_version").and_then(Value::as_str) != Some(translation.charon_version.as_str()) {
        diagnostics.push("translation.json charon_version does not match the LLBC".to_owned());
    }
    let files = llbc_source_files(translated, &mut diagnostics);
    let llbc_declarations = llbc_function_declarations(translated, &files, &mut diagnostics);
    let declarations = translation
        .functions
        .into_iter()
        .filter(|function| function.is_local)
        .map(|function| {
            let rust_span =
                line_envelope(&files, &function.source.file, function.source.begin_line, function.source.end_line)
                    .unwrap_or_else(|| {
                        diagnostics.push(format!(
                            "translation.json function `{}` names lines absent from the LLBC source `{}`",
                            function.rust_name, function.source.file
                        ));
                        empty_span(&function.source.file)
                    });
            AeneasTranslationDeclaration {
                def_id: function.def_id.to_string(),
                rust_name: function.rust_name,
                rust_span,
                lean_name: function.lean_name,
                lean_file: function.lean_file,
                is_opaque: function.is_opaque,
                can_fail: function.can_fail,
                can_diverge: function.can_diverge,
                is_recursive: function.is_rec,
                loop_parent_def_id: None,
                durable_identity: None,
            }
        })
        .collect();
    let mut generated = generated_files
        .iter()
        .map(|file| AeneasGeneratedLeanFile {
            locator: file.locator.to_owned(),
            blake3: blake3::hash(file.contents).to_hex().to_string(),
        })
        .collect::<Vec<_>>();
    generated.sort();
    if diagnostics.is_empty() {
        Ok((
            AeneasTranslationManifest {
                schema_version: AENEAS_TRANSLATION_MANIFEST_SCHEMA_VERSION.to_owned(),
                declarations,
                generated_files: generated,
            },
            llbc_declarations,
        ))
    } else {
        diagnostics.sort();
        diagnostics.dedup();
        Err(diagnostics)
    }
}

/// Observe and validate `translation.json` against the produced LLBC and generated Lean files.
#[must_use]
pub fn validate_observed_aeneas_translation(
    llbc_text: &str,
    translation_text: &str,
    generated_files: &[MeasuredLeanFile<'_>],
) -> Vec<String> {
    match observe_aeneas_translation(llbc_text, translation_text, generated_files) {
        Ok((manifest, llbc_declarations)) => validate_aeneas_translation_manifest(&manifest, &llbc_declarations),
        Err(diagnostics) => diagnostics,
    }
}

struct LlbcSourceFile<'a> {
    id: u64,
    path: &'a str,
    contents: &'a str,
}

fn llbc_source_files<'a>(translated: &'a Value, diagnostics: &mut Vec<String>) -> Vec<LlbcSourceFile<'a>> {
    let Some(files) = translated.get("files").and_then(Value::as_array) else {
        diagnostics.push("Charon LLBC translated.files is missing".to_owned());
        return Vec::new();
    };
    files
        .iter()
        .filter_map(|file| {
            Some(LlbcSourceFile {
                id: file.get("id")?.as_u64()?,
                path: file.pointer("/name/Local")?.as_str()?,
                contents: file.get("contents")?.as_str()?,
            })
        })
        .collect()
}

fn llbc_function_declarations(
    translated: &Value,
    files: &[LlbcSourceFile<'_>],
    diagnostics: &mut Vec<String>,
) -> Vec<AeneasLlbcDeclaration> {
    let Some(functions) = translated.get("fun_decls").and_then(Value::as_array) else {
        diagnostics.push("Charon LLBC translated.fun_decls is missing".to_owned());
        return Vec::new();
    };
    functions
        .iter()
        .filter(|function| function.pointer("/item_meta/is_local").and_then(Value::as_bool) == Some(true))
        .filter_map(|function| {
            let Some(def_id) = function.get("def_id").and_then(Value::as_u64) else {
                diagnostics.push("Charon LLBC local function declaration is missing def_id".to_owned());
                return None;
            };
            let span = function.pointer("/item_meta/span/data");
            let file = span
                .and_then(|span| span.get("file_id"))
                .and_then(Value::as_u64)
                .and_then(|file_id| files.iter().find(|file| file.id == file_id));
            let lines = span
                .and_then(|span| Some((span.pointer("/beg/line")?.as_u64()?, span.pointer("/end/line")?.as_u64()?)));
            let rust_span = file
                .zip(lines)
                .and_then(|(file, (begin, end))| line_envelope(files, file.path, begin, end))
                .unwrap_or_else(|| {
                    diagnostics.push(format!("Charon LLBC function {def_id} has no local source span"));
                    empty_span("")
                });
            Some(AeneasLlbcDeclaration {
                def_id: def_id.to_string(),
                rust_name: rendered_name(function.pointer("/item_meta/name")),
                rust_span,
            })
        })
        .collect()
}

fn rendered_name(name: Option<&Value>) -> String {
    name.and_then(Value::as_array)
        .map(|elements| {
            elements
                .iter()
                .map(|element| {
                    element
                        .pointer("/Ident/0")
                        .and_then(Value::as_str)
                        .map_or_else(|| element.to_string(), str::to_owned)
                })
                .collect::<Vec<_>>()
                .join("::")
        })
        .unwrap_or_default()
}

fn line_envelope(files: &[LlbcSourceFile<'_>], path: &str, begin_line: u64, end_line: u64) -> Option<AeneasSourceSpan> {
    let file = files.iter().find(|file| file.path == path)?;
    let (start_byte, end_byte) = crate::aeneas_consumer::selected_line_range(file.contents, begin_line, end_line)?;
    Some(AeneasSourceSpan {
        locator: path.to_owned(),
        start_byte,
        end_byte,
    })
}

fn empty_span(locator: &str) -> AeneasSourceSpan {
    AeneasSourceSpan {
        locator: locator.to_owned(),
        start_byte: 0,
        end_byte: 0,
    }
}

/// A trust-relevant Lean declaration form found in an Octet-bound Lean source.
#[derive(Clone, Debug, Eq, PartialEq)]
enum LeanTrustForm {
    /// `axiom <name>`.
    Axiom(String),
    /// `opaque <name>`.
    Opaque(String),
    /// `sorry`, `admit`, or `sorryAx`.
    Admitted(String),
    /// `native_decide`, `ofReduceBool`, `ofReduceNat`, `implemented_by`, or `extern`, which trust
    /// compiled code.
    NativeTrust(String),
}

/// Observe the theorem trust closure from the maintained theorem and generated Lean sources.
///
/// Both sources are scanned as Lean tokens, skipping comments, string text, and character
/// literals. `axiom` declarations are declared only when named by the assumption policy's axioms
/// or external models; `opaque` definitions only when named by its external models. Admitted
/// declarations (`sorry`, `admit`, `sorryAx`) are recorded for rejection, and native-code trust
/// (`native_decide`, `ofReduceBool`, `ofReduceNat`, `implemented_by`, `extern`) is always
/// undeclared because the policy has no way to declare it.
///
/// This scan is only the early filter that stops before Lean runs; it cannot see trust that Lean
/// constructs without a banned token (the `stop` tactic, meta-level `addDecl`). The admitted
/// closure is [`observe_aeneas_lean_trust_closure`], which adds the axioms Lean itself reports.
// r[impl octet.aeneas_lean_proof_rail.theorem_adequacy]
#[must_use]
pub fn observe_aeneas_trust_closure(
    theorem_source: &str,
    generated_lean: &str,
    policy: &AeneasAssumptionPolicy,
) -> AeneasTrustClosure {
    let mut opaque_definitions = BTreeSet::new();
    let mut admitted = BTreeSet::new();
    let mut undeclared = BTreeSet::new();
    for (origin, source) in [(THEOREM_ORIGIN, theorem_source), (GENERATED_ORIGIN, generated_lean)] {
        for form in lean_trust_forms(source) {
            match form {
                LeanTrustForm::Axiom(name)
                    if policy.axioms.contains(&name) || policy.external_models.contains(&name) => {}
                LeanTrustForm::Axiom(name) => {
                    undeclared.insert(format!("{origin}: axiom {name}"));
                }
                LeanTrustForm::Opaque(name) if policy.external_models.contains(&name) => {
                    opaque_definitions.insert(name);
                }
                LeanTrustForm::Opaque(name) => {
                    undeclared.insert(format!("{origin}: opaque {name}"));
                }
                LeanTrustForm::Admitted(token) => {
                    admitted.insert(format!("{origin}: {token}"));
                }
                LeanTrustForm::NativeTrust(token) => {
                    undeclared.insert(format!("{origin}: {token}"));
                }
            }
        }
    }
    AeneasTrustClosure {
        complete: true,
        declared_external_models: policy.external_models.clone(),
        declared_axioms: policy.axioms.clone(),
        declared_opaque_definitions: opaque_definitions.into_iter().collect(),
        declared_admitted_declarations: admitted.into_iter().collect(),
        undeclared_inputs: undeclared.into_iter().collect(),
    }
}

/// The axiom report Octet's Lean check program prints for the approved theorem.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LeanAxiomReport {
    schema_version: String,
    theorem: String,
    axioms: Vec<String>,
}

/// Observe the admitted trust closure: the source token scan plus every axiom Lean reports.
///
/// `axiom_report` is the JSON line Octet's Lean check program prints after `lake build`; it lists
/// each axiom that any declaration of the `Theorem` module, the approved theorem among them,
/// reaches through kernel-checked types and values.
/// The core axioms `propext`, `Classical.choice`, and `Quot.sound` are accepted. `sorryAx` is an
/// admitted declaration. `Lean.ofReduceBool`, `Lean.ofReduceNat`, `Lean.trustCompiler`, and the
/// `_native` auxiliary axioms `native_decide` adds trust compiled code and are declared only when
/// named by the policy's axioms. Any other axiom is declared only when named by the policy's
/// axioms or external models.
///
/// # Errors
///
/// Returns a diagnostic when the report is malformed, has another schema version, or reports a
/// theorem other than `theorem_name`.
// r[impl octet.aeneas_lean_proof_rail.theorem_adequacy]
pub fn observe_aeneas_lean_trust_closure(
    theorem_source: &str,
    generated_lean: &str,
    theorem_name: &str,
    axiom_report: &str,
    policy: &AeneasAssumptionPolicy,
) -> Result<AeneasTrustClosure, String> {
    let report = serde_json::from_str::<LeanAxiomReport>(axiom_report.trim())
        .map_err(|error| format!("Lean axiom report is not valid JSON: {error}"))?;
    if report.schema_version != AENEAS_LEAN_AXIOM_REPORT_SCHEMA_VERSION {
        return Err(format!(
            "Lean axiom report schema `{}` is not `{AENEAS_LEAN_AXIOM_REPORT_SCHEMA_VERSION}`",
            report.schema_version
        ));
    }
    if report.theorem != theorem_name {
        return Err(format!(
            "Lean axiom report covers theorem `{}`, not the approved theorem `{theorem_name}`",
            report.theorem
        ));
    }
    let mut closure = observe_aeneas_trust_closure(theorem_source, generated_lean, policy);
    let mut admitted = closure.declared_admitted_declarations.into_iter().collect::<BTreeSet<_>>();
    let mut undeclared = closure.undeclared_inputs.into_iter().collect::<BTreeSet<_>>();
    for axiom in report.axioms {
        if LEAN_CORE_AXIOMS.contains(&axiom.as_str()) {
            continue;
        }
        if axiom == LEAN_ADMITTED_AXIOM {
            admitted.insert(format!("{LEAN_ORIGIN}: {axiom}"));
        } else if lean_native_axiom(&axiom) {
            if !policy.axioms.contains(&axiom) {
                undeclared.insert(format!("{LEAN_ORIGIN}: native {axiom}"));
            }
        } else if !policy.axioms.contains(&axiom) && !policy.external_models.contains(&axiom) {
            undeclared.insert(format!("{LEAN_ORIGIN}: axiom {axiom}"));
        }
    }
    closure.declared_admitted_declarations = admitted.into_iter().collect();
    closure.undeclared_inputs = undeclared.into_iter().collect();
    Ok(closure)
}

fn lean_native_axiom(axiom: &str) -> bool {
    LEAN_NATIVE_AXIOMS.contains(&axiom) || axiom.split('.').any(|component| component == LEAN_NATIVE_NAME_COMPONENT)
}

fn lean_trust_forms(source: &str) -> Vec<LeanTrustForm> {
    let tokens = lean_identifier_tokens(source);
    let mut forms = Vec::new();
    for (index, token) in tokens.iter().enumerate() {
        let next = || tokens.get(index + 1).map(|next| next.text.clone()).unwrap_or_default();
        // `«…»` makes a name out of keyword text, so only unquoted tokens can be Lean keywords;
        // constant references keep their meaning however they are spelled.
        let keyword = if token.quoted { "" } else { token.text.as_str() };
        let reference = token.text.strip_prefix(ROOT_NAMESPACE_PREFIX).unwrap_or(&token.text);
        match (keyword, reference) {
            ("axiom", _) => forms.push(LeanTrustForm::Axiom(next())),
            ("opaque", _) => forms.push(LeanTrustForm::Opaque(next())),
            ("sorry" | "admit", _) | (_, "sorryAx") => forms.push(LeanTrustForm::Admitted(token.text.clone())),
            ("native_decide" | "extern", _)
            | (_, "ofReduceBool" | "Lean.ofReduceBool" | "ofReduceNat" | "Lean.ofReduceNat") => {
                forms.push(LeanTrustForm::NativeTrust(token.text.clone()));
            }
            ("implemented_by", _) => forms.push(LeanTrustForm::NativeTrust(format!("implemented_by {}", next()))),
            _ => {}
        }
    }
    forms
}

/// One Lean identifier with its `«…»` quotes removed.
#[derive(Clone, Debug, Eq, PartialEq)]
struct LeanToken {
    text: String,
    /// Whether any component was written as a `«…»` quoted name, which Lean never reads as a
    /// keyword.
    quoted: bool,
}

/// Split Lean source into identifier tokens, skipping comments, string text, and character
/// literals; the `{…}` holes of interpolated strings are scanned as code.
fn lean_identifier_tokens(source: &str) -> Vec<LeanToken> {
    let characters = source.chars().collect::<Vec<_>>();
    let mut scanner = LeanTokenScanner {
        characters: &characters,
        index: 0,
        tokens: Vec::new(),
        last_token_end: 0,
    };
    scanner.scan_code(false);
    scanner.tokens
}

struct LeanTokenScanner<'a> {
    characters: &'a [char],
    index: usize,
    tokens: Vec<LeanToken>,
    /// Character index just past the most recent identifier token.
    last_token_end: usize,
}

impl LeanTokenScanner<'_> {
    fn peek(&self, offset: usize) -> Option<char> {
        self.characters.get(self.index + offset).copied()
    }

    /// Scan code up to the end of the source or, inside an interpolation hole, its closing `}`.
    fn scan_code(&mut self, interpolation_hole: bool) {
        let mut brace_depth = 0_usize;
        while let Some(character) = self.peek(0) {
            let next = self.peek(1);
            if character == '-' && next == Some('-') {
                while self.peek(0).is_some_and(|character| character != '\n') {
                    self.index += 1;
                }
            } else if character == '/' && next == Some('-') {
                self.index = skip_block_comment(self.characters, self.index);
            } else if character == '"' {
                let interpolated = self.opens_interpolated_string();
                self.index += 1;
                self.scan_string(interpolated);
            } else if character == 'r' && raw_string_start(self.characters, self.index) {
                self.index = skip_raw_string(self.characters, self.index + 1);
            } else if character == '\'' {
                self.index = skip_character_literal(self.characters, self.index);
            } else if character == '«' || identifier_start(character) {
                self.scan_identifier();
            } else if interpolation_hole && character == '{' {
                brace_depth += 1;
                self.index += 1;
            } else if interpolation_hole && character == '}' {
                self.index += 1;
                if brace_depth == 0 {
                    return;
                }
                brace_depth -= 1;
            } else {
                self.index += 1;
            }
        }
    }

    /// Whether the string literal at the cursor is parsed as `interpolatedStr`.
    ///
    /// Lean decides this syntactically, and only for the syntax that asks for it: the `s!`, `m!`,
    /// and `f!` macros, `println!`, `dbg_trace`, the `throwError` and `logNamed…` family, and
    /// `trace[cls]`-style macros. A string after any other name, such as `panic!` or
    /// `HashMap.get!`, is plain text.
    fn opens_interpolated_string(&self) -> bool {
        let before = &self.characters[..self.index];
        let preceding = before.iter().rposition(|character| !character.is_whitespace());
        if preceding.is_some_and(|close| before[close] == ']' && closes_trace_macro_class(before, close)) {
            return true;
        }
        let adjacent_token = before[self.last_token_end..].iter().all(|character| character.is_whitespace());
        let mut recent = self.tokens.iter().rev().map(|token| if token.quoted { "" } else { token.text.as_str() });
        let [last, second, third] = [recent.next(), recent.next(), recent.next()];
        let names = |allowed: &[&str], token: Option<&str>| token.is_some_and(|token| allowed.contains(&token));
        (adjacent_token && names(&INTERPOLATING_KEYWORDS, last))
            || names(&INTERPOLATING_AFTER_ONE_ARGUMENT, second)
            || names(&INTERPOLATING_AFTER_TWO_ARGUMENTS, third)
    }

    /// Skip string text up to the closing quote, scanning interpolation holes as code.
    fn scan_string(&mut self, interpolated: bool) {
        while let Some(character) = self.peek(0) {
            match character {
                '\\' => self.index += 2,
                '"' => {
                    self.index += 1;
                    return;
                }
                '{' if interpolated => {
                    self.index += 1;
                    self.scan_code(true);
                }
                _ => self.index += 1,
            }
        }
    }

    /// Read one dotted identifier whose components are plain or `«…»` quoted.
    fn scan_identifier(&mut self) {
        let mut token = LeanToken {
            text: String::new(),
            quoted: false,
        };
        loop {
            if self.peek(0) == Some('«') {
                token.quoted = true;
                self.index += 1;
                while let Some(character) = self.peek(0).filter(|character| *character != '»') {
                    token.text.push(character);
                    self.index += 1;
                }
                self.index += 1;
            } else {
                while let Some(character) = self.peek(0).filter(|character| identifier_continue(*character)) {
                    token.text.push(character);
                    self.index += 1;
                }
            }
            let continues = self.peek(0) == Some('.')
                && self.peek(1).is_some_and(|character| character == '«' || identifier_start(character));
            if !continues {
                break;
            }
            token.text.push('.');
            self.index += 1;
        }
        self.index = self.index.min(self.characters.len());
        self.last_token_end = self.index;
        self.tokens.push(token);
    }
}

/// Whether the `]` at `close` ends the class of a `trace[cls]`-style macro: its `[` directly
/// follows one of [`INTERPOLATING_TRACE_MACROS`].
fn closes_trace_macro_class(before: &[char], close: usize) -> bool {
    let Some(open) = before[..close].iter().rposition(|character| *character == '[') else {
        return false;
    };
    let head_start = before[..open]
        .iter()
        .rposition(|character| !identifier_continue(*character) && *character != '.')
        .map_or(0, |separator| separator + 1);
    let head = before[head_start..open].iter().collect::<String>();
    INTERPOLATING_TRACE_MACROS.contains(&head.as_str())
}

fn skip_block_comment(characters: &[char], start: usize) -> usize {
    let mut depth = 0_usize;
    let mut index = start;
    while index < characters.len() {
        let pair = (characters[index], characters.get(index + 1).copied());
        if pair == ('/', Some('-')) {
            depth += 1;
            index += 2;
        } else if pair == ('-', Some('/')) {
            depth -= 1;
            index += 2;
            if depth == 0 {
                return index;
            }
        } else {
            index += 1;
        }
    }
    index
}

fn raw_string_start(characters: &[char], index: usize) -> bool {
    if index > 0 && identifier_continue(characters[index - 1]) {
        return false;
    }
    let mut cursor = index + 1;
    while characters.get(cursor) == Some(&'#') {
        cursor += 1;
    }
    characters.get(cursor) == Some(&'"')
}

fn skip_raw_string(characters: &[char], mut index: usize) -> usize {
    let mut hashes = 0;
    while characters.get(index) == Some(&'#') {
        hashes += 1;
        index += 1;
    }
    index += 1;
    while index < characters.len() {
        if characters[index] == '"' && (1..=hashes).all(|offset| characters.get(index + offset) == Some(&'#')) {
            return index + 1 + hashes;
        }
        index += 1;
    }
    index
}

fn skip_character_literal(characters: &[char], index: usize) -> usize {
    // A quote directly after an identifier character is part of that identifier (`h'`), and the
    // identifier scanner already consumed it; any other quote opens a character literal.
    let body = index + 1;
    let close = if characters.get(body) == Some(&'\\') {
        (body + 1..characters.len()).find(|&cursor| characters[cursor] == '\'')
    } else {
        Some(body + 1).filter(|&cursor| characters.get(cursor) == Some(&'\''))
    };
    close.map_or(index + 1, |cursor| cursor + 1)
}

fn identifier_start(character: char) -> bool {
    character.is_alphabetic() || character == '_'
}

fn identifier_continue(character: char) -> bool {
    character.is_alphanumeric() || matches!(character, '_' | '\'' | '!' | '?')
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::formal_proof::validate_charon_extraction_contract;

    const FIXTURE_ROOT: &str = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../standards/proof-rails/aeneas-lean/fixtures/valid/artifacts/proofs/aeneas/"
    );
    const PROFILE_JSON: &str = include_str!("../../../standards/proof-rails/aeneas-lean/generated/profile.json");

    fn fixture(name: &str) -> String {
        std::fs::read_to_string(format!("{FIXTURE_ROOT}{name}")).expect("frozen Aeneas fixture artifact")
    }

    fn reviewed_policy() -> CharonExtractionPolicy {
        let profile = serde_json::from_str::<Value>(PROFILE_JSON).expect("generated provider profile");
        serde_json::from_value(profile["extraction_policy"].clone()).expect("reviewed extraction policy")
    }

    fn empty_policy() -> AeneasAssumptionPolicy {
        AeneasAssumptionPolicy {
            external_models: Vec::new(),
            axioms: Vec::new(),
            requires_complete_declaration: true,
        }
    }

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[test]
    fn aeneas_observed_charon_policy_matches_reviewed_contract_for_real_llbc() {
        let observed = observe_charon_extraction_policy(&fixture("llbc.llbc")).expect("observed policy");

        assert!(observed.include_rules.is_empty(), "the executed Charon argv applies no include rules");
        assert_eq!(validate_charon_extraction_contract(&observed, &reviewed_policy()), Vec::<String>::new());
    }

    /// A named LLBC mutation and the extraction-policy field it must drift.
    type LlbcMutation = (&'static str, fn(&mut Value));

    // r[verify octet.aeneas_lean_proof_rail.charon_contract]
    #[test]
    fn aeneas_observed_charon_policy_rejects_weakened_or_missing_options() {
        let original = serde_json::from_str::<Value>(&fixture("llbc.llbc")).expect("LLBC JSON");
        let mutations: [LlbcMutation; 9] = [
            ("has_errors", |llbc| llbc["has_errors"] = Value::Bool(true)),
            ("skip_borrowck", |llbc| llbc["translated"]["options"]["skip_borrowck"] = Value::Bool(true)),
            ("pointer_width_bits", |llbc| {
                llbc["translated"]["target_information"][0]["value"]["target_pointer_size"] = Value::from(4_u64);
            }),
            ("endianness", |llbc| {
                llbc["translated"]["target_information"][0]["value"]["is_little_endian"] = Value::Bool(false);
            }),
            ("mir_start_stage", |llbc| llbc["translated"]["options"]["mir"] = Value::from("Optimized")),
            ("drop_policy", |llbc| llbc["translated"]["options"]["precise_drops"] = Value::Bool(true)),
            ("start_from", |llbc| {
                llbc["translated"]["options"]["start_from"] = serde_json::json!(["real_fixture::other"]);
            }),
            ("transformation_settings", |llbc| {
                llbc["translated"]["fun_decls"][0]["item_meta"].as_object_mut().expect("item meta").remove("span");
            }),
            ("serialization_format", |llbc| llbc["translated"]["options"]["ullbc"] = Value::Bool(true)),
        ];
        for (field, mutate) in mutations {
            let mut llbc = original.clone();
            mutate(&mut llbc);
            let observed = observe_charon_extraction_policy(&llbc.to_string()).expect("observed mutated policy");
            let diagnostics = validate_charon_extraction_contract(&observed, &reviewed_policy());
            assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(field)), "{field}: {diagnostics:?}");
        }

        let mut missing = original;
        missing["translated"]["options"].as_object_mut().expect("options").remove("skip_borrowck");
        assert!(observe_charon_extraction_policy(&missing.to_string()).is_err());
        assert!(observe_charon_extraction_policy("not json").is_err());
    }

    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    #[test]
    fn aeneas_observed_translation_links_real_llbc_and_generated_lean() {
        let generated = fixture("Llbc.lean");
        let files = [MeasuredLeanFile {
            locator: "Llbc.lean",
            contents: generated.as_bytes(),
        }];
        let diagnostics =
            validate_observed_aeneas_translation(&fixture("llbc.llbc"), &fixture("translation.json"), &files);

        assert_eq!(diagnostics, Vec::<String>::new());
    }

    // r[verify octet.aeneas_lean_proof_rail.translation_manifest]
    #[test]
    fn aeneas_observed_translation_rejects_stale_spans_names_files_and_crates() {
        let generated = fixture("Llbc.lean");
        let files = [MeasuredLeanFile {
            locator: "Llbc.lean",
            contents: generated.as_bytes(),
        }];
        let llbc = fixture("llbc.llbc");
        let original = serde_json::from_str::<Value>(&fixture("translation.json")).expect("translation JSON");
        let mutations: [fn(&mut Value); 5] = [
            |translation| translation["functions"][0]["source"]["end_line"] = Value::from(2_u64),
            |translation| translation["functions"][0]["rust_name"] = Value::from("real_fixture::other"),
            |translation| translation["functions"][0]["def_id"] = Value::from(7_u64),
            |translation| translation["functions"][0]["lean_file"] = Value::from("Other.lean"),
            |translation| translation["crate"] = Value::from("other_crate"),
        ];
        for mutate in mutations {
            let mut translation = original.clone();
            mutate(&mut translation);
            let diagnostics = validate_observed_aeneas_translation(&llbc, &translation.to_string(), &files);
            assert!(!diagnostics.is_empty(), "mutated translation unexpectedly validated: {translation}");
        }
        assert!(!validate_observed_aeneas_translation(&llbc, &original.to_string(), &[]).is_empty());
        assert!(!validate_observed_aeneas_translation(&llbc, "{}", &files).is_empty());
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_trust_closure_accepts_clean_sources_and_ignores_comments_and_strings() {
        let theorem = "import Llbc\n/-- no axiom here; sorry is prose -/\ntheorem t : f 1 = .ok 2 := by\n  -- native_decide would be wrong\n  rfl\ndef s := \"axiom cheat\"\ndef c := 'a'\ntheorem h' : True := trivial\n";
        let closure = observe_aeneas_trust_closure(theorem, &fixture("Llbc.lean"), &empty_policy());

        assert!(closure.complete);
        assert_eq!(closure.undeclared_inputs, Vec::<String>::new());
        assert_eq!(closure.declared_admitted_declarations, Vec::<String>::new());
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_trust_closure_reports_undeclared_axioms_opaque_and_native_trust() {
        let theorem = "axiom cheat : 1 = 2\ntheorem t : 1 = 2 := cheat\nopaque hidden : Nat\ntheorem n : 2 = 2 := by native_decide\n@[implemented_by fast] def slow := 1\n";
        let generated = "/- generated -/\naxiom core.model : Nat\ntheorem g : True := sorry\n";
        let closure = observe_aeneas_trust_closure(theorem, generated, &empty_policy());

        assert_eq!(closure.undeclared_inputs, vec![
            "generated: axiom core.model".to_owned(),
            "theorem: axiom cheat".to_owned(),
            "theorem: implemented_by fast".to_owned(),
            "theorem: native_decide".to_owned(),
            "theorem: opaque hidden".to_owned(),
        ]);
        assert_eq!(closure.declared_admitted_declarations, vec!["generated: sorry".to_owned()]);

        let declared = AeneasAssumptionPolicy {
            external_models: vec!["core.model".to_owned(), "hidden".to_owned()],
            axioms: vec!["cheat".to_owned()],
            requires_complete_declaration: true,
        };
        let closure = observe_aeneas_trust_closure(theorem, "axiom core.model : Nat\n", &declared);
        assert_eq!(closure.undeclared_inputs, vec![
            "theorem: implemented_by fast".to_owned(),
            "theorem: native_decide".to_owned()
        ]);
        assert_eq!(closure.declared_opaque_definitions, vec!["hidden".to_owned()]);
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_trust_closure_scans_string_interpolations_as_code() {
        let theorem = "def q := s!\"{'\"'}\"\naxiom cheat : False\ndef r := m!\"{\"}\" ++ \"x\"} sorry\"\ndef f := f! \"{ (\"a\" : String) }\"\naxiom other : False\ndef plain := \"{ axiom hidden\"\n";
        let closure = observe_aeneas_trust_closure(theorem, "", &empty_policy());

        assert_eq!(closure.undeclared_inputs, vec![
            "theorem: axiom cheat".to_owned(),
            "theorem: axiom other".to_owned()
        ]);
        assert_eq!(closure.declared_admitted_declarations, Vec::<String>::new());

        let commands = "def e := throwError \"{'\"'}\"\naxiom third : False\ndef t := trace[cls] \"{'\"'}\"\naxiom fourth : False\n";
        let closure = observe_aeneas_trust_closure(commands, "", &empty_policy());
        assert_eq!(closure.undeclared_inputs, vec![
            "theorem: axiom fourth".to_owned(),
            "theorem: axiom third".to_owned()
        ]);

        let interpolated_code = "def q := s!\"{sorry}\"\n";
        let closure = observe_aeneas_trust_closure(interpolated_code, "", &empty_policy());
        assert_eq!(closure.declared_admitted_declarations, vec!["theorem: sorry".to_owned()]);
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_trust_closure_interpolates_only_strings_lean_parses_as_interpolated() {
        let plain = "def p := panic! \"bad {\"\naxiom after_panic : False\ndef g := HashMap.get! m \"{\"\n@[extern \"c_fn\"] def native : Nat := 0\ndef k := pair [1] \"{\"\naxiom after_list : False\ndef t := xs[0]! \"{\"\nopaque after_index : Nat\n";
        let closure = observe_aeneas_trust_closure(plain, "", &empty_policy());
        assert_eq!(closure.undeclared_inputs, vec![
            "theorem: axiom after_list".to_owned(),
            "theorem: axiom after_panic".to_owned(),
            "theorem: extern".to_owned(),
            "theorem: opaque after_index".to_owned(),
        ]);

        for interpolated in [
            "def a := IO.println s!\"{sorry}\"",
            "def a := logInfo m!\"{sorry}\"",
            "def a := f! \"{sorry}\"",
            "def a := println! \"{sorry}\"",
            "def a := dbg_trace \"{sorry}\"; 0",
            "def a := throwError \"{sorry}\"",
            "def a := throwErrorAt ref \"{sorry}\"",
            "def a := do trace[Meta.debug] \"{sorry}\"",
            "def a := do trace_goal[grind] \"{sorry}\"",
            "def a := Macro.trace[cls] \"{sorry}\"",
        ] {
            let closure = observe_aeneas_trust_closure(interpolated, "", &empty_policy());
            assert_eq!(closure.declared_admitted_declarations, vec!["theorem: sorry".to_owned()], "{interpolated}");
        }
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_trust_closure_treats_french_quoted_identifiers_as_names_not_keywords() {
        let theorem = "theorem «axiom» : True := trivial\ndef Foo.«sorry» := 1\ndef «opaque» := Foo.«sorry»\n";
        let closure = observe_aeneas_trust_closure(theorem, "", &empty_policy());

        assert_eq!(closure.undeclared_inputs, Vec::<String>::new());
        assert_eq!(closure.declared_admitted_declarations, Vec::<String>::new());

        let references = "axiom «cheat» : False\ntheorem t : 1 = 2 := «sorryAx» _ false\n";
        let closure = observe_aeneas_trust_closure(references, "", &empty_policy());
        assert_eq!(closure.undeclared_inputs, vec!["theorem: axiom cheat".to_owned()]);
        assert_eq!(closure.declared_admitted_declarations, vec!["theorem: sorryAx".to_owned()]);
    }

    fn lean_axiom_report(theorem: &str, axioms: &[&str]) -> String {
        serde_json::json!({
            "schema_version": AENEAS_LEAN_AXIOM_REPORT_SCHEMA_VERSION,
            "theorem": theorem,
            "axioms": axioms,
        })
        .to_string()
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_lean_trust_closure_classifies_axioms_lean_reports() {
        // `stop` elaborates to `repeat sorry`: no token reveals the admitted proof, Lean does.
        let theorem = "import Llbc\ntheorem stopped : 1 = 2 := by stop rfl\n";
        let scanned = observe_aeneas_trust_closure(theorem, "", &empty_policy());
        assert_eq!(scanned.declared_admitted_declarations, Vec::<String>::new());

        let observe = |axioms: &[&str], policy: &AeneasAssumptionPolicy| {
            observe_aeneas_lean_trust_closure(theorem, "", "stopped", &lean_axiom_report("stopped", axioms), policy)
                .expect("valid Lean axiom report")
        };
        let core = observe(&["propext", "Classical.choice", "Quot.sound"], &empty_policy());
        assert_eq!(core.undeclared_inputs, Vec::<String>::new());
        assert_eq!(core.declared_admitted_declarations, Vec::<String>::new());

        let admitted = observe(&["propext", "sorryAx"], &empty_policy());
        assert_eq!(admitted.declared_admitted_declarations, vec!["lean: sorryAx".to_owned()]);
        assert_eq!(admitted.undeclared_inputs, Vec::<String>::new());

        let native_and_planted = [
            "Lean.ofReduceBool",
            "Lean.ofReduceNat",
            "Lean.trustCompiler",
            "nd._native.native_decide.ax_1_1",
            "planted",
        ];
        let undeclared = observe(&native_and_planted, &empty_policy());
        assert_eq!(undeclared.undeclared_inputs, vec![
            "lean: axiom planted".to_owned(),
            "lean: native Lean.ofReduceBool".to_owned(),
            "lean: native Lean.ofReduceNat".to_owned(),
            "lean: native Lean.trustCompiler".to_owned(),
            "lean: native nd._native.native_decide.ax_1_1".to_owned(),
        ]);

        let declared = AeneasAssumptionPolicy {
            external_models: vec!["planted".to_owned(), "Lean.ofReduceBool".to_owned()],
            axioms: vec![
                "Lean.ofReduceNat".to_owned(),
                "Lean.trustCompiler".to_owned(),
                "nd._native.native_decide.ax_1_1".to_owned(),
            ],
            requires_complete_declaration: true,
        };
        let partially_declared = observe(&native_and_planted, &declared);
        assert_eq!(partially_declared.undeclared_inputs, vec!["lean: native Lean.ofReduceBool".to_owned()]);
    }

    // r[verify octet.aeneas_lean_proof_rail.theorem_adequacy]
    #[test]
    fn aeneas_lean_trust_closure_rejects_malformed_or_foreign_reports() {
        let theorem = "theorem approved : 1 = 1 := rfl\n";
        let policy = empty_policy();
        let observe = |report: &str| observe_aeneas_lean_trust_closure(theorem, "", "approved", report, &policy);

        assert!(observe(&lean_axiom_report("approved", &[])).is_ok());
        assert!(observe(&lean_axiom_report("other", &[])).is_err());
        assert!(observe("'approved' does not depend on any axioms").is_err());
        assert!(observe("").is_err());
        let mut report = serde_json::from_str::<Value>(&lean_axiom_report("approved", &[])).expect("report JSON");
        report["schema_version"] = Value::from("octet-aeneas-lean-axioms/v0");
        assert!(observe(&report.to_string()).is_err());
        report["schema_version"] = Value::from(AENEAS_LEAN_AXIOM_REPORT_SCHEMA_VERSION);
        report["extra"] = Value::Bool(true);
        assert!(observe(&report.to_string()).is_err());
    }
}

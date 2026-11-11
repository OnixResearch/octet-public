// r[impl cairn.structured_lifecycle.typed_authoring]
// r[verify cairn.structured_lifecycle.typed_authoring.nickel_contracts]
// r[verify cairn.structured_lifecycle.typed_authoring.rust_revalidation]
// r[impl cairn.structured_lifecycle.runtime_bundle]
// r[verify cairn.structured_lifecycle.runtime_bundle.nickel_free_runtime]
// r[verify cairn.structured_lifecycle.runtime_bundle.stale_generated_bundle]
// r[verify cairn.structured_lifecycle.runtime_bundle.receipt_hash_binding]
// r[impl cairn.structured_lifecycle.markdown_projection]
// r[verify cairn.structured_lifecycle.markdown_projection.render_view]
// r[verify cairn.structured_lifecycle.markdown_projection.lossy_import]
// r[verify cairn.structured_lifecycle.markdown_projection.compatibility]
// r[impl cairn.structured_lifecycle.migration]
// r[verify cairn.structured_lifecycle.migration.preview_first]
// r[verify cairn.structured_lifecycle.migration.mutation_manifest]
// r[verify cairn.structured_lifecycle.migration.no_silent_delete]
// r[impl cairn.structured_lifecycle.format_strategy]
// r[verify cairn.structured_lifecycle.format_strategy.nickel_authoring]
// r[verify cairn.structured_lifecycle.format_strategy.generated_json]
// r[verify cairn.structured_lifecycle.format_strategy.preserves_candidate]
// r[verify cairn.structured_lifecycle.format_strategy.steel_not_source]
// r[verify cairn.structured_lifecycle.format_strategy.steel_probe_dsl]
use std::{
    error::Error,
    fs,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, structured_lifecycle_bundle_hash, structured_lifecycle_issue_json,
    validate_structured_lifecycle_bundle, STRUCTURED_LIFECYCLE_SCHEMA_V1,
};
use serde_json::{json, Value};

use crate::nickel_export_boundary::export_json_dual_run;
use crate::support::{
    collect_validation_spec_sources, detect_layout, has_flag, print_json, read_option,
    read_string_option, with_receipt_hash,
};

const STRUCTURED_LIFECYCLE_EXPORT_SCHEMA: &str = "cairn.structured-lifecycle.export-receipt.v1";
const STRUCTURED_LIFECYCLE_RENDER_SCHEMA: &str = "cairn.structured-lifecycle.render-receipt.v1";
const STRUCTURED_LIFECYCLE_IMPORT_SCHEMA: &str = "cairn.structured-lifecycle.import-receipt.v1";
const MARKDOWN_SPEC_FILE: &str = "spec.md";
const GIVEN_PREFIX: &str = "- GIVEN ";
const WHEN_PREFIX: &str = "- WHEN ";
const THEN_PREFIX: &str = "- THEN ";

pub(crate) fn run_lifecycle_data(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err(
            "lifecycle requires a command: check, export, render-markdown, or import-markdown"
                .into(),
        );
    };
    match command {
        "check" => run_lifecycle_check(args.into_iter().skip(1).collect()),
        "export" => run_lifecycle_export(args.into_iter().skip(1).collect()),
        "render-markdown" => run_lifecycle_render_markdown(args.into_iter().skip(1).collect()),
        "import-markdown" => run_lifecycle_import_markdown(args.into_iter().skip(1).collect()),
        unknown => Err(format!("unknown lifecycle command: {unknown}").into()),
    }
}

fn run_lifecycle_check(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let source =
        read_string_option(&args, "--source").ok_or("lifecycle check requires --source")?;
    let bundle = load_lifecycle_bundle(Path::new(&source))?;
    print_json(with_receipt_hash(lifecycle_check_receipt(
        "lifecycle check",
        &source,
        &bundle,
        Vec::new(),
    )))
}

fn run_lifecycle_export(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.structured_lifecycle.typed_authoring]
    // r[impl cairn.structured_lifecycle.typed_authoring.nickel_contracts]
    // r[impl cairn.structured_lifecycle.typed_authoring.rust_revalidation]
    // r[impl cairn.structured_lifecycle.runtime_bundle]
    // r[impl cairn.structured_lifecycle.runtime_bundle.nickel_free_runtime]
    // r[impl cairn.structured_lifecycle.runtime_bundle.stale_generated_bundle]
    // r[impl cairn.structured_lifecycle.runtime_bundle.receipt_hash_binding]
    let source =
        read_string_option(&args, "--source").ok_or("lifecycle export requires --source")?;
    let output =
        read_string_option(&args, "--output").ok_or("lifecycle export requires --output")?;
    let is_check = has_flag(&args, "--check");
    let bundle = load_lifecycle_bundle(Path::new(&source))?;
    let rendered = serde_json::to_string_pretty(&bundle)? + "\n";
    let mut export_issues = Vec::new();
    if is_check {
        let existing = fs::read_to_string(&output).unwrap_or_default();
        if existing != rendered {
            export_issues.push(json!({
                "code": "structured_lifecycle.generated.stale",
                "message": "generated lifecycle bundle is stale",
                "field_path": "output",
            }));
        }
    } else {
        if let Some(parent) = Path::new(&output).parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&output, rendered)?;
    }
    let mut receipt = lifecycle_check_receipt("lifecycle export", &source, &bundle, export_issues);
    receipt["output"] = Value::String(output);
    receipt["check"] = Value::Bool(is_check);
    receipt["valid"] = Value::Bool(receipt["issues"].as_array().is_some_and(Vec::is_empty));
    receipt["verdict"] = Value::String(if receipt["valid"].as_bool() == Some(true) {
        "PASS".to_string()
    } else {
        "FAIL".to_string()
    });
    print_json(with_receipt_hash(receipt))
}

fn run_lifecycle_render_markdown(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.structured_lifecycle.markdown_projection]
    // r[impl cairn.structured_lifecycle.markdown_projection.render_view]
    let source = read_string_option(&args, "--source")
        .ok_or("lifecycle render-markdown requires --source")?;
    let bundle = load_lifecycle_bundle(Path::new(&source))?;
    let out_dir = read_string_option(&args, "--out-dir");
    let rendered = render_bundle_markdown(&bundle);
    let artifacts = write_rendered_markdown(out_dir.as_deref(), &rendered)?;
    let report = validate_structured_lifecycle_bundle(&bundle);
    let mut issues = report
        .issues
        .iter()
        .map(structured_lifecycle_issue_json)
        .collect::<Vec<_>>();
    if rendered.is_empty() {
        issues.push(json!({
            "code": "structured_lifecycle.render.empty",
            "message": "bundle did not render any specs",
            "field_path": "specs",
        }));
    }
    let valid = issues.is_empty();
    print_json(with_receipt_hash(json!({
        "schema": STRUCTURED_LIFECYCLE_RENDER_SCHEMA,
        "command": "lifecycle render-markdown",
        "source": source,
        "source_format": source_format(Path::new(&source)),
        "bundle_hash": structured_lifecycle_bundle_hash(&bundle),
        "artifacts": artifacts,
        "issues": issues,
        "valid": valid,
        "verdict": if valid {"PASS"} else {"FAIL"},
    })))
}

fn run_lifecycle_import_markdown(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.structured_lifecycle.markdown_projection.lossy_import]
    // r[impl cairn.structured_lifecycle.markdown_projection.compatibility]
    // r[impl cairn.structured_lifecycle.migration]
    // r[impl cairn.structured_lifecycle.migration.preview_first]
    // r[impl cairn.structured_lifecycle.migration.mutation_manifest]
    // r[impl cairn.structured_lifecycle.migration.no_silent_delete]
    let root = read_option(args.clone(), "--root", ".");
    let out =
        read_string_option(&args, "--out").ok_or("lifecycle import-markdown requires --out")?;
    let execute = has_flag(&args, "--execute");
    let selected = detect_layout(&root);
    let sources = collect_validation_spec_sources(&root, selected.layout)?;
    let import = import_markdown_sources(&sources);
    let bundle = import.bundle;
    let rendered = serde_json::to_string_pretty(&bundle)? + "\n";
    let out_path = root.join(&out);
    let before_hash = fs::read(&out_path).ok().map(|bytes| blake3_hex(&bytes));
    let mut mutation_manifest = Value::Null;
    if execute {
        if let Some(parent) = out_path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&out_path, rendered.as_bytes())?;
        mutation_manifest = json!({
            "entries": [{
                "path": out,
                "before_hash": before_hash,
                "after_hash": blake3_hex(rendered.as_bytes()),
            }],
        });
    }
    let report = validate_structured_lifecycle_bundle(&bundle);
    let mut issues = report
        .issues
        .iter()
        .map(structured_lifecycle_issue_json)
        .collect::<Vec<_>>();
    issues.extend(import.issues);
    let valid = issues.is_empty();
    print_json(with_receipt_hash(json!({
        "schema": STRUCTURED_LIFECYCLE_IMPORT_SCHEMA,
        "command": "lifecycle import-markdown",
        "root": ".",
        "layout": cairn_core::layout_name(selected.layout),
        "out": out,
        "dry_run": !execute,
        "mutated": execute,
        "source_count": sources.len(),
        "bundle_hash": structured_lifecycle_bundle_hash(&bundle),
        "mutation_manifest": mutation_manifest,
        "issues": issues,
        "valid": valid,
        "verdict": if valid {"PASS"} else {"WARN"},
    })))
}

fn lifecycle_check_receipt(
    command: &str,
    source: &str,
    bundle: &Value,
    mut issues: Vec<Value>,
) -> Value {
    let report = validate_structured_lifecycle_bundle(bundle);
    issues.extend(report.issues.iter().map(structured_lifecycle_issue_json));
    let valid = issues.is_empty();
    json!({
        "schema": STRUCTURED_LIFECYCLE_EXPORT_SCHEMA,
        "command": command,
        "source": source,
        "source_format": source_format(Path::new(source)),
        "bundle_schema": bundle.get("schema").cloned().unwrap_or(Value::Null),
        "bundle_hash": structured_lifecycle_bundle_hash(bundle),
        "issues": issues,
        "valid": valid,
        "verdict": if valid {"PASS"} else {"FAIL"},
    })
}

fn load_lifecycle_bundle(source: &Path) -> Result<Value, Box<dyn Error>> {
    let output = if source.extension().and_then(|value| value.to_str()) == Some("ncl") {
        // r[impl cairn.nickel_export_cutover.boundary]
        // r[impl cairn.nickel_export_cutover.dual_run]
        export_json_dual_run(
            "cairn.structured-lifecycle",
            source,
            Path::new("target/cairn-lifecycle-export.json"),
            false,
        )?
        .bytes
    } else {
        fs::read(source)?
    };
    Ok(serde_json::from_slice(&output)?)
}

fn source_format(source: &Path) -> &'static str {
    if source.extension().and_then(|value| value.to_str()) == Some("ncl") {
        "nickel"
    } else {
        "json"
    }
}

fn render_bundle_markdown(bundle: &Value) -> Vec<RenderedSpec> {
    let Some(specs) = bundle.get("specs").and_then(Value::as_array) else {
        return Vec::new();
    };
    specs.iter().filter_map(render_spec_markdown).collect()
}

fn render_spec_markdown(spec: &Value) -> Option<RenderedSpec> {
    let id = spec.get("id")?.as_str()?.to_string();
    let purpose = spec
        .get("purpose")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut markdown = format!(
        "# {} Specification\n\n## Purpose\n\n{}\n\n## Requirements\n",
        title_case(&id),
        purpose
    );
    if let Some(requirements) = spec.get("requirements").and_then(Value::as_array) {
        for requirement in requirements {
            markdown.push_str(&render_requirement_markdown(requirement));
        }
    }
    Some(RenderedSpec { id, markdown })
}

fn render_requirement_markdown(requirement: &Value) -> String {
    let id = requirement
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let statement = requirement
        .get("statement")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let strength = requirement
        .get("strength")
        .and_then(Value::as_str)
        .unwrap_or("MUST");
    let mut markdown =
        format!("\n### Requirement: {statement}\nr[{id}] The system {strength} {statement}.\n");
    if let Some(scenarios) = requirement.get("scenarios").and_then(Value::as_array) {
        for scenario in scenarios {
            markdown.push_str(&render_scenario_markdown(id, scenario));
        }
    }
    markdown
}

fn render_scenario_markdown(requirement_id: &str, scenario: &Value) -> String {
    let id = scenario
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or("scenario");
    let mut markdown = format!(
        "\n#### Scenario: {}\nr[{requirement_id}.{id}]\n",
        title_case(id)
    );
    append_scenario_lines(&mut markdown, scenario, "given", "GIVEN");
    append_scenario_lines(&mut markdown, scenario, "when", "WHEN");
    append_scenario_lines(&mut markdown, scenario, "then", "THEN");
    markdown
}

fn append_scenario_lines(markdown: &mut String, scenario: &Value, field: &str, keyword: &str) {
    if let Some(lines) = scenario.get(field).and_then(Value::as_array) {
        for line in lines.iter().filter_map(Value::as_str) {
            markdown.push_str(&format!("- {keyword} {line}\n"));
        }
    }
}

fn write_rendered_markdown(
    out_dir: Option<&str>,
    rendered: &[RenderedSpec],
) -> Result<Vec<Value>, Box<dyn Error>> {
    let mut artifacts = Vec::new();
    for spec in rendered {
        let relative = PathBuf::from("specs")
            .join(&spec.id)
            .join(MARKDOWN_SPEC_FILE);
        if let Some(out_dir) = out_dir {
            let path = Path::new(out_dir).join(&relative);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, &spec.markdown)?;
        }
        artifacts.push(json!({
            "path": relative.to_string_lossy().replace('\\', "/"),
            "hash": blake3_hex(spec.markdown.as_bytes()),
        }));
    }
    Ok(artifacts)
}

struct RenderedSpec {
    id: String,
    markdown: String,
}

struct ImportedBundle {
    bundle: Value,
    issues: Vec<Value>,
}

fn import_markdown_sources(sources: &[cairn_core::SpecSource]) -> ImportedBundle {
    let mut issues = Vec::new();
    let specs = sources
        .iter()
        .map(|source| import_markdown_spec(source, &mut issues))
        .collect::<Vec<_>>();
    ImportedBundle {
        bundle: json!({
            "schema": STRUCTURED_LIFECYCLE_SCHEMA_V1,
            "format_metadata": {
                "source_format": "markdown-import",
                "importer": "cairn lifecycle import-markdown",
            },
            "specs": specs,
            "changes": [],
        }),
        issues,
    }
}

fn import_markdown_spec(source: &cairn_core::SpecSource, issues: &mut Vec<Value>) -> Value {
    let requirements = markdown_requirements(&source.markdown, &source.path, issues);
    json!({
        "id": source.id,
        "purpose": markdown_purpose(&source.markdown),
        "source_path": source.path,
        "requirements": requirements,
    })
}

fn markdown_purpose(markdown: &str) -> String {
    let mut in_purpose = false;
    let mut lines = Vec::new();
    for line in markdown.lines() {
        if line.trim() == "## Purpose" {
            in_purpose = true;
            continue;
        }
        if in_purpose && line.starts_with("## ") {
            break;
        }
        if in_purpose && !line.trim().is_empty() {
            lines.push(line.trim().to_string());
        }
    }
    if lines.is_empty() {
        "Imported Markdown lifecycle spec".to_string()
    } else {
        lines.join(" ")
    }
}

fn markdown_requirements(markdown: &str, path: &str, issues: &mut Vec<Value>) -> Vec<Value> {
    let lines = markdown.lines().collect::<Vec<_>>();
    let mut requirements = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        let line = lines[index].trim();
        if line.starts_with("### Requirement:") {
            let (requirement, next_index) = import_requirement(&lines, index, path, issues);
            requirements.push(requirement);
            index = next_index;
        } else {
            index += 1;
        }
    }
    if requirements.is_empty() {
        issues.push(json!({
            "code": "structured_lifecycle.import.no_requirements",
            "message": "Markdown import found no requirement headings",
            "field_path": path,
        }));
    }
    requirements
}

fn import_requirement(
    lines: &[&str],
    start: usize,
    path: &str,
    issues: &mut Vec<Value>,
) -> (Value, usize) {
    let heading = lines[start].trim();
    let id = marker_id(heading).unwrap_or_else(|| fallback_id("imported.requirement", start));
    let statement = heading
        .trim_start_matches("### Requirement:")
        .trim()
        .to_string();
    let mut scenarios = Vec::new();
    let mut index = start + 1;
    while index < lines.len() && !lines[index].trim().starts_with("### Requirement:") {
        if lines[index].trim().starts_with("#### Scenario:") {
            let (scenario, next_index) = import_scenario(lines, index);
            scenarios.push(scenario);
            index = next_index;
        } else {
            index += 1;
        }
    }
    if scenarios.is_empty() {
        issues.push(json!({
            "code": "structured_lifecycle.import.lossy_missing_scenario",
            "message": "Markdown requirement had no scenario block; inserted placeholder scenario",
            "field_path": path,
        }));
        scenarios.push(json!({
            "id": "imported",
            "given": ["imported Markdown requirement"],
            "when": ["Cairn migrates the requirement"],
            "then": ["operator reviews lossy conversion diagnostics"]
        }));
    }
    (
        json!({
            "id": id,
            "strength": markdown_strength(&statement),
            "statement": statement,
            "scenarios": scenarios,
        }),
        index,
    )
}

fn import_scenario(lines: &[&str], start: usize) -> (Value, usize) {
    let heading = lines[start].trim();
    let id = marker_id(heading).unwrap_or_else(|| fallback_id("imported.scenario", start));
    let mut given = Vec::new();
    let mut when = Vec::new();
    let mut then = Vec::new();
    let mut index = start + 1;
    while index < lines.len()
        && !lines[index].trim().starts_with("#### Scenario:")
        && !lines[index].trim().starts_with("### Requirement:")
    {
        let line = lines[index].trim();
        if let Some(rest) = line.strip_prefix(GIVEN_PREFIX) {
            given.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix(WHEN_PREFIX) {
            when.push(rest.to_string());
        } else if let Some(rest) = line.strip_prefix(THEN_PREFIX) {
            then.push(rest.to_string());
        }
        index += 1;
    }
    if given.is_empty() {
        given.push("imported Markdown scenario".to_string());
    }
    if when.is_empty() {
        when.push("Cairn migrates the scenario".to_string());
    }
    if then.is_empty() {
        then.push("operator reviews imported scenario content".to_string());
    }
    (
        json!({"id": id, "given": given, "when": when, "then": then}),
        index,
    )
}

fn marker_id(line: &str) -> Option<String> {
    let marker_start = line.find("r[")? + "r[".len();
    let marker_end = line[marker_start..].find(']')? + marker_start;
    Some(line[marker_start..marker_end].to_string())
}

fn fallback_id(prefix: &str, index: usize) -> String {
    format!("{prefix}.{index}")
}

fn markdown_strength(statement: &str) -> &'static str {
    for strength in ["MUST", "SHALL", "SHOULD", "MAY"] {
        if statement.contains(strength) {
            return strength;
        }
    }
    "SHOULD"
}

fn title_case(id: &str) -> String {
    id.split(['-', '_', '.'])
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

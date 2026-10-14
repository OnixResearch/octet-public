use std::fs;
use std::path::Path;
use std::path::PathBuf;

use serde::Serialize;

use crate::cli::FingerprintArgs;
use crate::cli::FingerprintMode;
use crate::cli::OutputFormat;
use crate::determinism;
use crate::evidence;

const HASH_PREFIX: &str = "b3:";
const COMMENT_MARKER: &str = "// octet-fingerprint:";
const LEGACY_SLOTCAR_COMMENT_MARKER: &str = "// slotcar-fingerprint:";
const LEGACY_TIGERSTYLE_COMMENT_MARKER: &str = "// tigerstyle-fingerprint:";
const ATTR_MARKER: &str = "#[fingerprint(";
const REVIEW_KEY: &str = "review=";
const EVIDENCE_KEY: &str = "evidence=";
const MACRO_ALLOW_KEY: &str = "macro=allow";
const FINGERPRINT_PROVENANCE_SOURCE_ID: &str = "octet.fingerprint.review-manifest";
const FINGERPRINT_REVIEW_ACTION: &str = "review fingerprint manifest records before using them as release evidence";

#[derive(Debug, Clone, Serialize)]
struct FunctionRecord {
    path: String,
    name: String,
    stored_hash: Option<String>,
    computed_hash: String,
    normalized_body_hash: String,
    semantic_id: String,
    dependency_set_hash: String,
    dependency_identity_status: String,
    tool_version: &'static str,
    status: FingerprintStatus,
    review: Option<String>,
    evidence: Option<String>,
    macro_calls: bool,
    macro_warning: bool,
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
enum FingerprintStatus {
    Match,
    Mismatch,
    Missing,
    Written,
}

#[derive(Debug, Serialize)]
struct FingerprintReport {
    schema_version: u32,
    mode: &'static str,
    records: Vec<FunctionRecord>,
}

#[derive(Debug, Serialize)]
struct FingerprintReviewManifest {
    schema_version: u32,
    tool_name: &'static str,
    tool_version: &'static str,
    mode: &'static str,
    input_paths: Vec<String>,
    manifest_hash: String,
    records: Vec<ReviewManifestRecord>,
}

#[derive(Debug, Serialize)]
struct ReviewManifestRecord {
    stable_id: String,
    path: String,
    name: String,
    status: FingerprintStatus,
    stored_hash: Option<String>,
    computed_hash: String,
    normalized_body_hash: String,
    semantic_id: String,
    dependency_set_hash: String,
    dependency_identity_status: String,
    tool_version: &'static str,
    review: Option<String>,
    evidence: Option<String>,
    requires_review: bool,
    macro_warning: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct FunctionSpan {
    pub(crate) name: String,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) marker: Option<Marker>,
    pub(crate) macro_calls: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct Marker {
    start: usize,
    end: usize,
    stored_hash: String,
    review: Option<String>,
    evidence: Option<String>,
    macro_allowed: bool,
}

impl Marker {
    pub(crate) fn stored_hash(&self) -> &str {
        &self.stored_hash
    }

    pub(crate) fn review(&self) -> Option<&str> {
        self.review.as_deref()
    }

    pub(crate) fn evidence(&self) -> Option<&str> {
        self.evidence.as_deref()
    }
}

#[allow(clippy::too_many_lines)]
pub fn run(args: &FingerprintArgs) -> Result<i32, String> {
    let paths = if args.paths.is_empty() {
        vec![PathBuf::from(".")]
    } else {
        args.paths.clone()
    };
    let input_paths: Vec<String> = paths.iter().map(|path| determinism::normalize_path(path)).collect();
    let mut files = Vec::new();
    for path in paths {
        collect_rust_files(&path, &mut files)?;
    }
    files.sort();
    files.dedup();

    let mut records = Vec::new();
    for path in files {
        let source = fs::read_to_string(&path).map_err(|error| format!("read {}: {error}", path.display()))?;
        let spans = find_functions(&source);
        let mut replacements = Vec::new();
        for span in spans {
            let function_source = &source[span.start..span.end];
            let computed_hash = function_hash(function_source);
            let normalized_body_hash = determinism::full_hash(&normalize_tokens(function_source));
            let dependency_set_hash = dependency_set_hash(function_source);
            let dependency_identity_status = dependency_identity_status(function_source).to_owned();
            let semantic_id = determinism::short_hash(&format!(
                "body={normalized_body_hash}\ndeps={dependency_set_hash}\ndep_status={dependency_identity_status}\ntool={}\n",
                env!("CARGO_PKG_VERSION"),
            ));
            let marker = span.marker.clone();
            if !matches!(args.mode, FingerprintMode::Write) && marker.is_none() {
                continue;
            }
            let status = match marker.as_ref().map(|m| m.stored_hash.as_str()) {
                Some(stored) if stored == computed_hash => FingerprintStatus::Match,
                Some(_) => FingerprintStatus::Mismatch,
                None => FingerprintStatus::Missing,
            };
            let macro_warning = span.macro_calls && !marker.as_ref().is_some_and(|m| m.macro_allowed);
            let record = FunctionRecord {
                path: path.display().to_string(),
                name: span.name.clone(),
                stored_hash: marker.as_ref().map(|m| m.stored_hash.clone()),
                computed_hash: computed_hash.clone(),
                normalized_body_hash,
                semantic_id,
                dependency_set_hash,
                dependency_identity_status,
                tool_version: env!("CARGO_PKG_VERSION"),
                status: if matches!(args.mode, FingerprintMode::Write | FingerprintMode::Update)
                    && status != FingerprintStatus::Match
                {
                    FingerprintStatus::Written
                } else {
                    status
                },
                review: marker.as_ref().and_then(|m| m.review.clone()),
                evidence: marker.as_ref().and_then(|m| m.evidence.clone()),
                macro_calls: span.macro_calls,
                macro_warning,
            };
            if matches!(args.mode, FingerprintMode::Write | FingerprintMode::Update)
                && status != FingerprintStatus::Match
            {
                let review = marker.as_ref().and_then(|m| m.review.as_deref()).unwrap_or("TODO");
                let evidence = marker.as_ref().and_then(|m| m.evidence.as_deref()).unwrap_or("TODO");
                let macro_suffix = if marker.as_ref().is_some_and(|m| m.macro_allowed) {
                    "; macro=allow"
                } else if span.macro_calls {
                    "; macro=review-required"
                } else {
                    ""
                };
                let line = format!(
                    "// octet-fingerprint: b3:{computed_hash}; review={review}; evidence={evidence}{macro_suffix}\n"
                );
                if let Some(marker) = marker {
                    replacements.push((marker.start, marker.end, line));
                } else {
                    replacements.push((span.start, span.start, line));
                }
            }
            records.push(record);
        }
        if !replacements.is_empty() {
            let mut updated = source;
            for (start, end, replacement) in replacements.into_iter().rev() {
                updated.replace_range(start..end, &replacement);
            }
            fs::write(&path, updated).map_err(|error| format!("write {}: {error}", path.display()))?;
        }
    }

    let report = FingerprintReport {
        schema_version: 1,
        mode: mode_name(args.mode),
        records,
    };
    if let Some(path) = args.review_manifest.as_deref() {
        write_review_manifest(path, &report, input_paths)?;
    }
    match args.output_format {
        OutputFormat::Human => print_human(&report),
        OutputFormat::Json => println!("{}", serde_json::to_string_pretty(&report).map_err(|error| error.to_string())?),
        OutputFormat::Sarif => return Err("fingerprint does not support SARIF output; use human or json".to_owned()),
    }

    let failing = report.records.iter().any(|record| {
        matches!(args.mode, FingerprintMode::Check | FingerprintMode::Diff)
            && (record.status != FingerprintStatus::Match
                || record.macro_warning
                || record.review.is_none()
                || record.evidence.is_none())
    });
    Ok(i32::from(failing))
}

fn write_review_manifest(path: &Path, report: &FingerprintReport, input_paths: Vec<String>) -> Result<(), String> {
    let mut records: Vec<ReviewManifestRecord> = report
        .records
        .iter()
        .map(|record| ReviewManifestRecord {
            stable_id: fingerprint_record_stable_id(record),
            path: record.path.clone(),
            name: record.name.clone(),
            status: record.status,
            stored_hash: record.stored_hash.clone(),
            computed_hash: record.computed_hash.clone(),
            normalized_body_hash: record.normalized_body_hash.clone(),
            semantic_id: record.semantic_id.clone(),
            dependency_set_hash: record.dependency_set_hash.clone(),
            dependency_identity_status: record.dependency_identity_status.clone(),
            tool_version: record.tool_version,
            review: record.review.clone(),
            evidence: record.evidence.clone(),
            requires_review: record.status != FingerprintStatus::Match
                || record.review.is_none()
                || record.evidence.is_none()
                || record.macro_warning,
            macro_warning: record.macro_warning,
        })
        .collect();
    records.sort_by(|left, right| {
        left.path
            .cmp(&right.path)
            .then(left.name.cmp(&right.name))
            .then(left.stable_id.cmp(&right.stable_id))
    });

    let hash_payload = serde_json::json!({
        "mode": report.mode,
        "input_paths": &input_paths,
        "records": &records,
    });
    let manifest = FingerprintReviewManifest {
        schema_version: 1,
        tool_name: "cargo-octet",
        tool_version: env!("CARGO_PKG_VERSION"),
        mode: report.mode,
        input_paths,
        manifest_hash: determinism::full_hash(&hash_payload.to_string()),
        records,
    };
    let contents =
        serde_json::to_string_pretty(&manifest).map_err(|error| format!("write review manifest JSON: {error}"))?;
    let rendered = format!("{contents}\n");
    fs::write(path, &rendered).map_err(|error| format!("write {}: {error}", path.display()))?;
    write_review_manifest_provenance(path, rendered.as_bytes(), &manifest.input_paths)
}

fn write_review_manifest_provenance(path: &Path, contents: &[u8], input_paths: &[String]) -> Result<(), String> {
    let provenance_path = path.with_extension("provenance.jsonl");
    let record = evidence::ProvenanceRecord {
        schema_version: evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: FINGERPRINT_PROVENANCE_SOURCE_ID.to_owned(),
        checker_id: evidence::CHECKER_OCTET_FINGERPRINT.to_owned(),
        evidence_tier: evidence::EvidenceTier::ManualReviewRequired,
        artifact_locator: path.display().to_string(),
        blake3: evidence::artifact_blake3_hex(contents),
        checked_scope: format!("fingerprint:{}", input_paths.join(",")),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: Some(FINGERPRINT_REVIEW_ACTION.to_owned()),
        parent_artifacts: Vec::new(),
    };
    let line = serde_json::to_string(&record).map_err(|error| format!("write fingerprint provenance JSON: {error}"))?;
    fs::write(&provenance_path, format!("{line}\n"))
        .map_err(|error| format!("write {}: {error}", provenance_path.display()))
}

fn fingerprint_record_stable_id(record: &FunctionRecord) -> String {
    record.semantic_id.clone()
}

pub(crate) fn collect_rust_files(path: &Path, files: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path.to_path_buf());
        }
        return Ok(());
    }
    if path.is_dir() {
        for entry in fs::read_dir(path).map_err(|error| format!("read dir {}: {error}", path.display()))? {
            let entry = entry.map_err(|error| format!("read dir entry {}: {error}", path.display()))?;
            let child = entry.path();
            if child.file_name().is_some_and(|name| name == "target" || name == ".git") {
                continue;
            }
            collect_rust_files(&child, files)?;
        }
    }
    Ok(())
}

pub(crate) fn find_functions(source: &str) -> Vec<FunctionSpan> {
    let mut spans = Vec::new();
    let bytes = source.as_bytes();
    let macro_rules_ranges = macro_rules_ranges(source);
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find("fn ") {
        let fn_pos = cursor + relative;
        if fn_pos > 0 && is_ident(bytes[fn_pos - 1] as char) {
            cursor = fn_pos + 3;
            continue;
        }
        if macro_rules_ranges.iter().any(|(start, end)| (*start..*end).contains(&fn_pos)) {
            cursor = fn_pos + 3;
            continue;
        }
        let name_start = fn_pos + 3;
        if !source[name_start..].chars().next().is_some_and(is_ident) {
            cursor = name_start;
            continue;
        }
        let Some(name_end) = source[name_start..].find(|c: char| !is_ident(c)).map(|offset| name_start + offset) else {
            break;
        };
        let next_open_brace = source[name_end..].find('{').map(|offset| name_end + offset);
        let next_semicolon = source[name_end..].find(';').map(|offset| name_end + offset);
        if next_semicolon.is_some_and(|semicolon| next_open_brace.is_none_or(|open_brace| semicolon < open_brace)) {
            cursor = next_semicolon.unwrap_or(name_end) + 1;
            continue;
        }
        let Some(open_brace) = next_open_brace else {
            break;
        };
        let Some(end) = matching_brace(source, open_brace) else {
            break;
        };
        let item_start = function_item_start(source, fn_pos);
        let marker = preceding_marker(source, item_start);
        let body = &source[open_brace..end];
        spans.push(FunctionSpan {
            name: source[name_start..name_end].to_owned(),
            start: marker.as_ref().map_or(item_start, |m| m.end),
            end,
            marker,
            macro_calls: contains_macro_call(body),
        });
        cursor = end;
    }
    spans
}

fn macro_rules_ranges(source: &str) -> Vec<(usize, usize)> {
    let mut ranges = Vec::new();
    let mut cursor = 0;
    while let Some(relative) = source[cursor..].find("macro_rules!") {
        let macro_start = cursor + relative;
        let Some(open_brace) = source[macro_start..].find('{').map(|offset| macro_start + offset) else {
            break;
        };
        let Some(end) = matching_brace(source, open_brace) else {
            break;
        };
        ranges.push((macro_start, end));
        cursor = end;
    }
    ranges
}

fn function_item_start(source: &str, fn_pos: usize) -> usize {
    let mut line_start = source[..fn_pos].rfind('\n').map_or(0, |index| index + 1);

    loop {
        let previous_end = line_start.saturating_sub(1);
        if previous_end == 0 {
            return line_start;
        }

        let previous_start = source[..previous_end].rfind('\n').map_or(0, |index| index + 1);
        let previous = source[previous_start..previous_end].trim();
        if previous.starts_with("#[") {
            line_start = previous_start;
            continue;
        }
        if (previous.starts_with(']') || previous.starts_with(')'))
            && let Some(attribute_start) = multiline_attribute_start(source, previous_start)
        {
            line_start = attribute_start;
            continue;
        }

        return line_start;
    }
}

fn multiline_attribute_start(source: &str, before_line_start: usize) -> Option<usize> {
    let mut line_end = before_line_start.saturating_sub(1);
    loop {
        let line_start = source[..line_end].rfind('\n').map_or(0, |index| index + 1);
        let line = source[line_start..line_end].trim();
        if line.starts_with("#[") {
            return Some(line_start);
        }
        if line.is_empty() || (!line.ends_with(',') && !line.contains('=')) {
            return None;
        }
        if line_start == 0 {
            return None;
        }
        line_end = line_start.saturating_sub(1);
    }
}

fn preceding_marker(source: &str, item_start: usize) -> Option<Marker> {
    let prefix = &source[..item_start];
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let previous_end = line_start.saturating_sub(1);
    let previous_start = source[..previous_end].rfind('\n').map_or(0, |index| index + 1);
    let previous = source[previous_start..previous_end].trim();
    if previous.starts_with(COMMENT_MARKER)
        || previous.starts_with(LEGACY_SLOTCAR_COMMENT_MARKER)
        || previous.starts_with(LEGACY_TIGERSTYLE_COMMENT_MARKER)
    {
        let line_end = if previous_end < source.len() {
            previous_end + 1
        } else {
            previous_end
        };
        return parse_marker(previous, previous_start, line_end);
    }
    if previous.starts_with(ATTR_MARKER) {
        let line_end = if previous_end < source.len() {
            previous_end + 1
        } else {
            previous_end
        };
        return parse_marker(previous, previous_start, line_end);
    }
    None
}

fn parse_marker(line: &str, start: usize, end: usize) -> Option<Marker> {
    let hash_start = line.find(HASH_PREFIX)? + HASH_PREFIX.len();
    let hash = line[hash_start..].chars().take_while(char::is_ascii_hexdigit).collect::<String>();
    if hash.is_empty() {
        return None;
    }
    Some(Marker {
        start,
        end,
        stored_hash: hash,
        review: value_after(line, REVIEW_KEY),
        evidence: value_after(line, EVIDENCE_KEY),
        macro_allowed: line.contains(MACRO_ALLOW_KEY),
    })
}

fn value_after(line: &str, key: &str) -> Option<String> {
    let bare_key = key.trim_end_matches('=');
    let start = line.find(bare_key)? + bare_key.len();
    let value_start = line[start..].find('=')? + start + 1;
    let value = line[value_start..]
        .split([';', ',', ')', ']'])
        .next()
        .unwrap_or("")
        .trim()
        .trim_matches('"')
        .to_owned();
    (!value.is_empty() && value != "TODO").then_some(value)
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0u32;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth = depth.checked_sub(1)?;
                if depth == 0 {
                    return Some(open + offset + ch.len_utf8());
                }
            }
            _ => {}
        }
    }
    None
}

fn function_hash(function_source: &str) -> String {
    let normalized = normalize_tokens(function_source);
    blake3::hash(normalized.as_bytes()).to_hex().to_string()
}

pub(crate) fn dependency_set_hash(function_source: &str) -> String {
    let mut calls = Vec::new();
    for token in normalize_tokens(function_source).split_whitespace() {
        if token.chars().next().is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_') {
            calls.push(token.to_owned());
        }
    }
    calls.sort();
    calls.dedup();
    determinism::full_hash(&calls.join("\n"))
}

pub(crate) fn dependency_identity_status(function_source: &str) -> &'static str {
    if contains_macro_call(function_source) {
        "unsupported-macro-context"
    } else {
        "source-scanner-token-set"
    }
}

pub(crate) fn normalize_tokens(source: &str) -> String {
    let without_comments = strip_comments_and_strings(source);
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in without_comments.chars() {
        if is_ident(ch) {
            current.push(ch);
        } else {
            if !current.is_empty() {
                tokens.push(std::mem::take(&mut current));
            }
            if !ch.is_whitespace() {
                tokens.push(ch.to_string());
            }
        }
    }
    if !current.is_empty() {
        tokens.push(current);
    }
    tokens.join(" ")
}

fn strip_comments_and_strings(source: &str) -> String {
    let mut out = String::new();
    for line in source.lines() {
        let code = line.split("//").next().unwrap_or("");
        out.push_str(code);
        out.push('\n');
    }
    out
}

fn contains_macro_call(source: &str) -> bool {
    let stripped = strip_comments_and_strings(source);
    let chars = stripped.char_indices().collect::<Vec<_>>();
    for (index, (byte_index, ch)) in chars.iter().enumerate() {
        if *ch != '!' {
            continue;
        }
        let previous = chars[..index].iter().rev().find_map(|(_, c)| (!c.is_whitespace()).then_some(*c));
        let next = stripped[*byte_index + ch.len_utf8()..].chars().find(|c| !c.is_whitespace());
        if previous.is_some_and(is_ident) && next.is_some_and(|c| matches!(c, '(' | '[' | '{')) {
            return true;
        }
    }
    false
}

fn is_ident(ch: char) -> bool {
    ch.is_ascii_alphanumeric() || ch == '_'
}

fn mode_name(mode: FingerprintMode) -> &'static str {
    match mode {
        FingerprintMode::Check => "check",
        FingerprintMode::Write => "write",
        FingerprintMode::Update => "update",
        FingerprintMode::Diff => "diff",
    }
}

fn print_human(report: &FingerprintReport) {
    println!("octet fingerprint {}", report.mode);
    for record in &report.records {
        println!(
            "{} {} {} semantic_id={} stored={} computed=b3:{} normalized_body_hash={} dependency_identity_status={} review={} evidence={} macro_warning={}",
            record.path,
            record.name,
            status_name(record.status),
            record.semantic_id,
            record.stored_hash.as_deref().unwrap_or("<none>"),
            record.computed_hash,
            record.normalized_body_hash,
            record.dependency_identity_status,
            record.review.as_deref().unwrap_or("<missing>"),
            record.evidence.as_deref().unwrap_or("<missing>"),
            record.macro_warning,
        );
    }

    let match_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Match).count();
    let mismatch_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Mismatch).count();
    let missing_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Missing).count();
    let written_count = report.records.iter().filter(|record| record.status == FingerprintStatus::Written).count();
    let macro_warning_count = report.records.iter().filter(|record| record.macro_warning).count();
    let metadata_missing_count =
        report.records.iter().filter(|record| record.review.is_none() || record.evidence.is_none()).count();
    println!(
        "summary total={} match={} mismatch={} missing={} written={} macro_warnings={} metadata_missing={}",
        report.records.len(),
        match_count,
        mismatch_count,
        missing_count,
        written_count,
        macro_warning_count,
        metadata_missing_count,
    );
}

fn status_name(status: FingerprintStatus) -> &'static str {
    match status {
        FingerprintStatus::Match => "match",
        FingerprintStatus::Mismatch => "mismatch",
        FingerprintStatus::Missing => "missing",
        FingerprintStatus::Written => "written",
    }
}

#[cfg(test)]
mod tests {
    use super::contains_macro_call;
    use super::find_functions;
    use super::function_hash;
    use super::normalize_tokens;

    #[test]
    fn formatting_and_comments_do_not_change_hash() {
        let one = "fn checked(input: u32) -> u32 { // comment\n input + 1 }";
        let two = "fn checked( input:u32)->u32{input+1}";
        assert_eq!(function_hash(one), function_hash(two));
    }

    #[test]
    fn token_changes_update_hash() {
        assert_ne!(function_hash("fn f() -> u32 { 1 }"), function_hash("fn f() -> u32 { 2 }"));
    }

    #[test]
    fn marker_is_not_part_of_function_span() {
        let source = "// octet-fingerprint: b3:abc; review=R; evidence=E\nfn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("fn f"));
    }

    #[test]
    fn legacy_marker_is_not_part_of_function_span() {
        let source = "// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\nfn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("fn f"));
    }

    #[test]
    fn marker_before_attributes_attaches_to_function() {
        let source =
            "// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\n#[inline]\npub const fn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(
            normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] pub const fn f")
        );
    }

    #[test]
    fn insertion_start_includes_visibility_before_fn_keyword() {
        let source = "#[inline]\npub fn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_none());
        assert_eq!(functions[0].start, 0);
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] pub fn f"));
    }

    #[test]
    fn marker_before_multiline_attribute_attaches_to_function() {
        let source = "// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\n#[inline]\n#[allow(\n    ambient_clock,\n    reason = \"clock boundary\"\n)]\npub fn f() -> u32 { 1 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] # [ allow"));
    }

    #[test]
    fn macro_detection_ignores_unary_not_and_inequality() {
        assert!(contains_macro_call("{ assert!(value != 0); }"));
        assert!(contains_macro_call("{ path::macro_name! { value } }"));
        assert!(!contains_macro_call("{ if !is_ready && value != 0 { return true; } }"));
        assert!(!contains_macro_call("{ let message = \"not a macro!\"; }"));
    }

    #[test]
    fn macro_generated_functions_are_not_source_fingerprinted() {
        let source = "macro_rules! define_checked {\n    ($name:ident) => {\n        pub fn $name(input: u32) -> u32 { input + 1 }\n    };\n}\n\ndefine_checked!(generated_checked);\n\n// tigerstyle-fingerprint: b3:abc; review=R; evidence=E\npub fn explicit_wrapper(input: u32) -> u32 { generated_checked(input) }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "explicit_wrapper");
        assert!(functions[0].marker.is_some());
    }

    #[test]
    fn literal_functions_inside_macro_rules_are_not_source_fingerprinted() {
        let source = "macro_rules! define_literal {\n    () => {\n        pub fn generated_literal() -> u32 { 1 }\n    };\n}\n\npub fn explicit_function() -> u32 { 2 }\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "explicit_function");
    }

    #[test]
    fn trait_declarations_do_not_swallow_later_impl_methods() {
        let source = "trait T {\n    fn f(&self) -> u64;\n}\nimpl T for S {\n    // tigerstyle-fingerprint: b3:abc; review=R; evidence=E\n    #[inline]\n    fn f(&self) -> u64 { 1 }\n}\nstruct S;\n";
        let functions = find_functions(source);
        assert_eq!(functions.len(), 1);
        assert_eq!(functions[0].name, "f");
        assert!(functions[0].marker.is_some());
        assert!(normalize_tokens(&source[functions[0].start..functions[0].end]).starts_with("# [ inline ] fn f"));
    }
}

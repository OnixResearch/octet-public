use std::{collections::BTreeSet, error::Error, fs, path::PathBuf};

use cairn_core::canonical_preserves_hash;
use serde_json::{json, Value};

use crate::support::{has_flag, print_json, read_option, with_receipt_hash};

pub(crate) fn run_preserves_corpus(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.iter();
    let Some(subcommand) = iter.next() else {
        return Err("preserves-corpus requires a subcommand: update|explain".into());
    };
    match subcommand.as_str() {
        "update" => run_preserves_corpus_update(args),
        "explain" => run_preserves_corpus_explain(args),
        unknown => Err(format!("unknown preserves-corpus subcommand: {unknown}").into()),
    }
}

fn run_preserves_corpus_explain(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let path = read_option(
        args,
        "--path",
        "fixtures/preserves-canonicalization/corpus.json",
    );
    let receipt = preserves_corpus_explain_receipt(path);
    let valid = receipt["valid"].as_bool().unwrap_or(false);
    print_json(with_receipt_hash(receipt))?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

// r[impl cairn.determinism.preserves_corpus_explain_cli.command]
// r[impl cairn.determinism.preserves_corpus_explain_cli.read_only]
fn preserves_corpus_explain_receipt(path: PathBuf) -> Value {
    let path_text = path.to_string_lossy().to_string();
    let mut issues = Vec::new();
    let input = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            issues.push(json!({
                "code": "preserves_corpus.read_failed",
                "message": error.to_string(),
            }));
            return base_explain_receipt(path_text, false, issues, Vec::new());
        }
    };
    let corpus: Value = match serde_json::from_str(&input) {
        Ok(value) => value,
        Err(error) => {
            issues.push(json!({
                "code": "preserves_corpus.parse_failed",
                "message": error.to_string(),
            }));
            return base_explain_receipt(path_text, false, issues, Vec::new());
        }
    };
    let Some(cases) = corpus.as_array() else {
        issues.push(json!({
            "code": "preserves_corpus.not_array",
            "message": "Preserves corpus must be a JSON array",
        }));
        return base_explain_receipt(path_text, false, issues, Vec::new());
    };

    let mut seen_ids = BTreeSet::new();
    let mut case_summaries = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let id = case
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("<case-{index}>"));
        if !seen_ids.insert(id.clone()) {
            issues.push(json!({
                "code": "preserves_corpus.duplicate_case_id",
                "case_id": id,
                "index": index,
            }));
        }

        let description = case
            .get("description")
            .and_then(Value::as_str)
            .unwrap_or("");
        let value_present = case.get("value").is_some();
        if !value_present {
            issues.push(json!({
                "code": "preserves_corpus.case_missing_value",
                "case_id": id,
                "index": index,
            }));
        }
        let expected_hash = case.get("expected_hash").and_then(Value::as_str);
        if expected_hash.is_none() {
            issues.push(json!({
                "code": "preserves_corpus.case_missing_expected_hash",
                "case_id": id,
                "index": index,
            }));
        }
        let expected_hash_shape = expected_hash.map(hash_shape).unwrap_or("missing");
        if expected_hash_shape == "malformed" {
            issues.push(json!({
                "code": "preserves_corpus.expected_hash_malformed",
                "case_id": id,
                "index": index,
            }));
        }

        case_summaries.push(json!({
            "id": id,
            "index": index,
            "description": description,
            "value_present": value_present,
            "value_redacted": true,
            "expected_hash_present": expected_hash.is_some(),
            "expected_hash_shape": expected_hash_shape,
        }));
    }

    let valid = issues.is_empty();
    base_explain_receipt(path_text, valid, issues, case_summaries)
}

fn hash_shape(hash: &str) -> &'static str {
    if hash.len() == 64 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
        "hex64"
    } else {
        "malformed"
    }
}

fn base_explain_receipt(path: String, valid: bool, issues: Vec<Value>, cases: Vec<Value>) -> Value {
    json!({
        "command": "preserves-corpus explain",
        "path": path,
        "mutated": false,
        "valid": valid,
        "case_count": cases.len(),
        "cases": cases,
        "issues": issues,
        "review_guidance": [
            "Values are intentionally redacted; use preserves-corpus update for hash recomputation.",
            "Duplicate ids, missing values, and malformed expected_hash fields require operator review."
        ],
    })
}

fn run_preserves_corpus_update(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let path = read_option(
        args.clone(),
        "--path",
        "fixtures/preserves-canonicalization/corpus.json",
    );
    let execute = has_flag(&args, "--execute");
    let (mut receipt, updated) = preserves_corpus_update_receipt(path.clone(), execute);
    let valid = receipt["valid"].as_bool().unwrap_or(false);
    let should_write = valid && execute && updated.is_some();
    if should_write {
        let updated = updated.expect("updated corpus text exists when should_write");
        fs::write(&path, updated)?;
        receipt["mutated"] = json!(true);
    }
    print_json(with_receipt_hash(receipt))?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

pub(crate) fn preserves_corpus_freshness_receipt(path: PathBuf) -> Value {
    with_receipt_hash(preserves_corpus_update_receipt(path, false).0)
}

// r[impl cairn.determinism.preserves_operator_docs]
// r[impl cairn.determinism.preserves_operator_docs.workflow]
// r[impl cairn.determinism.preserves_operator_docs.discoverability]
// r[impl cairn.determinism.preserves_operator_docs.validation]
// r[impl cairn.determinism.preserves_corpus_ci_guard]
// r[impl cairn.determinism.preserves_corpus_ci_guard.clean_check]
// r[impl cairn.determinism.preserves_corpus_ci_guard.stale_detection]
// r[impl cairn.determinism.preserves_corpus_ci_guard.validation]
// r[impl cairn.determinism.preserves_corpus_update_rail.command]
// r[impl cairn.determinism.preserves_corpus_update_rail.hash_recompute]
// r[impl cairn.determinism.preserves_corpus_update_rail.receipt]
fn preserves_corpus_update_receipt(path: PathBuf, execute: bool) -> (Value, Option<String>) {
    let path_text = path.to_string_lossy().to_string();
    let mut issues = Vec::new();
    let input = match fs::read_to_string(&path) {
        Ok(text) => text,
        Err(error) => {
            issues.push(json!({
                "code": "preserves_corpus.read_failed",
                "message": error.to_string(),
            }));
            return (
                base_receipt(path_text, execute, false, issues, Vec::new()),
                None,
            );
        }
    };
    let corpus: Value = match serde_json::from_str(&input) {
        Ok(value) => value,
        Err(error) => {
            issues.push(json!({
                "code": "preserves_corpus.parse_failed",
                "message": error.to_string(),
            }));
            return (
                base_receipt(path_text, execute, false, issues, Vec::new()),
                None,
            );
        }
    };
    let Some(cases) = corpus.as_array() else {
        issues.push(json!({
            "code": "preserves_corpus.not_array",
            "message": "Preserves corpus must be a JSON array",
        }));
        return (
            base_receipt(path_text, execute, false, issues, Vec::new()),
            None,
        );
    };

    let mut case_summaries = Vec::new();
    let mut replacement_hashes = Vec::new();
    for (index, case) in cases.iter().enumerate() {
        let id = case
            .get("id")
            .and_then(Value::as_str)
            .map(str::to_string)
            .unwrap_or_else(|| format!("<case-{index}>"));
        let Some(value) = case.get("value") else {
            issues.push(json!({
                "code": "preserves_corpus.case_missing_value",
                "case_id": id,
                "index": index,
            }));
            continue;
        };
        let Some(expected_hash) = case.get("expected_hash").and_then(Value::as_str) else {
            issues.push(json!({
                "code": "preserves_corpus.case_missing_expected_hash",
                "case_id": id,
                "index": index,
            }));
            continue;
        };
        match canonical_preserves_hash(value) {
            Ok(computed_hash) => {
                let stale = computed_hash != expected_hash;
                replacement_hashes.push(computed_hash.clone());
                case_summaries.push(json!({
                    "id": id,
                    "index": index,
                    "expected_hash": expected_hash,
                    "computed_hash": computed_hash,
                    "stale": stale,
                }));
            }
            Err(error) => issues.push(json!({
                "code": "preserves_corpus.hash_failed",
                "case_id": id,
                "index": index,
                "message": error,
            })),
        }
    }

    let stale_count = case_summaries
        .iter()
        .filter(|case| case["stale"].as_bool().unwrap_or(false))
        .count();
    let mut valid = issues.is_empty() && (execute || stale_count == 0);
    let updated = if issues.is_empty() && stale_count > 0 {
        match rewrite_expected_hashes_in_order(&input, &replacement_hashes) {
            Ok(text) => Some(text),
            Err(issue) => {
                valid = false;
                issues.push(issue);
                None
            }
        }
    } else {
        None
    };
    let mut receipt = base_receipt(path_text, execute, valid, issues, case_summaries);
    receipt["case_count"] = json!(cases.len());
    receipt["stale_count"] = json!(stale_count);
    (receipt, updated)
}

fn rewrite_expected_hashes_in_order(input: &str, hashes: &[String]) -> Result<String, Value> {
    let mut output = String::with_capacity(input.len());
    let mut cursor = 0;
    for hash in hashes {
        let Some(field_start) = input[cursor..]
            .find("\"expected_hash\"")
            .map(|index| cursor + index)
        else {
            return Err(json!({
                "code": "preserves_corpus.rewrite_failed",
                "message": "expected_hash field count did not match parsed corpus cases",
            }));
        };
        let Some(colon_offset) = input[field_start..].find(':') else {
            return Err(json!({
                "code": "preserves_corpus.rewrite_failed",
                "message": "expected_hash field has no colon",
            }));
        };
        let value_start_search = field_start + colon_offset + 1;
        let Some(open_quote_offset) = input[value_start_search..].find('"') else {
            return Err(json!({
                "code": "preserves_corpus.rewrite_failed",
                "message": "expected_hash field has no string value",
            }));
        };
        let value_start = value_start_search + open_quote_offset + 1;
        let Some(close_quote_offset) = input[value_start..].find('"') else {
            return Err(json!({
                "code": "preserves_corpus.rewrite_failed",
                "message": "expected_hash string is unterminated",
            }));
        };
        let value_end = value_start + close_quote_offset;
        output.push_str(&input[cursor..value_start]);
        output.push_str(hash);
        cursor = value_end;
    }
    output.push_str(&input[cursor..]);
    Ok(output)
}

fn base_receipt(
    path: String,
    execute: bool,
    valid: bool,
    issues: Vec<Value>,
    cases: Vec<Value>,
) -> Value {
    json!({
        "command": "preserves-corpus update",
        "path": path,
        "execute": execute,
        "mutated": false,
        "valid": valid,
        "case_count": cases.len(),
        "stale_count": 0,
        "cases": cases,
        "issues": issues,
    })
}

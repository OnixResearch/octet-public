use super::replay::{
    replay_attestation_policy_hash, replay_entropy_inventory_from_policy,
    replay_entropy_policy_hash, replay_environment_facts_from_policy,
};
use super::*;

struct ReplayVerifyOptions {
    root: PathBuf,
    path: PathBuf,
}

pub(super) fn run_determinism_verify_receipt(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = replay_verify_options(&args)?;
    let receipt = determinism_verify_replay_receipt(&options.root, &options.path);
    let valid = receipt
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    print_json(receipt)?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

fn replay_verify_options(args: &[String]) -> Result<ReplayVerifyOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let path = args
        .iter()
        .find(|arg| !arg.starts_with("--") && *arg != &root)
        .ok_or("determinism verify-receipt requires <receipt.json>")?;
    Ok(ReplayVerifyOptions {
        root,
        path: PathBuf::from(path),
    })
}

pub(crate) fn determinism_verify_replay_receipt(root: &Path, path: &Path) -> Value {
    // r[impl cairn.supersede.replay_receipt_verification.contract]
    // r[impl cairn.supersede.replay_receipt_verification.cli]
    // r[impl cairn.supersede.replay_receipt_verification.validation]
    // r[impl cairn.supersede.replay_receipt_verification.receipt]
    let policy = match load_policy_file(default_policy_path().into()) {
        Ok(policy) => policy,
        Err(error) => {
            return replay_verification_receipt(
                root,
                path,
                vec![replay_verification_issue(
                    "policy.load_failed",
                    "policy",
                    "generated policy loads",
                    &error.to_string(),
                )],
                String::new(),
                String::new(),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            )
        }
    };
    let current_policy_hash = canonical_json_hash(&policy.raw);
    let all_cases = replay_cases_from_policy(&policy).unwrap_or_default();
    let groups = replay_groups_from_policy(&policy).unwrap_or_default();
    let entropy_inventory = replay_entropy_inventory_from_policy(&policy).unwrap_or_default();
    let environment_facts = replay_environment_facts_from_policy(&policy).unwrap_or_default();
    let expected_entropy_policy_hash = replay_entropy_policy_hash(&entropy_inventory);
    let expected_attestation_policy_hash = replay_attestation_policy_hash(&environment_facts);

    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) => {
            return replay_verification_receipt(
                root,
                path,
                vec![replay_verification_issue(
                    "receipt.read_failed",
                    "path",
                    "readable replay receipt",
                    &error.to_string(),
                )],
                current_policy_hash,
                String::new(),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            )
        }
    };
    let receipt: Value = match serde_json::from_str(&text) {
        Ok(value) => value,
        Err(error) => {
            return replay_verification_receipt(
                root,
                path,
                vec![replay_verification_issue(
                    "receipt.malformed_json",
                    "json",
                    "valid JSON replay receipt",
                    &error.to_string(),
                )],
                current_policy_hash,
                String::new(),
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
                Value::Null,
            )
        }
    };

    let mut issues = Vec::new();
    if receipt.get("command").and_then(Value::as_str) != Some("determinism replay") {
        issues.push(replay_verification_issue(
            "receipt.command_mismatch",
            "command",
            "determinism replay",
            receipt
                .get("command")
                .and_then(Value::as_str)
                .unwrap_or("<missing>"),
        ));
    }
    if receipt.get("valid").and_then(Value::as_bool) != Some(true)
        || receipt.get("verdict").and_then(Value::as_str) != Some("pass")
    {
        issues.push(replay_verification_issue(
            "receipt.replay_not_passing",
            "valid/verdict",
            "valid=true verdict=pass",
            "non-passing replay receipt",
        ));
    }

    let stated_receipt_hash = receipt
        .get("receipt_hash")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if !is_blake3_hex_text(&stated_receipt_hash) {
        issues.push(replay_verification_issue(
            "receipt_hash.malformed",
            "receipt_hash",
            "BLAKE3 hex",
            stated_receipt_hash.as_str(),
        ));
    }
    let mut receipt_hash_input = receipt.clone();
    if let Some(object) = receipt_hash_input.as_object_mut() {
        object.remove("receipt_hash");
    }
    let replay_receipt_hash = portable_json_hash(root, &receipt_hash_input);
    if stated_receipt_hash != replay_receipt_hash {
        issues.push(replay_verification_issue(
            "receipt_hash.mismatch",
            "receipt_hash",
            &replay_receipt_hash,
            &stated_receipt_hash,
        ));
    }

    let selected_cases = receipt
        .get("selected_cases")
        .cloned()
        .unwrap_or(Value::Null);
    let selected_groups = receipt
        .get("selected_groups")
        .cloned()
        .unwrap_or(Value::Null);
    verify_replay_string_members(
        &mut issues,
        &selected_cases,
        "selected_cases",
        "selected_cases.invalid",
        all_cases.iter().map(|case| case.id.as_str()).collect(),
    );
    verify_replay_string_members(
        &mut issues,
        &selected_groups,
        "selected_groups",
        "selected_groups.invalid",
        groups.iter().map(|group| group.id.as_str()).collect(),
    );

    let entropy_policy_hash = receipt
        .get("entropy_policy_hash")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if entropy_policy_hash != expected_entropy_policy_hash {
        issues.push(replay_verification_issue(
            "entropy_policy_hash.mismatch",
            "entropy_policy_hash",
            &expected_entropy_policy_hash,
            &entropy_policy_hash,
        ));
    }

    let attestation = receipt
        .get("environment_attestation")
        .cloned()
        .unwrap_or(Value::Null);
    let replay_policy_hash = attestation
        .get("generated_policy_hash")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if replay_policy_hash != current_policy_hash {
        issues.push(replay_verification_issue(
            "replay_policy_hash.stale",
            "environment_attestation.generated_policy_hash",
            &current_policy_hash,
            &replay_policy_hash,
        ));
    }
    let environment_attestation_hash = attestation
        .get("attestation_hash")
        .and_then(Value::as_str)
        .unwrap_or_default()
        .to_string();
    if !attestation.is_object() {
        issues.push(replay_verification_issue(
            "environment_attestation.missing",
            "environment_attestation",
            "object",
            "missing",
        ));
    } else {
        if !is_blake3_hex_text(&environment_attestation_hash) {
            issues.push(replay_verification_issue(
                "environment_attestation_hash.malformed",
                "environment_attestation.attestation_hash",
                "BLAKE3 hex",
                &environment_attestation_hash,
            ));
        }
        let mut attestation_hash_input = attestation.clone();
        if let Some(object) = attestation_hash_input.as_object_mut() {
            object.remove("attestation_hash");
        }
        let expected_attestation_hash = canonical_json_hash(&attestation_hash_input);
        if environment_attestation_hash != expected_attestation_hash {
            issues.push(replay_verification_issue(
                "environment_attestation_hash.mismatch",
                "environment_attestation.attestation_hash",
                &expected_attestation_hash,
                &environment_attestation_hash,
            ));
        }
        let stated_attestation_policy_hash = attestation
            .get("attestation_policy_hash")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if stated_attestation_policy_hash != expected_attestation_policy_hash {
            issues.push(replay_verification_issue(
                "attestation_policy_hash.mismatch",
                "environment_attestation.attestation_policy_hash",
                &expected_attestation_policy_hash,
                stated_attestation_policy_hash,
            ));
        }
        let stated_attestation_entropy_hash = attestation
            .get("entropy_policy_hash")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if stated_attestation_entropy_hash != expected_entropy_policy_hash {
            issues.push(replay_verification_issue(
                "attestation_entropy_policy_hash.mismatch",
                "environment_attestation.entropy_policy_hash",
                &expected_entropy_policy_hash,
                stated_attestation_entropy_hash,
            ));
        }
    }
    issues.sort_by_key(canonical_json_hash);

    replay_verification_receipt(
        root,
        path,
        issues,
        current_policy_hash,
        replay_receipt_hash,
        Value::String(stated_receipt_hash),
        Value::String(replay_policy_hash),
        selected_cases,
        selected_groups,
        Value::String(environment_attestation_hash),
    )
}

fn verify_replay_string_members(
    issues: &mut Vec<Value>,
    value: &Value,
    field_path: &str,
    code: &str,
    allowed: std::collections::BTreeSet<&str>,
) {
    let Some(items) = value.as_array() else {
        issues.push(replay_verification_issue(
            code,
            field_path,
            "array",
            "missing/non-array",
        ));
        return;
    };
    for item in items {
        let Some(text) = item.as_str() else {
            issues.push(replay_verification_issue(
                code,
                field_path,
                "string ids",
                "non-string",
            ));
            continue;
        };
        if !allowed.contains(text) {
            issues.push(replay_verification_issue(
                code,
                field_path,
                "known policy id",
                text,
            ));
        }
    }
}

fn replay_verification_issue(code: &str, field_path: &str, expected: &str, actual: &str) -> Value {
    json!({
        "code": code,
        "field_path": field_path,
        "expected": expected,
        "actual": actual,
    })
}

#[allow(clippy::too_many_arguments)]
fn replay_verification_receipt(
    root: &Path,
    path: &Path,
    issues: Vec<Value>,
    current_replay_policy_hash: String,
    replay_receipt_hash: String,
    stated_replay_receipt_hash: Value,
    replay_policy_hash: Value,
    selected_cases: Value,
    selected_groups: Value,
    environment_attestation_hash: Value,
) -> Value {
    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        root,
        json!({
            "command": "determinism verify-receipt",
            "root": path_string(root),
            "path": portable_text(root, &path_string(path)),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "current_replay_policy_hash": current_replay_policy_hash,
            "replay_policy_hash": replay_policy_hash,
            "replay_receipt_hash": replay_receipt_hash,
            "stated_replay_receipt_hash": stated_replay_receipt_hash,
            "selected_cases": selected_cases,
            "selected_groups": selected_groups,
            "environment_attestation_hash": environment_attestation_hash,
        }),
    )
}

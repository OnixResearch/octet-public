// r[impl cli_maintainability.determinism_replay_submodules.focused_modules]
// r[impl cli_maintainability.determinism_replay_submodules.focused_modules.visible_boundaries]
// r[impl cli_maintainability.determinism_replay_submodules.focused_modules.narrow_bridges]
mod execution;
mod options;
mod policy;
mod receipt;
mod render;
mod support;
mod types;

use execution::{
    replay_case_root, replay_case_source_root, replay_manifest_for_case, replay_output_mutated,
    replay_reproduce_command, run_replay_case,
};
use options::replay_options;
use policy::{select_replay_cases, validate_replay_groups};
use receipt::{
    replay_entropy_assumptions, replay_entropy_inventory_json, replay_environment_attestation,
    replay_mismatch, replay_nondeterminism_failures, replay_receipt,
};
use render::render_replay_receipt;
use support::*;

pub(crate) use policy::{replay_cases_from_policy, replay_groups_from_policy};
pub(super) use policy::{
    replay_entropy_inventory_from_policy, replay_environment_facts_from_policy, required_string,
    required_string_array,
};
pub(super) use receipt::{replay_attestation_policy_hash, replay_entropy_policy_hash};
pub(crate) use types::{ReplayCase, ReplayGroup};

pub(super) fn run_determinism_replay(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = replay_options(&args)?;
    let root = options.root.clone();
    if has_flag(&args, "--execute") || has_flag(&args, "--allow-mutations") {
        let issue = json!({
            "case": "plan",
            "class": "mutation_denied",
            "code": "replay.mutation_denied",
            "comparison": "preflight",
            "field_path": "args",
            "expected": "read-only/dry-run replay",
            "actual": "mutation flag requested",
            "reproduce": replay_reproduce_command(&root, None, None),
        });
        let mut receipt = replay_receipt(
            root.clone(),
            Vec::new(),
            vec![issue],
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            Vec::new(),
            canonical_json_hash(&json!([])),
            Value::Null,
        );
        receipt["verdict"] = json!("fail");
        receipt["valid"] = json!(false);
        let receipt = with_portable_content_hash_receipt(&root, receipt);
        render_replay_receipt(&receipt, options.format, options.out.as_deref())?;
        std::process::exit(1);
    }
    let receipt = determinism_replay_receipt(
        root.clone(),
        options.selected_case.as_deref(),
        options.selected_group.as_deref(),
    )?;
    let valid = receipt
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    render_replay_receipt(&receipt, options.format, options.out.as_deref())?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

pub(crate) fn determinism_replay_receipt(
    root: PathBuf,
    selected_case: Option<&str>,
    selected_group: Option<&str>,
) -> Result<Value, Box<dyn Error>> {
    // r[impl cairn.supersede.determinism_replay.workspace_oracle]
    // r[impl cairn.supersede.determinism_replay.repeated]
    // r[impl cairn.supersede.determinism_replay.cross_root]
    // r[impl cairn.supersede.determinism_replay.receipt]
    // r[impl cairn.supersede.determinism_replay.mcp_cases]
    // r[impl cairn.supersede.determinism_replay.no_mutation]
    // r[impl cairn.supersede.determinism_replay.policy_plan]
    // r[impl cairn.supersede.determinism_replay.expanded_surfaces]
    // r[impl cairn.supersede.determinism_replay.ci_case_selection]
    // r[impl cairn.supersede.determinism_replay.ci_failure_ergonomics]
    // r[impl cairn.supersede.determinism_replay.groups.policy]
    // r[impl cairn.supersede.determinism_replay.groups.cli]
    // r[impl cairn.supersede.determinism_replay.groups.failures]
    // r[impl cairn.supersede.determinism_replay.groups.receipt]
    let policy = load_policy_file(default_policy_path().into())?;
    let all_cases = replay_cases_from_policy(&policy)?;
    let groups = replay_groups_from_policy(&policy)?;
    let entropy_inventory = replay_entropy_inventory_from_policy(&policy)?;
    let environment_facts = replay_environment_facts_from_policy(&policy)?;
    validate_replay_groups(&all_cases, &groups)?;
    let available_cases = all_cases
        .iter()
        .map(|case| case.id.clone())
        .collect::<Vec<_>>();
    let available_groups = groups
        .iter()
        .map(|group| group.id.clone())
        .collect::<Vec<_>>();
    let selection = select_replay_cases(&all_cases, &groups, selected_case, selected_group)?;
    let selected_groups = selection.selected_groups.clone();
    let cases = selection.cases;
    let mut case_receipts = Vec::new();
    let mut mismatches = Vec::new();
    let nondeterminism_failures = replay_nondeterminism_failures(&entropy_inventory);
    let mut mutability_observations = Vec::new();
    let mut skipped = Vec::new();
    if selected_case.is_some() && selected_group.is_some() {
        let actual = format!(
            "--case {} --group {}",
            selected_case.unwrap_or_default(),
            selected_group.unwrap_or_default()
        );
        mismatches.push(json!({
            "case": "selection",
            "comparison": "selection",
            "field_path": "--case/--group",
            "class": "selector_conflict",
            "expected": "one replay selector",
            "actual": actual,
            "reproduce": replay_reproduce_command(&root, selected_case, selected_group),
        }));
    } else if cases.is_empty() {
        if let Some(case_id) = selected_case {
            mismatches.push(json!({
                "case": case_id,
                "comparison": "selection",
                "field_path": "--case",
                "class": "unknown_case",
                "expected": available_cases,
                "actual": case_id,
                "reproduce": replay_reproduce_command(&root, Some(case_id), None),
            }));
        } else if let Some(group_id) = selected_group {
            mismatches.push(json!({
                "case": group_id,
                "comparison": "selection",
                "field_path": "--group",
                "class": "unknown_group",
                "expected": available_groups,
                "actual": group_id,
                "reproduce": replay_reproduce_command(&root, None, Some(group_id)),
            }));
        }
    }

    for case in &cases {
        let source_root = replay_case_source_root(&root, case)?;
        let repeated_a = replay_case_root(&source_root, case, "a")?;
        let repeated_b = if case
            .comparison_modes
            .iter()
            .any(|mode| mode == "cross_root")
        {
            Some(replay_case_root(&source_root, case, "b")?)
        } else {
            None
        };
        let before_a = replay_manifest_for_case(&repeated_a, case)?;
        let before_b = match &repeated_b {
            Some(path) => Some(replay_manifest_for_case(path, case)?),
            None => None,
        };

        let first = run_replay_case(case, &repeated_a)?;
        let second = run_replay_case(case, &repeated_a)?;
        let other = match &repeated_b {
            Some(path) => Some(run_replay_case(case, path)?),
            None => None,
        };
        if replay_output_mutated(&first)
            || replay_output_mutated(&second)
            || other.as_ref().is_some_and(replay_output_mutated)
        {
            mismatches.push(replay_mismatch(
                case,
                "mutation",
                "mutated",
                "false",
                "true",
                "mutation_observed",
            ));
            mutability_observations.push(json!({"case": case.id, "mutated": true}));
        } else {
            mutability_observations.push(json!({"case": case.id, "mutated": false}));
        }

        let repeated_first = portable_json_hash(&repeated_a, &first);
        let repeated_second = portable_json_hash(&repeated_a, &second);
        if repeated_first != repeated_second {
            mismatches.push(replay_mismatch(
                case,
                "repeated",
                "$",
                &repeated_first,
                &repeated_second,
                "normalized_output_hash",
            ));
        }
        let cross_other = match (&repeated_b, &other) {
            (Some(path), Some(value)) => Some(portable_json_hash(path, value)),
            _ => None,
        };
        if let (Some(path), Some(value), Some(cross_hash)) = (&repeated_b, &other, &cross_other) {
            let cross_first = portable_json_hash(&repeated_a, &first);
            if cross_first != *cross_hash {
                mismatches.push(replay_mismatch(
                    case,
                    "cross_root",
                    "$",
                    &cross_first,
                    cross_hash,
                    "portable_output_hash",
                ));
            }
            for field in &case.hash_fields {
                let left = first.get(field).and_then(Value::as_str).unwrap_or_default();
                let right = value.get(field).and_then(Value::as_str).unwrap_or_default();
                if left != right {
                    mismatches.push(replay_mismatch(
                        case,
                        "cross_root",
                        field,
                        left,
                        right,
                        "portable_hash_field",
                    ));
                }
            }
            let after_b = replay_manifest_for_case(path, case)?;
            if let Some(before) = before_b {
                if before["manifest_hash"] != after_b["manifest_hash"] {
                    mismatches.push(json!({"case": case.id, "comparison": "mutation", "field_path": "root_b", "class": "mutation_observed", "expected": before["manifest_hash"], "actual": after_b["manifest_hash"], "reproduce": replay_reproduce_command(&root, Some(&case.id), None)}));
                }
            }
        } else {
            skipped.push(json!({"case": case.id, "comparison": "cross_root", "reason": "not requested by policy"}));
        }

        let after_a = replay_manifest_for_case(&repeated_a, case)?;
        if before_a["manifest_hash"] != after_a["manifest_hash"] {
            mismatches.push(json!({"case": case.id, "comparison": "mutation", "field_path": "root_a", "class": "mutation_observed", "expected": before_a["manifest_hash"], "actual": after_a["manifest_hash"], "reproduce": replay_reproduce_command(&root, Some(&case.id), None)}));
        }

        let mut runs = vec![
            json!({"root": "a", "iteration": 1, "output_hash": repeated_first, "normalized_output_hash": repeated_first}),
            json!({"root": "a", "iteration": 2, "output_hash": repeated_second, "normalized_output_hash": repeated_second}),
        ];
        if let Some(cross_hash) = &cross_other {
            runs.push(json!({"root": "b", "iteration": 1, "output_hash": cross_hash, "normalized_output_hash": cross_hash}));
        }
        case_receipts.push(json!({
            "id": case.id,
            "command_kind": case.command_kind,
            "args": case.args,
            "fixture_root": case.fixture_root,
            "expected_mutability": "read_only_or_dry_run",
            "comparison_modes": case.comparison_modes,
            "normalization_rules": case.normalization_rules,
            "display_only_paths": case.display_only_paths,
            "hash_fields": case.hash_fields,
            "runs": runs,
            "portable_evidence_hash": canonical_json_hash(&json!({"case": case.id, "runs": runs})),
        }));
    }

    case_receipts.sort_by_key(|value| value["id"].as_str().unwrap_or_default().to_string());
    mismatches.sort_by_key(canonical_json_hash);
    mutability_observations.sort_by_key(canonical_json_hash);
    skipped.sort_by_key(canonical_json_hash);
    let selected_cases = cases.iter().map(|case| case.id.clone()).collect::<Vec<_>>();
    let entropy_policy_hash = replay_entropy_policy_hash(&entropy_inventory);
    let environment_attestation = replay_environment_attestation(
        &policy,
        &environment_facts,
        &entropy_policy_hash,
        &selected_cases,
        &selected_groups,
        selected_case,
        selected_group,
    );
    let receipt = replay_receipt(
        root.clone(),
        case_receipts,
        mismatches,
        mutability_observations,
        skipped,
        nondeterminism_failures,
        available_cases,
        available_groups,
        selected_cases,
        selected_groups,
        replay_entropy_inventory_json(&entropy_inventory),
        replay_entropy_assumptions(&entropy_inventory),
        entropy_policy_hash,
        environment_attestation,
    );
    Ok(with_portable_content_hash_receipt(&root, receipt))
}

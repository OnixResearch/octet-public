use super::replay::{required_string, required_string_array};
use super::*;

pub(super) fn run_determinism_coverage_audit(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.determinism_surface_coverage_audit.cli]
    // r[impl cairn.supersede.determinism_surface_coverage_audit.inventory]
    // r[impl cairn.supersede.determinism_surface_coverage_audit.failures]
    let root = read_option(args, "--root", ".");
    let policy = load_policy_file(default_policy_path().into())?;
    let receipt = determinism_coverage_audit_receipt(&root, &policy);
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

#[derive(Clone)]
pub(crate) struct DeterminismSurface {
    pub(crate) id: String,
    pub(crate) command: String,
    pub(crate) surface_kind: String,
    pub(crate) receipt_command: String,
    pub(crate) required_replay_cases: Vec<String>,
    pub(crate) required_replay_groups: Vec<String>,
    pub(crate) determinism_claim: String,
}

pub(crate) fn determinism_coverage_audit_receipt(root: &Path, policy: &Policy) -> Value {
    let current_policy_hash = canonical_json_hash(&policy.raw);
    let surfaces = determinism_surfaces_from_policy(policy).unwrap_or_default();
    let cases = replay_cases_from_policy(policy).unwrap_or_default();
    let groups = replay_groups_from_policy(policy).unwrap_or_default();
    let (covered_surfaces, missing_surfaces, mut issues) =
        audit_determinism_surface_coverage(&surfaces, &cases, &groups);
    issues.sort_by_key(canonical_json_hash);
    let surface_inventory = determinism_surfaces_json(&surfaces);
    let valid = issues.is_empty();
    with_portable_receipt_hash(
        root,
        json!({
            "command": "determinism coverage-audit",
            "root": path_string(root),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "issues": issues,
            "current_replay_policy_hash": current_policy_hash,
            "surface_inventory_hash": canonical_json_hash(&json!(surface_inventory)),
            "surface_count": surfaces.len(),
            "covered_surfaces": covered_surfaces,
            "missing_surfaces": missing_surfaces,
        }),
    )
}

pub(crate) fn determinism_surfaces_from_policy(
    policy: &Policy,
) -> Result<Vec<DeterminismSurface>, Box<dyn Error>> {
    let surfaces = policy
        .raw
        .get("determinism_surface_inventory")
        .and_then(Value::as_array)
        .ok_or("policy missing determinism_surface_inventory")?;
    let mut parsed = surfaces
        .iter()
        .map(determinism_surface_from_policy_value)
        .collect::<Result<Vec<_>, _>>()?;
    parsed.sort_by_key(|surface| surface.id.clone());
    Ok(parsed)
}

fn determinism_surface_from_policy_value(
    value: &Value,
) -> Result<DeterminismSurface, Box<dyn Error>> {
    Ok(DeterminismSurface {
        id: required_string(value, "id")?.to_owned(),
        command: required_string(value, "command")?.to_owned(),
        surface_kind: required_string(value, "surface_kind")?.to_owned(),
        receipt_command: required_string(value, "receipt_command")?.to_owned(),
        required_replay_cases: required_string_array(value, "required_replay_cases")?,
        required_replay_groups: required_string_array(value, "required_replay_groups")?,
        determinism_claim: required_string(value, "determinism_claim")?.to_owned(),
    })
}

fn determinism_surfaces_json(surfaces: &[DeterminismSurface]) -> Vec<Value> {
    surfaces
        .iter()
        .map(|surface| {
            json!({
                "id": surface.id,
                "command": surface.command,
                "surface_kind": surface.surface_kind,
                "receipt_command": surface.receipt_command,
                "required_replay_cases": surface.required_replay_cases,
                "required_replay_groups": surface.required_replay_groups,
                "determinism_claim": surface.determinism_claim,
            })
        })
        .collect()
}

fn audit_determinism_surface_coverage(
    surfaces: &[DeterminismSurface],
    cases: &[ReplayCase],
    groups: &[ReplayGroup],
) -> (Vec<Value>, Vec<Value>, Vec<Value>) {
    let case_ids = cases
        .iter()
        .map(|case| case.id.as_str())
        .collect::<std::collections::BTreeSet<_>>();
    let group_map = groups
        .iter()
        .map(|group| (group.id.as_str(), group))
        .collect::<std::collections::BTreeMap<_, _>>();
    let mut covered = Vec::new();
    let mut missing = Vec::new();
    let mut issues = Vec::new();
    for surface in surfaces {
        let mut surface_issues = Vec::new();
        for case_id in &surface.required_replay_cases {
            if !case_ids.contains(case_id.as_str()) {
                surface_issues.push(coverage_issue(
                    "coverage.required_case_missing",
                    &surface.id,
                    "required_replay_cases",
                    case_id,
                ));
            }
        }
        for group_id in &surface.required_replay_groups {
            match group_map.get(group_id.as_str()) {
                Some(group) => {
                    for case_id in &surface.required_replay_cases {
                        if !group.cases.iter().any(|candidate| candidate == case_id) {
                            surface_issues.push(coverage_issue(
                                "coverage.group_missing_case",
                                &surface.id,
                                group_id,
                                case_id,
                            ));
                        }
                    }
                }
                None => surface_issues.push(coverage_issue(
                    "coverage.required_group_missing",
                    &surface.id,
                    "required_replay_groups",
                    group_id,
                )),
            }
        }
        let surface_json = json!({
            "id": surface.id,
            "command": surface.command,
            "required_replay_cases": surface.required_replay_cases,
            "required_replay_groups": surface.required_replay_groups,
        });
        if surface_issues.is_empty() {
            covered.push(surface_json);
        } else {
            issues.extend(surface_issues);
            missing.push(surface_json);
        }
    }
    covered.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    missing.sort_by(|left, right| left["id"].as_str().cmp(&right["id"].as_str()));
    (covered, missing, issues)
}

fn coverage_issue(code: &str, surface_id: &str, field_path: &str, actual: &str) -> Value {
    json!({
        "code": code,
        "surface_id": surface_id,
        "field_path": field_path,
        "expected": "policy replay coverage",
        "actual": actual,
    })
}

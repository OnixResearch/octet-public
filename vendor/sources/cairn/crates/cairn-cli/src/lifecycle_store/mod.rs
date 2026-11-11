mod local_state;
mod repository;

use std::{
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    apply_lifecycle_store_locator_plan, canonical_json_hash, plan_lifecycle_store_registration,
    plan_lifecycle_store_unregistration, LifecycleStoreDiagnostic, LifecycleStoreLocatorPlan,
    LIFECYCLE_STORE_LOCAL_EVIDENCE_CLASS, LIFECYCLE_STORE_REQUIRED_NON_CLAIMS,
};
use serde::Serialize;
use serde_json::{json, Value};

use crate::support::{print_json, with_receipt_hash};

use self::{
    local_state::{data_directory, load_locator, locator_path, with_locked_locator},
    repository::{
        atomic_materialize_store, build_setup_material, inspect_store,
        normalized_setup_destination, setup_file_hashes, StoreInspection, StoreSetupMaterial,
        LIFECYCLE_DIRECTORIES_FOR_RECEIPT,
    },
};

const COMMAND_SCHEMA_V1: &str = "cairn.lifecycle-store-command.v1";
const SETUP_COMMAND: &str = "store setup";
const REGISTER_COMMAND: &str = "store register";
const UNREGISTER_COMMAND: &str = "store unregister";
const LIST_COMMAND: &str = "store list";
const SHOW_COMMAND: &str = "store show";
const DOCTOR_COMMAND: &str = "store doctor";
const DIAGNOSTIC_IDENTITY_INVALID: &str = "lifecycle_store.identity.invalid";
const DIAGNOSTIC_IDENTITY_CHANGED: &str = "lifecycle_store.locator.identity_changed";
const POSITIONAL_LIMIT_WITH_SUBCOMMAND: usize = 2;

// r[impl cairn.lifecycle_stores.commands]
// r[impl cairn.lifecycle_stores.transport_boundary]
// r[impl cairn.lifecycle_stores.functional_boundary.shell]
pub(crate) fn run_store(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(command) = iter.next() else {
        return Err("store requires setup, register, unregister, list, show, or doctor".into());
    };
    let rest = iter.collect::<Vec<_>>();
    match command.as_str() {
        "setup" => run_setup(&rest),
        "register" => run_register(&rest),
        "unregister" => run_unregister(&rest),
        "list" => run_list(&rest),
        "show" => run_show(&rest),
        "doctor" => run_doctor(&rest),
        unknown => Err(format!("unknown store command: {unknown}").into()),
    }
}

#[derive(Debug)]
struct SetupOptions {
    store_id: String,
    path: PathBuf,
    remote_hint: Option<String>,
    execute: bool,
}

#[derive(Debug)]
struct RegisterOptions {
    path: PathBuf,
    store_id: Option<String>,
    execute: bool,
}

#[derive(Debug)]
struct UnregisterOptions {
    store_id: String,
    execute: bool,
}

#[derive(Debug, Serialize)]
struct SetupAction {
    kind: &'static str,
    path: String,
    before_blake3: Option<String>,
    after_blake3: Option<String>,
}

#[derive(Debug)]
struct LocatorExecution {
    mutated: bool,
    diagnostics: Vec<LifecycleStoreDiagnostic>,
}

fn run_setup(args: &[String]) -> Result<(), Box<dyn Error>> {
    let options = parse_setup_options(args)?;
    let material = match build_setup_material(&options.store_id, options.remote_hint.as_deref()) {
        Ok(material) => material,
        Err(diagnostics) => {
            return print_json(setup_receipt(&options, None, None, diagnostics, false));
        }
    };
    let destination = match normalized_setup_destination(&options.path) {
        Ok(destination) => destination,
        Err(diagnostic) => {
            return print_json(setup_receipt(
                &options,
                Some(&material),
                None,
                vec![diagnostic],
                false,
            ));
        }
    };
    let mutated = if options.execute {
        atomic_materialize_store(&destination, &material)?;
        true
    } else {
        false
    };
    print_json(setup_receipt(
        &options,
        Some(&material),
        Some(&destination),
        Vec::new(),
        mutated,
    ))
}

fn run_register(args: &[String]) -> Result<(), Box<dyn Error>> {
    let options = parse_register_options(args)?;
    let data_dir = data_directory()?;
    let inspection = inspect_store(&options.path, options.store_id.as_deref());
    let current = load_locator(&data_dir)?;
    let Some(state) = current.state.as_ref() else {
        return print_json(locator_receipt(
            REGISTER_COMMAND,
            options.execute,
            false,
            None,
            Some(&inspection),
            current.diagnostics,
        ));
    };
    let Some(identity) = inspection.identity.as_ref() else {
        return print_json(locator_receipt(
            REGISTER_COMMAND,
            options.execute,
            false,
            None,
            Some(&inspection),
            inspection.diagnostics.clone(),
        ));
    };
    let path_text = inspection.path.to_string_lossy().into_owned();
    let plan = plan_lifecycle_store_registration(state, identity, &path_text);
    if !options.execute || plan.blocked {
        return print_json(locator_receipt(
            REGISTER_COMMAND,
            options.execute,
            false,
            Some(&plan),
            Some(&inspection),
            plan.diagnostics.clone(),
        ));
    }
    let initial_identity = identity.clone();
    let initial_path = inspection.path.clone();
    let execution = with_locked_locator(&data_dir, |locked| {
        let Some(locked_state) = locked.state.as_ref() else {
            return Ok((
                LocatorExecution {
                    mutated: false,
                    diagnostics: locked.diagnostics.clone(),
                },
                None,
            ));
        };
        let reloaded = inspect_store(&initial_path, Some(initial_identity.store_id.as_str()));
        if !reloaded.valid() {
            return Ok((
                LocatorExecution {
                    mutated: false,
                    diagnostics: reloaded.diagnostics,
                },
                None,
            ));
        }
        if reloaded.identity.as_ref() != Some(&initial_identity) {
            return Ok((
                LocatorExecution {
                    mutated: false,
                    diagnostics: vec![diagnostic(
                        DIAGNOSTIC_IDENTITY_CHANGED,
                        "store_identity",
                        initial_identity.store_id.as_str(),
                        "store identity changed after registration planning",
                    )],
                },
                None,
            ));
        }
        let application = apply_lifecycle_store_locator_plan(locked_state, &plan);
        let replacement = application
            .accepted
            .then_some(application.next_state.clone())
            .flatten()
            .filter(|_| plan.changed);
        Ok((
            LocatorExecution {
                mutated: application.accepted && plan.changed,
                diagnostics: application.diagnostics,
            },
            replacement,
        ))
    })?;
    print_json(locator_receipt(
        REGISTER_COMMAND,
        true,
        execution.mutated,
        Some(&plan),
        Some(&inspection),
        execution.diagnostics,
    ))
}

fn run_unregister(args: &[String]) -> Result<(), Box<dyn Error>> {
    let options = parse_unregister_options(args)?;
    let data_dir = data_directory()?;
    let current = load_locator(&data_dir)?;
    let Some(state) = current.state.as_ref() else {
        return print_json(locator_receipt(
            UNREGISTER_COMMAND,
            options.execute,
            false,
            None,
            None,
            current.diagnostics,
        ));
    };
    let plan = plan_lifecycle_store_unregistration(state, &options.store_id);
    if !options.execute || plan.blocked {
        return print_json(locator_receipt(
            UNREGISTER_COMMAND,
            options.execute,
            false,
            Some(&plan),
            None,
            plan.diagnostics.clone(),
        ));
    }
    let execution = with_locked_locator(&data_dir, |locked| {
        let Some(locked_state) = locked.state.as_ref() else {
            return Ok((
                LocatorExecution {
                    mutated: false,
                    diagnostics: locked.diagnostics.clone(),
                },
                None,
            ));
        };
        let application = apply_lifecycle_store_locator_plan(locked_state, &plan);
        let replacement = application
            .accepted
            .then_some(application.next_state.clone())
            .flatten()
            .filter(|_| plan.changed);
        Ok((
            LocatorExecution {
                mutated: application.accepted && plan.changed,
                diagnostics: application.diagnostics,
            },
            replacement,
        ))
    })?;
    print_json(locator_receipt(
        UNREGISTER_COMMAND,
        true,
        execution.mutated,
        Some(&plan),
        None,
        execution.diagnostics,
    ))
}

fn run_list(args: &[String]) -> Result<(), Box<dyn Error>> {
    require_no_arguments(args, LIST_COMMAND)?;
    let data_dir = data_directory()?;
    let locator = load_locator(&data_dir)?;
    let stores = locator.state.as_ref().map_or_else(Vec::new, |state| {
        state
            .mappings
            .iter()
            .map(|mapping| {
                json!({
                    "store_id": mapping.store_id.as_str(),
                    "path": mapping.checkout_path.as_str(),
                    "store_identity": mapping.store_identity.as_str(),
                })
            })
            .collect()
    });
    print_json(read_receipt(
        LIST_COMMAND,
        locator.valid,
        json!({
            "registry": locator_path(&data_dir),
            "stores": stores,
            "diagnostics": locator.diagnostics,
        }),
    ))
}

fn run_show(args: &[String]) -> Result<(), Box<dyn Error>> {
    let store_id = require_one_positional(args, SHOW_COMMAND)?;
    let data_dir = data_directory()?;
    let locator = load_locator(&data_dir)?;
    let Some(state) = locator.state.as_ref() else {
        return print_json(read_receipt(
            SHOW_COMMAND,
            false,
            json!({
                "store_id": store_id,
                "registration": null,
                "projection": null,
                "diagnostics": locator.diagnostics,
            }),
        ));
    };
    let Some(mapping) = state
        .mappings
        .iter()
        .find(|mapping| mapping.store_id.as_str() == store_id)
    else {
        return print_json(read_receipt(
            SHOW_COMMAND,
            false,
            json!({
                "store_id": store_id,
                "registration": null,
                "projection": null,
                "diagnostics": [diagnostic(
                    DIAGNOSTIC_IDENTITY_INVALID,
                    "store_id",
                    &store_id,
                    "store ID is not registered",
                )],
            }),
        ));
    };
    let inspection = inspect_store(
        PathBuf::from(mapping.checkout_path.as_str()).as_path(),
        Some(&store_id),
    );
    let mut diagnostics = inspection.diagnostics.clone();
    if inspection
        .identity
        .as_ref()
        .is_some_and(|identity| identity.store_identity.as_str() != mapping.store_identity.as_str())
    {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_IDENTITY_CHANGED,
            "store_identity",
            &store_id,
            "registered store identity differs from the current projection",
        ));
    }
    diagnostics.sort();
    diagnostics.dedup();
    print_json(read_receipt(
        SHOW_COMMAND,
        diagnostics.is_empty(),
        json!({
            "store_id": store_id,
            "registration": {
                "path": mapping.checkout_path.as_str(),
                "store_identity": mapping.store_identity.as_str(),
            },
            "projection": projection_facts(&inspection),
            "checks": inspection.checks,
            "diagnostics": diagnostics,
        }),
    ))
}

fn run_doctor(args: &[String]) -> Result<(), Box<dyn Error>> {
    let selected = optional_one_positional(args, DOCTOR_COMMAND)?;
    let data_dir = data_directory()?;
    let locator = load_locator(&data_dir)?;
    let Some(state) = locator.state.as_ref() else {
        return print_json(read_receipt(
            DOCTOR_COMMAND,
            false,
            json!({
                "registry": locator_path(&data_dir),
                "stores": [],
                "diagnostics": locator.diagnostics,
            }),
        ));
    };
    let mut entries = Vec::new();
    for mapping in &state.mappings {
        if selected
            .as_deref()
            .is_some_and(|store_id| store_id != mapping.store_id.as_str())
        {
            continue;
        }
        entries.push(doctor_entry(
            mapping.store_id.as_str(),
            mapping.checkout_path.as_str(),
            mapping.store_identity.as_str(),
        ));
    }
    let mut diagnostics = Vec::new();
    if let Some(store_id) = &selected {
        if entries.is_empty() {
            diagnostics.push(diagnostic(
                DIAGNOSTIC_IDENTITY_INVALID,
                "store_id",
                store_id,
                "store ID is not registered",
            ));
        }
    }
    let entries_valid = entries
        .iter()
        .all(|entry| entry["valid"].as_bool() == Some(true));
    print_json(read_receipt(
        DOCTOR_COMMAND,
        diagnostics.is_empty() && entries_valid,
        json!({
            "registry": locator_path(&data_dir),
            "stores": entries,
            "diagnostics": diagnostics,
        }),
    ))
}

fn doctor_entry(store_id: &str, path: &str, registered_identity: &str) -> Value {
    let inspection = inspect_store(PathBuf::from(path).as_path(), Some(store_id));
    let mut diagnostics = inspection.diagnostics.clone();
    let identity_matches = inspection
        .identity
        .as_ref()
        .is_some_and(|identity| identity.store_identity.as_str() == registered_identity);
    if inspection.identity.is_some() && !identity_matches {
        diagnostics.push(diagnostic(
            DIAGNOSTIC_IDENTITY_CHANGED,
            "store_identity",
            store_id,
            "registered store identity differs from the current projection",
        ));
    }
    diagnostics.sort();
    diagnostics.dedup();
    let repair_actions = if diagnostics.is_empty() {
        Vec::new()
    } else {
        vec![
            format!(
                "cairn store unregister {} --execute",
                quote_argument(store_id)
            ),
            format!(
                "cairn store register {} --id {} --execute",
                quote_argument(path),
                quote_argument(store_id)
            ),
        ]
    };
    json!({
        "store_id": store_id,
        "path": path,
        "registered_store_identity": registered_identity,
        "valid": diagnostics.is_empty(),
        "checks": {
            "path_safe": inspection.checks.path_safe,
            "native_root_present": inspection.checks.native_root_present,
            "source_present": inspection.checks.source_present,
            "projection_present": inspection.checks.projection_present,
            "projection_current": inspection.checks.projection_current,
            "identity_matches": identity_matches,
            "locator_conflict": false,
        },
        "projection": projection_facts(&inspection),
        "diagnostics": diagnostics,
        "repair_actions": repair_actions,
    })
}

fn setup_receipt(
    options: &SetupOptions,
    material: Option<&StoreSetupMaterial>,
    destination: Option<&PathBuf>,
    diagnostics: Vec<LifecycleStoreDiagnostic>,
    mutated: bool,
) -> Value {
    let actions = match (material, destination) {
        (Some(material), Some(destination)) => setup_actions(destination, material),
        _ => Vec::new(),
    };
    let portable_hash = material.map(|material| {
        canonical_json_hash(&json!({
            "command": SETUP_COMMAND,
            "store_id": material.identity.store_id.as_str(),
            "store_identity": material.identity.store_identity.as_str(),
            "remote_hint": material.identity.remote_hint.as_ref().map(|value| value.as_str()),
            "artifacts": setup_file_hashes(material),
            "non_claims": required_non_claims(),
        }))
    });
    with_receipt_hash(json!({
        "schema": COMMAND_SCHEMA_V1,
        "command": SETUP_COMMAND,
        "valid": diagnostics.is_empty(),
        "blocked": !diagnostics.is_empty(),
        "dry_run": !options.execute,
        "mutated": mutated,
        "store_id": options.store_id,
        "path": destination.unwrap_or(&options.path),
        "remote_hint": options.remote_hint,
        "store_identity": material.map(|value| value.identity.store_identity.as_str()),
        "actions": actions,
        "diagnostics": diagnostics,
        "evidence_class": LIFECYCLE_STORE_LOCAL_EVIDENCE_CLASS,
        "portable_hash": portable_hash,
        "non_claims": required_non_claims(),
    }))
}

fn setup_actions(destination: &Path, material: &StoreSetupMaterial) -> Vec<SetupAction> {
    let mut actions = LIFECYCLE_DIRECTORIES_FOR_RECEIPT
        .iter()
        .map(|relative| SetupAction {
            kind: "create_directory",
            path: destination.join(relative).display().to_string(),
            before_blake3: None,
            after_blake3: None,
        })
        .collect::<Vec<_>>();
    for (relative, hash) in setup_file_hashes(material) {
        actions.push(SetupAction {
            kind: "create_file",
            path: destination.join(relative).display().to_string(),
            before_blake3: None,
            after_blake3: Some(hash),
        });
    }
    actions.sort_by(|left, right| left.path.cmp(&right.path));
    actions
}

fn locator_receipt(
    command: &str,
    execute: bool,
    mutated: bool,
    plan: Option<&LifecycleStoreLocatorPlan>,
    inspection: Option<&StoreInspection>,
    mut diagnostics: Vec<LifecycleStoreDiagnostic>,
) -> Value {
    diagnostics.sort();
    diagnostics.dedup();
    with_receipt_hash(json!({
        "schema": COMMAND_SCHEMA_V1,
        "command": command,
        "valid": diagnostics.is_empty(),
        "blocked": !diagnostics.is_empty(),
        "dry_run": !execute,
        "mutated": mutated,
        "store_id": plan.map(|value| value.local_facts.store_id.as_str()).or_else(|| inspection.and_then(|value| value.identity.as_ref().map(|identity| identity.store_id.as_str()))),
        "path": inspection.map(|value| value.path.as_path()),
        "store_identity": inspection.and_then(|value| value.identity.as_ref().map(|identity| identity.store_identity.as_str())),
        "expected_prior_state_hash": plan.map(|value| value.expected_prior_state_identity.as_str()),
        "next_state_hash": plan.and_then(|value| value.next_state.as_ref().map(|state| state.state_identity.as_str())),
        "changed": plan.is_some_and(|value| value.changed),
        "plan_hash": plan.map(|value| value.plan_hash.as_str()),
        "portable_hash": plan.map(|value| value.portable_hash.as_str()),
        "diagnostics": diagnostics,
        "evidence_class": LIFECYCLE_STORE_LOCAL_EVIDENCE_CLASS,
        "non_claims": required_non_claims(),
    }))
}

fn read_receipt(command: &str, valid: bool, fields: Value) -> Value {
    let mut receipt = json!({
        "schema": COMMAND_SCHEMA_V1,
        "command": command,
        "valid": valid,
        "evidence_class": LIFECYCLE_STORE_LOCAL_EVIDENCE_CLASS,
        "non_claims": required_non_claims(),
    });
    if let (Some(receipt_object), Some(fields_object)) =
        (receipt.as_object_mut(), fields.as_object())
    {
        for (key, value) in fields_object {
            receipt_object.insert(key.clone(), value.clone());
        }
    }
    with_receipt_hash(receipt)
}

fn projection_facts(inspection: &StoreInspection) -> Value {
    let Some(projection) = &inspection.projection else {
        return Value::Null;
    };
    json!({
        "schema": projection.schema,
        "store_id": projection.store_id,
        "remote_hint": projection.remote_hint,
        "source_blake3": projection.source_identity,
        "non_claims": projection.non_claims,
        "store_identity": inspection.identity.as_ref().map(|identity| identity.store_identity.as_str()),
    })
}

fn required_non_claims() -> Vec<String> {
    LIFECYCLE_STORE_REQUIRED_NON_CLAIMS
        .iter()
        .map(|value| (*value).to_string())
        .collect()
}

fn parse_setup_options(args: &[String]) -> Result<SetupOptions, Box<dyn Error>> {
    let store_id = required_leading_value(args, SETUP_COMMAND)?;
    let path = required_unique_option(args, "--path", SETUP_COMMAND)?;
    let remote_hint = unique_option(args, "--remote")?;
    validate_argument_tokens(args, &["--path", "--remote"], true, SETUP_COMMAND)?;
    Ok(SetupOptions {
        store_id,
        path: PathBuf::from(path),
        remote_hint,
        execute: flag_present_once(args, "--execute")?,
    })
}

fn parse_register_options(args: &[String]) -> Result<RegisterOptions, Box<dyn Error>> {
    let path = required_leading_value(args, REGISTER_COMMAND)?;
    let store_id = unique_option(args, "--id")?;
    validate_argument_tokens(args, &["--id"], true, REGISTER_COMMAND)?;
    Ok(RegisterOptions {
        path: PathBuf::from(path),
        store_id,
        execute: flag_present_once(args, "--execute")?,
    })
}

fn parse_unregister_options(args: &[String]) -> Result<UnregisterOptions, Box<dyn Error>> {
    let store_id = required_leading_value(args, UNREGISTER_COMMAND)?;
    validate_argument_tokens(args, &[], true, UNREGISTER_COMMAND)?;
    Ok(UnregisterOptions {
        store_id,
        execute: flag_present_once(args, "--execute")?,
    })
}

fn required_leading_value(args: &[String], command: &str) -> Result<String, Box<dyn Error>> {
    let Some(value) = args.first() else {
        return Err(format!("{command} requires one positional value").into());
    };
    if value.starts_with('-') {
        return Err(format!("{command} requires one positional value before options").into());
    }
    Ok(value.clone())
}

fn required_unique_option(
    args: &[String],
    option: &str,
    command: &str,
) -> Result<String, Box<dyn Error>> {
    unique_option(args, option)?.ok_or_else(|| format!("{command} requires {option} VALUE").into())
}

fn unique_option(args: &[String], option: &str) -> Result<Option<String>, Box<dyn Error>> {
    let matches = args
        .windows(POSITIONAL_LIMIT_WITH_SUBCOMMAND)
        .filter(|pair| pair[0] == option)
        .map(|pair| pair[1].clone())
        .collect::<Vec<_>>();
    if matches.len() > 1 {
        return Err(format!("option occurs more than once: {option}").into());
    }
    if args.last().is_some_and(|value| value == option) {
        return Err(format!("option requires a value: {option}").into());
    }
    Ok(matches.into_iter().next())
}

fn flag_present_once(args: &[String], flag: &str) -> Result<bool, Box<dyn Error>> {
    let count = args.iter().filter(|value| value.as_str() == flag).count();
    if count > 1 {
        Err(format!("flag occurs more than once: {flag}").into())
    } else {
        Ok(count == 1)
    }
}

fn validate_argument_tokens(
    args: &[String],
    valued_options: &[&str],
    allow_execute: bool,
    command: &str,
) -> Result<(), Box<dyn Error>> {
    let mut index = 1;
    while index < args.len() {
        let token = &args[index];
        if valued_options.contains(&token.as_str()) {
            let value_index = index.saturating_add(1);
            let Some(value) = args.get(value_index) else {
                return Err(format!("option requires a value: {token}").into());
            };
            if value.starts_with("--") {
                return Err(format!("option requires a value: {token}").into());
            }
            index = value_index.saturating_add(1);
        } else if allow_execute && token == "--execute" {
            index = index.saturating_add(1);
        } else {
            return Err(format!("unexpected argument for {command}: {token}").into());
        }
    }
    Ok(())
}

fn require_no_arguments(args: &[String], command: &str) -> Result<(), Box<dyn Error>> {
    if args.is_empty() {
        Ok(())
    } else {
        Err(format!("{command} does not accept arguments").into())
    }
}

fn require_one_positional(args: &[String], command: &str) -> Result<String, Box<dyn Error>> {
    if args.len() == 1 && !args[0].starts_with('-') {
        Ok(args[0].clone())
    } else {
        Err(format!("{command} requires exactly one ID").into())
    }
}

fn optional_one_positional(
    args: &[String],
    command: &str,
) -> Result<Option<String>, Box<dyn Error>> {
    match args {
        [] => Ok(None),
        [value] if !value.starts_with('-') => Ok(Some(value.clone())),
        _ => Err(format!("{command} accepts at most one ID").into()),
    }
}

fn quote_argument(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn diagnostic(code: &str, field: &str, subject: &str, message: &str) -> LifecycleStoreDiagnostic {
    LifecycleStoreDiagnostic {
        code: code.to_string(),
        field: field.to_string(),
        subject: subject.to_string(),
        message: message.to_string(),
    }
}

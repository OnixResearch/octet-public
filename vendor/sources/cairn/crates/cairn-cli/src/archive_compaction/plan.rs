use std::{
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    build_compaction_plan, canonical_json_hash, compaction_genesis_hash, validate_compaction_plan,
    CompactionPlan, Policy, LOSSLESS_PACK_PROFILE,
};

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_option, read_string_option,
    CapabilityRoot, CapabilityRootKind,
};

use super::{
    execute::execute_saved_plan,
    io::{read_json, write_json_atomic},
    scan::{scan_repository, RepositorySnapshot},
};

pub(super) fn run_compact_archive(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.sync_archive.compaction.plan.read_only]
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    if args.iter().any(|arg| arg == "--execute") {
        let plan_path = read_string_option(&args, "--plan")
            .map(PathBuf::from)
            .ok_or("compact archive --execute requires --plan <file>")?;
        let receipt = execute_saved_plan(&root, &policy, &plan_path)?;
        print_json(serde_json::to_value(receipt)?)?;
        return Ok(());
    }
    let profile =
        read_string_option(&args, "--profile").unwrap_or_else(|| LOSSLESS_PACK_PROFILE.to_string());
    let (plan, _) = current_plan(&root, &policy, &profile)?;
    if let Some(out) = read_string_option(&args, "--out").map(PathBuf::from) {
        write_external_json(&out, serde_json::to_value(&plan)?)?;
    }
    print_json(serde_json::to_value(&plan)?)?;
    if !plan.blockers.is_empty() {
        return Err("archive compaction plan is blocked".into());
    }
    Ok(())
}

pub(super) fn current_plan(
    root: &Path,
    policy: &Policy,
    profile: &str,
) -> Result<(CompactionPlan, RepositorySnapshot), Box<dyn Error>> {
    let policy_hash = canonical_json_hash(&policy.raw);
    let snapshot = scan_repository(root, &policy_hash)?;
    let previous = snapshot
        .ledger
        .as_ref()
        .map(|ledger| ledger.latest_chain_hash.clone())
        .unwrap_or_else(|| compaction_genesis_hash(&policy_hash));
    let plan = build_compaction_plan(
        &snapshot.archives,
        &snapshot.pointers,
        &policy_hash,
        profile,
        &previous,
        &snapshot.issues,
    );
    Ok((plan, snapshot))
}

pub(super) fn read_saved_plan(path: &Path) -> Result<CompactionPlan, Box<dyn Error>> {
    let (parent, relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Repository, path, false)?;
    let plan: CompactionPlan = read_json(&parent, &relative)?;
    let issues = validate_compaction_plan(&plan);
    if issues.is_empty() {
        Ok(plan)
    } else {
        Err(format!("saved compaction plan is invalid: {issues:?}").into())
    }
}

fn write_external_json(path: &Path, value: serde_json::Value) -> Result<(), Box<dyn Error>> {
    let (parent, relative) =
        CapabilityRoot::open_file_parent(CapabilityRootKind::Repository, path, true)?;
    write_json_atomic(&parent, &relative, &value)
}

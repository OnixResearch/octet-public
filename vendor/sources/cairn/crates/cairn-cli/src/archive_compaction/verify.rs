use std::{collections::BTreeSet, error::Error, path::Path};

use cairn_core::{
    build_compaction_plan, build_unpack_plan, build_verification_receipt, canonical_json_hash,
    compaction_genesis_hash, empty_compaction_ledger, mutation_manifest_path, pack_manifest_path,
    validate_compaction_ledger, verify_pack_objects, CompactionIssue,
    CompactionVerificationReceipt, Policy, LOSSLESS_PACK_PROFILE,
};
use serde_json::Value;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_option, CapabilityRoot,
    CapabilityRootKind,
};

use super::{
    io::{path_exists, read_json},
    scan::{issue, load_pack_and_objects, scan_repository},
};

pub(super) fn run_compact_verify(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args, "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let receipt = verify_repository(&root, &policy)?;
    let valid = receipt.valid;
    print_json(serde_json::to_value(receipt)?)?;
    if !valid {
        return Err("archive compaction verification failed".into());
    }
    Ok(())
}

pub(super) fn verify_repository(
    root: &Path,
    policy: &Policy,
) -> Result<CompactionVerificationReceipt, Box<dyn Error>> {
    // r[impl cairn.sync_archive.compaction.verify]
    // r[impl cairn.sync_archive.compaction.verify.valid]
    // r[impl cairn.sync_archive.compaction.verify.unsafe_pointer]
    // r[impl cairn.sync_archive.compaction.boundary]
    // r[impl cairn.sync_archive.compaction.boundary.cli_shell]
    let policy_hash = canonical_json_hash(&policy.raw);
    let snapshot = scan_repository(root, &policy_hash)?;
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    let ledger = snapshot
        .ledger
        .clone()
        .unwrap_or_else(|| empty_compaction_ledger(&policy_hash));
    let mut issues = snapshot.issues.clone();
    issues.extend(validate_compaction_ledger(&ledger, &snapshot.receipts));

    let previous = snapshot
        .ledger
        .as_ref()
        .map(|ledger| ledger.latest_chain_hash.clone())
        .unwrap_or_else(|| compaction_genesis_hash(&policy_hash));
    let state = build_compaction_plan(
        &snapshot.archives,
        &snapshot.pointers,
        &policy_hash,
        LOSSLESS_PACK_PROFILE,
        &previous,
        &[],
    );
    issues.extend(
        state
            .blockers
            .into_iter()
            .filter(|blocker| blocker.code != "archive_compaction.no_candidates"),
    );

    let receipt_pack_roots = snapshot
        .receipts
        .iter()
        .map(|receipt| receipt.pack_root.as_str())
        .collect::<BTreeSet<_>>();
    let mut verified_pack_roots = Vec::new();
    let mut observed_pack_roots = BTreeSet::new();
    let mut referenced_objects = BTreeSet::new();
    for pointer in &snapshot.pointers {
        if pointer.pack_manifest_path != pack_manifest_path(&pointer.pack_root) {
            issues.push(issue(
                "archive_compaction.pointer_manifest_path_mismatch",
                &pointer.archive_id,
                "pointer manifest path must be canonical for its pack root",
            ));
            continue;
        }
        if !receipt_pack_roots.contains(pointer.pack_root.as_str()) {
            issues.push(issue(
                "archive_compaction.pointer_receipt_missing",
                &pointer.archive_id,
                "every retained pointer pack must be bound by a ledger receipt",
            ));
        }
        let first_pack_observation = observed_pack_roots.insert(pointer.pack_root.clone());
        match load_pack_and_objects(&repository, &pointer.pack_manifest_path) {
            Ok(loaded) => {
                referenced_objects.extend(
                    loaded
                        .pack
                        .objects
                        .iter()
                        .map(|object| object.content_hash.clone()),
                );
                let mut pack_issues = loaded.issues;
                pack_issues.extend(verify_pack_objects(&loaded.pack, &loaded.objects));
                if first_pack_observation {
                    let manifest_archives = loaded
                        .pack
                        .archives
                        .iter()
                        .map(|archive| archive.archive_id.as_str())
                        .collect::<BTreeSet<_>>();
                    let retained_archives = snapshot
                        .pointers
                        .iter()
                        .filter(|candidate| candidate.pack_root == pointer.pack_root)
                        .map(|candidate| candidate.archive_id.as_str())
                        .collect::<BTreeSet<_>>();
                    if manifest_archives != retained_archives {
                        pack_issues.push(issue(
                            "archive_compaction.pack_pointer_coverage_mismatch",
                            &pointer.pack_root,
                            "every archive in a retained pack must have exactly one retained pointer",
                        ));
                    }
                }
                let unpack = build_unpack_plan(pointer, &loaded.pack, &loaded.objects);
                pack_issues.extend(unpack.blockers);
                if pack_issues.is_empty() {
                    verified_pack_roots.push(loaded.pack.pack_root);
                } else {
                    issues.append(&mut pack_issues);
                }
            }
            Err(_) => issues.push(issue(
                "archive_compaction.pack_unreadable",
                &pointer.pack_manifest_path,
                "retained pointer pack must be readable and valid",
            )),
        }
    }
    for receipt in &snapshot.receipts {
        if !observed_pack_roots.contains(&receipt.pack_root) {
            issues.push(issue(
                "archive_compaction.receipt_pointer_missing",
                &receipt.pack_root,
                "every compaction receipt pack must retain at least one pointer",
            ));
        }
        verify_mutation_manifest(&repository, receipt, &mut issues);
    }
    verify_no_orphaned_artifacts(
        &repository,
        &observed_pack_roots,
        &referenced_objects,
        &snapshot,
        &mut issues,
    )?;
    Ok(build_verification_receipt(
        &policy_hash,
        &ledger,
        verified_pack_roots,
        issues,
    ))
}

fn verify_no_orphaned_artifacts(
    repository: &CapabilityRoot,
    pack_roots: &BTreeSet<String>,
    object_hashes: &BTreeSet<String>,
    snapshot: &super::scan::RepositorySnapshot,
    issues: &mut Vec<CompactionIssue>,
) -> Result<(), Box<dyn Error>> {
    verify_directory_members(
        repository,
        Path::new(".cairn/compaction/v1/packs"),
        pack_roots,
        true,
        "archive_compaction.orphan_pack",
        issues,
    )?;
    verify_directory_members(
        repository,
        Path::new(".cairn/compaction/v1/objects/blake3"),
        object_hashes,
        false,
        "archive_compaction.orphan_object",
        issues,
    )?;
    let receipt_names = snapshot
        .receipts
        .iter()
        .map(|receipt| format!("{}.json", receipt.pack_root))
        .collect::<BTreeSet<_>>();
    verify_directory_members(
        repository,
        Path::new(".cairn/compaction/v1/receipts"),
        &receipt_names,
        false,
        "archive_compaction.orphan_receipt",
        issues,
    )?;
    let mutation_names = snapshot
        .receipts
        .iter()
        .map(|receipt| format!("{}.json", receipt.pack_root))
        .collect::<BTreeSet<_>>();
    verify_directory_members(
        repository,
        Path::new(".cairn/compaction/v1/mutations"),
        &mutation_names,
        false,
        "archive_compaction.orphan_mutation_manifest",
        issues,
    )?;
    Ok(())
}

fn verify_directory_members(
    repository: &CapabilityRoot,
    directory: &Path,
    expected_names: &BTreeSet<String>,
    expect_directories: bool,
    orphan_code: &str,
    issues: &mut Vec<CompactionIssue>,
) -> Result<(), Box<dyn Error>> {
    if !path_exists(repository, directory)? {
        return Ok(());
    }
    for entry in repository.entries(directory)? {
        let name = entry.file_name().to_string_lossy().to_string();
        let expected_type = if expect_directories {
            entry.file_type().is_dir()
        } else {
            entry.file_type().is_file()
        };
        if entry.file_type().is_symlink() || !expected_type || !expected_names.contains(&name) {
            issues.push(issue(
                orphan_code,
                &directory.join(&name).to_string_lossy(),
                "compaction storage artifacts must be regular, typed, and reachable from retained pointers and the ledger",
            ));
        }
    }
    Ok(())
}

fn verify_mutation_manifest(
    repository: &CapabilityRoot,
    receipt: &cairn_core::CompactionReceipt,
    issues: &mut Vec<CompactionIssue>,
) {
    let relative = mutation_manifest_path(&receipt.pack_root);
    let manifest: Value = match read_json(repository, Path::new(&relative)) {
        Ok(manifest) => manifest,
        Err(_) => {
            issues.push(issue(
                "archive_compaction.mutation_manifest_unreadable",
                &relative,
                "receipt mutation manifest must be readable bounded JSON",
            ));
            return;
        }
    };
    let stated_hash = manifest
        .get("mutation_manifest_hash")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let observed_hash = manifest
        .get("actions")
        .map(canonical_json_hash)
        .unwrap_or_default();
    if stated_hash != receipt.mutation_manifest_hash
        || observed_hash != receipt.mutation_manifest_hash
    {
        issues.push(issue(
            "archive_compaction.mutation_manifest_hash_mismatch",
            &relative,
            "mutation manifest actions must match the receipt-bound hash",
        ));
    }
    if manifest.get("plan_hash").and_then(Value::as_str) != Some(receipt.plan_hash.as_str()) {
        issues.push(issue(
            "archive_compaction.mutation_plan_hash_mismatch",
            &relative,
            "mutation manifest must bind the receipt plan hash",
        ));
    }
}

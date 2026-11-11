use std::{
    collections::BTreeMap,
    error::Error,
    io,
    path::{Path, PathBuf},
};

use cairn_core::{
    append_compaction_receipt, blake3_hex, build_compaction_plan, build_compaction_receipt,
    canonical_json_hash, compaction_genesis_hash, mutation_manifest_path, object_path,
    pack_manifest_path, pointer_path, prepared_pointer_path, receipt_path,
    validate_compaction_plan, verify_pack_objects, ArchiveInput, CompactionPlan, CompactionReceipt,
    PackObjectInput, Policy,
};
use serde_json::json;

use crate::support::{CapabilityRoot, CapabilityRootKind};

use super::{
    io::{path_exists, write_bytes_atomic, write_json_atomic},
    plan::read_saved_plan,
    scan::{load_pack_and_objects, scan_repository, RepositorySnapshot},
};

pub(super) fn execute_saved_plan(
    root: &Path,
    policy: &Policy,
    plan_path: &Path,
) -> Result<CompactionReceipt, Box<dyn Error>> {
    // r[impl cairn.sync_archive.compaction.lossless_pack]
    // r[impl cairn.sync_archive.compaction.lossless_pack.write_first]
    // r[impl cairn.sync_archive.compaction.lossless_pack.blocked]
    // r[impl cairn.sync_archive.compaction.plan]
    // r[impl cairn.sync_archive.compaction.plan.stale]
    // r[impl cairn.sync_archive.compaction.boundary]
    // r[impl cairn.sync_archive.compaction.boundary.cli_shell]
    let saved = read_saved_plan(plan_path)?;
    let policy_hash = canonical_json_hash(&policy.raw);
    if saved.policy_hash != policy_hash {
        return Err("saved compaction plan policy hash is stale".into());
    }
    let snapshot = scan_repository(root, &policy_hash)?;
    let previous = snapshot
        .ledger
        .as_ref()
        .map(|ledger| ledger.latest_chain_hash.clone())
        .unwrap_or_else(|| compaction_genesis_hash(&policy_hash));
    let current = build_compaction_plan(
        &snapshot.archives,
        &snapshot.pointers,
        &policy_hash,
        &saved.profile,
        &previous,
        &snapshot.issues,
    );
    if current.plan_hash != saved.plan_hash || !current.blockers.is_empty() {
        return Err(format!(
            "saved compaction plan is stale or blocked: {:?}",
            current.blockers
        )
        .into());
    }
    let plan_issues = validate_compaction_plan(&saved);
    if !plan_issues.is_empty() {
        return Err(format!("saved compaction plan is invalid: {plan_issues:?}").into());
    }

    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    let objects = source_objects(&saved, &snapshot)?;
    preflight_outputs(&repository, &saved, &objects)?;
    write_pack_artifacts(&repository, &saved, &objects)?;
    verify_written_pack(&repository, &saved)?;
    write_prepared_pointers(&repository, &saved)?;
    write_mutation_manifest(&repository, &saved)?;

    let sequence = next_sequence(&snapshot)?;
    let receipt = build_compaction_receipt(&saved, sequence);
    let ledger = append_compaction_receipt(snapshot.ledger.as_ref(), &snapshot.receipts, &receipt)
        .map_err(|issues| format!("compaction receipt cannot append: {issues:?}"))?;
    write_json_atomic(
        &repository,
        Path::new(&receipt_path(&saved.pack.pack_root)),
        &serde_json::to_value(&receipt)?,
    )?;
    write_json_atomic(
        &repository,
        Path::new(&cairn_core::ledger_path()),
        &serde_json::to_value(&ledger)?,
    )?;

    verify_written_pack(&repository, &saved)?;
    prune_and_finalize(&repository, &saved)?;
    verify_post_state(root, policy, &saved)?;
    Ok(receipt)
}

fn source_objects(
    plan: &CompactionPlan,
    snapshot: &RepositorySnapshot,
) -> Result<Vec<PackObjectInput>, Box<dyn Error>> {
    let candidates = plan
        .candidates
        .iter()
        .map(String::as_str)
        .collect::<std::collections::BTreeSet<_>>();
    let mut by_hash = BTreeMap::<String, Vec<u8>>::new();
    for archive in &snapshot.archives {
        if !candidates.contains(archive.archive_id.as_str()) {
            continue;
        }
        add_archive_objects(archive, &mut by_hash)?;
    }
    let objects = plan
        .pack
        .objects
        .iter()
        .map(|descriptor| {
            by_hash
                .get(&descriptor.content_hash)
                .cloned()
                .map(|bytes| PackObjectInput {
                    content_hash: descriptor.content_hash.clone(),
                    bytes,
                })
                .ok_or_else(|| {
                    format!(
                        "source bytes missing for planned object {}",
                        descriptor.content_hash
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(objects)
}

fn add_archive_objects(
    archive: &ArchiveInput,
    by_hash: &mut BTreeMap<String, Vec<u8>>,
) -> Result<(), Box<dyn Error>> {
    for file in &archive.files {
        let content_hash = blake3_hex(&file.bytes);
        match by_hash.get(&content_hash) {
            Some(existing) if existing != &file.bytes => {
                return Err("BLAKE3 collision encountered while building archive pack".into())
            }
            Some(_) => {}
            None => {
                by_hash.insert(content_hash, file.bytes.clone());
            }
        }
    }
    Ok(())
}

fn preflight_outputs(
    repository: &CapabilityRoot,
    plan: &CompactionPlan,
    objects: &[PackObjectInput],
) -> Result<(), Box<dyn Error>> {
    for object in objects {
        let relative = object_path(&object.content_hash);
        match repository.read(Path::new(&relative)) {
            Ok(existing) if existing == object.bytes => {}
            Ok(_) => return Err(format!("existing object differs from plan: {relative}").into()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    let mut must_not_exist = vec![
        pack_manifest_path(&plan.pack.pack_root),
        mutation_manifest_path(&plan.pack.pack_root),
        receipt_path(&plan.pack.pack_root),
    ];
    for pointer in &plan.planned_pointers {
        must_not_exist.push(pointer_path(&pointer.archive_id));
        must_not_exist.push(prepared_pointer_path(
            &plan.pack.pack_root,
            &pointer.archive_id,
        ));
    }
    for relative in must_not_exist {
        if path_exists(repository, Path::new(&relative))? {
            return Err(format!("planned output already exists: {relative}").into());
        }
    }
    Ok(())
}

fn write_pack_artifacts(
    repository: &CapabilityRoot,
    plan: &CompactionPlan,
    objects: &[PackObjectInput],
) -> Result<(), Box<dyn Error>> {
    for object in objects {
        write_bytes_atomic(
            repository,
            Path::new(&object_path(&object.content_hash)),
            &object.bytes,
        )?;
    }
    write_json_atomic(
        repository,
        Path::new(&pack_manifest_path(&plan.pack.pack_root)),
        &serde_json::to_value(&plan.pack)?,
    )
}

fn verify_written_pack(
    repository: &CapabilityRoot,
    plan: &CompactionPlan,
) -> Result<(), Box<dyn Error>> {
    let loaded = load_pack_and_objects(repository, &pack_manifest_path(&plan.pack.pack_root))?;
    let mut issues = loaded.issues;
    issues.extend(verify_pack_objects(&loaded.pack, &loaded.objects));
    if loaded.pack != plan.pack {
        return Err("written pack manifest differs from the saved plan".into());
    }
    if !issues.is_empty() {
        return Err(format!("written archive pack did not verify: {issues:?}").into());
    }
    Ok(())
}

fn write_prepared_pointers(
    repository: &CapabilityRoot,
    plan: &CompactionPlan,
) -> Result<(), Box<dyn Error>> {
    for pointer in &plan.planned_pointers {
        write_json_atomic(
            repository,
            Path::new(&prepared_pointer_path(
                &plan.pack.pack_root,
                &pointer.archive_id,
            )),
            &serde_json::to_value(pointer)?,
        )?;
    }
    Ok(())
}

fn write_mutation_manifest(
    repository: &CapabilityRoot,
    plan: &CompactionPlan,
) -> Result<(), Box<dyn Error>> {
    let manifest = json!({
        "schema_version": "cairn.archive-compaction-mutation-manifest.v1",
        "plan_hash": plan.plan_hash,
        "mutation_manifest_hash": plan.mutation_manifest_hash,
        "actions": plan.mutation_manifest,
    });
    write_json_atomic(
        repository,
        Path::new(&mutation_manifest_path(&plan.pack.pack_root)),
        &manifest,
    )
}

fn next_sequence(snapshot: &RepositorySnapshot) -> Result<u64, Box<dyn Error>> {
    let current = snapshot
        .ledger
        .as_ref()
        .map_or(0_usize, |ledger| ledger.entries.len());
    let current = u64::try_from(current).map_err(|_| "compaction ledger is too large")?;
    current
        .checked_add(1)
        .ok_or_else(|| "compaction ledger sequence overflow".into())
}

fn prune_and_finalize(
    repository: &CapabilityRoot,
    plan: &CompactionPlan,
) -> Result<(), Box<dyn Error>> {
    let pointer_parent = Path::new(cairn_core::COMPACTION_ROOT).join("pointers");
    repository.create_dir_all(&pointer_parent)?;
    for pointer in &plan.planned_pointers {
        let archive_path = PathBuf::from(".cairn/archive").join(&pointer.archive_id);
        repository.remove_dir_all(&archive_path)?;
        let prepared = prepared_pointer_path(&plan.pack.pack_root, &pointer.archive_id);
        let retained = pointer_path(&pointer.archive_id);
        repository.rename(Path::new(&prepared), repository, Path::new(&retained))?;
    }
    let prepared_root = PathBuf::from(cairn_core::COMPACTION_ROOT)
        .join("prepared")
        .join(&plan.pack.pack_root);
    repository.remove_dir_all(&prepared_root)?;
    Ok(())
}

fn verify_post_state(
    root: &Path,
    policy: &Policy,
    saved: &CompactionPlan,
) -> Result<(), Box<dyn Error>> {
    let policy_hash = canonical_json_hash(&policy.raw);
    let snapshot = scan_repository(root, &policy_hash)?;
    let previous = snapshot
        .ledger
        .as_ref()
        .map(|ledger| ledger.latest_chain_hash.clone())
        .unwrap_or_else(|| compaction_genesis_hash(&policy_hash));
    let observed = build_compaction_plan(
        &snapshot.archives,
        &snapshot.pointers,
        &policy_hash,
        &saved.profile,
        &previous,
        &[],
    );
    if observed.pre_state_root != saved.post_state_root {
        return Err("archive compaction post-state root does not match the saved plan".into());
    }
    let unexpected = snapshot
        .issues
        .into_iter()
        .filter(|issue| issue.code != "archive_compaction.no_candidates")
        .collect::<Vec<_>>();
    if !unexpected.is_empty() {
        return Err(format!("archive compaction post-state is invalid: {unexpected:?}").into());
    }
    Ok(())
}

use std::{
    collections::BTreeMap,
    error::Error,
    path::{Path, PathBuf},
};

use cairn_core::{
    blake3_hex, build_archive_manifest, build_unpack_plan, pointer_path, safe_archive_id,
    ArchivePointer, Policy,
};
use serde_json::json;

use crate::support::{
    default_policy_path, load_policy_file, print_json, read_option, read_string_option,
    CapabilityRoot, CapabilityRootKind,
};

use super::{
    io::{path_exists, read_json},
    scan::{load_pack_and_objects, scan_repository},
    verify::verify_repository,
};

const UNPACK_PREPARED_DIRECTORY: &str = ".cairn/compaction/v1/unpack-prepared";
const PACK_ROOT_DISPLAY_PREFIX_LENGTH: usize = 12;

pub(super) fn run_compact_unpack(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let root = read_option(args.clone(), "--root", ".");
    let policy_path = read_option(args.clone(), "--policy", default_policy_path());
    let policy = load_policy_file(policy_path)?;
    let archive_id =
        read_string_option(&args, "--archive").ok_or("compact unpack requires --archive <id>")?;
    if !safe_archive_id(&archive_id) {
        return Err("compact unpack archive identity is unsafe".into());
    }
    let execute = args.iter().any(|arg| arg == "--execute");
    let output = unpack_archive(&root, &policy, &archive_id, execute)?;
    print_json(output)
}

pub(super) fn unpack_archive(
    root: &Path,
    policy: &Policy,
    archive_id: &str,
    execute: bool,
) -> Result<serde_json::Value, Box<dyn Error>> {
    // r[impl cairn.sync_archive.compaction.lossless_pack]
    // r[impl cairn.sync_archive.compaction.boundary]
    let verification = verify_repository(root, policy)?;
    if !verification.valid {
        return Err(format!(
            "archive compaction state must verify before unpacking: {:?}",
            verification.issues
        )
        .into());
    }
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    let pointer: ArchivePointer = read_json(&repository, Path::new(&pointer_path(archive_id)))?;
    let loaded = load_pack_and_objects(&repository, &pointer.pack_manifest_path)?;
    let mut plan = build_unpack_plan(&pointer, &loaded.pack, &loaded.objects);
    plan.blockers.extend(loaded.issues);
    let objects = loaded.objects;
    plan.blockers.sort();
    plan.blockers.dedup();
    let destination = PathBuf::from(".cairn/archive").join(archive_id);
    let already_restored = restored_archive_matches(root, policy, archive_id, &plan.archive_root)?;
    if path_exists(&repository, &destination)? && !already_restored {
        return Err("archive unpack destination exists with different content".into());
    }
    if !plan.blockers.is_empty() {
        return Err(format!("archive unpack plan is blocked: {:?}", plan.blockers).into());
    }
    if !execute || already_restored {
        return Ok(json!({
            "command": "compact unpack",
            "execute": execute,
            "mutated": false,
            "already_restored": already_restored,
            "plan": plan,
        }));
    }

    let temporary = unpack_temporary_path(archive_id, &plan.pack_root);
    if path_exists(&repository, &temporary)? {
        return Err("prepared unpack directory already exists; verify interrupted state".into());
    }
    repository.create_dir_all(&temporary)?;
    let object_map = objects
        .iter()
        .map(|object| (object.content_hash.as_str(), object.bytes.as_slice()))
        .collect::<BTreeMap<_, _>>();
    for file in &plan.files {
        let bytes = object_map
            .get(file.content_hash.as_str())
            .ok_or("unpack object bytes are missing")?;
        let output_path = temporary.join(&file.path);
        let parent = output_path.parent().unwrap_or(temporary.as_path());
        repository.create_dir_all(parent)?;
        repository.write(&output_path, bytes)?;
        let restored = repository.read(&output_path)?;
        let restored_length = u64::try_from(restored.len())
            .map_err(|_| "restored archive file length does not fit in u64")?;
        if restored_length != file.byte_length || blake3_hex(&restored) != file.content_hash {
            return Err(format!("restored archive file failed verification: {}", file.path).into());
        }
    }
    repository.create_dir_all(Path::new(".cairn/archive"))?;
    repository.rename(&temporary, &repository, &destination)?;
    let post_verification = verify_repository(root, policy)?;
    if !post_verification.valid {
        return Err(format!(
            "archive state failed verification after unpack: {:?}",
            post_verification.issues
        )
        .into());
    }
    Ok(json!({
        "command": "compact unpack",
        "execute": true,
        "mutated": true,
        "already_restored": false,
        "plan": plan,
        "verification_receipt_hash": post_verification.receipt_hash,
    }))
}

fn restored_archive_matches(
    root: &Path,
    policy: &Policy,
    archive_id: &str,
    expected_root: &str,
) -> Result<bool, Box<dyn Error>> {
    let policy_hash = cairn_core::canonical_json_hash(&policy.raw);
    let snapshot = scan_repository(root, &policy_hash)?;
    let Some(archive) = snapshot
        .archives
        .iter()
        .find(|archive| archive.archive_id == archive_id)
    else {
        return Ok(false);
    };
    let manifest = build_archive_manifest(archive)
        .map_err(|issues| format!("restored archive manifest is invalid: {issues:?}"))?;
    Ok(manifest.archive_root == expected_root)
}

fn unpack_temporary_path(archive_id: &str, pack_root: &str) -> PathBuf {
    let prefix_length = PACK_ROOT_DISPLAY_PREFIX_LENGTH.min(pack_root.len());
    PathBuf::from(UNPACK_PREPARED_DIRECTORY)
        .join(format!("{archive_id}-{}", &pack_root[..prefix_length]))
}

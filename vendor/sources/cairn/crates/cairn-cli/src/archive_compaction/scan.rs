use std::{
    collections::BTreeSet,
    io,
    path::{Path, PathBuf},
};

use bounded_tree_core::RelativePath;
use cairn_core::{
    object_path, pointer_path, safe_relative_path, validate_compaction_ledger, ArchiveFileInput,
    ArchiveInput, ArchivePointer, CompactionIssue, CompactionLedger, CompactionReceipt,
    PackManifest, PackObjectInput,
};

use crate::support::{CapabilityRoot, CapabilityRootKind};

use super::io::{path_exists, read_json, read_json_if_exists};

const ARCHIVE_DIRECTORY: &str = ".cairn/archive";
const POINTER_DIRECTORY: &str = ".cairn/compaction/v1/pointers";
const PREPARED_DIRECTORY: &str = ".cairn/compaction/v1/prepared";
const UNPACK_PREPARED_DIRECTORY: &str = ".cairn/compaction/v1/unpack-prepared";
const LEDGER_FILE: &str = ".cairn/compaction/v1/ledger.json";
const ARCHIVE_SCAN_MAX_DEPTH: usize = 32;
const ARCHIVE_SCAN_MAX_FILES: usize = 10_000;
const ARCHIVE_FILE_MAX_BYTES: u64 = 64 * 1_048_576;
const ARCHIVE_TOTAL_MAX_BYTES: u64 = 1_073_741_824;

// r[impl cairn.bounded_tree_adoption.prerequisite]
// r[impl cairn.bounded_tree_adoption.archive_policy]
// r[impl cairn.bounded_tree_adoption.boundary]
// r[impl cairn.bounded_tree_adoption.rollback]

pub(super) struct RepositorySnapshot {
    pub(super) archives: Vec<ArchiveInput>,
    pub(super) pointers: Vec<ArchivePointer>,
    pub(super) ledger: Option<CompactionLedger>,
    pub(super) receipts: Vec<CompactionReceipt>,
    pub(super) issues: Vec<CompactionIssue>,
}

pub(super) struct LoadedPack {
    pub(super) pack: PackManifest,
    pub(super) objects: Vec<PackObjectInput>,
    pub(super) issues: Vec<CompactionIssue>,
}

pub(super) fn scan_repository(
    root: &Path,
    _policy_hash: &str,
) -> Result<RepositorySnapshot, Box<dyn std::error::Error>> {
    // r[impl cairn.sync_archive.compaction.boundary]
    let repository = CapabilityRoot::open(CapabilityRootKind::Repository, root)?;
    let mut issues = Vec::new();
    let archives = scan_archives(&repository, &mut issues)?;
    let pointers = scan_pointers(&repository, &mut issues)?;
    scan_prepared_state(&repository, &mut issues)?;
    let ledger: Option<CompactionLedger> =
        read_json_if_exists(&repository, Path::new(LEDGER_FILE))?;
    let receipts = load_ledger_receipts(&repository, ledger.as_ref(), &mut issues);
    match &ledger {
        Some(ledger) => {
            issues.extend(validate_compaction_ledger(ledger, &receipts));
        }
        None if !pointers.is_empty() => issues.push(issue(
            "archive_compaction.ledger_missing",
            LEDGER_FILE,
            "retained pointers require an append-only compaction ledger",
        )),
        None => {}
    }
    issues.sort();
    issues.dedup();
    Ok(RepositorySnapshot {
        archives,
        pointers,
        ledger,
        receipts,
        issues,
    })
}

pub(super) fn load_pack_and_objects(
    repository: &CapabilityRoot,
    manifest_path: &str,
) -> Result<LoadedPack, Box<dyn std::error::Error>> {
    let mut issues = Vec::new();
    if !safe_relative_path(manifest_path) {
        return Ok(LoadedPack {
            pack: empty_pack(),
            objects: Vec::new(),
            issues: vec![issue(
                "archive_compaction.pointer_path_unsafe",
                manifest_path,
                "pack manifest path must be normalized and repository-relative",
            )],
        });
    }
    let pack: PackManifest = read_json(repository, Path::new(manifest_path))?;
    let mut objects = Vec::with_capacity(pack.objects.len());
    for descriptor in &pack.objects {
        if descriptor.byte_length > ARCHIVE_FILE_MAX_BYTES {
            issues.push(issue(
                "archive_compaction.object_too_large",
                &descriptor.content_hash,
                "pack object exceeds the bounded archive file limit",
            ));
            continue;
        }
        let relative = object_path(&descriptor.content_hash);
        match repository.read_bounded(Path::new(&relative), ARCHIVE_FILE_MAX_BYTES) {
            Ok(bytes) => objects.push(PackObjectInput {
                content_hash: descriptor.content_hash.clone(),
                bytes,
            }),
            Err(error) if error.kind() == io::ErrorKind::NotFound => issues.push(issue(
                "archive_compaction.object_missing",
                &relative,
                "declared pack object must exist",
            )),
            Err(_) => issues.push(issue(
                "archive_compaction.object_unreadable",
                &relative,
                "declared pack object must be a readable bounded regular file",
            )),
        }
    }
    Ok(LoadedPack {
        pack,
        objects,
        issues,
    })
}

fn scan_archives(
    repository: &CapabilityRoot,
    issues: &mut Vec<CompactionIssue>,
) -> Result<Vec<ArchiveInput>, Box<dyn std::error::Error>> {
    let archive_root = Path::new(ARCHIVE_DIRECTORY);
    if !path_exists(repository, archive_root)? {
        return Ok(Vec::new());
    }
    let mut entries = repository.entries(archive_root)?;
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    let mut archives = Vec::new();
    for entry in entries {
        let archive_id = entry.file_name().to_string_lossy().to_string();
        let archive_path = archive_root.join(&archive_id);
        if entry.file_type().is_symlink() {
            issues.push(issue(
                "archive_compaction.archive_symlink_rejected",
                &archive_path.to_string_lossy(),
                "archive candidates must be real directories",
            ));
            continue;
        }
        if !entry.file_type().is_dir() {
            continue;
        }
        let mut files = Vec::new();
        let mut total_bytes = 0_u64;
        collect_archive_files(
            repository,
            &archive_path,
            Path::new("."),
            0,
            &mut files,
            &mut total_bytes,
            issues,
        );
        archives.push(ArchiveInput { archive_id, files });
    }
    archives.sort_by(|left, right| left.archive_id.cmp(&right.archive_id));
    Ok(archives)
}

#[allow(clippy::too_many_arguments)]
fn collect_archive_files(
    repository: &CapabilityRoot,
    archive_root: &Path,
    relative_directory: &Path,
    depth: usize,
    files: &mut Vec<ArchiveFileInput>,
    total_bytes: &mut u64,
    issues: &mut Vec<CompactionIssue>,
) {
    if depth > ARCHIVE_SCAN_MAX_DEPTH {
        issues.push(issue(
            "archive_compaction.scan_depth_exceeded",
            &archive_root.to_string_lossy(),
            "archive scan depth must stay within the configured bound",
        ));
        return;
    }
    let current = if relative_directory == Path::new(".") {
        archive_root.to_path_buf()
    } else {
        archive_root.join(relative_directory)
    };
    let Ok(mut entries) = repository.entries(&current) else {
        issues.push(issue(
            "archive_compaction.directory_unreadable",
            &current.to_string_lossy(),
            "archive directory must remain readable during planning",
        ));
        return;
    };
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    for entry in entries {
        if files.len() >= ARCHIVE_SCAN_MAX_FILES {
            issues.push(issue(
                "archive_compaction.file_count_exceeded",
                &archive_root.to_string_lossy(),
                "archive file count exceeds the configured bound",
            ));
            return;
        }
        let relative = if relative_directory == Path::new(".") {
            PathBuf::from(entry.file_name())
        } else {
            relative_directory.join(entry.file_name())
        };
        // r[impl cairn.bounded_tree_adoption.archive_scan]
        // r[impl cairn.bounded_tree_adoption.core_boundary]
        if admit_shared_path(&relative, issues).is_none() {
            continue;
        }
        let full_path = archive_root.join(&relative);
        if entry.file_type().is_symlink() {
            issues.push(issue(
                "archive_compaction.archive_symlink_rejected",
                &full_path.to_string_lossy(),
                "archive packs do not follow symlinks",
            ));
            continue;
        }
        if entry.file_type().is_dir() {
            collect_archive_files(
                repository,
                archive_root,
                &relative,
                depth.saturating_add(1),
                files,
                total_bytes,
                issues,
            );
            continue;
        }
        if !entry.file_type().is_file() {
            issues.push(issue(
                "archive_compaction.special_file_rejected",
                &full_path.to_string_lossy(),
                "archive packs accept regular files only",
            ));
            continue;
        }
        match repository.read_bounded(&full_path, ARCHIVE_FILE_MAX_BYTES) {
            Ok(bytes) => {
                let byte_length = match u64::try_from(bytes.len()) {
                    Ok(length) => length,
                    Err(_) => {
                        issues.push(issue(
                            "archive_compaction.file_too_large",
                            &full_path.to_string_lossy(),
                            "archive file byte length must fit in u64",
                        ));
                        continue;
                    }
                };
                *total_bytes = total_bytes.saturating_add(byte_length);
                if *total_bytes > ARCHIVE_TOTAL_MAX_BYTES {
                    issues.push(issue(
                        "archive_compaction.total_bytes_exceeded",
                        &archive_root.to_string_lossy(),
                        "archive candidate bytes exceed the configured bound",
                    ));
                    return;
                }
                files.push(ArchiveFileInput {
                    path: relative.to_string_lossy().replace('\\', "/"),
                    bytes,
                });
            }
            Err(_) => issues.push(issue(
                "archive_compaction.file_unreadable",
                &full_path.to_string_lossy(),
                "archive file must remain readable during planning",
            )),
        }
    }
}

/// Admits one repository-relative path with shared bounded-tree exact-byte rules.
///
/// Returns the path only when every exact byte component is admitted. A
/// malformed path emits a bounded-path issue and yields `None` (permissive
/// continue), so a single bad member never blocks the rest of an archive.
fn admit_shared_path(relative: &Path, issues: &mut Vec<CompactionIssue>) -> Option<PathBuf> {
    let components = shared_path_components(relative)?;
    if RelativePath::new(components).is_err() {
        issues.push(issue(
            "archive_compaction.bounded_path_rejected",
            &relative.to_string_lossy(),
            "shared bounded observation rejected the exact component bytes",
        ));
        return None;
    }
    Some(relative.to_path_buf())
}

fn shared_path_components(relative: &Path) -> Option<Vec<Vec<u8>>> {
    let mut components = Vec::new();
    for component in relative.components() {
        let bytes = component.as_os_str().as_encoded_bytes().to_vec();
        if bytes.is_empty() {
            return None;
        }
        components.push(bytes);
    }
    if components.is_empty() {
        return None;
    }
    Some(components)
}

fn scan_pointers(
    repository: &CapabilityRoot,
    issues: &mut Vec<CompactionIssue>,
) -> Result<Vec<ArchivePointer>, Box<dyn std::error::Error>> {
    let pointer_root = Path::new(POINTER_DIRECTORY);
    if !path_exists(repository, pointer_root)? {
        return Ok(Vec::new());
    }
    let mut entries = repository.entries(pointer_root)?;
    entries.sort_by(|left, right| left.file_name().cmp(right.file_name()));
    let mut pointers = Vec::<ArchivePointer>::new();
    for entry in entries {
        let path = pointer_root.join(entry.file_name());
        if entry.file_type().is_symlink() || !entry.file_type().is_file() {
            issues.push(issue(
                "archive_compaction.pointer_file_invalid",
                &path.to_string_lossy(),
                "retained pointers must be regular JSON files",
            ));
            continue;
        }
        match read_json::<ArchivePointer>(repository, &path) {
            Ok(pointer) => {
                let expected_path = pointer_path(&pointer.archive_id);
                if path != Path::new(&expected_path) {
                    issues.push(issue(
                        "archive_compaction.pointer_location_mismatch",
                        &path.to_string_lossy(),
                        "retained pointer file name must be canonical for its archive identity",
                    ));
                }
                pointers.push(pointer);
            }
            Err(_) => issues.push(issue(
                "archive_compaction.pointer_unreadable",
                &path.to_string_lossy(),
                "retained pointer must be valid bounded JSON",
            )),
        }
    }
    pointers.sort_by(|left, right| left.archive_id.cmp(&right.archive_id));
    Ok(pointers)
}

fn scan_prepared_state(
    repository: &CapabilityRoot,
    issues: &mut Vec<CompactionIssue>,
) -> Result<(), Box<dyn std::error::Error>> {
    for directory in [PREPARED_DIRECTORY, UNPACK_PREPARED_DIRECTORY] {
        let prepared_root = Path::new(directory);
        if !path_exists(repository, prepared_root)? {
            continue;
        }
        let entries = repository.entries(prepared_root)?;
        if !entries.is_empty() {
            issues.push(issue(
                "archive_compaction.interrupted_execution",
                directory,
                "prepared state remains from an interrupted compaction or unpack; verify and recover before another execution",
            ));
        }
    }
    Ok(())
}

fn load_ledger_receipts(
    repository: &CapabilityRoot,
    ledger: Option<&CompactionLedger>,
    issues: &mut Vec<CompactionIssue>,
) -> Vec<CompactionReceipt> {
    let mut receipts = Vec::new();
    let Some(ledger) = ledger else {
        return receipts;
    };
    let mut seen_paths = BTreeSet::new();
    for entry in &ledger.entries {
        if !safe_relative_path(&entry.receipt_path)
            || !seen_paths.insert(entry.receipt_path.clone())
        {
            issues.push(issue(
                "archive_compaction.receipt_path_unsafe",
                &entry.receipt_path,
                "ledger receipt paths must be unique normalized repository-relative paths",
            ));
            continue;
        }
        match read_json(repository, Path::new(&entry.receipt_path)) {
            Ok(receipt) => receipts.push(receipt),
            Err(_) => issues.push(issue(
                "archive_compaction.receipt_unreadable",
                &entry.receipt_path,
                "every ledger entry must resolve to a valid bounded receipt",
            )),
        }
    }
    receipts
}

pub(super) fn issue(code: &str, path: &str, message: &str) -> CompactionIssue {
    CompactionIssue {
        code: code.to_string(),
        path: path.to_string(),
        message: message.to_string(),
    }
}

fn empty_pack() -> PackManifest {
    PackManifest {
        schema_version: String::new(),
        archives: Vec::new(),
        objects: Vec::new(),
        packed_archives_root: String::new(),
        pack_root: String::new(),
    }
}

use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicUsize, Ordering},
};

use cairn_core::LOSSLESS_PACK_PROFILE;

use crate::support::load_policy_file;

use super::{
    execute::execute_saved_plan,
    plan::{current_plan, run_compact_archive},
    scan::scan_repository,
    unpack::unpack_archive,
    verify::verify_repository,
};

const TEST_DIRECTORY_PREFIX: &str = "cairn-archive-compaction";
const ARCHIVE_ID: &str = "2026-07-13-example-change";
const SECOND_ARCHIVE_ID: &str = "2026-07-13-second-change";
const PROPOSAL_CONTENT: &[u8] = b"## Why\n\nPreserve exact bytes.\n";
const TASKS_CONTENT: &[u8] = b"## Tasks\n\n- [x] Complete\n";
static TEST_SEQUENCE: AtomicUsize = AtomicUsize::new(0);

struct TestRoot(PathBuf);

impl TestRoot {
    fn create(label: &str) -> Self {
        let sequence = TEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
        let root = std::env::temp_dir().join(format!(
            "{TEST_DIRECTORY_PREFIX}-{label}-{}-{sequence}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join(".cairn/archive").join(ARCHIVE_ID))
            .expect("create archive fixture");
        fs::write(
            root.join(".cairn/archive")
                .join(ARCHIVE_ID)
                .join("proposal.md"),
            PROPOSAL_CONTENT,
        )
        .expect("write proposal fixture");
        fs::write(
            root.join(".cairn/archive")
                .join(ARCHIVE_ID)
                .join("tasks.md"),
            TASKS_CONTENT,
        )
        .expect("write tasks fixture");
        Self(root)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TestRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn policy_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../cairn-policy/generated/cairn-policy.json")
}

fn policy() -> cairn_core::Policy {
    load_policy_file(policy_path()).expect("load generated test policy")
}

fn save_plan(root: &Path, plan: &cairn_core::CompactionPlan) -> PathBuf {
    let path = root.join("compact-plan.json");
    let mut bytes = serde_json::to_vec_pretty(plan).expect("serialize plan");
    bytes.push(b'\n');
    fs::write(&path, bytes).expect("write plan");
    path
}

#[test]
// r[verify cairn.sync_archive.compaction.plan]
// r[verify cairn.sync_archive.compaction.tests]
fn dry_run_command_writes_only_the_requested_plan_output() {
    let root = TestRoot::create("dry-run");
    let out = root.path().join("compact-plan.json");

    run_compact_archive(vec![
        "--root".to_string(),
        root.path().to_string_lossy().to_string(),
        "--out".to_string(),
        out.to_string_lossy().to_string(),
        "--policy".to_string(),
        policy_path().to_string_lossy().to_string(),
    ])
    .expect("run dry compaction plan");

    assert!(out.is_file());
    assert!(root.path().join(".cairn/archive").join(ARCHIVE_ID).is_dir());
    assert!(!root.path().join(".cairn/compaction").exists());
}

#[test]
// r[verify cairn.sync_archive.compaction.lossless_pack]
// r[verify cairn.sync_archive.compaction.lossless_pack.write_first]
// r[verify cairn.sync_archive.compaction.lossless_pack.unpack]
// r[verify cairn.sync_archive.compaction.verify]
// r[verify cairn.sync_archive.compaction.verify.valid]
// r[verify cairn.sync_archive.compaction.tests]
// r[verify cairn.sync_archive.compaction.tests.positive]
// r[verify cairn.sync_archive.compaction.final_validation]
// r[verify cairn.sync_archive.compaction.final_validation.evidence]
fn lossless_pack_verifies_and_unpack_restores_exact_bytes() {
    let root = TestRoot::create("positive");
    let policy = policy();
    let (plan, _) = current_plan(root.path(), &policy, LOSSLESS_PACK_PROFILE).expect("build plan");
    assert!(plan.blockers.is_empty(), "{:?}", plan.blockers);
    let plan_path = save_plan(root.path(), &plan);

    execute_saved_plan(root.path(), &policy, &plan_path).expect("execute compaction");
    assert!(!root.path().join(".cairn/archive").join(ARCHIVE_ID).exists());
    let verification = verify_repository(root.path(), &policy).expect("verify compacted state");
    assert!(verification.valid, "{:?}", verification.issues);

    let output = unpack_archive(root.path(), &policy, ARCHIVE_ID, true).expect("unpack archive");
    assert_eq!(
        output.get("mutated").and_then(serde_json::Value::as_bool),
        Some(true)
    );
    assert_eq!(
        fs::read(
            root.path()
                .join(".cairn/archive")
                .join(ARCHIVE_ID)
                .join("proposal.md")
        )
        .expect("read restored proposal"),
        PROPOSAL_CONTENT
    );
    assert_eq!(
        fs::read(
            root.path()
                .join(".cairn/archive")
                .join(ARCHIVE_ID)
                .join("tasks.md")
        )
        .expect("read restored tasks"),
        TASKS_CONTENT
    );
}

#[test]
// r[verify cairn.sync_archive.compaction.plan]
// r[verify cairn.sync_archive.compaction.plan.stale]
// r[verify cairn.sync_archive.compaction.lossless_pack.blocked]
// r[verify cairn.sync_archive.compaction.tests]
// r[verify cairn.sync_archive.compaction.tests.negative]
fn stale_plan_is_rejected_before_mutation() {
    let root = TestRoot::create("stale");
    let policy = policy();
    let (plan, _) = current_plan(root.path(), &policy, LOSSLESS_PACK_PROFILE).expect("build plan");
    let plan_path = save_plan(root.path(), &plan);
    fs::write(
        root.path()
            .join(".cairn/archive")
            .join(ARCHIVE_ID)
            .join("tasks.md"),
        b"changed after planning\n",
    )
    .expect("change archive after planning");

    let result = execute_saved_plan(root.path(), &policy, &plan_path);

    assert!(result.is_err());
    assert!(root.path().join(".cairn/archive").join(ARCHIVE_ID).is_dir());
    assert!(!root.path().join(".cairn/compaction").exists());
}

#[test]
// r[verify cairn.sync_archive.compaction.verify]
// r[verify cairn.sync_archive.compaction.verify.unsafe_pointer]
// r[verify cairn.sync_archive.compaction.tests]
// r[verify cairn.sync_archive.compaction.tests.negative]
fn unsafe_pointer_and_interrupted_state_fail_verification() {
    let root = TestRoot::create("negative");
    let second_archive = root.path().join(".cairn/archive").join(SECOND_ARCHIVE_ID);
    fs::create_dir_all(&second_archive).expect("create second archive");
    fs::write(second_archive.join("proposal.md"), PROPOSAL_CONTENT).expect("write second archive");
    let policy = policy();
    let (plan, _) = current_plan(root.path(), &policy, LOSSLESS_PACK_PROFILE).expect("build plan");
    let plan_path = save_plan(root.path(), &plan);
    execute_saved_plan(root.path(), &policy, &plan_path).expect("execute compaction");
    let pointer_path = root
        .path()
        .join(".cairn/compaction/v1/pointers")
        .join(format!("{ARCHIVE_ID}.json"));
    let mut pointer: serde_json::Value =
        serde_json::from_slice(&fs::read(&pointer_path).expect("read pointer"))
            .expect("parse pointer");
    let unsafe_pointer_id = "unsafe-pointer";
    pointer["archive_id"] = serde_json::Value::String(unsafe_pointer_id.to_string());
    pointer["pack_manifest_path"] = serde_json::Value::String("../escape.json".to_string());
    fs::write(
        pointer_path.with_file_name(format!("{unsafe_pointer_id}.json")),
        serde_json::to_vec_pretty(&pointer).expect("serialize pointer"),
    )
    .expect("write unsafe pointer");
    fs::remove_file(
        root.path()
            .join(".cairn/compaction/v1/pointers")
            .join(format!("{SECOND_ARCHIVE_ID}.json")),
    )
    .expect("remove second retained pointer");
    fs::create_dir_all(
        root.path()
            .join(".cairn/compaction/v1/prepared/interrupted"),
    )
    .expect("create interrupted state");
    let orphan_pack = root
        .path()
        .join(".cairn/compaction/v1/packs")
        .join("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff");
    fs::create_dir_all(&orphan_pack).expect("create orphan pack directory");
    fs::write(orphan_pack.join("manifest.json"), b"{}\n").expect("write orphan pack");

    let verification = verify_repository(root.path(), &policy).expect("verification receipt");

    assert!(!verification.valid);
    let codes = verification
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"archive_compaction.pointer_path_unsafe"));
    assert!(codes.contains(&"archive_compaction.pack_pointer_coverage_mismatch"));
    assert!(codes.contains(&"archive_compaction.interrupted_execution"));
    assert!(codes.contains(&"archive_compaction.orphan_pack"));
}

#[test]
// r[verify cairn.bounded_tree_adoption.archive_scan]
// r[verify cairn.bounded_tree_adoption.parity]
fn shared_bounded_observation_preserves_accepted_archive_inputs_and_order() {
    let root = TestRoot::create("parity-valid");
    let snapshot = scan_repository(root.path(), "").expect("scan repository");
    let archive = snapshot
        .archives
        .iter()
        .find(|archive| archive.archive_id == ARCHIVE_ID)
        .expect("discover scanned archive");

    let paths = archive
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(paths, vec!["proposal.md", "tasks.md"]);
    assert_eq!(archive.files[0].bytes, PROPOSAL_CONTENT);
    assert_eq!(archive.files[1].bytes, TASKS_CONTENT);
    assert!(snapshot.issues.is_empty());
}

#[test]
// r[verify cairn.bounded_tree_adoption.archive_policy]
// r[verify cairn.bounded_tree_adoption.parity]
fn shared_bounded_observation_retains_link_and_special_file_policy() {
    let root = TestRoot::create("parity-negative");
    let archive = root.path().join(".cairn/archive").join(ARCHIVE_ID);
    std::os::unix::fs::symlink("proposal.md", archive.join("alias.md")).expect("create symlink");
    let fifo = archive.join("pipe");
    let status = std::process::Command::new("mkfifo")
        .arg(&fifo)
        .status()
        .expect("create fifo");
    assert!(status.success());

    let snapshot = scan_repository(root.path(), "").expect("scan repository");
    let codes = snapshot
        .issues
        .iter()
        .map(|issue| issue.code.as_str())
        .collect::<Vec<_>>();
    assert!(codes.contains(&"archive_compaction.archive_symlink_rejected"));
    assert!(codes.contains(&"archive_compaction.special_file_rejected"));

    let archive = snapshot
        .archives
        .iter()
        .find(|archive| archive.archive_id == ARCHIVE_ID)
        .expect("discover scanned archive");
    let paths = archive
        .files
        .iter()
        .map(|file| file.path.as_str())
        .collect::<Vec<_>>();
    assert_eq!(paths, vec!["proposal.md", "tasks.md"]);
}

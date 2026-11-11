use std::{env, fs};

use super::super::check_index::{
    parse_release_evidence_index_payload, release_evidence_check_index_receipt, CheckIndexMode,
    ReleaseEvidenceCheckIndexOptions,
};
use super::super::common::git_head_state;
use super::helpers::{git_stdout, run_git};

#[test]
fn check_index_parses_generated_payload_fields() {
    // r[verify release_evidence.current_head_staleness_guard.parser]
    let markdown = "# Cairn current HEAD release evidence\n\n## Payload\n\n- Payload commit: `abc1234`\n- Branch: `main`\n- Worktree clean: `true`\n- Upstream: `origin/main`\n- Ahead/behind: `0\\t0`\n";
    let mut issues = Vec::new();
    let payload = parse_release_evidence_index_payload(markdown, &mut issues)
        .expect("generated payload parses");

    assert!(issues.is_empty(), "unexpected issues: {issues:?}");
    assert_eq!(payload.payload_commit, "abc1234");
    assert_eq!(payload.branch, "main");
    assert!(payload.clean);
}

#[test]
fn check_index_rejects_stale_current_head_but_allows_audited_snapshot() {
    // r[verify release_evidence.current_head_staleness_guard.git_checks]
    // r[verify release_evidence.current_head_staleness_guard.tests]
    // r[verify release_evidence.current_head_staleness_guard.fail_closed]
    let root = env::temp_dir().join(format!(
        "cairn-release-evidence-check-index-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("temp repo directory");
    run_git(&root, &["init"]);
    run_git(&root, &["config", "user.email", "cairn@example.invalid"]);
    run_git(&root, &["config", "user.name", "Cairn Test"]);
    fs::write(root.join("tracked.txt"), "first\n").expect("write tracked file");
    run_git(&root, &["add", "tracked.txt"]);
    run_git(&root, &["commit", "-m", "first"]);
    let old_head = git_stdout(&root, &["rev-parse", "--short", "HEAD"]);
    fs::write(root.join("tracked.txt"), "second\n").expect("write tracked file");
    run_git(&root, &["commit", "-am", "second"]);
    let index = root.join("docs/release-evidence-current-head.md");
    fs::create_dir_all(index.parent().expect("index parent")).expect("index parent dir");
    fs::write(
        &index,
        format!(
            "# Cairn current HEAD release evidence\n\n## Payload\n\n- Payload commit: `{old_head}`\n- Branch: `main`\n- Worktree clean: `true`\n- Upstream: `origin/main`\n- Ahead/behind: `0\\t0`\n"
        ),
    )
    .expect("write stale index");

    let stale_receipt = release_evidence_check_index_receipt(&ReleaseEvidenceCheckIndexOptions {
        root: root.clone(),
        index: index.clone(),
        mode: Some(CheckIndexMode::CurrentHead),
    });
    assert_eq!(stale_receipt["valid"].as_bool(), Some(false));
    assert!(stale_receipt["issues"]
        .as_array()
        .expect("issues")
        .iter()
        .any(|issue| {
            issue["code"].as_str() == Some("release_evidence.check_index.stale_current_head")
        }));

    let snapshot_receipt =
        release_evidence_check_index_receipt(&ReleaseEvidenceCheckIndexOptions {
            root: root.clone(),
            index,
            mode: Some(CheckIndexMode::AuditedPayload),
        });
    assert_eq!(snapshot_receipt["valid"].as_bool(), Some(true));
    assert_eq!(
        snapshot_receipt["payload_ancestor_of_head"].as_bool(),
        Some(true)
    );
    fs::remove_dir_all(&root).expect("remove temp repo");
}

#[test]
fn git_state_reports_dirty_worktree_as_failure_issue() {
    // r[verify release_evidence.automated_current_head_index.fail_closed]
    // r[verify release_evidence.automated_current_head_index.fail_closed.dirty_or_mismatched]
    let root = env::temp_dir().join(format!(
        "cairn-release-evidence-dirty-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("temp repo directory");
    run_git(&root, &["init"]);
    run_git(&root, &["config", "user.email", "cairn@example.invalid"]);
    run_git(&root, &["config", "user.name", "Cairn Test"]);
    fs::write(root.join("tracked.txt"), "clean\n").expect("write tracked file");
    run_git(&root, &["add", "tracked.txt"]);
    run_git(&root, &["commit", "-m", "initial"]);
    fs::write(root.join("tracked.txt"), "dirty\n").expect("dirty tracked file");

    let mut issues = Vec::new();
    let state = git_head_state(&root, &mut issues);

    assert_eq!(state["clean"].as_bool(), Some(false));
    assert!(issues
        .iter()
        .any(|issue| { issue["code"].as_str() == Some("release_evidence.worktree_dirty") }));
    fs::remove_dir_all(&root).expect("remove temp repo");
}

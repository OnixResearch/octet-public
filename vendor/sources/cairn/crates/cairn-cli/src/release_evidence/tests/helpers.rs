use std::{fs, process::Command};

use serde_json::json;

use cairn_core::blake3_hex;

pub(super) fn write_publish_tree(
    path: &std::path::Path,
    channel: &str,
    release: &str,
    retention: &str,
) {
    fs::create_dir_all(path).expect("publish tree");
    let manifest = json!({
        "schema": "cairn.release-evidence.publish-manifest.v1",
        "labels": {"channel": channel, "release": release, "retention": retention},
        "materialized": [],
    });
    let manifest_bytes = serde_json::to_vec_pretty(&manifest).expect("manifest bytes");
    fs::write(path.join("manifest.json"), &manifest_bytes).expect("manifest");
    let manifest_hash = blake3_hex(&manifest_bytes);
    fs::write(
        path.join("receipt.json"),
        serde_json::to_vec_pretty(&json!({
            "command": "release-evidence publish",
            "valid": true,
            "verdict": "pass",
            "manifest_hash": manifest_hash,
        }))
        .expect("receipt bytes"),
    )
    .expect("receipt");
}

pub(super) fn git_stdout(root: &std::path::Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command runs");
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:{}\nstderr:{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

pub(super) fn run_git(root: &std::path::Path, args: &[&str]) {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .expect("git command runs");
    assert!(
        output.status.success(),
        "git {:?} failed\nstdout:{}\nstderr:{}",
        args,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

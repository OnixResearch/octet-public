use std::{fs, path::PathBuf};

fn workspace_file(path: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(path)
}

fn read_workspace_file(path: &str) -> String {
    fs::read_to_string(workspace_file(path)).unwrap()
}

#[test]
fn post_tag_remote_install_smoke_docs_are_bounded() {
    // r[verify release_evidence.post_tag_remote_install_smoke.build]
    // r[verify release_evidence.post_tag_remote_install_smoke.lifecycle]
    // r[verify release_evidence.post_tag_remote_install_smoke.docs]
    // r[verify release_evidence.post_tag_remote_install_smoke.validation]
    let docs = read_workspace_file("docs/post-tag-remote-install-smoke.md");
    assert!(docs.contains("github:OnixResearch/cairn/v0.1.0#cairn"));
    assert!(docs.contains("init --root"));
    assert!(docs.contains("validate` reported `valid=true`"));
    assert!(docs.contains("does not move or create release tags"));
}

#[test]
fn v010_release_evidence_publication_docs_are_bounded() {
    // r[verify release_evidence.v010_publication.index]
    // r[verify release_evidence.v010_publication.publish]
    // r[verify release_evidence.v010_publication.docs]
    // r[verify release_evidence.v010_publication.validation]
    let docs = read_workspace_file("docs/release-evidence/v0.1.0.md");
    assert!(docs.contains("Release tag: `v0.1.0`"));
    assert!(docs.contains("Payload commit: `2e6ca1f`"));
    assert!(docs.contains("publish_materialization"));
    assert!(docs.contains("Bulky generated receipts remain under `target/`"));
}

#[test]
fn git_remote_relocation_docs_are_secret_free() {
    // r[verify developer_experience.git_remote_relocation_cleanup.configure]
    // r[verify developer_experience.git_remote_relocation_cleanup.verify]
    // r[verify developer_experience.git_remote_relocation_cleanup.docs]
    // r[verify developer_experience.git_remote_relocation_cleanup.validation]
    let docs = read_workspace_file("docs/git-remote.md");
    assert!(docs.contains("git@github.com:OnixResearch/cairn.git"));
    assert!(docs.contains("avoid GitHub relocation warnings"));
    assert!(docs.contains("does not validate credentials"));
    assert!(!docs.contains("SSH_AUTH_SOCK"));
}

#[test]
fn maintainability_hotspot_inventory_is_advisory() {
    // r[verify developer_experience.cli_maintainability_hotspot_inventory.scan]
    // r[verify developer_experience.cli_maintainability_hotspot_inventory.docs]
    // r[verify developer_experience.cli_maintainability_hotspot_inventory.boundary]
    // r[verify developer_experience.cli_maintainability_hotspot_inventory.validation]
    let docs = read_workspace_file("docs/maintainability-hotspots.md");
    assert!(docs.contains("Largest Rust files sampled"));
    assert!(docs.contains("semantic_review/metrics_regression.rs"));
    assert!(docs.contains("does not move modules"));
    assert!(docs.contains("focused behavior-preserving checks"));
}

#[test]
fn packaged_policy_export_runtime_is_documented_and_checked() {
    // r[verify cairn.supersede.policy_export_packaged_runtime]
    // r[verify cairn.supersede.policy_export_packaged_runtime.wrapper]
    // r[verify cairn.supersede.policy_export_packaged_runtime.check]
    // r[verify cairn.supersede.policy_export_packaged_runtime.docs]
    // r[verify cairn.supersede.policy_export_packaged_runtime.validation]
    let flake = read_workspace_file("flake.nix");
    assert!(flake.contains("pkgs.makeWrapper"));
    assert!(flake.contains("wrapProgram $out/bin/cairn"));
    assert!(flake.contains("wrapProgram $out/bin/ironspec"));
    assert!(flake.contains("pkgs.nickel"));
    assert!(flake.contains("cairn-packaged-policy-export-runtime"));
    assert!(flake.contains("env -i"));

    let readme = read_workspace_file("README.md");
    assert!(readme.contains("nix run path:$PWD#cairn -- policy export --check"));

    let workflow = read_workspace_file("docs/hermes/cairn-workflow-skill.md");
    assert!(workflow.contains("nix run path:$PWD#cairn -- policy export --check"));
    assert!(!workflow.contains("nix develop -c cairn policy export --check"));
}

#[test]
fn aarch64_release_support_boundary_is_a_non_claim() {
    // r[verify release_evidence.aarch64_support_boundary.docs]
    // r[verify release_evidence.aarch64_support_boundary.non_claims]
    // r[verify release_evidence.aarch64_support_boundary.validation]
    // r[verify release_evidence.aarch64_support_boundary.priority]
    let docs = read_workspace_file("docs/aarch64-release-support.md");
    assert!(docs.contains("must not be cited as a completed aarch64 release validation"));
    assert!(docs.contains("aarch64 release support remains unclaimed"));
    assert!(docs.contains("nix flake check --all-systems -L"));
    assert!(docs.contains("infrastructure noise"));
}

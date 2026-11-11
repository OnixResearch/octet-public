use std::{env, error::Error, fs, path::PathBuf};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

#[test]
// r[verify cairn.determinism.preserves_workflow_test_split.docs_target]
// r[verify cairn.determinism.preserves_workflow_test_split.tests]
// r[verify cairn.determinism.preserves_workflow_test_split.validation]
// r[verify cairn.determinism.preserves_operator_docs]
// r[verify cairn.determinism.preserves_operator_docs.workflow]
// r[verify cairn.determinism.preserves_operator_docs.discoverability]
// r[verify cairn.determinism.preserves_operator_docs.validation]
fn preserves_operator_docs_cover_receipt_workflow() -> Result<(), Box<dyn Error>> {
    let docs = fs::read_to_string(repo_root().join("docs/preserves-receipts.md"))?;
    let readme = fs::read_to_string(repo_root().join("README.md"))?;
    let agent_docs = fs::read_to_string(repo_root().join("docs/agent-run-receipts.md"))?;

    for required in [
        "JSON `receipt_hash`",
        "agent-receipt preserves-hash",
        "preserves-corpus update --path fixtures/preserves-canonicalization/corpus.json",
        "--execute",
        "preserves-corpus-fresh",
        "preserves-corpus-stale-detects-drift",
        "Preserves is not a replacement for JSON public/stored receipt schemas",
    ] {
        assert!(
            docs.contains(required),
            "Preserves operator docs missing required text: {required}"
        );
    }
    assert!(readme.contains("docs/preserves-receipts.md"));
    assert!(agent_docs.contains("preserves-receipts.md"));
    Ok(())
}

#[test]
// r[verify cairn.determinism.preserves_workflow_test_split.docs_target]
// r[verify cairn.determinism.preserves_workflow_test_split.tests]
// r[verify cairn.determinism.preserves_dependency_bump_runbook]
// r[verify cairn.determinism.preserves_dependency_bump_runbook.workflow]
// r[verify cairn.determinism.preserves_dependency_bump_runbook.intentional_drift]
// r[verify cairn.determinism.preserves_dependency_bump_runbook.discoverability]
// r[verify cairn.determinism.preserves_dependency_bump_runbook.validation]
fn preserves_dependency_bump_runbook_covers_review_paths() -> Result<(), Box<dyn Error>> {
    let runbook = fs::read_to_string(repo_root().join("docs/preserves-dependency-bumps.md"))?;
    let workflow_docs = fs::read_to_string(repo_root().join("docs/preserves-receipts.md"))?;
    let readme = fs::read_to_string(repo_root().join("README.md"))?;

    for required in [
        "preserves-corpus update --path fixtures/preserves-canonicalization/corpus.json",
        "preserves-corpus-fresh",
        "preserves-corpus-stale-detects-drift",
        "--execute",
        "JSON `receipt_hash`",
        "must not change public/stored JSON receipt schemas",
        "CI must never run `--execute` automatically",
        "cargo test -p cairn-core preserves",
        "cargo test -p cairn-cli --test preserves_corpus",
        "cargo test --workspace",
    ] {
        assert!(
            runbook.contains(required),
            "Preserves dependency-bump runbook missing required text: {required}"
        );
    }
    assert!(workflow_docs.contains("preserves-dependency-bumps.md"));
    assert!(readme.contains("docs/preserves-dependency-bumps.md"));
    Ok(())
}

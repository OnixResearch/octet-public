use serde_json::json;

use super::super::index::render_release_evidence_index;

#[test]
fn rendered_index_binds_payload_head_without_final_commit_claim() {
    // r[verify release_evidence.cli_source_split.behavior_preserved.focused_checks]
    // r[verify release_evidence.cli_source_split.repository_checks]
    // r[verify release_evidence.cli_source_split.archive_ready]
    // r[verify release_evidence.automated_current_head_index.generated_index]
    // r[verify release_evidence.automated_current_head_index.generated_index.payload_head]
    // r[verify release_evidence.automated_current_head_index.generated_index.required_hashes]
    let markdown = render_release_evidence_index(
        &json!({
            "payload_commit": "abc1234",
            "branch": "main",
            "upstream": "origin/main",
            "ahead_behind": "0\t0",
            "clean": true,
        }),
        &[json!({
            "role": "release_readiness",
            "verdict": "pass",
            "receipt_hash": "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        })],
        &[json!({
            "role": "release_readiness",
            "path": "release-readiness.json",
            "hash": "abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789",
        })],
    );

    assert!(markdown.contains("Payload commit: `abc1234`"));
    assert!(markdown.contains("release-readiness --root <root>"));
    assert!(markdown.contains("No tag creation or tag movement"));
    assert!(markdown.contains("not a later documentation commit"));
}

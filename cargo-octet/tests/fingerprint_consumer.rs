use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

fn cargo_octet() -> &'static str {
    env!("CARGO_BIN_EXE_cargo-octet")
}

fn write_consumer_fixture(root: &Path) {
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-consumer-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r#"// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=evidence/check.txt
pub fn checked_add_one(input: u32) -> u32 {
    input + 1
}

// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=evidence/macro.txt; macro=allow
pub fn rendered(input: u32) -> String {
    format!("value={input}")
}
"#,
    )
    .expect("write lib.rs");
}

fn write_selective_fixture(root: &Path) {
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-selective-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r#"// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=evidence/check.txt
#[inline]
pub const fn checked_age(current_ms: u64, started_ms: u64) -> u64 {
    current_ms.saturating_sub(started_ms)
}

#[cfg(test)]
mod tests {
    #[test]
    fn macro_heavy_unmarked_test_is_not_part_of_selective_check() {
        assert_eq!(format!("{}", 1), "1");
    }
}
"#,
    )
    .expect("write lib.rs");
}

fn write_macro_generated_fixture(root: &Path) {
    let src_dir = root.join("src");
    fs::create_dir_all(&src_dir).expect("create src dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-macro-generated-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r"macro_rules! define_parametric {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input + 1
        }
    };
}

macro_rules! define_literal {
    () => {
        pub fn literal_generated() -> u32 {
            7
        }
    };
}

define_parametric!(generated_checked);
define_literal!();

// tigerstyle-fingerprint: b3:0; review=explicit-wrapper; evidence=evidence/wrapper; macro=allow
pub fn explicit_wrapper(input: u32) -> u32 {
    generated_checked(input)
}
",
    )
    .expect("write lib.rs");
}

fn write_directory_fixture(root: &Path) {
    let src_dir = root.join("src");
    let nested_dir = src_dir.join("nested");
    let tests_dir = root.join("tests");
    fs::create_dir_all(&nested_dir).expect("create nested src dir");
    fs::create_dir_all(&tests_dir).expect("create tests dir");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"fingerprint-directory-smoke\"\nversion = \"0.1.0\"\nedition = \"2024\"\n",
    )
    .expect("write Cargo.toml");
    fs::write(
        src_dir.join("lib.rs"),
        r"pub mod nested;

macro_rules! define_generated {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input + 1
        }
    };
}

define_generated!(generated_from_root);

// tigerstyle-fingerprint: b3:0; review=directory-root-plain; evidence=evidence/root-plain
pub fn root_plain(input: u32) -> u32 {
    input.saturating_add(1)
}

// tigerstyle-fingerprint: b3:0; review=directory-root-macro; evidence=evidence/root-macro; macro=allow
#[inline]
pub fn root_with_macro(input: u32) -> u32 {
    assert!(input < 10);
    input + 2
}
",
    )
    .expect("write lib.rs");
    fs::write(
        nested_dir.join("mod.rs"),
        r#"trait Reader {
    fn read_value(&self) -> u32;
}

pub struct FixedReader;

impl Reader for FixedReader {
    // tigerstyle-fingerprint: b3:0; review=directory-trait-impl; evidence=evidence/trait-impl
    fn read_value(&self) -> u32 {
        7
    }
}

// tigerstyle-fingerprint: b3:0; review=directory-attributed; evidence=evidence/attributed
#[allow(
    dead_code,
    reason = "directory-wide fingerprint smoke"
)]
pub fn attributed_nested(input: u32) -> u32 {
    input * 2
}

#[cfg(test)]
mod tests {
    // tigerstyle-fingerprint: b3:0; review=directory-cfg-test; evidence=evidence/cfg-test; macro=allow
    #[test]
    fn nested_test_helper() {
        assert_eq!(2 + 2, 4);
    }
}
"#,
    )
    .expect("write nested mod.rs");
    fs::write(
        tests_dir.join("integration.rs"),
        r"macro_rules! literal_test_fn {
    () => {
        fn generated_test_literal() -> u32 {
            99
        }
    };
}

literal_test_fn!();

// tigerstyle-fingerprint: b3:0; review=directory-integration-helper; evidence=evidence/integration-helper
fn explicit_test_helper(input: u32) -> u32 {
    input + 3
}
",
    )
    .expect("write integration fixture");
}

fn repo_root() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace root").to_path_buf()
}

#[test]
fn fingerprint_lifecycle_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-lifecycle");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("lifecycle-transcript.md")).expect("read lifecycle transcript");

    let final_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run docs final check");
    assert!(
        final_check.status.success(),
        "docs final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    let final_stdout = String::from_utf8_lossy(&final_check.stdout);
    assert!(final_stdout.contains("bounded_retry_delay_ms match"));
    assert!(final_stdout.contains("summary total=1 match=1 mismatch=0 missing=0 written=0"));
    assert!(transcript.contains(final_stdout.as_ref()), "transcript missing final check output:\n{final_stdout}");

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r"// octet-fingerprint: b3:0; review=safety-critical; evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md; macro=allow
#[rustfmt::skip]
pub fn bounded_retry_delay_ms(attempt: u32) -> u32 {
    assert!(attempt <= 5);
    50_u32.saturating_mul(attempt.saturating_add(1))
}
",
    )
    .expect("write docs lifecycle initial fixture");

    let initial_update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run initial docs update");
    assert!(
        initial_update.status.success(),
        "initial update stderr: {}",
        String::from_utf8_lossy(&initial_update.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "initial update");

    let clean_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run initial docs check");
    assert!(clean_check.status.success(), "clean check stderr: {}", String::from_utf8_lossy(&clean_check.stderr));
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "clean check");

    let source = fs::read_to_string(&fixture).expect("read lifecycle fixture");
    fs::write(
        &fixture,
        source.replace(
            "50_u32.saturating_mul(attempt.saturating_add(1))",
            "75_u32.saturating_mul(attempt.saturating_add(1))",
        ),
    )
    .expect("mutate lifecycle fixture");

    let drift_diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run docs drift diff");
    assert!(!drift_diff.status.success(), "drift diff should fail");
    assert_transcript_contains_stdout(&transcript, &drift_diff.stdout, "drift diff");

    let repair_update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run docs repair update");
    assert!(
        repair_update.status.success(),
        "repair update stderr: {}",
        String::from_utf8_lossy(&repair_update.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &repair_update.stdout, "repair update");

    let repaired_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run docs repaired check");
    assert!(
        repaired_check.status.success(),
        "repaired check stderr: {}",
        String::from_utf8_lossy(&repaired_check.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &repaired_check.stdout, "repaired check");

    let repaired_source = fs::read_to_string(&fixture).expect("read repaired lifecycle fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final docs fixture");
    assert_eq!(repaired_source, checked_in_source);
}

fn assert_transcript_contains_stdout(transcript: &str, stdout: &[u8], label: &str) {
    let text = String::from_utf8_lossy(stdout);
    assert!(transcript.contains(text.as_ref()), "transcript missing {label} output:\n{text}");
}

// This name is selected by the existing releaseOperatorSmokeDocs Nix rail;
// exercise the transcript command, not the document's incidental wording.
#[test]
fn release_operator_smoke_transcript_and_help_cover_operator_surfaces() {
    let root = repo_root();
    let transcript_path = root.join("docs/examples/release-operator-smoke/transcript.md");

    let transcript_output = Command::new(cargo_octet())
        .args(["transcript", transcript_path.to_str().expect("transcript path utf8")])
        .output()
        .expect("run release smoke transcript validation");
    assert!(
        transcript_output.status.success(),
        "release smoke transcript stderr: {}",
        String::from_utf8_lossy(&transcript_output.stderr)
    );
    let transcript_stdout = String::from_utf8_lossy(&transcript_output.stdout);
    assert!(transcript_stdout.contains("command_blocks=7"));
    assert!(transcript_stdout.contains("expected_output_blocks=8"));
    assert!(transcript_stdout.contains("status=passed"));
}

#[test]
fn fingerprint_review_manifest_is_stable_and_sorted() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let manifest = temp.path().join("review-manifest.json");

    let output = Command::new(cargo_octet())
        .args([
            "fingerprint",
            "--check",
            "--review-manifest",
            manifest.to_str().expect("manifest path utf8"),
            "src/lib.rs",
        ])
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint review manifest");
    assert!(!output.status.success(), "zero hashes should require review/update");

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&manifest).expect("read manifest")).expect("parse manifest");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["tool_name"], "cargo-octet");
    assert_eq!(json["mode"], "check");
    assert!(json["manifest_hash"].as_str().expect("manifest hash").starts_with("b3:"));

    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 2);
    assert_eq!(records[0]["name"], "checked_add_one");
    assert_eq!(records[1]["name"], "rendered");
    for record in records {
        assert!(record["stable_id"].as_str().expect("stable id").starts_with("b3:"));
        assert_eq!(record["requires_review"], true);
    }

    let provenance_path = manifest.with_extension("provenance.jsonl");
    let provenance_line = fs::read_to_string(&provenance_path).expect("read fingerprint provenance sidecar");
    let provenance: serde_json::Value = serde_json::from_str(provenance_line.trim()).expect("parse provenance sidecar");
    assert_eq!(provenance["checker_id"], "octet-fingerprint");
    assert_eq!(provenance["source_id"], "octet.fingerprint.review-manifest");
    assert_eq!(provenance["evidence_tier"], "manual_review_required");
    assert_eq!(provenance["artifact_locator"], manifest.display().to_string());
    assert!(provenance["human_action"].as_str().expect("human action").contains("review fingerprint manifest"));
}

#[test]
fn fingerprint_human_summary_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-human-summary");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read human summary transcript");

    let final_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run human summary final check");
    assert!(
        final_check.status.success(),
        "human summary final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    let final_stdout = String::from_utf8_lossy(&final_check.stdout);
    assert!(final_stdout.contains("capped_batch_size match"));
    assert!(final_stdout.contains("describe_batch match"));
    assert!(final_stdout.contains("summary total=2 match=2 mismatch=0 missing=0 written=0"));
    assert!(
        transcript.contains(final_stdout.as_ref()),
        "transcript missing final human summary check output:\n{final_stdout}"
    );

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"// tigerstyle-fingerprint: b3:0; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md
pub fn capped_batch_size(requested: u32) -> u32 {
    requested.min(128)
}

// tigerstyle-fingerprint: b3:0; review=operator-human-summary; evidence=docs/examples/function-fingerprint-human-summary/transcript.md; macro=allow
pub fn describe_batch(size: u32) -> String {
    format!("batch={size}")
}
"#,
    )
    .expect("write human summary initial fixture");

    let initial_update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary initial update");
    assert!(
        initial_update.status.success(),
        "human summary initial update stderr: {}",
        String::from_utf8_lossy(&initial_update.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "human summary initial update");

    let clean_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary clean check");
    assert!(
        clean_check.status.success(),
        "human summary clean check stderr: {}",
        String::from_utf8_lossy(&clean_check.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "human summary clean check");

    let source = fs::read_to_string(&fixture).expect("read human summary fixture");
    fs::write(&fixture, source.replace("requested.min(128)", "requested.min(256)"))
        .expect("mutate human summary fixture");

    let drift_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary drift check");
    assert!(!drift_check.status.success(), "human summary drift check should fail");
    let drift_stdout = String::from_utf8_lossy(&drift_check.stdout);
    assert!(drift_stdout.contains("summary total=2 match=1 mismatch=1 missing=0 written=0"));
    assert_transcript_contains_stdout(&transcript, &drift_check.stdout, "human summary drift check");

    let repair_update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary repair update");
    assert!(
        repair_update.status.success(),
        "human summary repair update stderr: {}",
        String::from_utf8_lossy(&repair_update.stderr)
    );
    let repair_stdout = String::from_utf8_lossy(&repair_update.stdout);
    assert!(repair_stdout.contains("summary total=2 match=1 mismatch=0 missing=0 written=1"));
    assert_transcript_contains_stdout(&transcript, &repair_update.stdout, "human summary repair update");

    let repaired_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run human summary repaired check");
    assert!(
        repaired_check.status.success(),
        "human summary repaired check stderr: {}",
        String::from_utf8_lossy(&repaired_check.stderr)
    );
    assert_transcript_contains_stdout(&transcript, &repaired_check.stdout, "human summary repaired check");

    let repaired_source = fs::read_to_string(&fixture).expect("read repaired human summary fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final human summary fixture");
    assert_eq!(repaired_source, checked_in_source);
}

#[test]
fn fingerprint_warning_summary_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-warning-summary");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read warning summary transcript");

    let final_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run warning summary final check");
    assert!(!final_check.status.success(), "warning summary final check should fail on warning counts");
    let final_stdout = String::from_utf8_lossy(&final_check.stdout);
    assert!(final_stdout.contains("unreviewed_macro_path match"));
    assert!(final_stdout.contains("missing_gate_metadata match"));
    assert!(
        final_stdout
            .contains("summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=1 metadata_missing=1")
    );
    assert!(
        transcript.contains(final_stdout.as_ref()),
        "transcript missing final warning summary check output:\n{final_stdout}"
    );

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"// tigerstyle-fingerprint: b3:0; review=macro-warning-example; evidence=docs/examples/function-fingerprint-warning-summary/transcript.md
pub fn unreviewed_macro_path(size: u32) -> String {
    format!("size={size}")
}

// tigerstyle-fingerprint: b3:0
pub fn missing_gate_metadata(limit: u32) -> u32 {
    limit.min(64)
}
"#,
    )
    .expect("write warning summary initial fixture");

    let initial_update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run warning summary initial update");
    assert!(
        initial_update.status.success(),
        "warning summary initial update stderr: {}",
        String::from_utf8_lossy(&initial_update.stderr)
    );
    let initial_stdout = String::from_utf8_lossy(&initial_update.stdout);
    assert!(
        initial_stdout
            .contains("summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=1 metadata_missing=1")
    );
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "warning summary initial update");

    let clean_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run warning summary clean check");
    assert!(!clean_check.status.success(), "warning summary clean check should fail on warning counts");
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "warning summary clean check");

    let updated_source = fs::read_to_string(&fixture).expect("read updated warning summary fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final warning summary fixture");
    assert_eq!(updated_source, checked_in_source);
}

fn assert_fingerprint_json_record(output: &[u8], mode: &str, status: &str, stored_hash: &str) {
    let json: serde_json::Value = serde_json::from_slice(output).expect("fingerprint json output");
    assert_eq!(json["schema_version"], 1);
    assert_eq!(json["mode"], mode);
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    let record = &records[0];
    assert_eq!(record["path"], "reviewed.rs");
    assert_eq!(record["name"], "capped_retries");
    assert_eq!(record["status"], status);
    assert_eq!(record["stored_hash"], stored_hash);
    assert_eq!(record["review"], "json-script-example");
    assert_eq!(record["evidence"], "docs/examples/function-fingerprint-json-output/transcript.md");
    assert_eq!(record["macro_calls"], false);
    assert_eq!(record["macro_warning"], false);
}

#[test]
fn fingerprint_json_output_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-json-output");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read json output transcript");

    let final_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run json output final check");
    assert!(
        final_check.status.success(),
        "json output final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    assert_fingerprint_json_record(
        &final_check.stdout,
        "check",
        "match",
        "ea3ca43821de8c24558ea6c5d43beae6c6731f49e62ebc6b2675f96e4c2019f7",
    );
    assert_transcript_contains_stdout(&transcript, &final_check.stdout, "json output final check");

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r"// tigerstyle-fingerprint: b3:0; review=json-script-example; evidence=docs/examples/function-fingerprint-json-output/transcript.md; macro=allow
pub fn capped_retries(requested: u32) -> u32 {
    requested.min(3)
}
",
    )
    .expect("write json output initial fixture");

    let initial_update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output initial update");
    assert!(initial_update.status.success());
    assert_fingerprint_json_record(&initial_update.stdout, "update", "written", "0");
    assert_transcript_contains_stdout(&transcript, &initial_update.stdout, "json output initial update");

    let clean_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output clean check");
    assert!(clean_check.status.success());
    assert_fingerprint_json_record(
        &clean_check.stdout,
        "check",
        "match",
        "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
    );
    assert_transcript_contains_stdout(&transcript, &clean_check.stdout, "json output clean check");

    let source = fs::read_to_string(&fixture).expect("read json output fixture");
    fs::write(&fixture, source.replace("requested.min(3)", "requested.min(5)")).expect("mutate json output fixture");

    let drift_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output drift check");
    assert!(!drift_check.status.success(), "json output drift check should fail");
    assert_fingerprint_json_record(
        &drift_check.stdout,
        "check",
        "mismatch",
        "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
    );
    assert_transcript_contains_stdout(&transcript, &drift_check.stdout, "json output drift check");

    let accepted_update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output accepted update");
    assert!(accepted_update.status.success());
    assert_fingerprint_json_record(
        &accepted_update.stdout,
        "update",
        "written",
        "6b4c3f4e650e7cbc286f37ba67d94bc3bc4676b06ac870d7f157ebec09215a0c",
    );
    assert_transcript_contains_stdout(&transcript, &accepted_update.stdout, "json output accepted update");

    let accepted_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run json output accepted check");
    assert!(accepted_check.status.success());
    assert_fingerprint_json_record(
        &accepted_check.stdout,
        "check",
        "match",
        "ea3ca43821de8c24558ea6c5d43beae6c6731f49e62ebc6b2675f96e4c2019f7",
    );
    assert_transcript_contains_stdout(&transcript, &accepted_check.stdout, "json output accepted check");

    let accepted_source = fs::read_to_string(&fixture).expect("read accepted json output fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final json output fixture");
    assert_eq!(accepted_source, checked_in_source);
}

#[test]
fn fingerprint_source_scanner_docs_fixture_matches_transcript() {
    let root = repo_root();
    let docs_dir = root.join("docs/examples/function-fingerprint-source-scanner");
    let final_fixture = docs_dir.join("reviewed.rs");
    let transcript = fs::read_to_string(docs_dir.join("transcript.md")).expect("read source scanner transcript");

    let final_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(&docs_dir)
        .output()
        .expect("run source scanner final check");
    assert!(
        final_check.status.success(),
        "source scanner final check stderr: {}",
        String::from_utf8_lossy(&final_check.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&final_check.stdout).expect("source scanner json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "explicit_generated_boundary");
    assert_ne!(records[0]["name"], "generated_boundary");
    assert_eq!(records[0]["status"], "match");
    assert_eq!(records[0]["review"], "source-scanner-boundary");
    assert_transcript_contains_stdout(&transcript, &final_check.stdout, "source scanner final json check");

    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r"macro_rules! define_generated_boundary {
    ($name:ident) => {
        pub fn $name(input: u32) -> u32 {
            input.saturating_add(1)
        }
    };
}

define_generated_boundary!(generated_boundary);

// tigerstyle-fingerprint: b3:0; review=source-scanner-boundary; evidence=docs/examples/function-fingerprint-source-scanner/transcript.md
pub fn explicit_generated_boundary(input: u32) -> u32 {
    generated_boundary(input).min(64)
}
",
    )
    .expect("write source scanner initial fixture");

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run source scanner update");
    assert!(update.status.success());
    let update_stdout = String::from_utf8_lossy(&update.stdout);
    assert!(update_stdout.contains("summary total=1 match=0 mismatch=0 missing=0 written=1"));
    assert!(update_stdout.contains("explicit_generated_boundary written"));
    assert!(!update_stdout.contains("reviewed.rs generated_boundary "));
    assert_transcript_contains_stdout(&transcript, &update.stdout, "source scanner update");

    let json_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run source scanner json check");
    assert!(json_check.status.success());
    assert_transcript_contains_stdout(&transcript, &json_check.stdout, "source scanner json check");
    let json: serde_json::Value = serde_json::from_slice(&json_check.stdout).expect("source scanner json check");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "explicit_generated_boundary");

    let human_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "reviewed.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run source scanner human check");
    assert!(human_check.status.success());
    assert_transcript_contains_stdout(&transcript, &human_check.stdout, "source scanner human check");

    let updated_source = fs::read_to_string(&fixture).expect("read source scanner fixture");
    let checked_in_source = fs::read_to_string(final_fixture).expect("read final source scanner fixture");
    assert_eq!(updated_source, checked_in_source);
}

#[test]
fn fingerprint_records_and_manifest_use_relative_paths_and_reject_outside_paths() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let absolute_source = temp.path().join("src/lib.rs");
    let output = Command::new(cargo_octet())
        .args([
            "fingerprint",
            "--check",
            "--output-format=json",
            "--review-manifest",
            "manifest.json",
        ])
        .arg(&absolute_source)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint with absolute path");
    assert_eq!(output.status.code(), Some(1), "stderr={}", String::from_utf8_lossy(&output.stderr));
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fingerprint json");
    let manifest: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(temp.path().join("manifest.json")).expect("read manifest"))
            .expect("parse manifest");
    assert_eq!(manifest["input_paths"], serde_json::json!(["src/lib.rs"]));
    for records in [&report["records"], &manifest["records"]] {
        let paths =
            records.as_array().expect("records").iter().map(|record| record["path"].clone()).collect::<Vec<_>>();
        assert_eq!(paths, [serde_json::json!("src/lib.rs"), serde_json::json!("src/lib.rs")]);
    }

    let outside = tempfile::tempdir().expect("create outside tempdir");
    write_consumer_fixture(outside.path());
    let outside_source = outside.path().join("src/lib.rs");
    let before = fs::read_to_string(&outside_source).expect("read outside source");
    let escaping = format!("../{}/src/lib.rs", outside.path().file_name().expect("tempdir name").to_string_lossy());
    for named in [outside_source.to_str().expect("utf8 path"), escaping.as_str()] {
        let output = Command::new(cargo_octet())
            .args(["fingerprint", "--update", named])
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint with outside path");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{named}: stderr={stderr}");
        assert!(stderr.contains("outside the current directory"), "{named}: stderr={stderr}");
    }
    assert_eq!(fs::read_to_string(&outside_source).expect("reread outside source"), before);
}

#[test]
fn fingerprint_write_then_check_works_for_external_crate_file() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let write = Command::new(cargo_octet())
        .args(["fingerprint", "--write"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));

    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 2);
    assert!(records.iter().all(|record| record["status"] == "match"));
    assert!(records.iter().all(|record| record["review"] == "safety-critical"));
    assert!(records.iter().any(|record| record["macro_calls"] == true && record["macro_warning"] == false));
}

#[test]
fn fingerprint_update_then_check_only_tracks_marked_functions() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_selective_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));

    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "checked_age");
    assert_eq!(records[0]["status"], "match");
    assert_eq!(records[0]["macro_warning"], false);
}

#[test]
fn fingerprint_write_skips_functions_inside_macro_rules_definitions() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_macro_generated_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let write = Command::new(cargo_octet())
        .args(["fingerprint", "--write", "--output-format=json"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));

    let json: serde_json::Value = serde_json::from_slice(&write.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_eq!(records.len(), 1);
    assert_eq!(records[0]["name"], "explicit_wrapper");
    assert_eq!(records[0]["status"], "written");

    let updated = fs::read_to_string(&fixture).expect("read updated fixture");
    assert!(!updated.contains("review=TODO"));
    assert_eq!(updated.matches("octet-fingerprint").count(), 1);

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
}

#[test]
fn fingerprint_marker_detects_code_after_quoted_slashes() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    let reviewed = "pub fn fetch() -> u32 {\n    let _endpoint = \"https://example.invalid/api\"; 1\n}\n";
    fs::write(&fixture, reviewed).expect("write reviewed fixture");
    let write = Command::new(cargo_octet())
        .args(["fingerprint", "--write"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));
    let marked = fs::read_to_string(&fixture)
        .expect("read marked fixture")
        .replace("review=TODO; evidence=TODO", "review=R-1; evidence=E-1");
    let check = |text: String| {
        fs::write(&fixture, text).expect("write edited fixture");
        let output = Command::new(cargo_octet())
            .args(["fingerprint", "--check", "--output-format=json"])
            .arg(&fixture)
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint check");
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("check json");
        (output.status.success(), json["records"][0]["status"].clone())
    };

    assert_eq!(check(marked.replace("; 1", "; 1 // still one")), (true, serde_json::json!("match")));
    assert_eq!(check(marked.replace("; 1", "; std::process::abort()")), (false, serde_json::json!("mismatch")));
}

#[test]
fn fingerprint_rejects_sarif_before_rewriting_sources_or_manifest() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    let source = "// octet-fingerprint: b3:0; review=R-1; evidence=E-1\npub fn marked() -> u32 { 1 }\n\npub fn unmarked() -> u32 { 2 }\n";
    fs::write(&fixture, source).expect("write sarif fixture");
    for mode in ["--write", "--update"] {
        let output = Command::new(cargo_octet())
            .args([
                "fingerprint",
                mode,
                "--output-format=sarif",
                "--review-manifest",
                "manifest.json",
                "reviewed.rs",
            ])
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint with sarif");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{mode}: stderr={stderr}");
        assert!(stderr.contains("SARIF"), "{mode}: stderr={stderr}");
        assert_eq!(fs::read_to_string(&fixture).expect("reread fixture"), source, "{mode} rewrote the source");
        assert!(!temp.path().join("manifest.json").exists(), "{mode} wrote the review manifest");
    }
}

#[test]
fn fingerprint_rejects_untagged_v1_markers() {
    // v1 marker hashes were untagged BLAKE3 over token text that split literal
    // contents into tokens, so `"batch=1"` hashed like `" batch = 1 "`.
    let legacy = |tokens: &str| blake3::hash(tokens.as_bytes()).to_hex().to_string();
    let capped = legacy("pub fn capped_batch_size ( requested : u32 ) - > u32 { requested . min ( 128 ) }");
    let label = legacy("pub fn label ( ) - > & ' static str { \" batch = 1 \" }");
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    let source = r#"// octet-fingerprint: b3:CAPPED; review=R-1; evidence=E-1
pub fn capped_batch_size(requested: u32) -> u32 {
    requested.min(128)
}

// octet-fingerprint: b3:LABEL; review=R-1; evidence=E-1
pub fn label() -> &'static str {
    " batch = 1 "
}
"#;
    fs::write(&fixture, source.replace("CAPPED", &capped).replace("LABEL", &label)).expect("write legacy fixture");

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(!check.status.success(), "untagged v1 markers must fail closed");
    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check json");
    let statuses = json["records"]
        .as_array()
        .expect("records array")
        .iter()
        .map(|record| (record["name"].as_str().expect("name"), record["status"].as_str().expect("status")))
        .collect::<Vec<_>>();
    assert_eq!(statuses, [("capped_batch_size", "mismatch"), ("label", "mismatch")]);
}

#[test]
fn fingerprint_and_object_reject_missing_or_non_rust_named_paths() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    fs::write(temp.path().join("src/notes.txt"), "fn f() {}\n").expect("write non-rust file");
    for (args, named) in [
        (&["fingerprint", "--check", "srcc"][..], "srcc"),
        (&["fingerprint", "--check", "src/notes.txt"][..], "src/notes.txt"),
        (&["object", "check", "src/tpyo.rs"][..], "src/tpyo.rs"),
        (&["object", "emit", "src/notes.txt"][..], "src/notes.txt"),
    ] {
        let output = Command::new(cargo_octet())
            .args(args)
            .current_dir(temp.path())
            .output()
            .expect("run named-path command");
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert_eq!(output.status.code(), Some(2), "{args:?} stderr={stderr}");
        assert!(stderr.contains(named), "{args:?} stderr={stderr}");
    }
}

#[test]
fn fingerprint_markers_without_a_hash_are_missing_and_fail_check() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        "// octet-fingerprint: b3:; review=R-1; evidence=E-1\npub fn empty_hash() -> u32 { 1 }\n\n// octet-fingerprint: b3:TODO; review=R-1; evidence=E-1\npub fn placeholder_hash() -> u32 { 2 }\n",
    )
    .expect("write missing-hash fixture");
    let run = |mode: &str| {
        let output = Command::new(cargo_octet())
            .args(["fingerprint", mode, "--output-format=json", "reviewed.rs"])
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint");
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fingerprint json");
        let records = json["records"]
            .as_array()
            .expect("records array")
            .iter()
            .map(|record| {
                (
                    record["name"].as_str().expect("name").to_owned(),
                    record["status"].as_str().expect("status").to_owned(),
                    record["stored_hash"].is_null(),
                )
            })
            .collect::<Vec<_>>();
        (output.status.success(), records)
    };

    for mode in ["--check", "--diff"] {
        assert_eq!(
            run(mode),
            (false, vec![
                ("empty_hash".to_owned(), "missing".to_owned(), true),
                ("placeholder_hash".to_owned(), "missing".to_owned(), true),
            ]),
            "{mode}"
        );
    }
    let (updated, records) = run("--update");
    assert!(updated);
    assert!(records.iter().all(|(_, status, _)| status == "written"), "{records:?}");
    let reviewed = fs::read_to_string(&fixture).expect("read updated fixture");
    assert_eq!(reviewed.matches("// octet-fingerprint: b3:").count(), 2, "{reviewed}");
    assert!(!reviewed.contains("b3:TODO") && !reviewed.contains("b3:;"), "{reviewed}");
    let (checked, records) = run("--check");
    assert!(checked, "{records:?}");
    assert!(records.iter().all(|(_, status, _)| status == "match"), "{records:?}");
}

#[test]
fn fingerprint_check_fails_on_review_warnings_even_when_hashes_match() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        r#"// octet-fingerprint: b3:0; review=macro-path; evidence=evidence/macro
pub fn unreviewed_macro_path(size: u32) -> String {
    format!("size={size}")
}

// octet-fingerprint: b3:0
pub fn missing_gate_metadata(limit: u32) -> u32 {
    limit.min(64)
}
"#,
    )
    .expect("write warning fixture");
    let run = |mode: &str| {
        let output = Command::new(cargo_octet())
            .args(["fingerprint", mode, "reviewed.rs"])
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint");
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let summary = stdout.lines().find(|line| line.starts_with("summary ")).expect("summary line").to_owned();
        (output.status.success(), summary)
    };

    let (updated, summary) = run("--update");
    assert!(updated);
    assert_eq!(
        summary,
        "summary total=2 match=0 mismatch=0 missing=0 written=2 macro_warnings=1 metadata_missing=1"
    );
    assert_eq!(
        run("--check"),
        (
            false,
            "summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=1 metadata_missing=1".to_owned()
        )
    );

    let reviewed = fs::read_to_string(&fixture)
        .expect("read updated fixture")
        .replace("macro=review-required", "macro=allow")
        .replace("review=TODO; evidence=TODO", "review=gate; evidence=evidence/gate");
    fs::write(&fixture, reviewed).expect("record review metadata");
    assert_eq!(
        run("--check"),
        (
            true,
            "summary total=2 match=2 mismatch=0 missing=0 written=0 macro_warnings=0 metadata_missing=0".to_owned()
        )
    );
}

/// Runs cargo-octet in `root`, returning the exit code, stdout, and stderr.
fn run_octet(root: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(cargo_octet()).args(args).current_dir(root).output().expect("run cargo-octet");
    (
        output.status.code(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

fn parse_json(stdout: &str, context: &str) -> serde_json::Value {
    serde_json::from_str(stdout).unwrap_or_else(|error| panic!("{context}: {error}; stdout={stdout}"))
}

#[test]
fn fingerprint_markers_that_bind_to_no_function_fail_check_and_block_rewrites() {
    let marker = "// octet-fingerprint: b3:deadbeef; review=R-1; evidence=E-1";
    let attribute = "#[fingerprint(\"b3:deadbeef\", review = \"R-1\", evidence = \"E-1\")]";
    for (source, line) in [
        (format!("#[inline]\n{marker}\npub fn between_attribute_and_fn() -> u32 {{ 1 }}\n"), 2),
        (format!("{marker}\n/// Docs.\npub fn above_doc_comment() -> u32 {{ 1 }}\n"), 1),
        (
            format!("#[must_use]\n{attribute}\npub fn attribute_marker_between_attributes() -> u32 {{ 1 }}\n"),
            2,
        ),
        (format!("pub fn inside_body() -> u32 {{\n    {marker}\n    1\n}}\n"), 2),
    ] {
        let temp = tempfile::tempdir().expect("create tempdir");
        let fixture = temp.path().join("lib.rs");
        fs::write(&fixture, &source).expect("write misplaced marker");
        for mode in ["--check", "--diff"] {
            let (code, stdout, _) = run_octet(temp.path(), &["fingerprint", mode, "--output-format=json", "lib.rs"]);
            assert_eq!(code, Some(1), "{mode}: {source}");
            let report = parse_json(&stdout, mode);
            assert_eq!(report["unbound_markers"], serde_json::json!([{ "path": "lib.rs", "line": line }]), "{source}");
            assert_eq!(report["records"], serde_json::json!([]), "{mode}: {source}");
            let (code, stdout, _) = run_octet(temp.path(), &["fingerprint", mode, "lib.rs"]);
            assert_eq!(code, Some(1), "{mode}: {source}");
            assert!(stdout.contains(&format!("lib.rs:{line} unbound-marker")), "{mode}: {stdout}");
        }
        for mode in ["--write", "--update"] {
            let (code, _, stderr) = run_octet(temp.path(), &["fingerprint", mode, "lib.rs"]);
            assert_eq!(code, Some(2), "{mode}: {source}");
            assert!(stderr.contains(&format!("lib.rs:{line}")), "{mode}: {stderr}");
            assert_eq!(fs::read_to_string(&fixture).expect("reread fixture"), source, "{mode} rewrote the source");
        }
    }

    // Markers directly above the first outer attribute bind, and the hash
    // covers the attributes below them.
    for source in [
        format!("{marker}\n#[must_use]\npub fn comment_marker() -> u32 {{ 1 }}\n"),
        format!("/// Docs.\n{attribute}\n#[must_use]\npub fn attribute_marker() -> u32 {{ 1 }}\n"),
    ] {
        let temp = tempfile::tempdir().expect("create tempdir");
        let fixture = temp.path().join("lib.rs");
        fs::write(&fixture, &source).expect("write placed marker");
        let (code, stdout, _) = run_octet(temp.path(), &["fingerprint", "--update", "--output-format=json", "lib.rs"]);
        assert_eq!(code, Some(0), "{source}");
        let report = parse_json(&stdout, "--update");
        assert_eq!(report["records"][0]["status"], "written", "{source}");
        assert!(report.get("unbound_markers").is_none(), "{report}");
        let reviewed = fs::read_to_string(&fixture).expect("read updated fixture");
        assert_eq!(reviewed.matches("octet-fingerprint:").count(), 1, "{reviewed}");
        assert!(!reviewed.contains("#[fingerprint("), "{reviewed}");
        assert!(reviewed.contains("evidence=E-1\n#[must_use]\n"), "{reviewed}");

        let check = |text: &str| {
            fs::write(&fixture, text).expect("write reviewed fixture");
            let (code, stdout, _) =
                run_octet(temp.path(), &["fingerprint", "--check", "--output-format=json", "lib.rs"]);
            (code, parse_json(&stdout, "--check")["records"][0]["status"].clone())
        };
        assert_eq!(check(&reviewed), (Some(0), serde_json::json!("match")), "{reviewed}");
        let attribute_drift = reviewed.replace("#[must_use]", "#[must_use = \"use the value\"]");
        assert_eq!(check(&attribute_drift), (Some(1), serde_json::json!("mismatch")), "{attribute_drift}");
    }
}

#[test]
fn fingerprint_macro_calls_anywhere_in_the_item_need_macro_review() {
    for (item, macro_calls) in [
        ("#[doc = include_str!(\"f.md\")]\npub fn attribute_macro() -> u32 { 1 }\n", true),
        ("pub fn signature_macro(bytes: [u8; width!()]) -> u32 { bytes.len() as u32 }\n", true),
        ("pub fn body_macro() -> u32 { width!() }\n", true),
        ("#[doc = \"include_str!(f.md)\"]\npub fn quoted_macro_text() -> u32 { 1 }\n", false),
        ("#[inline]\npub fn no_macro(bytes: [u8; 4]) -> u32 { u32::from_le_bytes(bytes) }\n", false),
    ] {
        let temp = tempfile::tempdir().expect("create tempdir");
        let fixture = temp.path().join("lib.rs");
        fs::write(&fixture, format!("// octet-fingerprint: b3:; review=R-1; evidence=E-1\n{item}")).expect("write");
        let (code, _, stderr) = run_octet(temp.path(), &["fingerprint", "--update", "lib.rs"]);
        assert_eq!(code, Some(0), "{stderr}");
        let reviewed = fs::read_to_string(&fixture).expect("read updated fixture");
        assert_eq!(reviewed.contains("macro=review-required"), macro_calls, "{reviewed}");

        let (code, stdout, _) = run_octet(temp.path(), &["fingerprint", "--check", "--output-format=json", "lib.rs"]);
        let record = &parse_json(&stdout, "--check")["records"][0];
        assert_eq!(
            (record["macro_calls"].as_bool(), record["macro_warning"].as_bool()),
            (Some(macro_calls), Some(macro_calls))
        );
        assert_eq!(code, Some(i32::from(macro_calls)), "{item}");

        let (code, stdout, stderr) = run_octet(temp.path(), &["object", "emit", "--output-format=json", "lib.rs"]);
        assert_eq!(code, Some(0), "{stderr}");
        let object = &parse_json(&stdout, "object emit")["objects"][0];
        let macro_expansion = object["caveat_summary"]["macro_expansion"].as_str().expect("macro caveat summary");
        assert_eq!(macro_expansion != "none-detected", macro_calls, "{item}: {object}");
    }
}

#[test]
fn fingerprint_hash_reads_crlf_line_endings_in_literals_and_doc_comments_as_lf() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(
        &fixture,
        "// octet-fingerprint: b3:; review=R-1; evidence=E-1\n#[inline]\n/// Returns the banner.\n/** Block\n    docs. */\npub fn banner() -> &'static str {\n    \"first\nsecond\"\n}\n",
    )
    .expect("write LF fixture");
    let (code, _, stderr) = run_octet(temp.path(), &["fingerprint", "--update", "lib.rs"]);
    assert_eq!(code, Some(0), "{stderr}");
    let reviewed = fs::read_to_string(&fixture).expect("read reviewed LF fixture");
    let check = |text: &str| {
        fs::write(&fixture, text).expect("write fixture");
        let (code, stdout, _) = run_octet(temp.path(), &["fingerprint", "--check", "--output-format=json", "lib.rs"]);
        (code, parse_json(&stdout, "--check")["records"][0]["status"].clone())
    };
    let crlf = reviewed.replace('\n', "\r\n");
    assert_eq!(check(&reviewed), (Some(0), serde_json::json!("match")));
    assert_eq!(check(&crlf), (Some(0), serde_json::json!("match")), "CRLF checkout of the reviewed LF source");
    for drift in [crlf.replace("second", "third"), crlf.replace("the banner", "a banner")] {
        assert_eq!(check(&drift), (Some(1), serde_json::json!("mismatch")), "{drift:?}");
    }
}

#[test]
fn object_update_rejects_sarif_before_rewriting_sources() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    let source = "pub fn unmarked() -> u32 { 1 }\n";
    fs::write(&fixture, source).expect("write sarif fixture");
    let (code, _, stderr) = run_octet(temp.path(), &["object", "update", "--output-format=sarif", "lib.rs"]);
    assert_eq!(code, Some(2), "stderr={stderr}");
    assert!(stderr.contains("SARIF"), "stderr={stderr}");
    assert_eq!(fs::read_to_string(&fixture).expect("reread fixture"), source, "object update rewrote the source");
}

#[test]
fn object_markers_without_a_hash_are_missing_and_replaced_in_place() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(
        &fixture,
        "// octet-object: b3:; review=R-1; evidence=E-1\npub fn empty_hash() -> u32 { 1 }\n\n// octet-object: b3:TODO; review=R-2; evidence=E-2\npub fn placeholder_hash() -> u32 { 2 }\n",
    )
    .expect("write hashless object markers");
    let records = |mode: &str| {
        let (code, stdout, _) = run_octet(temp.path(), &["object", mode, "--output-format=json", "lib.rs"]);
        let report = parse_json(&stdout, mode);
        let records = report["records"]
            .as_array()
            .expect("records")
            .iter()
            .map(|record| {
                (
                    record["status"].as_str().expect("status").to_owned(),
                    record["stored_hash"].is_null(),
                    record["review"].as_str().map(str::to_owned),
                )
            })
            .collect::<Vec<_>>();
        (code, records)
    };
    let reviewed = |status: &str, hashless: bool| {
        vec![
            (status.to_owned(), hashless, Some("R-1".to_owned())),
            (status.to_owned(), hashless, Some("R-2".to_owned())),
        ]
    };

    assert_eq!(records("check"), (Some(1), reviewed("missing", true)));
    assert_eq!(records("update"), (Some(0), reviewed("written", true)));
    let updated = fs::read_to_string(&fixture).expect("read updated fixture");
    assert_eq!(updated.matches("// octet-object: b3:").count(), 2, "{updated}");
    assert!(!updated.contains("b3:;") && !updated.contains("b3:TODO"), "{updated}");
    assert_eq!(records("check"), (Some(0), reviewed("match", false)));
}

#[test]
fn fingerprint_json_records_report_stored_and_computed_hashes_across_drift() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("reviewed.rs");
    fs::write(
        &fixture,
        "// octet-fingerprint: b3:0; review=json-script; evidence=evidence/json\npub fn capped_retries(requested: u32) -> u32 {\n    requested.min(3)\n}\n",
    )
    .expect("write json fixture");
    let run = |mode: &str| {
        let output = Command::new(cargo_octet())
            .args(["fingerprint", mode, "--output-format=json", "reviewed.rs"])
            .current_dir(temp.path())
            .output()
            .expect("run fingerprint");
        let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("fingerprint json");
        assert_eq!(json["schema_version"], 1);
        assert_eq!(json["mode"], mode.trim_start_matches('-'));
        assert_eq!(json["records"].as_array().map(Vec::len), Some(1));
        let field = |name: &str| json["records"][0][name].as_str().expect("string field").to_owned();
        (output.status.success(), field("status"), field("stored_hash"), field("computed_hash"))
    };

    let (updated, status, stored, reviewed) = run("--update");
    assert_eq!((updated, status.as_str(), stored.as_str()), (true, "written", "0"));
    assert_eq!(run("--check"), (true, "match".to_owned(), reviewed.clone(), reviewed.clone()));

    let source = fs::read_to_string(&fixture).expect("read reviewed fixture");
    fs::write(&fixture, source.replace("requested.min(3)", "requested.min(5)")).expect("mutate fixture");
    let (clean, status, stored, drifted) = run("--check");
    assert_eq!((clean, status.as_str(), stored.as_str()), (false, "mismatch", reviewed.as_str()));
    assert_ne!(drifted, reviewed);

    assert_eq!(run("--update"), (true, "written".to_owned(), reviewed, drifted.clone()));
    assert_eq!(run("--check"), (true, "match".to_owned(), drifted.clone(), drifted));
}

#[test]
fn fingerprint_update_check_and_diff_walk_directory_tree() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_directory_fixture(temp.path());

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let expected = BTreeSet::from([
        "attributed_nested",
        "explicit_test_helper",
        "nested_test_helper",
        "read_value",
        "root_plain",
        "root_with_macro",
    ]);
    assert_directory_records(&update.stdout, &expected, "written");

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
    assert_directory_records(&check.stdout, &expected, "match");

    let diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint diff");
    assert!(diff.status.success(), "diff stderr: {}", String::from_utf8_lossy(&diff.stderr));
    assert_directory_records(&diff.stdout, &expected, "match");
}

fn assert_directory_records(stdout: &[u8], expected: &BTreeSet<&str>, status: &str) {
    let json: serde_json::Value = serde_json::from_slice(stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    let names = records.iter().map(|record| record["name"].as_str().expect("record name")).collect::<BTreeSet<_>>();
    assert_eq!(&names, expected);
    assert!(!names.iter().any(|name| name.starts_with("generated")));
    assert!(records.iter().all(|record| record["status"] == status));
    assert!(records.iter().all(|record| !record["review"].is_null()));
    assert!(records.iter().all(|record| !record["evidence"].is_null()));
    assert!(records.iter().all(|record| record["macro_warning"] == false));
}

#[test]
fn fingerprint_check_reports_single_directory_drift() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_directory_fixture(temp.path());

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let lib_rs = temp.path().join("src/lib.rs");
    let source = fs::read_to_string(&lib_rs).expect("read lib.rs");
    fs::write(&lib_rs, source.replace("input.saturating_add(1)", "input.saturating_add(2)"))
        .expect("mutate marked function");

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(!check.status.success(), "drift check should fail");

    let human_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run human fingerprint check");
    assert!(!human_check.status.success(), "human drift check should fail");
    assert_human_summary(&human_check.stdout, "match=5 mismatch=1 missing=0 written=0");
    assert_human_contains_record(&human_check.stdout, "root_plain", "mismatch");

    let json: serde_json::Value = serde_json::from_slice(&check.stdout).expect("json output");
    let records = json["records"].as_array().expect("records array");
    assert_single_directory_mismatch(records);

    let diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint drift diff");
    assert!(!diff.status.success(), "drift diff should fail");

    let diff_json: serde_json::Value = serde_json::from_slice(&diff.stdout).expect("diff json output");
    let diff_records = diff_json["records"].as_array().expect("diff records array");
    assert_single_directory_mismatch(diff_records);

    let repair = Command::new(cargo_octet())
        .args(["fingerprint", "--update"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint repair update");
    assert!(repair.status.success(), "repair stderr: {}", String::from_utf8_lossy(&repair.stderr));
    assert_human_summary(&repair.stdout, "match=5 mismatch=0 missing=0 written=1");
    assert_human_contains_record(&repair.stdout, "root_plain", "written");

    let repaired_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run repaired fingerprint check");
    assert!(
        repaired_check.status.success(),
        "repaired check stderr: {}",
        String::from_utf8_lossy(&repaired_check.stderr)
    );
    assert_directory_records(
        &repaired_check.stdout,
        &BTreeSet::from([
            "attributed_nested",
            "explicit_test_helper",
            "nested_test_helper",
            "read_value",
            "root_plain",
            "root_with_macro",
        ]),
        "match",
    );

    let repaired_diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run repaired fingerprint diff");
    assert!(
        repaired_diff.status.success(),
        "repaired diff stderr: {}",
        String::from_utf8_lossy(&repaired_diff.stderr)
    );
    assert_directory_records(
        &repaired_diff.stdout,
        &BTreeSet::from([
            "attributed_nested",
            "explicit_test_helper",
            "nested_test_helper",
            "read_value",
            "root_plain",
            "root_with_macro",
        ]),
        "match",
    );
}

fn assert_single_directory_mismatch(records: &[serde_json::Value]) {
    assert_eq!(records.len(), 6);
    let mismatches = records.iter().filter(|record| record["status"] == "mismatch").collect::<Vec<_>>();
    assert_eq!(mismatches.len(), 1);
    assert_eq!(mismatches[0]["name"], "root_plain");
    assert!(records.iter().all(|record| !record["review"].is_null()));
    assert!(records.iter().all(|record| !record["evidence"].is_null()));
    assert!(records.iter().all(|record| record["macro_warning"] == false));
}

fn assert_human_summary(stdout: &[u8], expected_counts: &str) {
    let text = String::from_utf8_lossy(stdout);
    let summary = text.lines().find(|line| line.starts_with("summary total=6 ")).expect("human summary line");
    assert!(summary.contains(expected_counts), "summary `{summary}` missing `{expected_counts}`");
    assert!(summary.contains("macro_warnings=0"));
    assert!(summary.contains("metadata_missing=0"));
}

fn assert_human_contains_record(stdout: &[u8], name: &str, status: &str) {
    let text = String::from_utf8_lossy(stdout);
    assert!(
        text.lines().any(|line| line.contains(name) && line.contains(status)),
        "human output did not include {name} as {status}:\n{text}"
    );
}

#[test]
fn object_support_does_not_require_fingerprint_markers() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let fingerprint = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
        .current_dir(temp.path())
        .output()
        .expect("run fingerprint check");
    assert!(!fingerprint.status.success(), "zeroed fixture should report fingerprint mismatches");
    let json: serde_json::Value = serde_json::from_slice(&fingerprint.stdout).expect("fingerprint json");
    assert_eq!(json["records"].as_array().expect("records").len(), 2);
    assert!(json["records"].as_array().unwrap().iter().all(|record| record["stored_hash"] == "0"));
}

#[test]
fn object_pointer_set_records_move_and_validate_fails_closed() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let old_hash = blake3_hex(b"checked_value input + 1");
    let new_hash = blake3_hex(b"checked_value input + 2");
    let old_objects = temp.path().join("old-objects.json");
    let new_objects = temp.path().join("new-objects.json");
    write_object_records(&old_objects, &[("checked_value", &old_hash)]);
    write_object_records(&new_objects, &[("checked_value", &new_hash)]);

    let manifest = temp.path().join("pointers.json");
    let receipt1 = temp.path().join("receipt-old.json");
    let first_set = Command::new(cargo_octet())
        .args(["object", "pointer", "set", "--manifest"])
        .arg(&manifest)
        .args(["--name", "crate::checked_value", "--hash", &old_hash, "--objects"])
        .arg(&old_objects)
        .args(["--receipt"])
        .arg(&receipt1)
        .output()
        .expect("set old pointer");
    assert!(first_set.status.success(), "first pointer stderr: {}", String::from_utf8_lossy(&first_set.stderr));

    let receipt2 = temp.path().join("receipt-new.json");
    let second_set = Command::new(cargo_octet())
        .args(["object", "pointer", "set", "--manifest"])
        .arg(&manifest)
        .args(["--name", "crate::checked_value", "--hash", &new_hash, "--objects"])
        .arg(&new_objects)
        .args(["--receipt"])
        .arg(&receipt2)
        .output()
        .expect("set new pointer");
    assert!(
        second_set.status.success(),
        "second pointer stderr: {}",
        String::from_utf8_lossy(&second_set.stderr)
    );
    let receipt_json: serde_json::Value = serde_json::from_slice(&second_set.stdout).expect("receipt json");
    assert_eq!(receipt_json["old_hash"], old_hash);
    assert_eq!(receipt_json["new_hash"], new_hash);
    assert!(receipt_json["diff_summary"].as_str().unwrap().contains("pointer-moved"));

    let valid = Command::new(cargo_octet())
        .args(["object", "pointer", "validate", "--manifest"])
        .arg(&manifest)
        .args(["--objects"])
        .arg(&new_objects)
        .output()
        .expect("validate pointer");
    assert!(valid.status.success(), "valid pointer stderr: {}", String::from_utf8_lossy(&valid.stderr));

    let invalid = Command::new(cargo_octet())
        .args(["object", "pointer", "validate", "--manifest"])
        .arg(&manifest)
        .args(["--objects"])
        .arg(&old_objects)
        .output()
        .expect("validate stale pointer");
    assert!(!invalid.status.success(), "stale pointer unexpectedly resolved");
    let invalid_json: serde_json::Value = serde_json::from_slice(&invalid.stdout).expect("invalid validate json");
    assert_eq!(invalid_json["status"], "missing-object");
}

#[test]
fn object_diff_separates_body_signature_caveat_and_pointer_changes() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let old_path = temp.path().join("old.json");
    let new_path = temp.path().join("new.json");
    fs::write(
        &old_path,
        r#"{"object_hash":"old","name":"checked","body_hash":"body-a","signature_hash":"sig","dependency_caveats":["deps"],"effect_caveats":["none"],"macro_caveats":["none"]}"#,
    )
    .expect("write old object");
    fs::write(
        &new_path,
        r#"{"object_hash":"new","name":"checked","body_hash":"body-b","signature_hash":"sig","dependency_caveats":["deps"],"effect_caveats":["panic"],"macro_caveats":["none"]}"#,
    )
    .expect("write new object");
    let manifest = temp.path().join("pointers.json");
    fs::write(
        &manifest,
        r#"{"schema_version":1,"pointers":{"crate::checked":{"object_hash":"new","semantic_name":"crate::checked"}},"history":{}}"#,
    )
    .expect("write pointer manifest");

    let diff = Command::new(cargo_octet())
        .args(["object", "diff", "--old"])
        .arg(&old_path)
        .args(["--new"])
        .arg(&new_path)
        .args(["--pointer-manifest"])
        .arg(&manifest)
        .args(["--output-format=json"])
        .output()
        .expect("run object diff");
    assert!(diff.status.success(), "diff stderr: {}", String::from_utf8_lossy(&diff.stderr));
    let json: serde_json::Value = serde_json::from_slice(&diff.stdout).expect("diff json");
    assert_eq!(json["body_changed"], true);
    assert_eq!(json["signature_changed"], false);
    assert_eq!(json["effect_caveats_changed"], true);
    assert_eq!(json["macro_caveats_changed"], false);
    assert_eq!(json["pointer_changes"].as_array().unwrap().len(), 1);
}

#[test]
fn object_bundle_export_is_deterministic_and_records_replay() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let objects_path = temp.path().join("objects.json");
    let alpha_hash = blake3_hex(b"alpha input + 1");
    write_object_records(&objects_path, &[("beta", &blake3_hex(b"beta input + 2")), ("alpha", &alpha_hash)]);
    let pointers = temp.path().join("pointers.json");
    let receipt = temp.path().join("receipt.json");
    let pointer_set = Command::new(cargo_octet())
        .args(["object", "pointer", "set", "--manifest"])
        .arg(&pointers)
        .args(["--name", "crate::alpha", "--hash", &alpha_hash, "--objects"])
        .arg(&objects_path)
        .args(["--receipt"])
        .arg(&receipt)
        .output()
        .expect("set pointer");
    assert!(pointer_set.status.success(), "pointer stderr: {}", String::from_utf8_lossy(&pointer_set.stderr));

    let one = temp.path().join("bundle-one.json");
    let two = temp.path().join("bundle-two.json");
    for output_path in [&one, &two] {
        let bundle = Command::new(cargo_octet())
            .args(["object", "bundle", "export", "--objects"])
            .arg(&objects_path)
            .args(["--pointers"])
            .arg(&pointers)
            .args(["--output"])
            .arg(output_path)
            .args([
                "--replay-command",
                "cargo octet object emit --output-format=json lib.rs",
            ])
            .output()
            .expect("export bundle");
        assert!(bundle.status.success(), "bundle stderr: {}", String::from_utf8_lossy(&bundle.stderr));
    }
    let first = fs::read_to_string(&one).expect("read first bundle");
    let second = fs::read_to_string(&two).expect("read second bundle");
    assert_eq!(first, second);
    let json: serde_json::Value = serde_json::from_str(&first).expect("bundle json");
    assert_eq!(json["schema"], "octet.function-object-bundle.v1");
    assert_eq!(json["object_count"], 2);
    assert_eq!(json["pointer_count"], 1);
    assert!(json["bundle_hash"].as_str().unwrap().starts_with("b3:"));
    assert_eq!(json["replay"]["command"], "cargo octet object emit --output-format=json lib.rs");
    assert!(json["replay"]["determinism_note"].as_str().unwrap().contains("not correctness"));

    let missing_pointer = temp.path().join("missing-pointer.json");
    let missing_hash = blake3_hex(b"no such object");
    fs::write(
        &missing_pointer,
        format!(
            r#"{{"schema_version":1,"pointers":{{"crate::missing":{{"object_hash":"{missing_hash}","semantic_name":"crate::missing"}}}},"history":{{}}}}"#
        ),
    )
    .expect("write missing pointer manifest");
    let unresolved = Command::new(cargo_octet())
        .args(["object", "bundle", "export", "--objects"])
        .arg(&objects_path)
        .args(["--pointers"])
        .arg(&missing_pointer)
        .args(["--output"])
        .arg(temp.path().join("unresolved-bundle.json"))
        .args([
            "--replay-command",
            "cargo octet object emit --output-format=json lib.rs",
        ])
        .output()
        .expect("export bundle with unresolved pointer");
    assert!(!unresolved.status.success(), "bundle with an unresolved pointer unexpectedly passed");
    assert!(
        String::from_utf8_lossy(&unresolved.stderr).contains("references missing object"),
        "unresolved pointer was not named: {}",
        String::from_utf8_lossy(&unresolved.stderr)
    );
    assert!(!temp.path().join("unresolved-bundle.json").exists());
}

#[test]
fn object_bundle_export_rejects_unknown_flags_before_writing() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let objects_path = temp.path().join("objects.json");
    write_object_records(&objects_path, &[("alpha", &blake3_hex(b"alpha input + 1"))]);
    let bundle_path = temp.path().join("bundle.json");

    let typo = Command::new(cargo_octet())
        .args(["object", "bundle", "export", "--objects"])
        .arg(&objects_path)
        .args(["--pointer"])
        .arg(temp.path().join("pointers.json"))
        .args(["--output"])
        .arg(&bundle_path)
        .args([
            "--replay-command",
            "cargo octet object emit --output-format=json lib.rs",
        ])
        .output()
        .expect("export bundle with typo flag");

    assert_eq!(typo.status.code(), Some(2), "stderr: {}", String::from_utf8_lossy(&typo.stderr));
    assert!(
        String::from_utf8_lossy(&typo.stderr).contains("unsupported object bundle export argument `--pointer`"),
        "typo flag was not named: {}",
        String::from_utf8_lossy(&typo.stderr)
    );
    assert!(!bundle_path.exists(), "rejected bundle export must not write a bundle");
}

#[test]
fn object_pointer_validate_and_bundle_export_reject_missing_manifest() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let objects_path = temp.path().join("objects.json");
    write_object_records(&objects_path, &[("alpha", &blake3_hex(b"alpha input + 1"))]);
    let missing_manifest = temp.path().join("typo-pointers.json");

    let validate = Command::new(cargo_octet())
        .args(["object", "pointer", "validate", "--manifest"])
        .arg(&missing_manifest)
        .args(["--objects"])
        .arg(&objects_path)
        .output()
        .expect("validate missing pointer manifest");
    assert!(
        !validate.status.success(),
        "missing manifest unexpectedly validated: {}",
        String::from_utf8_lossy(&validate.stdout)
    );
    assert!(
        String::from_utf8_lossy(&validate.stderr).contains("typo-pointers.json"),
        "missing manifest was not named: {}",
        String::from_utf8_lossy(&validate.stderr)
    );

    let bundle_path = temp.path().join("bundle.json");
    let bundle = Command::new(cargo_octet())
        .args(["object", "bundle", "export", "--objects"])
        .arg(&objects_path)
        .args(["--pointers"])
        .arg(&missing_manifest)
        .args(["--output"])
        .arg(&bundle_path)
        .args([
            "--replay-command",
            "cargo octet object emit --output-format=json lib.rs",
        ])
        .output()
        .expect("export bundle with missing pointer manifest");
    assert!(
        !bundle.status.success(),
        "bundle with a missing pointer manifest unexpectedly passed: {}",
        String::from_utf8_lossy(&bundle.stdout)
    );
    assert!(
        String::from_utf8_lossy(&bundle.stderr).contains("typo-pointers.json"),
        "missing manifest was not named: {}",
        String::from_utf8_lossy(&bundle.stderr)
    );
    assert!(!bundle_path.exists(), "rejected bundle export must not write a bundle");
    assert!(!missing_manifest.exists(), "validate and export must not create the manifest");
}

const INTERNAL_OBJECT_CORPUS: &[&str] = &[
    "cargo-octet/src/baseline.rs",
    "cargo-octet/src/bin/cargo-slotcar.rs",
    "cargo-octet/src/bin/cargo-tigerstyle.rs",
    "cargo-octet/src/cli.rs",
    "cargo-octet/src/config.rs",
    "cargo-octet/src/determinism.rs",
    "cargo-octet/src/engine.rs",
    "cargo-octet/src/evidence.rs",
    "cargo-octet/src/evidence_rails.rs",
    "cargo-octet/src/fingerprint.rs",
    "cargo-octet/src/lib.rs",
    "cargo-octet/src/main.rs",
    "cargo-octet/src/migrate.rs",
    "cargo-octet/src/object.rs",
    "cargo-octet/src/output.rs",
    "cargo-octet/src/provider_manifest.rs",
    "cargo-octet/src/roadmap.rs",
    "cargo-octet/src/transcript.rs",
];

const INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/internal-corpus-receipt.json";
const EXPANDED_INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-receipt.json";
const EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-taxonomy.json";
const EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE: &str =
    "cargo-octet/tests/fixtures/object-corpus/expanded-internal-corpus-caveat-taxonomy.json";
const OBJECT_CORPUS_COMPATIBILITY_DIR: &str = "cargo-octet/tests/fixtures/object-corpus/compatibility";
const OBJECT_CORPUS_COMPATIBILITY_SOURCE: &[&str] = &["src/lib.rs"];
const OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1: &str = "positive-v1-receipt.json";
const OBJECT_CORPUS_CANDIDATE_V2_DIR: &str = "cargo-octet/tests/fixtures/object-corpus/source-token-v2";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MISSING_OBJECT_SET_HASH: &str = "negative-missing-object-set-hash.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_FUTURE_SCHEMA_VERSION: &str = "negative-future-schema-version.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_BOUNDARY_TAMPER: &str = "negative-boundary-tamper.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_DUPLICATE_SOURCE_PATH: &str = "negative-duplicate-source-path.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_EMPTY_SOURCE_SELECTION: &str = "negative-empty-source-selection.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MALFORMED_JSON: &str = "negative-malformed-json.json";
const OBJECT_CORPUS_COMPATIBILITY_NON_RUST_SOURCE: &str = "notes.txt";

fn check_corpus_receipt(root: &Path, receipt_path: &Path, paths: &[&str]) -> std::process::Output {
    Command::new(cargo_octet())
        .args(["object", "corpus", "check", "--receipt"])
        .arg(receipt_path)
        .args(["--output-format", "json"])
        .args(paths)
        .current_dir(root)
        .output()
        .expect("check object corpus receipt")
}

fn check_corpus_receipt_fixture(root: &Path, receipt_name: &str) -> std::process::Output {
    check_corpus_receipt(
        &root.join(OBJECT_CORPUS_COMPATIBILITY_DIR),
        Path::new(receipt_name),
        OBJECT_CORPUS_COMPATIBILITY_SOURCE,
    )
}

fn read_json_fixture(root: &Path, relative: &str) -> serde_json::Value {
    serde_json::from_slice(&fs::read(root.join(relative)).unwrap_or_else(|error| panic!("read {relative}: {error}")))
        .unwrap_or_else(|error| panic!("parse {relative}: {error}"))
}

fn object_corpus_contract(root: &Path) -> serde_json::Value {
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "contract", "--output-format", "json"])
        .current_dir(root)
        .output()
        .expect("object corpus contract metadata");
    assert!(
        output.status.success(),
        "contract stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("contract metadata json")
}

fn blake3_hex(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

/// Writes `objects` in the `object emit` JSON shape. Commands that read
/// existing object evidence (`pointer`, `bundle export`) need only each
/// record's name and object hash.
fn write_object_records(path: &Path, objects: &[(&str, &str)]) {
    let objects = objects
        .iter()
        .map(|(name, hash)| serde_json::json!({ "schema": "octet.function-object.v1", "name": name, "object_hash": hash }))
        .collect::<Vec<_>>();
    let report = serde_json::json!({ "schema_version": 1, "mode": "emit", "objects": objects });
    fs::write(path, serde_json::to_vec_pretty(&report).expect("object records json")).expect("write object records");
}

/// Copies the checked-in v1 compatibility receipt to `dir/receipt.json` and its
/// source to `dir/lib.rs`, for receipt validation that runs before any object is
/// built.
fn copy_checked_in_receipt(dir: &Path) -> (std::path::PathBuf, serde_json::Value) {
    let compatibility = repo_root().join(OBJECT_CORPUS_COMPATIBILITY_DIR);
    fs::copy(compatibility.join("src/lib.rs"), dir.join("lib.rs")).expect("copy compatibility source");
    let receipt_path = dir.join("receipt.json");
    fs::copy(compatibility.join(OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1), &receipt_path)
        .expect("copy compatibility receipt");
    (receipt_path, read_json_fixture(&compatibility, OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1))
}

fn object_v2_command(root: &Path, args: &[&str]) -> serde_json::Value {
    let output = Command::new(cargo_octet())
        .args(args)
        .current_dir(root)
        .output()
        .expect("run candidate object command");
    assert!(
        output.status.success(),
        "{args:?}: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("candidate object command json")
}

fn object_v2_hash(root: &Path) -> serde_json::Value {
    object_v2_command(root, &["object", "emit", "--output-format=json", "lib.rs"])["objects"][0].clone()
}

fn reviewed_v2_selection(root: &Path, role: &str) -> Vec<String> {
    let manifest = read_json_fixture(root, &format!("{OBJECT_CORPUS_CANDIDATE_V2_DIR}/reviewed-source-manifest.json"));
    manifest["source_selections"][role]
        .as_array()
        .unwrap_or_else(|| panic!("missing reviewed {role} selection"))
        .iter()
        .map(|path| path.as_str().expect("reviewed source path").to_owned())
        .collect()
}

fn selected_corpus_receipt_bytes(root: &Path, paths: &[String]) -> Vec<u8> {
    let temp = tempfile::tempdir().expect("temporary corpus receipt directory");
    let output_path = temp.path().join("receipt.json");
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "receipt", "--output"])
        .arg(&output_path)
        .args(paths)
        .current_dir(root)
        .output()
        .expect("emit source-backed corpus receipt");
    assert!(output.status.success(), "corpus receipt stderr: {}", String::from_utf8_lossy(&output.stderr));
    fs::read(output_path).expect("read source-backed corpus receipt")
}

fn selected_corpus_check(root: &Path, receipt: &Path, paths: &[String]) -> serde_json::Value {
    let selection = paths.iter().map(String::as_str).collect::<Vec<_>>();
    let output = check_corpus_receipt(root, receipt, &selection);
    assert!(
        output.status.success(),
        "corpus check stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("parse source-backed corpus check");
    assert_eq!(report["status"], "match", "reviewed receipt mismatch: {report:#?}");
    report
}

#[test]
fn object_v2_source_identity_preserves_url_tail_and_trivia() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source = "\n// prior comment\npub fn fetch(input: u32) -> u32 {\n    let _endpoint = \"https://example.invalid/api\"; input + 1\n}\n";
    fs::write(temp.path().join("lib.rs"), source).expect("write source");
    let original = object_v2_hash(temp.path());
    assert_eq!(original["normalization"]["algorithm"], "octet-source-token-v2");
    assert_eq!(original["schema"], "octet.function-object.v1");
    assert_eq!(original["source_path"], "lib.rs");
    assert_eq!(original["source_span"], serde_json::json!({"start_line": 3, "end_line": 5}));
    assert_eq!(original["correctness_claim"]["claims_correctness"], false);
    assert_eq!(original["pure_cache_eligibility"]["eligible"], false);

    let assert_identity_changed = |changed: &str| {
        fs::write(temp.path().join("lib.rs"), changed).expect("mutate source");
        let object = object_v2_hash(temp.path());
        assert_ne!(object["normalized_body_hash"], original["normalized_body_hash"]);
        assert_ne!(object["semantic_id"], original["semantic_id"]);
        assert_ne!(object["object_hash"], original["object_hash"]);
    };
    assert_identity_changed(&source.replace("example.invalid", "other.invalid"));
    assert_identity_changed(&source.replace("input + 1", "input + 2"));

    let trivia = source
        .replace("// prior comment", "/* another prior comment */")
        .replace("let _endpoint", "/* scanner ignores only comments */ let _endpoint");
    fs::write(temp.path().join("lib.rs"), trivia).expect("write trivia-only edit");
    let unchanged = object_v2_hash(temp.path());
    assert_eq!(unchanged["normalized_body_hash"], original["normalized_body_hash"]);
    assert_eq!(unchanged["object_hash"], original["object_hash"]);
    assert_eq!(unchanged["correctness_claim"]["claims_correctness"], false);
}

#[test]
fn object_v2_macro_token_boundaries_change_identity_without_trivia_drift() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let object_for = |argument: &str| {
        fs::write(temp.path().join("lib.rs"), format!("pub fn selected() -> usize {{ {argument} }}\n"))
            .expect("write macro argument source");
        object_v2_hash(temp.path())
    };

    // These pairs are distinct TokenStreams accepted by pinned rustc: Joint
    // versus Alone punctuation, one numeric Literal versus three tokens, and
    // one raw Ident versus three tokens, a raw lifetime versus separate
    // punctuation and identifiers, a suffixed raw Literal versus two tokens,
    // and a doc-comment attribute versus ordinary trivia. Macro
    // caveats do not license a shared reviewed source identity.
    for (joined, separated) in [
        ("punct_probe::spacing!(=>)", "punct_probe::spacing!(= >)"),
        ("token_count::count!(1.0)", "token_count::count!(1 . 0)"),
        ("token_count::count!(0.5)", "token_count::count!(0 . 5)"),
        ("token_count::count!(0e3)", "token_count::count!(0 e3)"),
        ("token_count::count!(r#type)", "token_count::count!(r #type)"),
        ("token_count::count!('r#foo)", "token_count::count!('r #foo)"),
        ("token_count::count!(r#\"x\"#suffix)", "token_count::count!(r#\"x\"# suffix)"),
        ("token_count::count!(// ordinary\nfoo)", "token_count::count!(/// doc\nfoo)"),
    ] {
        let first = object_for(joined);
        let second = object_for(separated);
        assert_ne!(first["normalized_body_hash"], second["normalized_body_hash"], "{joined} vs {separated}");
        assert_ne!(first["object_hash"], second["object_hash"], "{joined} vs {separated}");
    }

    let alone = object_for("punct_probe::spacing!(= >)");
    assert_eq!(alone["object_hash"], object_for("punct_probe::spacing!(=/* trivia */>)")["object_hash"]);
    let group = object_for("token_count::count!(#[inline])");
    assert_eq!(group["object_hash"], object_for("token_count::count!(# [inline])")["object_hash"]);
    let literal = object_for("token_count::count!(r#\"a => b\"#)");
    assert_eq!(literal["object_hash"], object_for("token_count::count!( r#\"a => b\"# )")["object_hash"]);
}

#[test]
fn object_v2_export_name_change_across_comment_changes_identity_but_trivia_does_not() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source =
        "#[unsafe(export_name = \"alpha\")]\n// explanation of export\n\npub extern \"C\" fn exported() -> u32 { 1 }\n";
    fs::write(temp.path().join("lib.rs"), source).expect("write alpha source");
    let alpha = object_v2_hash(temp.path());
    assert_eq!(alpha["source_span"], serde_json::json!({"start_line": 1, "end_line": 4}));

    fs::write(temp.path().join("lib.rs"), source.replace("alpha", "omega")).expect("change exported symbol");
    let omega = object_v2_hash(temp.path());
    assert_ne!(alpha["normalized_body_hash"], omega["normalized_body_hash"]);
    assert_ne!(alpha["semantic_id"], omega["semantic_id"]);
    assert_ne!(alpha["object_hash"], omega["object_hash"]);

    let trivia = source.replace("// explanation of export\n\n", "/* another comment */\n");
    fs::write(temp.path().join("lib.rs"), trivia).expect("change only trivia");
    let unchanged = object_v2_hash(temp.path());
    assert_eq!(unchanged["normalized_body_hash"], alpha["normalized_body_hash"]);
    assert_eq!(unchanged["object_hash"], alpha["object_hash"]);
}

#[test]
fn object_v2_multiline_attribute_and_split_visibility_change_identity() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source = "#[doc = \"literal [brackets]\"]\n#[unsafe(export_name =\n    \"alpha\"\n)]\n// review gap\npub extern \"C\" fn exported() -> u32 { 1 }\n";
    fs::write(temp.path().join("lib.rs"), source).expect("write multiline alpha");
    let alpha = object_v2_hash(temp.path());
    assert_eq!(alpha["source_span"], serde_json::json!({"start_line": 1, "end_line": 6}));

    fs::write(temp.path().join("lib.rs"), source.replace("alpha", "omega")).expect("change exported symbol");
    let omega = object_v2_hash(temp.path());
    assert_ne!(alpha["normalized_body_hash"], omega["normalized_body_hash"]);
    assert_ne!(alpha["object_hash"], omega["object_hash"]);

    fs::write(temp.path().join("lib.rs"), source.replace("// review gap", "/* different explanation */"))
        .expect("change only trivia");
    assert_eq!(object_v2_hash(temp.path())["object_hash"], alpha["object_hash"]);

    fs::write(temp.path().join("lib.rs"), "pub\nfn exported() -> u32 { 1 }\n").expect("write split public function");
    let public = object_v2_hash(temp.path());
    assert_eq!(public["source_span"], serde_json::json!({"start_line": 1, "end_line": 2}));
    fs::write(temp.path().join("lib.rs"), "// private comparison\nfn exported() -> u32 { 1 }\n")
        .expect("write private function");
    let private = object_v2_hash(temp.path());
    assert_ne!(public["object_hash"], private["object_hash"]);

    fs::write(temp.path().join("lib.rs"), "pub(crate)\nfn exported() -> u32 { 1 }\n").expect("restrict visibility");
    assert_ne!(public["object_hash"], object_v2_hash(temp.path())["object_hash"]);
}

#[test]
fn object_v2_reviewed_marker_above_separated_attribute_goes_stale_on_export_change() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source = format!(
        "// octet-object: b3:{}; review=R-export; evidence=E-export\n#[unsafe(export_name =\n    \"alpha\"\n)]\n// review gap\npub extern \"C\" fn exported() -> u32 {{ 1 }}\n",
        "0".repeat(64)
    );
    fs::write(temp.path().join("lib.rs"), source).expect("write marked export");
    object_v2_command(temp.path(), &["object", "update", "--output-format=json", "lib.rs"]);
    let checked = object_v2_command(temp.path(), &["object", "check", "--output-format=json", "lib.rs"]);
    assert_eq!(checked["records"][0]["status"], "match");

    let reviewed = fs::read_to_string(temp.path().join("lib.rs")).expect("read reviewed export");
    fs::write(temp.path().join("lib.rs"), reviewed.replace("alpha", "omega")).expect("change exported symbol");
    let check = Command::new(cargo_octet())
        .args(["object", "check", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("check changed export");
    assert_eq!(check.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("parse marker mismatch");
    assert_eq!(report["records"][0]["status"], "mismatch");
}

#[test]
fn object_v2_comment_forgery_with_matching_digest_is_missing_not_reviewed() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source_path = temp.path().join("lib.rs");
    let forged = format!(
        "/* outer /* nested */\n// octet-object: b3:{}; review=R-stale; evidence=E-stale\n*/ pub fn selected() -> u32 {{ 1 }}\n",
        "0".repeat(64)
    );
    fs::write(&source_path, &forged).expect("write forged marker source");
    let actual = object_v2_hash(temp.path())["object_hash"].as_str().expect("object hash").to_owned();
    fs::write(&source_path, forged.replace(&format!("b3:{}", "0".repeat(64)), &actual))
        .expect("write matching forged marker");

    let checked = Command::new(cargo_octet())
        .args(["object", "check", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("check forged marker");
    assert_eq!(checked.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&checked.stdout).expect("parse denied marker");
    assert_eq!(report["records"][0]["status"], "missing");
    assert_eq!(report["records"][0]["computed_hash"], actual);
    assert!(report["records"][0]["stored_hash"].is_null());
    assert!(report["records"][0]["review"].is_null());
    assert!(report["records"][0]["evidence"].is_null());

    let updated = object_v2_command(temp.path(), &["object", "update", "--output-format=json", "lib.rs"]);
    assert_eq!(updated["records"][0]["status"], "written");
    let after = Command::new(cargo_octet())
        .args(["object", "check", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("check newly inserted active marker");
    assert_eq!(after.status.code(), Some(1), "new marker still requires explicit review metadata");
    let checked: serde_json::Value = serde_json::from_slice(&after.stdout).expect("parse updated marker");
    assert_eq!(checked["records"][0]["status"], "match");
    assert_eq!(checked["records"][0]["computed_hash"], actual);
    assert!(checked["records"][0]["review"].is_null());
}

#[test]
fn fingerprint_and_object_markers_stay_bound_when_stacked_in_either_order() {
    let run = |root: &Path, args: &[&str]| {
        let output = Command::new(cargo_octet()).args(args).current_dir(root).output().expect("run cargo-octet");
        let json: serde_json::Value = serde_json::from_slice(&output.stdout)
            .unwrap_or_else(|error| panic!("{args:?}: {error}; stderr={}", String::from_utf8_lossy(&output.stderr)));
        let statuses = json["records"]
            .as_array()
            .expect("records")
            .iter()
            .map(|record| record["status"].as_str().expect("status").to_owned())
            .collect::<Vec<_>>();
        (output.status.code(), statuses)
    };
    let fingerprint = |root: &Path, mode: &str| run(root, &["fingerprint", mode, "--output-format=json", "lib.rs"]);
    let object = |root: &Path, mode: &str| run(root, &["object", mode, "--output-format=json", "lib.rs"]);
    let review = |root: &Path| {
        let path = root.join("lib.rs");
        let source = fs::read_to_string(&path).expect("read lib.rs");
        fs::write(&path, source.replace("review=TODO; evidence=TODO", "review=R-1; evidence=E-1")).expect("review");
    };

    for fingerprint_first in [true, false] {
        let temp = tempfile::tempdir().expect("create tempdir");
        let root = temp.path();
        fs::write(root.join("lib.rs"), "#[inline]\npub fn selected(input: u32) -> u32 {\n    input + 1\n}\n")
            .expect("write lib.rs");
        if fingerprint_first {
            assert_eq!(fingerprint(root, "--write"), (Some(0), vec!["written".to_owned()]));
            assert_eq!(object(root, "update"), (Some(0), vec!["written".to_owned()]));
        } else {
            assert_eq!(object(root, "update"), (Some(0), vec!["written".to_owned()]));
            assert_eq!(fingerprint(root, "--write"), (Some(0), vec!["written".to_owned()]));
        }
        review(root);
        let order = format!("fingerprint_first={fingerprint_first}");
        assert_eq!(fingerprint(root, "--check"), (Some(0), vec!["match".to_owned()]), "{order}");
        assert_eq!(object(root, "check"), (Some(0), vec!["match".to_owned()]), "{order}");

        let source = fs::read_to_string(root.join("lib.rs")).expect("read lib.rs");
        fs::write(root.join("lib.rs"), source.replace("input + 1", "input + 2")).expect("drift lib.rs");
        assert_eq!(fingerprint(root, "--check"), (Some(1), vec!["mismatch".to_owned()]), "{order}");
        assert_eq!(object(root, "check"), (Some(1), vec!["mismatch".to_owned()]), "{order}");
        assert_eq!(fingerprint(root, "--update"), (Some(0), vec!["written".to_owned()]), "{order}");
        assert_eq!(object(root, "update"), (Some(0), vec!["written".to_owned()]), "{order}");
        assert_eq!(fingerprint(root, "--check"), (Some(0), vec!["match".to_owned()]), "{order}");
        assert_eq!(object(root, "check"), (Some(0), vec!["match".to_owned()]), "{order}");

        let source = fs::read_to_string(root.join("lib.rs")).expect("read lib.rs");
        assert_eq!(source.matches("// octet-fingerprint:").count(), 1, "{order}: {source}");
        assert_eq!(source.matches("// octet-object:").count(), 1, "{order}: {source}");
    }
}

#[test]
fn object_marker_metadata_keys_match_exactly() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source_path = temp.path().join("lib.rs");
    fs::write(&source_path, "pub fn selected() -> u32 { 1 }\n").expect("write unmarked source");
    object_v2_command(temp.path(), &["object", "update", "--output-format=json", "lib.rs"]);
    let written = fs::read_to_string(&source_path).expect("read marked source");
    fs::write(&source_path, written.replace("review=TODO; evidence=TODO", "preview=P; evidence=E; reevidence=X"))
        .expect("write lookalike metadata keys");

    let checked = Command::new(cargo_octet())
        .args(["object", "check", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("check lookalike metadata keys");
    assert_eq!(checked.status.code(), Some(1), "`preview` is not `review`");
    let report: serde_json::Value = serde_json::from_slice(&checked.stdout).expect("parse marker check");
    assert_eq!(report["records"][0]["status"], "match");
    assert!(report["records"][0]["review"].is_null(), "{report}");
    assert_eq!(report["records"][0]["evidence"], "E");
}

#[test]
fn object_internal_corpus_matches_reviewed_source_reference_and_rejects_tamper() {
    let root = repo_root();
    let sources = reviewed_v2_selection(&root, "internal_corpus");
    let saved_path = root.join(format!("{OBJECT_CORPUS_CANDIDATE_V2_DIR}/internal-corpus-receipt.json"));
    let expected = fs::read(&saved_path).expect("read independently reviewed internal receipt");
    assert_eq!(selected_corpus_receipt_bytes(&root, &sources), expected);
    selected_corpus_check(&root, &saved_path, &sources);

    let temp = tempfile::tempdir().expect("create tempdir");
    let tampered_path = temp.path().join("tampered-internal.json");
    let mut tampered: serde_json::Value = serde_json::from_slice(&expected).expect("reviewed internal receipt json");
    tampered["object_set_hash"] = serde_json::json!(blake3_hex(b"wrong-internal-object-set"));
    fs::write(&tampered_path, serde_json::to_vec(&tampered).expect("encode tampered internal receipt"))
        .expect("write tampered internal receipt");
    let selection = sources.iter().map(String::as_str).collect::<Vec<_>>();
    let mismatch = check_corpus_receipt(&root, &tampered_path, &selection);
    assert_eq!(mismatch.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&mismatch.stdout).expect("tampered internal report");
    assert_eq!(report["status"], "mismatch");
    assert!(
        report["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|comparison| { comparison["field"] == "object_set_hash" && comparison["status"] == "mismatch" })
    );
}

#[test]
fn object_corpus_compatibility_positive_v2_replays_and_rejects_source_and_boundary_drift() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = repo_root().join(OBJECT_CORPUS_CANDIDATE_V2_DIR);
    fs::create_dir(temp.path().join("src")).expect("create fixture source directory");
    let source = fs::read_to_string(fixture.join("src/lib.rs")).expect("read fixed candidate source");
    fs::write(temp.path().join("src/lib.rs"), &source).expect("copy candidate source");
    let receipt_path = temp.path().join("positive-v2-receipt.json");
    fs::copy(fixture.join("positive-v2-receipt.json"), &receipt_path).expect("copy saved candidate receipt");
    let saved: serde_json::Value = serde_json::from_slice(&fs::read(&receipt_path).expect("read saved receipt"))
        .expect("parse saved candidate receipt");
    assert_eq!(saved["boundary"]["claims_correctness"], false);
    assert_eq!(saved["boundary"]["claims_semantic_equivalence"], false);

    let replay = || check_corpus_receipt(temp.path(), &receipt_path, &["src/lib.rs"]);
    let matching = replay();
    assert!(matching.status.success(), "{}", String::from_utf8_lossy(&matching.stderr));
    let report: serde_json::Value = serde_json::from_slice(&matching.stdout).expect("parse matching replay");
    assert_eq!(report["status"], "match");

    for changed in [
        source.replace("example.invalid", "other.invalid"),
        source.replace("saturating_add(1)", "saturating_add(2)"),
    ] {
        fs::write(temp.path().join("src/lib.rs"), changed).expect("mutate candidate source");
        let mismatch = replay();
        assert_eq!(mismatch.status.code(), Some(1));
        let report: serde_json::Value = serde_json::from_slice(&mismatch.stdout).expect("parse source drift");
        assert_eq!(report["status"], "mismatch");
        assert!(
            report["comparisons"]
                .as_array()
                .expect("comparisons")
                .iter()
                .any(|comparison| { comparison["field"] == "object_set_hash" && comparison["status"] == "mismatch" })
        );
    }

    fs::write(temp.path().join("src/lib.rs"), source).expect("restore candidate source");
    let mut tampered = saved;
    tampered["boundary"]["claims_correctness"] = serde_json::json!(true);
    fs::write(&receipt_path, serde_json::to_vec(&tampered).expect("serialize tampered receipt"))
        .expect("tamper candidate receipt");
    let mismatch = replay();
    assert_eq!(mismatch.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&mismatch.stdout).expect("parse boundary drift");
    assert_eq!(report["status"], "mismatch");
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons")
            .iter()
            .any(|comparison| { comparison["field"] == "boundary" && comparison["status"] == "mismatch" })
    );
}

#[test]
fn object_caveat_taxonomy_matches_reviewed_expanded_reference_and_detects_added_function() {
    let root = repo_root();
    let sources = reviewed_v2_selection(&root, "expanded_internal_corpus");
    let receipt_path = root.join(format!("{OBJECT_CORPUS_CANDIDATE_V2_DIR}/expanded-internal-corpus-receipt.json"));
    assert_eq!(
        selected_corpus_receipt_bytes(&root, &sources),
        fs::read(&receipt_path).expect("read reviewed expanded receipt")
    );
    selected_corpus_check(&root, &receipt_path, &sources);
    let taxonomy_path =
        root.join(format!("{OBJECT_CORPUS_CANDIDATE_V2_DIR}/expanded-internal-corpus-caveat-taxonomy.json"));
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "caveats", "--output-format=json"])
        .args(&sources)
        .current_dir(&root)
        .output()
        .expect("emit expanded taxonomy from selected sources");
    assert!(output.status.success(), "expanded taxonomy stderr: {}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(output.stdout, fs::read(&taxonomy_path).expect("read reviewed taxonomy"));
    let taxonomy: serde_json::Value = serde_json::from_slice(&output.stdout).expect("expanded taxonomy json");
    assert_eq!(taxonomy["boundary"]["claims_correctness"], false);
    assert_eq!(taxonomy["boundary"]["claims_semantic_equivalence"], false);

    let temp = tempfile::tempdir().expect("temporary altered source corpus");
    for source in &sources {
        let destination = temp.path().join(source);
        fs::create_dir_all(destination.parent().expect("source parent")).expect("create corpus source directory");
        fs::copy(root.join(source), &destination).expect("copy selected source");
    }
    let changed_source = temp.path().join("cargo-octet/src/main.rs");
    let mut changed = fs::read_to_string(&changed_source).expect("read copied Rust source");
    changed.push_str("\npub fn added_taxonomy_probe() { std::process::exit(3); }\n");
    fs::write(&changed_source, changed).expect("add new function to copied Rust source");
    let changed_report = Command::new(cargo_octet())
        .args(["object", "corpus", "caveats", "--output-format=json"])
        .args(&sources)
        .current_dir(temp.path())
        .output()
        .expect("emit taxonomy for changed source");
    assert!(
        changed_report.status.success(),
        "changed taxonomy stderr: {}",
        String::from_utf8_lossy(&changed_report.stderr)
    );
    let changed: serde_json::Value = serde_json::from_slice(&changed_report.stdout).expect("changed taxonomy json");
    assert_eq!(changed["object_count"].as_u64(), Some(taxonomy["object_count"].as_u64().unwrap() + 1));
    assert_ne!(changed_report.stdout, output.stdout, "source-added function must stale reviewed taxonomy");
}

#[test]
fn object_corpus_matches_fresh_reviewed_references_and_rejects_missing_receipt() {
    let root = repo_root();
    let fixture = root.join(OBJECT_CORPUS_CANDIDATE_V2_DIR);
    let fixed = vec!["src/lib.rs".to_owned()];
    assert_eq!(
        selected_corpus_receipt_bytes(&fixture, &fixed),
        fs::read(fixture.join("positive-v2-receipt.json")).expect("read reviewed fixed receipt")
    );
    selected_corpus_check(&fixture, &fixture.join("positive-v2-receipt.json"), &fixed);
    for (role, filename) in [
        ("internal_corpus", "internal-corpus-receipt.json"),
        ("expanded_internal_corpus", "expanded-internal-corpus-receipt.json"),
    ] {
        let sources = reviewed_v2_selection(&root, role);
        assert_eq!(
            selected_corpus_receipt_bytes(&root, &sources),
            fs::read(fixture.join(filename)).expect("read independently saved reference"),
            "stale {role} receipt"
        );
    }
    let expanded = reviewed_v2_selection(&root, "expanded_internal_corpus");
    let taxonomy = Command::new(cargo_octet())
        .args(["object", "corpus", "caveats", "--output-format=json"])
        .args(&expanded)
        .current_dir(&root)
        .output()
        .expect("emit freshness taxonomy");
    assert!(
        taxonomy.status.success(),
        "freshness taxonomy stderr: {}",
        String::from_utf8_lossy(&taxonomy.stderr)
    );
    assert_eq!(
        taxonomy.stdout,
        fs::read(fixture.join("expanded-internal-corpus-caveat-taxonomy.json")).expect("read reviewed taxonomy")
    );

    let missing = check_corpus_receipt(&fixture, Path::new("missing-reviewed-reference.json"), &["src/lib.rs"]);
    assert_eq!(missing.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&missing.stdout).expect("missing reference report");
    assert_eq!(report["status"], "invalid");
    assert!(report["errors"].as_array().unwrap().iter().any(|error| error["kind"] == "read-receipt"));
}

#[test]
fn object_v2_corpus_receipt_detects_drift_and_boundary_tamper_without_rebinding_v1() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let source = "pub fn fetch() -> u32 { let _url = \"https://example.invalid/api\"; 1 }\n";
    fs::write(temp.path().join("lib.rs"), source).expect("write source");
    let receipt = Command::new(cargo_octet())
        .args(["object", "corpus", "receipt", "--output", "receipt.json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("emit candidate receipt");
    assert!(receipt.status.success(), "{}", String::from_utf8_lossy(&receipt.stderr));
    let receipt_path = temp.path().join("receipt.json");
    let saved: serde_json::Value =
        serde_json::from_slice(&fs::read(&receipt_path).expect("read receipt")).expect("parse receipt");
    assert_eq!(saved["boundary"]["claims_correctness"], false);
    assert_eq!(saved["boundary"]["claims_semantic_equivalence"], false);
    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    assert!(check.status.success(), "{}", String::from_utf8_lossy(&check.stderr));
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("parse matching check");
    assert_eq!(report["status"], "match");

    fs::write(temp.path().join("lib.rs"), source.replace("; 1", "; 2")).expect("change code after URL");
    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    assert_eq!(check.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("parse drift check");
    assert_eq!(report["status"], "mismatch");
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons")
            .iter()
            .any(|item| item["field"] == "object_set_hash" && item["status"] == "mismatch")
    );

    fs::write(temp.path().join("lib.rs"), source).expect("restore original source");
    let mut tampered = saved;
    tampered["boundary"]["claims_correctness"] = serde_json::json!(true);
    fs::write(&receipt_path, serde_json::to_vec(&tampered).expect("serialize tampered receipt"))
        .expect("tamper temp receipt");
    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    assert_eq!(check.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("parse boundary check");
    assert_eq!(report["status"], "mismatch");
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons")
            .iter()
            .any(|item| item["field"] == "boundary" && item["status"] == "mismatch")
    );

    let historical = check_corpus_receipt_fixture(&repo_root(), OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1);
    assert_eq!(historical.status.code(), Some(1));
    let report: serde_json::Value = serde_json::from_slice(&historical.stdout).expect("parse historical check");
    assert_eq!(report["status"], "mismatch");
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons")
            .iter()
            .any(|item| item["field"] == "object_set_hash" && item["status"] == "mismatch")
    );
}

#[test]
fn object_supplement_compatibility_marker_and_bounded_families_reject_tamper() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let marked_source = format!(
        "// octet-object: b3:{}; review=R-candidate; evidence=E-candidate\npub fn render(input: u32) -> String {{ format!(\"value={{input}}\") }}\n",
        "0".repeat(64)
    );
    fs::write(temp.path().join("lib.rs"), marked_source).expect("write source");
    let object = object_v2_hash(temp.path());
    assert_eq!(object["dependency_caveats"][0]["status"], "unsupported-macro-context");
    assert_eq!(object["correctness_claim"]["claims_correctness"], false);

    let marker = object_v2_command(temp.path(), &["object", "update", "--output-format=json", "lib.rs"]);
    assert_eq!(marker["records"][0]["computed_hash"], object["object_hash"]);
    let checked = object_v2_command(temp.path(), &["object", "check", "--output-format=json", "lib.rs"]);
    assert_eq!(checked["records"][0]["status"], "match");
    assert_eq!(checked["records"][0]["computed_hash"], object["object_hash"]);

    let supplement = object_v2_command(temp.path(), &[
        "object",
        "supplement",
        "emit",
        "--family",
        "hir",
        "--output-format=json",
        "lib.rs",
    ]);
    assert_eq!(supplement["supplements"][0]["function_object_hash"], object["object_hash"]);
    assert_eq!(supplement["boundary"]["claims_correctness"], false);
    assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);

    let root = repo_root();
    for (family, spike, fixture_path) in [
        ("hir", "rustc-hir-fixture", "cargo-octet/tests/fixtures/hir-spike/checked.rs"),
        ("thir", "rustc-thir-fixture", "cargo-octet/tests/fixtures/thir-spike/checked.rs"),
        ("mir", "rustc-mir-fixture", "cargo-octet/tests/fixtures/mir-spike/checked.rs"),
    ] {
        let fixture_object = object_v2_command(&root, &["object", "emit", "--output-format=json", fixture_path]);
        let fixture_supplement = object_v2_command(&root, &[
            "object",
            "supplement",
            "emit",
            "--family",
            family,
            "--spike",
            spike,
            "--fixture-shape",
            "checked-arithmetic",
            "--output-format=json",
        ]);
        assert_eq!(
            fixture_supplement["supplements"][0]["function_object_hash"], fixture_object["objects"][0]["object_hash"],
            "{family} did not attach to the v2 function identity"
        );
        assert_eq!(fixture_supplement["supplements"][0]["status"], "pilot-supported");
        assert_eq!(fixture_supplement["supplements"][0]["source_path"], fixture_path);
        assert_eq!(fixture_supplement["boundary"]["claims_correctness"], false);

        let receipt_path = temp.path().join(format!("{family}-supplement.json"));
        fs::write(&receipt_path, serde_json::to_vec(&fixture_supplement).expect("serialize supplement"))
            .expect("save temporary supplement receipt");
        let check = || {
            Command::new(cargo_octet())
                .args([
                    "object",
                    "supplement",
                    "check",
                    "--family",
                    family,
                    "--spike",
                    spike,
                    "--fixture-shape",
                    "checked-arithmetic",
                    "--receipt",
                ])
                .arg(&receipt_path)
                .args(["--output-format=json"])
                .current_dir(&root)
                .output()
                .expect("check selected fixture supplement receipt")
        };
        let matching = check();
        assert!(matching.status.success(), "{family}: {}", String::from_utf8_lossy(&matching.stderr));
        let report: serde_json::Value = serde_json::from_slice(&matching.stdout).expect("matching supplement check");
        assert_eq!(report["status"], "pass", "{family}: {report:#?}");

        let mut tampered = fixture_supplement;
        tampered["supplements"][0]["function_object_hash"] = serde_json::json!(blake3_hex(b"wrong object"));
        fs::write(&receipt_path, serde_json::to_vec(&tampered).expect("serialize tamper"))
            .expect("tamper temp receipt");
        let failed = check();
        assert_eq!(failed.status.code(), Some(1), "{family}: {}", String::from_utf8_lossy(&failed.stderr));
        let report: serde_json::Value = serde_json::from_slice(&failed.stdout).expect("tampered supplement check");
        assert_eq!(report["status"], "fail", "{family}: {report:#?}");
        assert!(
            report["comparisons"].as_array().expect("supplement comparisons").iter().any(|comparison| {
                comparison["field"] == "function_object_hash" && comparison["status"] == "mismatch"
            }),
            "{family} accepted a tampered v2 object attachment: {report:#?}"
        );
    }
}

#[test]
fn object_supplement_check_compares_every_pilot_supplement() {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::write(
        temp.path().join("lib.rs"),
        "pub fn first(input: u32) -> u32 { input + 1 }\n\npub fn second(input: u32) -> u32 { input * 2 }\n",
    )
    .expect("write source");
    let emitted = object_v2_command(temp.path(), &[
        "object",
        "supplement",
        "emit",
        "--family",
        "hir",
        "--output-format=json",
        "lib.rs",
    ]);
    let supplements = emitted["supplements"].as_array().expect("pilot supplements");
    assert_eq!(supplements.len(), 2, "pilot scope emits one supplement per function: {emitted:#?}");
    let second_key = serde_json::json!({
        "source_path": supplements[1]["source_path"],
        "function_name": supplements[1]["function_name"],
    });
    assert_eq!(second_key["function_name"], "second");

    let receipt = temp.path().join("hir-supplement.json");
    let check = |value: &serde_json::Value| {
        fs::write(&receipt, serde_json::to_vec(value).expect("serialize receipt")).expect("write receipt");
        let output = Command::new(cargo_octet())
            .args(["object", "supplement", "check", "--family", "hir", "--receipt"])
            .arg(&receipt)
            .args(["--output-format=json", "lib.rs"])
            .current_dir(temp.path())
            .output()
            .expect("check pilot supplement receipt");
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
            panic!("supplement check json ({error}): stderr={}", String::from_utf8_lossy(&output.stderr))
        });
        (output.status.code(), report)
    };

    let (code, report) = check(&emitted);
    assert_eq!(code, Some(0), "{report:#?}");
    assert_eq!(report["status"], "pass");

    let mut reordered = emitted.clone();
    reordered["supplements"].as_array_mut().expect("supplements").reverse();
    let (code, report) = check(&reordered);
    assert_eq!(code, Some(0), "entries are keyed by source path and function: {report:#?}");
    assert_eq!(report["status"], "pass");

    let mut tampered = emitted.clone();
    tampered["supplements"][1]["observation_hash"] = serde_json::json!(blake3_hex(b"tampered second"));
    let (code, report) = check(&tampered);
    assert_eq!(code, Some(1), "{report:#?}");
    assert_eq!(report["status"], "fail");
    assert!(
        report["comparisons"].as_array().expect("comparisons").iter().any(|comparison| {
            comparison["field"] == "observation_hash"
                && comparison["status"] == "mismatch"
                && comparison["supplement"] == second_key
        }),
        "tampered second supplement was not flagged: {report:#?}"
    );

    let mut truncated = emitted;
    truncated["supplements"].as_array_mut().expect("supplements").truncate(1);
    let (code, report) = check(&truncated);
    assert_eq!(code, Some(1), "{report:#?}");
    assert_eq!(report["status"], "fail");
    assert!(
        report["comparisons"].as_array().expect("comparisons").iter().any(|comparison| {
            comparison["field"] == "supplement_count"
                && comparison["status"] == "mismatch"
                && comparison["expected"] == 1
                && comparison["current"] == 2
        }),
        "dropped supplement was not flagged: {report:#?}"
    );
}

fn selected_fixture_supplement(root: &Path, family: &str, shape: &str) -> (String, serde_json::Value) {
    let spike = format!("rustc-{family}-fixture");
    let inventory = object_v2_command(root, &[
        "object",
        "supplement",
        "fixtures",
        "--family",
        family,
        "--spike",
        &spike,
        "--output-format=json",
    ]);
    let fixture = inventory["shapes"]
        .as_array()
        .expect("selected fixture shapes")
        .iter()
        .find(|row| row["shape"] == shape)
        .unwrap_or_else(|| panic!("missing {family} selected {shape} fixture: {inventory:#?}"));
    let path = fixture["fixture_path"].as_str().expect("selected fixture path").to_owned();
    let object = object_v2_command(root, &["object", "emit", "--output-format=json", &path]);
    let supplement = object_v2_command(root, &[
        "object",
        "supplement",
        "emit",
        "--family",
        family,
        "--spike",
        &spike,
        "--fixture-shape",
        shape,
        "--output-format=json",
    ]);
    let name = &supplement["supplements"][0]["function_name"];
    let selected_object = object["objects"]
        .as_array()
        .expect("fixture object records")
        .iter()
        .find(|row| &row["name"] == name)
        .expect("source-backed selected function object");
    assert_eq!(selected_object["normalization"]["algorithm"], "octet-source-token-v2");
    assert_eq!(supplement["supplements"][0]["function_object_hash"], selected_object["object_hash"]);
    assert_eq!(supplement["supplements"][0]["status"], "pilot-supported");
    assert_eq!(supplement["boundary"]["claims_correctness"], false);
    assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);
    (path, supplement)
}

fn replay_selected_supplement(root: &Path, family: &str, shape: &str, receipt: &Path) -> std::process::Output {
    Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "check",
            "--family",
            family,
            "--spike",
            &format!("rustc-{family}-fixture"),
            "--fixture-shape",
            shape,
            "--receipt",
        ])
        .arg(receipt)
        .args(["--output-format=json"])
        .current_dir(root)
        .output()
        .expect("replay selected source-backed supplement")
}

#[test]
fn object_thir_inventory_call_return_replay_rejects_true_boundary() {
    let root = repo_root();
    let (source_path, report) = selected_fixture_supplement(&root, "thir", "call-return");
    assert!(source_path.ends_with("thir-spike/call_return.rs"));
    assert_eq!(report["supplements"][0]["function_name"], "call_then_return");
    let temp = tempfile::tempdir().expect("create tempdir");
    let receipt = temp.path().join("thir-call-return.json");
    fs::write(&receipt, serde_json::to_vec(&report).expect("serialize THIR receipt")).expect("write THIR receipt");
    let matching = replay_selected_supplement(&root, "thir", "call-return", &receipt);
    assert!(matching.status.success(), "THIR replay stderr: {}", String::from_utf8_lossy(&matching.stderr));
    let matched: serde_json::Value = serde_json::from_slice(&matching.stdout).expect("THIR replay json");
    assert_eq!(matched["status"], "pass");

    let mut tampered = report;
    tampered["supplements"][0]["boundary"]["claims_correctness"] = serde_json::json!(true);
    fs::write(&receipt, serde_json::to_vec(&tampered).expect("serialize THIR boundary tamper"))
        .expect("write THIR boundary tamper");
    let failed = replay_selected_supplement(&root, "thir", "call-return", &receipt);
    assert_eq!(failed.status.code(), Some(1));
    let mismatch: serde_json::Value = serde_json::from_slice(&failed.stdout).expect("THIR boundary mismatch json");
    assert_eq!(mismatch["status"], "fail");
    assert!(
        mismatch["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| { row["field"] == "boundary" && row["status"] == "mismatch" })
    );
}

#[test]
fn object_mir_fixture_checked_arithmetic_replay_rejects_observation_tamper() {
    let root = repo_root();
    let (source_path, report) = selected_fixture_supplement(&root, "mir", "checked-arithmetic");
    assert!(source_path.ends_with("mir-spike/checked.rs"));
    let temp = tempfile::tempdir().expect("create tempdir");
    let receipt = temp.path().join("mir-checked.json");
    fs::write(&receipt, serde_json::to_vec(&report).expect("serialize MIR receipt")).expect("write MIR receipt");
    let matching = replay_selected_supplement(&root, "mir", "checked-arithmetic", &receipt);
    assert!(matching.status.success(), "MIR replay stderr: {}", String::from_utf8_lossy(&matching.stderr));
    let matched: serde_json::Value = serde_json::from_slice(&matching.stdout).expect("MIR replay json");
    assert_eq!(matched["status"], "pass");

    let mut tampered = report;
    tampered["supplements"][0]["observation_hash"] = serde_json::json!(blake3_hex(b"wrong-mir-observation"));
    fs::write(&receipt, serde_json::to_vec(&tampered).expect("serialize MIR observation tamper"))
        .expect("write MIR observation tamper");
    let failed = replay_selected_supplement(&root, "mir", "checked-arithmetic", &receipt);
    assert_eq!(failed.status.code(), Some(1));
    let mismatch: serde_json::Value = serde_json::from_slice(&failed.stdout).expect("MIR observation mismatch json");
    assert_eq!(mismatch["status"], "fail");
    assert!(
        mismatch["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| { row["field"] == "observation_hash" && row["status"] == "mismatch" })
    );
}

#[test]
fn object_mir_inventory_branch_source_replay_rejects_wrong_path() {
    let root = repo_root();
    let (source_path, report) = selected_fixture_supplement(&root, "mir", "branch-control-flow");
    assert!(source_path.ends_with("mir-spike/branch_control_flow.rs"));
    assert_eq!(report["supplements"][0]["function_name"], "choose_branch");
    assert_eq!(report["supplements"][0]["source_path"], source_path);
    let temp = tempfile::tempdir().expect("create tempdir");
    let receipt = temp.path().join("mir-branch.json");
    fs::write(&receipt, serde_json::to_vec(&report).expect("serialize MIR branch receipt"))
        .expect("write MIR branch receipt");
    let matching = replay_selected_supplement(&root, "mir", "branch-control-flow", &receipt);
    assert!(matching.status.success(), "MIR branch replay stderr: {}", String::from_utf8_lossy(&matching.stderr));
    let matched: serde_json::Value = serde_json::from_slice(&matching.stdout).expect("MIR branch replay json");
    assert_eq!(matched["status"], "pass");

    let mut tampered = report;
    tampered["supplements"][0]["source_path"] = serde_json::json!("cargo-octet/tests/fixtures/mir-spike/checked.rs");
    fs::write(&receipt, serde_json::to_vec(&tampered).expect("serialize MIR path tamper"))
        .expect("write MIR path tamper");
    let failed = replay_selected_supplement(&root, "mir", "branch-control-flow", &receipt);
    assert_eq!(failed.status.code(), Some(1));
    let mismatch: serde_json::Value = serde_json::from_slice(&failed.stdout).expect("MIR source-path mismatch json");
    assert_eq!(mismatch["status"], "fail");
    assert!(
        mismatch["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|row| { row["field"] == "source_path" && row["status"] == "mismatch" })
    );
}

#[test]
fn object_supplement_receipt_rejects_invalid_shapes() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let valid_shape = serde_json::json!({
        "schema": "octet.function-object-mir-supplement.v1",
        "schema_version": 1,
        "family": "mir",
        "status": "pilot-supported",
        "function_object_hash": blake3_hex(b"checked"),
        "source_path": "cargo-octet/tests/fixtures/mir-spike/checked.rs",
        "function_name": "checked",
        "rustc_version": "rustc fixture",
        "target_triple": "x86_64-unknown-linux-gnu",
        "extractor": {},
        "observations": {},
        "observation_hash": blake3_hex(b"observations"),
        "extraction_caveats": [],
        "boundary": {},
    });
    let check = |name: &str, receipt: &[u8]| {
        let path = temp.path().join(name);
        fs::write(&path, receipt).expect("write supplement receipt");
        Command::new(cargo_octet())
            .args([
                "object",
                "supplement",
                "check",
                "--family",
                "mir",
                "--spike",
                "rustc-mir-fixture",
            ])
            .args([
                "--fixture-shape",
                "checked-arithmetic",
                "--output-format",
                "json",
                "--receipt",
            ])
            .arg(&path)
            .current_dir(&root)
            .output()
            .expect("run supplement check")
    };

    let mut missing = valid_shape.clone();
    missing.as_object_mut().expect("receipt object").remove("function_object_hash");
    let mut future = valid_shape.clone();
    future["schema_version"] = serde_json::json!(99);
    let mut wrong_family = valid_shape.clone();
    wrong_family["family"] = serde_json::json!("thir");
    for (name, receipt, kind, field) in [
        ("malformed.json", b"{ not json".to_vec(), "invalid-json", "receipt"),
        ("missing.json", serde_json::to_vec(&missing).expect("json"), "missing-field", "function_object_hash"),
        (
            "future.json",
            serde_json::to_vec(&future).expect("json"),
            "unsupported-schema-version",
            "schema_version",
        ),
        ("family.json", serde_json::to_vec(&wrong_family).expect("json"), "family-mismatch", "family"),
        ("empty.json", br#"{"supplements":[]}"#.to_vec(), "empty-receipt", "supplements"),
    ] {
        let output = check(name, &receipt);
        assert_eq!(output.status.code(), Some(1), "{name}: {}", String::from_utf8_lossy(&output.stderr));
        let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("supplement report json");
        assert_eq!(report["status"], "invalid", "{name}: {report:#?}");
        assert_supplement_error(&report, kind, field);
    }
}

fn assert_supplement_error(report: &serde_json::Value, expected_kind: &str, expected_field: &str) {
    assert!(
        report["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| { error["kind"] == expected_kind && error["field"] == expected_field }),
        "missing supplement error {expected_kind}/{expected_field}: {report:#?}"
    );
    assert_eq!(report["boundary"]["claims_correctness"], false);
    assert_eq!(report["boundary"]["claims_semantic_equivalence"], false);
}

#[test]
fn object_supplement_fixture_inventory_lists_selected_shapes() {
    let root = repo_root();

    let inventory = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "fixtures",
            "--family",
            "hir",
            "--spike",
            "rustc-hir-fixture",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run HIR fixture inventory json smoke");
    assert!(
        inventory.status.success(),
        "inventory smoke failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("inventory smoke json");
    assert_eq!(json["schema"], "octet.hir-fixture-inventory.v1");
    assert_eq!(json["scope"], "rustc-hir-fixture-spike");
    assert_eq!(json["default_shape"], "checked-arithmetic");
    let shapes = json["shapes"].as_array().expect("fixture inventory shapes array");
    assert_eq!(shapes.len(), 2, "inventory should remain a closed two-shape fixture set: {json:#?}");
    for selector in ["checked-arithmetic", "branch-control-flow"] {
        let fixture = shapes
            .iter()
            .find(|fixture| fixture["shape"] == selector)
            .unwrap_or_else(|| panic!("missing fixture selector {selector}: {json:#?}"));
        assert_eq!(fixture["support_status"], "pilot-supported");
    }
}

#[test]
fn thir_spike_fixture_listing_stays_closed_over_selected_shapes() {
    let root = repo_root();
    let inventory = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "fixtures",
            "--family",
            "thir",
            "--spike",
            "rustc-thir-fixture",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run aggregate THIR fixture inventory source");
    assert!(
        inventory.status.success(),
        "aggregate THIR inventory source failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("aggregate THIR inventory json");
    let shapes = json["shapes"].as_array().expect("aggregate THIR fixture inventory shapes array");
    let listed: BTreeSet<_> =
        shapes.iter().map(|fixture| fixture["shape"].as_str().expect("fixture shape").to_owned()).collect();
    assert_eq!(
        listed,
        BTreeSet::from([
            "branch-control-flow".to_owned(),
            "call-return".to_owned(),
            "checked-arithmetic".to_owned(),
        ]),
        "aggregate THIR inventory gate must stay closed over selected fixture shapes: {json:#?}"
    );
    let branch = shapes
        .iter()
        .find(|fixture| fixture["shape"] == "branch-control-flow")
        .expect("aggregate THIR branch-control-flow fixture entry");
    assert_eq!(branch["function"], "choose_branch");
    assert!(
        branch["fixture_path"]
            .as_str()
            .expect("branch fixture path")
            .ends_with("thir-spike/branch_control_flow.rs"),
        "branch-control-flow must be a THIR-specific selected fixture: {branch:#?}"
    );
    for fixture in shapes {
        assert_eq!(fixture["support_status"], "pilot-supported");
    }

    let contract = object_corpus_contract(&root);
    let supplements = contract["supplemental_evidence"].as_array().expect("supplemental evidence list");
    let thir = supplements
        .iter()
        .find(|value| value["family"] == "thir")
        .unwrap_or_else(|| panic!("missing THIR supplement contract in {contract:#?}"));
    assert_eq!(thir["status"], "unsupported-planned");
    assert_eq!(thir["claims_correctness"], false);
    assert_eq!(thir["claims_semantic_equivalence"], false);
    assert_eq!(thir["claims_cross_toolchain_stability"], false);
}

#[test]
fn mir_spike_fixture_listing_stays_closed_over_selected_shapes() {
    let root = repo_root();
    let inventory = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "fixtures",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run aggregate MIR fixture inventory source");
    assert!(
        inventory.status.success(),
        "aggregate MIR inventory source failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("aggregate MIR inventory json");
    let shapes = json["shapes"].as_array().expect("aggregate MIR fixture inventory shapes array");
    let listed: BTreeSet<_> =
        shapes.iter().map(|fixture| fixture["shape"].as_str().expect("fixture shape").to_owned()).collect();
    assert_eq!(
        listed,
        BTreeSet::from([
            "branch-control-flow".to_owned(),
            "call-return".to_owned(),
            "checked-arithmetic".to_owned()
        ]),
        "aggregate MIR inventory gate must stay closed over selected fixture shapes: {json:#?}"
    );
    for fixture in shapes {
        assert_eq!(fixture["support_status"], "pilot-supported");
    }

    let contract = object_corpus_contract(&root);
    let supplements = contract["supplemental_evidence"].as_array().expect("supplemental evidence list");
    let mir = supplements
        .iter()
        .find(|value| value["family"] == "mir")
        .unwrap_or_else(|| panic!("missing MIR supplement contract in {contract:#?}"));
    assert_eq!(mir["status"], "unsupported-planned");
    assert_eq!(mir["claims_correctness"], false);
    assert_eq!(mir["claims_semantic_equivalence"], false);
    assert_eq!(mir["claims_cross_toolchain_stability"], false);
}
#[test]
fn object_corpus_contract_metadata_reports_receipt_and_check_shapes() {
    let root = repo_root();
    let contract = object_corpus_contract(&root);

    assert_eq!(contract["schema"], "octet.object-corpus-contract-metadata.v1");
    assert_eq!(contract["schema_version"], 1);
    assert_eq!(contract["source_of_truth"], "rust-dto-and-constants");
    assert_eq!(contract["receipt"]["schema"], "octet.function-object-corpus-receipt.v1");
    assert_eq!(contract["receipt"]["schema_version"], 1);
    for field in [
        "schema",
        "schema_version",
        "mode",
        "object_count",
        "source_paths",
        "object_set_hash",
        "caveat_counts",
        "pure_cache_blocked_count",
        "replay",
        "boundary",
    ] {
        assert!(
            contract["receipt"]["required_fields"].as_array().unwrap().iter().any(|value| value == field),
            "missing receipt required field {field} in {contract:#?}"
        );
    }
    assert!(contract["receipt"]["compared_fields"].as_array().unwrap().iter().any(|value| value == "boundary"));
    assert_eq!(contract["check_report"]["schema"], "octet.function-object-corpus-check-report.v1");
    for field in [
        "schema",
        "schema_version",
        "mode",
        "status",
        "receipt_path",
        "comparisons",
        "errors",
    ] {
        assert!(
            contract["check_report"]["required_fields"].as_array().unwrap().iter().any(|value| value == field),
            "missing check-report required field {field} in {contract:#?}"
        );
    }
    for status in ["match", "mismatch", "invalid"] {
        assert!(contract["check_report"]["status_values"].as_array().unwrap().iter().any(|value| value == status));
    }
    assert_eq!(contract["boundaries"]["claims_correctness"], false);
    assert_eq!(contract["boundaries"]["claims_semantic_equivalence"], false);
}

#[test]
fn object_corpus_contract_metadata_reports_pilot_and_planned_supplement_statuses() {
    let root = repo_root();
    let contract = object_corpus_contract(&root);
    let supplements = contract["supplemental_evidence"].as_array().expect("supplemental evidence list");
    assert_eq!(supplements.len(), 3, "expected HIR, THIR, and MIR supplement contracts: {contract:#?}");

    let hir = supplements.iter().find(|value| value["family"] == "hir").expect("missing HIR supplement contract");
    for fixture_shape in ["checked-arithmetic", "branch-control-flow"] {
        assert!(
            hir["selected_fixture_shapes"].as_array().unwrap().iter().any(|value| value == fixture_shape),
            "HIR contract should name selected fixture shape {fixture_shape}: {hir:#?}"
        );
    }

    for (family, status) in [
        ("hir", "pilot-supported"),
        ("thir", "unsupported-planned"),
        ("mir", "unsupported-planned"),
    ] {
        let supplement = supplements
            .iter()
            .find(|value| value["family"] == family)
            .unwrap_or_else(|| panic!("missing {family} supplement contract in {contract:#?}"));
        assert_eq!(supplement["status"], status);
        assert_eq!(supplement["claims_correctness"], false);
        assert_eq!(supplement["claims_semantic_equivalence"], false);
        assert_eq!(supplement["claims_cross_toolchain_stability"], false);
    }
}

#[test]
fn checked_in_internal_corpus_receipt_satisfies_contract_metadata() {
    let root = repo_root();
    let contract = object_corpus_contract(&root);
    let fixture: serde_json::Value = serde_json::from_slice(
        &fs::read(root.join(INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE)).expect("read checked-in receipt fixture"),
    )
    .expect("checked-in receipt fixture json");

    for field in contract["receipt"]["required_fields"].as_array().expect("required fields") {
        let field = field.as_str().expect("required field string");
        assert!(fixture.get(field).is_some(), "fixture missing contract-required field {field}: {fixture:#?}");
    }
    assert_eq!(fixture["schema"], contract["receipt"]["schema"]);
    assert_eq!(fixture["schema_version"], contract["receipt"]["schema_version"]);
    assert_eq!(fixture["boundary"]["identity_scope"], contract["boundaries"]["identity_scope"]);
    assert_eq!(fixture["boundary"]["claims_correctness"], false);
    assert_eq!(fixture["boundary"]["claims_semantic_equivalence"], false);
}

#[test]
fn object_corpus_contract_missing_required_field_fails_closed() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let (receipt_path, mut receipt) = copy_checked_in_receipt(temp.path());
    let contract = object_corpus_contract(temp.path());
    let required = contract["receipt"]["required_fields"]
        .as_array()
        .expect("required fields")
        .iter()
        .find_map(|value| value.as_str())
        .expect("at least one required field");
    receipt.as_object_mut().expect("receipt object").remove(required);
    fs::write(&receipt_path, format!("{}\n", serde_json::to_string_pretty(&receipt).unwrap()))
        .expect("write missing contract field receipt");

    let report =
        assert_invalid_check_error(&check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]), "missing-field");
    assert!(
        report["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| { error["kind"] == "missing-field" && error["field"] == required }),
        "missing contract field {required} was not named in {report:#?}"
    );
}

#[test]
fn checked_in_expanded_corpus_taxonomy_names_selected_source_buckets() {
    let root = repo_root();
    let taxonomy = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE);
    let receipt = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE);
    let taxonomy_paths = taxonomy["source_paths"].as_array().expect("taxonomy source paths");
    let receipt_paths = receipt["source_paths"].as_array().expect("receipt source paths");

    assert_eq!(taxonomy["schema"], "octet.expanded-internal-object-corpus-taxonomy.v1");
    assert_eq!(taxonomy["schema_version"], 1);
    assert_eq!(taxonomy_paths, receipt_paths);
    assert!(taxonomy_paths.len() > INTERNAL_OBJECT_CORPUS.len());
    for path in INTERNAL_OBJECT_CORPUS {
        assert!(taxonomy_paths.iter().any(|value| value == path), "missing core path {path}");
    }
    for bucket in ["cargo-octet", "octet-standards"] {
        assert!(
            taxonomy["buckets"]
                .as_array()
                .expect("taxonomy buckets")
                .iter()
                .any(|entry| entry["name"] == bucket && !entry["source_paths"].as_array().unwrap().is_empty()),
            "missing non-empty bucket {bucket}: {taxonomy:#?}"
        );
    }
    assert!(
        taxonomy_paths
            .iter()
            .any(|value| value.as_str().unwrap().starts_with("crates/octet-standards/src/")),
        "expanded taxonomy must include standards crate sources: {taxonomy:#?}"
    );
}

#[test]
fn checked_in_corpus_caveat_report_names_per_source_blockers() {
    let root = repo_root();
    let report = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE);
    assert_eq!(report["schema"], "octet.function-object-corpus-caveat-taxonomy.v1");
    assert_eq!(report["schema_version"], 1);
    let source_paths = report["source_paths"].as_array().expect("source paths");
    let source_buckets = report["source_buckets"].as_array().expect("source buckets");
    assert_eq!(source_paths.len(), source_buckets.len());
    let mut previous = "";
    for bucket in source_buckets {
        let source_path = bucket["source_path"].as_str().expect("bucket source path");
        assert!(source_path >= previous, "source buckets must be sorted");
        previous = source_path;
        assert!(bucket["object_count"].as_u64().expect("bucket object count") > 0);
        assert!(bucket["caveat_counts"].is_object(), "missing caveat counts for {source_path}");
        assert!(bucket["pure_cache_blocked_count"].as_u64().is_some());
        assert!(bucket["blocker_names"].is_array(), "missing blocker names for {source_path}");
    }
    assert!(
        source_buckets.iter().any(|bucket| {
            bucket["blocker_names"]
                .as_array()
                .unwrap()
                .iter()
                .any(|name| name == "dependency-identity-unresolved")
        }),
        "expected dependency blocker in caveat taxonomy: {report:#?}"
    );
    assert!(
        source_buckets.iter().any(|bucket| {
            bucket["blocker_names"].as_array().unwrap().iter().any(|name| name == "effect-summary-unresolved")
        }),
        "expected effect blocker in caveat taxonomy: {report:#?}"
    );
}

#[test]
fn checked_in_corpus_caveat_report_preserves_non_proof_boundaries() {
    let root = repo_root();
    let report = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE);
    assert_eq!(report["boundary"]["identity_scope"], "review-and-provenance-evidence");
    assert_eq!(report["boundary"]["claims_correctness"], false);
    assert_eq!(report["boundary"]["claims_semantic_equivalence"], false);
    assert!(
        report["boundary"]["detail"]
            .as_str()
            .expect("boundary detail")
            .contains("do not prove behavioral correctness")
    );
    assert!(
        report["pure_cache_blocked_count"].as_u64().expect("pure cache blocked") > 0,
        "taxonomy should explain blockers, not promote pure-cache eligibility"
    );
}

#[test]
fn object_corpus_compatibility_historical_v1_is_stale_under_v2() {
    let root = repo_root();
    let compatibility_dir = root.join(OBJECT_CORPUS_COMPATIBILITY_DIR);
    let fixture: serde_json::Value = serde_json::from_slice(
        &fs::read(compatibility_dir.join(OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1))
            .expect("read positive compatibility fixture"),
    )
    .expect("positive compatibility fixture json");

    assert_eq!(fixture["schema"], "octet.function-object-corpus-receipt.v1");
    assert_eq!(fixture["schema_version"], 1);
    assert_eq!(fixture["source_paths"], serde_json::json!(["src/lib.rs"]));
    assert_eq!(fixture["boundary"]["claims_correctness"], false);
    assert_eq!(fixture["boundary"]["claims_semantic_equivalence"], false);

    let check = check_corpus_receipt_fixture(&root, OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1);
    assert!(
        !check.status.success(),
        "historical v1 receipt unexpectedly matched current v2 source stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("historical check report json");
    assert_eq!(report["status"], "mismatch");
    assert!(report["errors"].as_array().expect("errors array").is_empty());
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons array")
            .iter()
            .any(|comparison| comparison["field"] == "object_set_hash" && comparison["status"] == "mismatch"),
        "historical v1 object set must differ from current v2 objects: {report:#?}"
    );
}

#[test]
// One pair of test-owned v1 objects flows through diff, pointer, and bundle commands in sequence.
#[allow(clippy::too_many_lines)]
fn object_corpus_compatibility_preserves_v1_artifact_read() {
    use valence_core::function_object::DependencyIdentityStatus;
    use valence_core::function_object::FunctionObjectInput;
    use valence_core::function_object::NORMALIZATION_ALGORITHM;
    use valence_core::function_object::NormalizedFunctionSource;
    use valence_core::function_object::SourceSpan;
    use valence_core::function_object::build_function_object;

    let root = repo_root();
    let receipt = read_json_fixture(
        &root,
        &format!("{OBJECT_CORPUS_COMPATIBILITY_DIR}/{OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1}"),
    );
    assert_eq!(receipt["schema"], "octet.function-object-corpus-receipt.v1");
    assert_eq!(receipt["schema_version"], 1);
    let historical_hash = receipt["object_set_hash"].as_str().expect("historical receipt object-set hash");
    assert_eq!(historical_hash, "b3:7f97a686c4c6697a58dd6d2754c7aa9cb4c2a2d81653814028fedf39a5b33883");

    // No standalone historical v1 object JSON is archived here. Construct
    // test-owned, genuine v1 objects with the public core API, independently
    // pinning the first identity to the golden captured from the v1 parent.
    // Neither object belongs to the saved historical corpus receipt.
    let temp = tempfile::tempdir().expect("create tempdir");
    let old_path = temp.path().join("old-v1-shape.json");
    let new_path = temp.path().join("new-v1-shape.json");
    let objects_path = temp.path().join("v1-shape-objects.json");
    let construct = |source: &str| {
        build_function_object(FunctionObjectInput {
            tool_name: "cargo-octet",
            tool_version: "0.1.0",
            name: "answer",
            source_path: "src/lib.rs".to_owned(),
            source_span: SourceSpan {
                start_line: 1,
                end_line: 1,
            },
            function_source: source,
            normalized_function_source: NormalizedFunctionSource::new(NORMALIZATION_ALGORITHM, source.to_owned())
                .expect("supported pinned v1 algorithm"),
            dependency_set_hash: blake3_hex(b"answer"),
            dependency_identity_status: DependencyIdentityStatus::SourceScannerTokenSet,
            macro_calls: false,
        })
        .expect("construct typed v1 object")
    };
    let old = construct("pub fn answer() -> u32 { 42 }");
    let new = construct("pub fn answer() -> u32 { 43 }");
    let old_hash = old.object_hash.clone();
    let new_hash = new.object_hash.clone();
    assert_eq!(old_hash, "b3:da550fa4897c0b6b3ba9adb819eb1c1bedf5f5b6ad6c1c4498d9afef0719c9b4");
    assert_ne!(new_hash, old_hash);
    let old_object = serde_json::to_value(&old).expect("serialize typed pinned-v1 object");
    let changed_object = serde_json::to_value(&new).expect("serialize changed typed pinned-v1 object");
    assert_eq!(old_object["schema"], "octet.function-object.v1");
    assert_eq!(old_object["normalization"]["algorithm"], NORMALIZATION_ALGORITHM);
    assert_eq!(old_object["dependency_caveats"][0]["status"], "source-scanner-token-set");
    assert_eq!(old_object["correctness_claim"]["claims_correctness"], false);
    fs::write(&old_path, serde_json::to_vec(&old_object).expect("encode test-owned object"))
        .expect("write test-owned object");
    fs::write(&new_path, serde_json::to_vec(&changed_object).expect("encode changed test-owned object"))
        .expect("write changed test-owned object");
    fs::write(
        &objects_path,
        serde_json::to_vec(&serde_json::json!({
            "schema_version": 1,
            "mode": "emit",
            "objects": [&old_object]
        }))
        .expect("encode test-owned object report"),
    )
    .expect("write test-owned object report");

    let diff = Command::new(cargo_octet())
        .args(["object", "diff", "--old"])
        .arg(&old_path)
        .args(["--new"])
        .arg(&new_path)
        .args(["--output-format", "json"])
        .output()
        .expect("diff artifact-only v1-shape object");
    assert!(diff.status.success(), "diff stderr: {}", String::from_utf8_lossy(&diff.stderr));
    let comparison: serde_json::Value = serde_json::from_slice(&diff.stdout).expect("diff json");
    assert_eq!(comparison["old_object_hash"], old_hash);
    assert_eq!(comparison["new_object_hash"], new_hash);
    assert_eq!(comparison["body_changed"], true);
    assert_eq!(comparison["signature_changed"], false);

    let manifest = temp.path().join("pointers.json");
    let pointer_receipt = temp.path().join("pointer-receipt.json");
    let set = Command::new(cargo_octet())
        .args(["object", "pointer", "set", "--manifest"])
        .arg(&manifest)
        .args(["--name", "crate::answer", "--hash", &old_hash, "--objects"])
        .arg(&objects_path)
        .args(["--receipt"])
        .arg(&pointer_receipt)
        .output()
        .expect("set artifact-only pointer");
    assert!(set.status.success(), "pointer stderr: {}", String::from_utf8_lossy(&set.stderr));
    let validate = Command::new(cargo_octet())
        .args(["object", "pointer", "validate", "--manifest"])
        .arg(&manifest)
        .args(["--objects"])
        .arg(&objects_path)
        .output()
        .expect("validate artifact-only pointer");
    assert!(
        validate.status.success(),
        "pointer validation stderr: {}",
        String::from_utf8_lossy(&validate.stderr)
    );
    let validation: serde_json::Value = serde_json::from_slice(&validate.stdout).expect("validation json");
    assert_eq!(validation["status"], "resolved");

    let bundle_path = temp.path().join("bundle.json");
    let bundle = Command::new(cargo_octet())
        .args(["object", "bundle", "export", "--objects"])
        .arg(&objects_path)
        .args(["--pointers"])
        .arg(&manifest)
        .args(["--output"])
        .arg(&bundle_path)
        .args(["--replay-command", "test-owned typed v1 artifact; no source replay"])
        .output()
        .expect("export artifact-only bundle");
    assert!(bundle.status.success(), "bundle stderr: {}", String::from_utf8_lossy(&bundle.stderr));
    let exported: serde_json::Value =
        serde_json::from_slice(&fs::read(bundle_path).expect("read bundle")).expect("bundle json");
    assert_eq!(exported["objects"][0], old_object);
    assert_eq!(exported["object_count"], 1);
    assert_eq!(exported["pointer_count"], 1);
}

#[test]
fn object_corpus_compatibility_negative_fixtures_fail_closed() {
    let root = repo_root();
    for (fixture, expected_kind, expected_field) in [
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MISSING_OBJECT_SET_HASH,
            "missing-field",
            Some("object_set_hash"),
        ),
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_FUTURE_SCHEMA_VERSION,
            "unsupported-schema-version",
            Some("schema_version"),
        ),
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_DUPLICATE_SOURCE_PATH,
            "duplicate-source-path",
            Some("source_paths"),
        ),
        (
            OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_EMPTY_SOURCE_SELECTION,
            "empty-source-selection",
            Some("source_paths"),
        ),
        (OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MALFORMED_JSON, "invalid-json", Some("receipt")),
    ] {
        let report = assert_invalid_check_error(&check_corpus_receipt_fixture(&root, fixture), expected_kind);
        if let Some(expected_field) = expected_field {
            assert!(
                report["errors"]
                    .as_array()
                    .expect("errors array")
                    .iter()
                    .any(|error| error["field"] == expected_field),
                "negative fixture {fixture} did not name expected field {expected_field}: {report:#?}"
            );
        }
    }
}

#[test]
fn object_corpus_compatibility_boundary_tamper_fixture_fails_closed() {
    let root = repo_root();
    let check = check_corpus_receipt_fixture(&root, OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_BOUNDARY_TAMPER);
    assert!(
        !check.status.success(),
        "boundary tamper compatibility fixture unexpectedly passed stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("boundary tamper report json");
    assert_eq!(report["status"], "mismatch", "report={report:#?}");
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons array")
            .iter()
            .any(|comparison| comparison["field"] == "boundary" && comparison["status"] == "mismatch"),
        "expected boundary mismatch for tamper fixture: {report:#?}"
    );
    assert!(report["errors"].as_array().expect("errors array").is_empty());
}

#[test]
fn object_corpus_compatibility_source_selection_fixtures_fail_closed() {
    let root = repo_root();
    let compatibility_dir = root.join(OBJECT_CORPUS_COMPATIBILITY_DIR);
    let non_rust_report = assert_invalid_check_error(
        &check_corpus_receipt(&compatibility_dir, Path::new(OBJECT_CORPUS_COMPATIBILITY_POSITIVE_V1), &[
            OBJECT_CORPUS_COMPATIBILITY_NON_RUST_SOURCE,
        ]),
        "unsupported-source-path",
    );
    assert!(
        non_rust_report["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["field"] == "paths"),
        "non-Rust selection should name paths field: {non_rust_report:#?}"
    );
}

fn assert_invalid_check_error(output: &std::process::Output, expected_kind: &str) -> serde_json::Value {
    assert!(
        !output.status.success(),
        "invalid check unexpectedly passed stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&output.stdout).expect("invalid check report json");
    assert_eq!(report["status"], "invalid", "report={report:#?}");
    assert!(
        report["errors"]
            .as_array()
            .expect("errors array")
            .iter()
            .any(|error| error["kind"] == expected_kind),
        "expected error kind {expected_kind} in {report:#?}"
    );
    assert!(report["comparisons"].as_array().expect("comparisons array").is_empty());
    report
}

#[test]
fn object_corpus_receipt_check_reports_malformed_receipts() {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    let receipt_path = temp.path().join("bad.json");
    fs::write(&receipt_path, "{not json\n").expect("write malformed receipt");

    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    let report = assert_invalid_check_error(&check, "invalid-json");
    assert!(
        report["errors"][0]["message"].as_str().unwrap().contains("parse"),
        "malformed receipt should explain parse failure: {report:#?}"
    );
}

#[test]
fn object_corpus_receipt_check_rejects_unsupported_schema_and_future_version() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let (receipt_path, mut receipt) = copy_checked_in_receipt(temp.path());

    receipt["schema"] = serde_json::Value::String("octet.future-corpus-receipt.v9".to_owned());
    fs::write(&receipt_path, format!("{}\n", serde_json::to_string_pretty(&receipt).unwrap()))
        .expect("write wrong schema receipt");
    assert_invalid_check_error(&check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]), "unsupported-schema");

    receipt["schema"] = serde_json::Value::String("octet.function-object-corpus-receipt.v1".to_owned());
    receipt["schema_version"] = serde_json::Value::Number(serde_json::Number::from(99));
    fs::write(&receipt_path, format!("{}\n", serde_json::to_string_pretty(&receipt).unwrap()))
        .expect("write future version receipt");
    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]),
        "unsupported-schema-version",
    );
}

#[test]
fn object_corpus_receipt_check_rejects_invalid_source_selection() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let (receipt_path, _) = copy_checked_in_receipt(temp.path());
    fs::write(temp.path().join("README.md"), "not rust\n").expect("write non-rust fixture");
    fs::write(temp.path().join("empty.rs"), "// no functions here\n").expect("write empty rust fixture");

    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["missing.rs"]),
        "source-path-missing",
    );
    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs", "lib.rs"]),
        "duplicate-source-path",
    );
    assert_invalid_check_error(
        &check_corpus_receipt(temp.path(), &receipt_path, &["README.md"]),
        "unsupported-source-path",
    );
    assert_invalid_check_error(&check_corpus_receipt(temp.path(), &receipt_path, &["empty.rs"]), "empty-corpus");
}

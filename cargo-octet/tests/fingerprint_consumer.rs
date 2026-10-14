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
        r"#[rustfmt::skip]
// tigerstyle-fingerprint: b3:0; review=safety-critical; evidence=docs/examples/function-fingerprint-lifecycle/lifecycle-transcript.md; macro=allow
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

#[test]
fn release_operator_smoke_transcript_and_help_cover_operator_surfaces() {
    let root = repo_root();
    let transcript_path = root.join("docs/examples/release-operator-smoke/transcript.md");
    let transcript = fs::read_to_string(&transcript_path).expect("read release smoke transcript");

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

    for surface in [
        "artifact verify",
        "fingerprint",
        "migrate",
        "effect-summary",
        "lint-cache",
        "transcript",
        "roadmap",
    ] {
        assert!(transcript.contains(surface), "release smoke transcript missing {surface}");
    }

    let help_output = Command::new(cargo_octet()).arg("--help").output().expect("run root help");
    assert!(help_output.status.success(), "root help should succeed");
    let help_stdout = String::from_utf8_lossy(&help_output.stdout);
    for surface in [
        "artifact verify",
        "fingerprint",
        "migrate",
        "effect-summary",
        "lint-cache",
        "transcript",
        "roadmap",
    ] {
        assert!(help_stdout.contains(surface), "root help missing {surface}");
    }
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
        "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
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
        "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
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
        "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
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
        "d849f27312de653868c870135a9c9ef4c006512d325cfb3e5c726f0492c4d03f",
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
        "9edc0f2555bf5f15be3d7b09806228695671427c8f8720f466af4280b4bd1130",
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
fn fingerprint_write_then_check_works_for_external_crate_file() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let fixture = temp.path().join("src/lib.rs");

    let write = Command::new(cargo_octet())
        .args(["fingerprint", "--write"])
        .arg(&fixture)
        .output()
        .expect("run fingerprint write");
    assert!(write.status.success(), "write stderr: {}", String::from_utf8_lossy(&write.stderr));

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
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
        .output()
        .expect("run fingerprint update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));

    let check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(&fixture)
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
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
}

#[test]
fn fingerprint_update_check_and_diff_walk_directory_tree() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_directory_fixture(temp.path());

    let update = Command::new(cargo_octet())
        .args(["fingerprint", "--update", "--output-format=json"])
        .arg(temp.path())
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
        .output()
        .expect("run fingerprint check");
    assert!(check.status.success(), "check stderr: {}", String::from_utf8_lossy(&check.stderr));
    assert_directory_records(&check.stdout, &expected, "match");

    let diff = Command::new(cargo_octet())
        .args(["fingerprint", "--diff", "--output-format=json"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint diff");
    assert!(diff.status.success(), "diff stderr: {}", String::from_utf8_lossy(&diff.stderr));
    assert_directory_records(&diff.stdout, &expected, "match");
}

#[test]
fn object_emit_outputs_function_object_v1_json() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_directory_fixture(temp.path());

    let output = Command::new(cargo_octet())
        .args(["object", "emit", "--function", "root_plain", "--output-format=json"])
        .arg(temp.path().join("src/lib.rs"))
        .output()
        .expect("run object emit");
    assert!(output.status.success(), "object stderr: {}", String::from_utf8_lossy(&output.stderr));

    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    let objects = json["objects"].as_array().expect("objects array");
    assert_eq!(objects.len(), 1);
    let object = &objects[0];
    assert_eq!(object["schema"], "octet.function-object.v1");
    assert_eq!(object["name"], "root_plain");
    assert!(object["object_hash"].as_str().expect("object hash").starts_with("b3:"));
    assert!(object["body_hash"].as_str().expect("body hash").starts_with("b3:"));
    assert_eq!(object["correctness_claim"]["claims_correctness"], false);
    assert_eq!(object["dependency_caveats"][0]["status"], "source-scanner-token-set");
}

#[test]
fn object_emit_hash_stable_across_roots() {
    let one = tempfile::tempdir().expect("create first tempdir");
    let two = tempfile::tempdir().expect("create second tempdir");
    write_directory_fixture(one.path());
    write_directory_fixture(two.path());

    let one_hash = emit_root_plain_hash(one.path());
    let two_hash = emit_root_plain_hash(two.path());
    assert_eq!(one_hash, two_hash);
}

fn emit_root_plain_hash(root: &Path) -> String {
    let output = Command::new(cargo_octet())
        .current_dir(root)
        .args([
            "object",
            "emit",
            "--function",
            "root_plain",
            "--output-format=json",
            "src/lib.rs",
        ])
        .output()
        .expect("run object emit");
    assert!(output.status.success(), "object stderr: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.contains(root.to_str().expect("utf8 temp path")));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    json["objects"][0]["object_hash"].as_str().expect("object hash").to_owned()
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
        .output()
        .expect("run fingerprint check");
    assert!(!check.status.success(), "drift check should fail");

    let human_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check"])
        .arg(temp.path())
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
        .output()
        .expect("run fingerprint drift diff");
    assert!(!diff.status.success(), "drift diff should fail");

    let diff_json: serde_json::Value = serde_json::from_slice(&diff.stdout).expect("diff json output");
    let diff_records = diff_json["records"].as_array().expect("diff records array");
    assert_single_directory_mismatch(diff_records);

    let repair = Command::new(cargo_octet())
        .args(["fingerprint", "--update"])
        .arg(temp.path())
        .output()
        .expect("run fingerprint repair update");
    assert!(repair.status.success(), "repair stderr: {}", String::from_utf8_lossy(&repair.stderr));
    assert_human_summary(&repair.stdout, "match=5 mismatch=0 missing=0 written=1");
    assert_human_contains_record(&repair.stdout, "root_plain", "written");

    let repaired_check = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
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
fn object_marker_check_update_and_drift_detection() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(
        &fixture,
        r"// octet-object: b3:0000; review=phase3; evidence=evidence/object-marker
pub fn checked_value(input: u32) -> u32 {
    input + 1
}
",
    )
    .expect("write object marker fixture");

    let initial = Command::new(cargo_octet())
        .args(["object", "check", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run initial object check");
    assert!(!initial.status.success(), "initial check unexpectedly passed");
    let initial_json: serde_json::Value = serde_json::from_slice(&initial.stdout).expect("initial object check json");
    assert_eq!(initial_json["records"][0]["status"], "mismatch");

    let update = Command::new(cargo_octet())
        .args(["object", "update", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run object marker update");
    assert!(update.status.success(), "update stderr: {}", String::from_utf8_lossy(&update.stderr));
    let update_json: serde_json::Value = serde_json::from_slice(&update.stdout).expect("update json");
    assert_eq!(update_json["records"][0]["status"], "written");

    let clean = Command::new(cargo_octet())
        .args(["object", "check", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run clean object check");
    assert!(clean.status.success(), "clean check stderr: {}", String::from_utf8_lossy(&clean.stderr));
    let clean_json: serde_json::Value = serde_json::from_slice(&clean.stdout).expect("clean json");
    assert_eq!(clean_json["records"][0]["status"], "match");

    let source = fs::read_to_string(&fixture).expect("read fixture");
    fs::write(&fixture, source.replace("input + 1", "input + 2")).expect("mutate fixture");
    let drift = Command::new(cargo_octet())
        .args(["object", "check", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("run drift object check");
    assert!(!drift.status.success(), "drift check unexpectedly passed");
    let drift_json: serde_json::Value = serde_json::from_slice(&drift.stdout).expect("drift json");
    assert_eq!(drift_json["records"][0]["status"], "mismatch");
}

#[test]
fn object_support_does_not_require_fingerprint_markers() {
    let temp = tempfile::tempdir().expect("create tempdir");
    write_consumer_fixture(temp.path());
    let fingerprint = Command::new(cargo_octet())
        .args(["fingerprint", "--check", "--output-format=json"])
        .arg(temp.path())
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
    let fixture = temp.path().join("lib.rs");
    fs::write(&fixture, "pub fn checked_value(input: u32) -> u32 { input + 1 }\n").expect("write old fixture");
    let old_objects = temp.path().join("old-objects.json");
    let old_output = Command::new(cargo_octet())
        .args([
            "object",
            "emit",
            "--function",
            "checked_value",
            "--output-format=json",
            "lib.rs",
        ])
        .current_dir(temp.path())
        .output()
        .expect("emit old object");
    assert!(old_output.status.success(), "old emit stderr: {}", String::from_utf8_lossy(&old_output.stderr));
    fs::write(&old_objects, &old_output.stdout).expect("write old objects");
    let old_json: serde_json::Value = serde_json::from_slice(&old_output.stdout).expect("old object json");
    let old_hash = old_json["objects"][0]["object_hash"].as_str().unwrap().to_owned();

    fs::write(&fixture, "pub fn checked_value(input: u32) -> u32 { input + 2 }\n").expect("write new fixture");
    let new_objects = temp.path().join("new-objects.json");
    let new_output = Command::new(cargo_octet())
        .args([
            "object",
            "emit",
            "--function",
            "checked_value",
            "--output-format=json",
            "lib.rs",
        ])
        .current_dir(temp.path())
        .output()
        .expect("emit new object");
    assert!(new_output.status.success(), "new emit stderr: {}", String::from_utf8_lossy(&new_output.stderr));
    fs::write(&new_objects, &new_output.stdout).expect("write new objects");
    let new_json: serde_json::Value = serde_json::from_slice(&new_output.stdout).expect("new object json");
    let new_hash = new_json["objects"][0]["object_hash"].as_str().unwrap().to_owned();
    assert_ne!(old_hash, new_hash);

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
fn object_caveats_keep_dependencies_unresolved_and_block_pure_cache() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(
        &fixture,
        r"pub fn checked_value(input: u32) -> u32 {
    input.saturating_add(1)
}
",
    )
    .expect("write fixture");

    let output = Command::new(cargo_octet())
        .args([
            "object",
            "emit",
            "--function",
            "checked_value",
            "--output-format=json",
            "lib.rs",
        ])
        .current_dir(temp.path())
        .output()
        .expect("emit object");
    assert!(output.status.success(), "emit stderr: {}", String::from_utf8_lossy(&output.stderr));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("object json");
    let object = &json["objects"][0];
    assert_eq!(object["dependency_caveats"][0]["status"], "source-scanner-token-set");
    assert_eq!(object["caveat_summary"]["dependency_identity"], "unresolved-source-scanner");
    assert_eq!(object["pure_cache_eligibility"]["eligible"], false);
    assert_eq!(object["pure_cache_eligibility"]["status"], "blocked-by-caveats");
    let blockers = object["pure_cache_eligibility"]["blockers"].as_array().expect("blockers");
    assert!(blockers.iter().any(|blocker| blocker == "dependency-identity-unresolved"));
    assert!(blockers.iter().any(|blocker| blocker == "effect-summary-unresolved"));
}

#[test]
fn object_effect_and_macro_caveats_are_structured() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(
        &fixture,
        r#"pub fn checked_value(input: Option<u32>) -> u32 {
    format!("value={}", input.unwrap()).len() as u32
}
"#,
    )
    .expect("write fixture");

    let output = Command::new(cargo_octet())
        .args([
            "object",
            "emit",
            "--function",
            "checked_value",
            "--output-format=json",
            "lib.rs",
        ])
        .current_dir(temp.path())
        .output()
        .expect("emit object");
    assert!(output.status.success(), "emit stderr: {}", String::from_utf8_lossy(&output.stderr));
    let json: serde_json::Value = serde_json::from_slice(&output.stdout).expect("object json");
    let object = &json["objects"][0];
    assert!(object["effect_caveats"].as_array().unwrap().iter().any(|caveat| caveat["kind"] == "panic"));
    assert_eq!(object["macro_caveats"][0]["status"], "review-required");
    assert_eq!(object["caveat_summary"]["macro_expansion"], "macro-expansion-not-modeled");
    let blockers = object["pure_cache_eligibility"]["blockers"].as_array().expect("blockers");
    assert!(blockers.iter().any(|blocker| blocker == "macro-expansion-unmodeled"));
}

#[test]
fn object_bundle_export_is_deterministic_and_records_replay() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(
        &fixture,
        r"pub fn beta(input: u32) -> u32 { input + 2 }
pub fn alpha(input: u32) -> u32 { input + 1 }
",
    )
    .expect("write fixture");
    let objects_path = temp.path().join("objects.json");
    let objects = Command::new(cargo_octet())
        .args(["object", "emit", "--output-format=json", "lib.rs"])
        .current_dir(temp.path())
        .output()
        .expect("emit objects");
    assert!(objects.status.success(), "emit stderr: {}", String::from_utf8_lossy(&objects.stderr));
    fs::write(&objects_path, &objects.stdout).expect("write objects");
    let objects_json: serde_json::Value = serde_json::from_slice(&objects.stdout).expect("objects json");
    let alpha_hash = objects_json["objects"]
        .as_array()
        .unwrap()
        .iter()
        .find(|object| object["name"] == "alpha")
        .and_then(|object| object["object_hash"].as_str())
        .unwrap()
        .to_owned();
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
}

const REAL_CODE_CORPUS: &[&str] = &[
    "cargo-octet/src/cli.rs",
    "cargo-octet/src/object.rs",
    "cargo-octet/src/fingerprint.rs",
];

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
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MISSING_OBJECT_SET_HASH: &str = "negative-missing-object-set-hash.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_FUTURE_SCHEMA_VERSION: &str = "negative-future-schema-version.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_BOUNDARY_TAMPER: &str = "negative-boundary-tamper.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_DUPLICATE_SOURCE_PATH: &str = "negative-duplicate-source-path.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_EMPTY_SOURCE_SELECTION: &str = "negative-empty-source-selection.json";
const OBJECT_CORPUS_COMPATIBILITY_NEGATIVE_MALFORMED_JSON: &str = "negative-malformed-json.json";
const OBJECT_CORPUS_COMPATIBILITY_NON_RUST_SOURCE: &str = "notes.txt";

const REAL_CODE_REPLAY_COMMAND: &str = "cargo octet object emit --output-format=json cargo-octet/src/cli.rs cargo-octet/src/object.rs cargo-octet/src/fingerprint.rs";

#[derive(Debug)]
struct CorpusSummary {
    object_count: usize,
    source_paths: BTreeSet<String>,
    macro_review_required: usize,
    panic_caveats: usize,
    pure_cache_blocked: usize,
}

fn emit_objects_json(root: &Path, paths: &[&str]) -> serde_json::Value {
    let output = Command::new(cargo_octet())
        .args(["object", "emit", "--output-format=json"])
        .args(paths)
        .current_dir(root)
        .output()
        .expect("emit objects");
    assert!(output.status.success(), "emit stderr: {}", String::from_utf8_lossy(&output.stderr));
    serde_json::from_slice(&output.stdout).expect("object json")
}

fn emit_real_code_corpus(root: &Path) -> serde_json::Value {
    emit_objects_json(root, REAL_CODE_CORPUS)
}

fn corpus_summary(json: &serde_json::Value) -> CorpusSummary {
    let objects = json["objects"].as_array().expect("objects array");
    CorpusSummary {
        object_count: objects.len(),
        source_paths: objects
            .iter()
            .filter_map(|object| object["source_path"].as_str().map(ToOwned::to_owned))
            .collect(),
        macro_review_required: objects
            .iter()
            .filter(|object| {
                object["macro_caveats"]
                    .as_array()
                    .expect("macro caveats")
                    .iter()
                    .any(|caveat| caveat["status"] == "review-required")
            })
            .count(),
        panic_caveats: objects
            .iter()
            .filter(|object| {
                object["effect_caveats"]
                    .as_array()
                    .expect("effect caveats")
                    .iter()
                    .any(|caveat| caveat["kind"] == "panic")
            })
            .count(),
        pure_cache_blocked: objects
            .iter()
            .filter(|object| object["pure_cache_eligibility"]["status"] == "blocked-by-caveats")
            .count(),
    }
}

fn object_hash(json: &serde_json::Value, name: &str) -> String {
    json["objects"]
        .as_array()
        .expect("objects array")
        .iter()
        .find(|object| object["name"] == name)
        .and_then(|object| object["object_hash"].as_str())
        .unwrap_or_else(|| panic!("missing object hash for {name}"))
        .to_owned()
}

fn write_objects(root: &Path, paths: &[&str], objects_path: &Path) -> serde_json::Value {
    let output = Command::new(cargo_octet())
        .args(["object", "emit", "--output-format=json"])
        .args(paths)
        .current_dir(root)
        .output()
        .expect("emit objects for file");
    assert!(output.status.success(), "emit stderr: {}", String::from_utf8_lossy(&output.stderr));
    fs::write(objects_path, &output.stdout).expect("write objects json");
    serde_json::from_slice(&output.stdout).expect("objects json")
}

fn write_corpus_receipt(root: &Path, paths: &[&str], output_path: &Path) -> serde_json::Value {
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "receipt", "--output"])
        .arg(output_path)
        .args(paths)
        .current_dir(root)
        .output()
        .expect("write object corpus receipt");
    assert!(
        output.status.success(),
        "corpus receipt stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&fs::read(output_path).expect("read corpus receipt")).expect("corpus receipt json")
}

fn write_corpus_receipt_with_fresh_target(
    root: &Path,
    paths: &[&str],
    output_path: &Path,
    target_dir: &Path,
) -> serde_json::Value {
    let output = Command::new(cargo_octet())
        .env_clear()
        .env("TZ", "UTC")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("CARGO_TARGET_DIR", target_dir)
        .args(["object", "corpus", "receipt", "--output"])
        .arg(output_path)
        .args(paths)
        .current_dir(root)
        .output()
        .expect("write object corpus receipt with fresh target");
    assert!(
        output.status.success(),
        "fresh-target corpus receipt stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&fs::read(output_path).expect("read fresh-target corpus receipt"))
        .expect("fresh-target corpus receipt json")
}

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

fn expanded_internal_object_corpus_paths(taxonomy: &serde_json::Value) -> Vec<&str> {
    taxonomy["source_paths"]
        .as_array()
        .expect("taxonomy source paths")
        .iter()
        .map(|value| value.as_str().expect("taxonomy source path string"))
        .collect()
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

fn object_corpus_caveat_taxonomy(root: &Path, paths: &[&str]) -> Vec<u8> {
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "caveats", "--output-format", "json"])
        .args(paths)
        .current_dir(root)
        .output()
        .expect("object corpus caveat taxonomy");
    assert!(
        output.status.success(),
        "caveat taxonomy stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output.stdout
}

fn export_bundle(
    root: &Path,
    objects_path: &Path,
    pointers_path: Option<&Path>,
    output_path: &Path,
) -> std::process::Output {
    let mut command = Command::new(cargo_octet());
    command.args(["object", "bundle", "export", "--objects"]).arg(objects_path);
    if let Some(pointers_path) = pointers_path {
        command.args(["--pointers"]).arg(pointers_path);
    }
    command
        .args(["--output"])
        .arg(output_path)
        .args(["--replay-command", REAL_CODE_REPLAY_COMMAND])
        .current_dir(root)
        .output()
        .expect("export bundle")
}

fn blake3_hex(bytes: &[u8]) -> String {
    format!("b3:{}", blake3::hash(bytes).to_hex())
}

fn emit_objects_bytes_with_env(root: &Path, paths: &[&str], target_dir: &Path) -> Vec<u8> {
    let output = Command::new(cargo_octet())
        .env_clear()
        .env("TZ", "UTC")
        .env("LC_ALL", "C")
        .env("LANG", "C")
        .env("CARGO_TARGET_DIR", target_dir)
        .args(["object", "emit", "--output-format=json"])
        .args(paths)
        .current_dir(root)
        .output()
        .expect("emit objects with scrubbed env");
    assert!(output.status.success(), "scrubbed emit stderr: {}", String::from_utf8_lossy(&output.stderr));
    output.stdout
}

fn object_hash_set(json: &serde_json::Value) -> BTreeSet<String> {
    json["objects"]
        .as_array()
        .expect("objects array")
        .iter()
        .map(|object| object["object_hash"].as_str().expect("object hash").to_owned())
        .collect()
}

fn copy_real_code_corpus(source_root: &Path, target_root: &Path) {
    for relative in REAL_CODE_CORPUS {
        let source = source_root.join(relative);
        let target = target_root.join(relative);
        fs::create_dir_all(target.parent().expect("corpus file parent")).expect("create corpus parent");
        fs::copy(&source, &target)
            .unwrap_or_else(|error| panic!("copy {} to {}: {error}", source.display(), target.display()));
    }
}

fn command_stdout(root: &Path, program: &str, args: &[&str]) -> String {
    let output = Command::new(program)
        .args(args)
        .current_dir(root)
        .output()
        .unwrap_or_else(|error| panic!("run {program} {args:?}: {error}"));
    assert!(output.status.success(), "{program} stderr: {}", String::from_utf8_lossy(&output.stderr));
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

#[test]
fn object_strong_local_determinism_receipt_checks_repeatability() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let binary_bytes = fs::read(cargo_octet()).expect("read cargo-octet binary");
    println!("receipt.git_head={}", command_stdout(&root, "git", &["rev-parse", "HEAD"]));
    println!("receipt.binary_path={}", cargo_octet());
    println!("receipt.binary_b3={}", blake3_hex(&binary_bytes));
    println!("receipt.rustc={}", command_stdout(&root, "rustc", &["--version"]));
    println!("receipt.nix={}", command_stdout(&root, "nix", &["--version"]));
    println!("receipt.system={}/{}", std::env::consts::OS, std::env::consts::ARCH);
    for relative in REAL_CODE_CORPUS {
        let bytes = fs::read(root.join(relative)).expect("read corpus file");
        println!("receipt.corpus_file_b3.{relative}={}", blake3_hex(&bytes));
    }
    let copied_root = temp.path().join("copied-root");
    copy_real_code_corpus(&root, &copied_root);

    let mut stdout_hashes = BTreeSet::new();
    let mut object_sets = Vec::new();
    for run in 0..10 {
        let target_dir = temp.path().join(format!("target-{run}"));
        let bytes = emit_objects_bytes_with_env(&root, REAL_CODE_CORPUS, &target_dir);
        stdout_hashes.insert(blake3_hex(&bytes));
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("objects json");
        object_sets.push(object_hash_set(&json));
    }
    assert_eq!(stdout_hashes.len(), 1, "repeated emits produced distinct stdout hashes: {stdout_hashes:?}");
    println!("receipt.emit_runs=10");
    println!("receipt.emit_stdout_b3={}", stdout_hashes.iter().next().expect("stdout hash"));
    assert!(object_sets.windows(2).all(|pair| pair[0] == pair[1]));
    println!("receipt.object_count={}", object_sets[0].len());

    let copied_bytes = emit_objects_bytes_with_env(&copied_root, REAL_CODE_CORPUS, &temp.path().join("target-copied"));
    let copied_json: serde_json::Value = serde_json::from_slice(&copied_bytes).expect("copied objects json");
    assert_eq!(object_sets[0], object_hash_set(&copied_json));
    println!("receipt.cross_root_object_set: PASS");

    let reversed = [
        "cargo-octet/src/fingerprint.rs",
        "cargo-octet/src/object.rs",
        "cargo-octet/src/cli.rs",
    ];
    let reversed_bytes = emit_objects_bytes_with_env(&root, &reversed, &temp.path().join("target-reversed"));
    let reversed_json: serde_json::Value = serde_json::from_slice(&reversed_bytes).expect("reversed objects json");
    assert_eq!(object_sets[0], object_hash_set(&reversed_json));
    println!("receipt.input_order_permutation_object_set: PASS");

    let objects_path = temp.path().join("objects.json");
    fs::write(
        &objects_path,
        emit_objects_bytes_with_env(&root, REAL_CODE_CORPUS, &temp.path().join("target-bundle-objects")),
    )
    .expect("write objects json");
    let mut bundle_hashes = BTreeSet::new();
    for run in 0..10 {
        let bundle_path = temp.path().join(format!("bundle-{run}.json"));
        let bundle = export_bundle(&root, &objects_path, None, &bundle_path);
        assert!(bundle.status.success(), "bundle stderr: {}", String::from_utf8_lossy(&bundle.stderr));
        let bytes = fs::read(&bundle_path).expect("read bundle");
        bundle_hashes.insert(blake3_hex(&bytes));
    }
    assert_eq!(bundle_hashes.len(), 1, "repeated bundles produced distinct byte hashes: {bundle_hashes:?}");
    println!("receipt.bundle_runs=10");
    println!("receipt.bundle_bytes_b3={}", bundle_hashes.iter().next().expect("bundle hash"));
}

#[test]
fn hir_mir_supplement_pilot_docs_name_commands_and_non_proofs() {
    let root = repo_root();
    let contributor = fs::read_to_string(root.join("docs/contributor-guide.md")).expect("read contributor guide");
    let boundaries = fs::read_to_string(root.join("docs/content-addressed-rust-evidence-boundaries.md"))
        .expect("read content-addressed boundaries doc");

    for needle in [
        "cargo octet object supplement emit --family hir",
        "cargo octet object supplement check --family hir",
        "cargo octet object supplement emit --family mir",
        "cargo octet object supplement check --family mir",
        "cargo octet object supplement emit --family thir",
        "cargo octet object supplement check --family thir",
        "cargo octet object corpus contract --output-format json",
        "cargo octet object supplement emit --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement check --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement emit --family mir --spike rustc-mir-fixture",
        "cargo octet object supplement check --family mir --spike rustc-mir-fixture",
        "--fixture-shape checked-arithmetic",
        "--fixture-shape branch-control-flow",
        "selected-fixture-backed/toolchain-scoped review",
        "checked.rs::checked",
        "clamp.rs::clamp_to_limit",
        "call_return.rs::call_then_return",
        "source-token-bounded review evidence",
        "selected-fixture-backed/toolchain-scoped review",
        "do not replace it as the",
        "The default MIR and THIR runtime seams",
        "correctness proof",
        "semantic-equivalence proof",
        "pure-cache eligibility proof",
        "cross-toolchain stability",
    ] {
        assert!(contributor.contains(needle), "contributor guide missing {needle}: {contributor}");
    }

    for needle in [
        "cargo octet object supplement emit --family hir",
        "cargo octet object supplement check --family hir",
        "cargo octet object supplement emit --family mir",
        "cargo octet object supplement check --family mir",
        "cargo octet object supplement emit --family thir",
        "cargo octet object supplement check --family thir",
        "cargo octet object supplement emit --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement check --family hir --spike rustc-hir-fixture",
        "cargo octet object supplement emit --family mir --spike rustc-mir-fixture",
        "cargo octet object supplement check --family mir --spike rustc-mir-fixture",
        "--fixture-shape checked-arithmetic",
        "--fixture-shape branch-control-flow",
        "attached supplement to `octet.function-object.v1`",
        "selected-fixture-backed/toolchain-scoped",
        "arbitrary-path and non-fixture HIR remain",
        "Default THIR runtime receipts MUST keep status `unsupported-planned`",
        "does not prove THIR",
        "MUST NOT claim stable identity across compiler versions",
    ] {
        assert!(boundaries.contains(needle), "boundary doc missing {needle}: {boundaries}");
    }
}

#[allow(clippy::too_many_lines)]
#[test]
fn thir_supplement_emit_and_check_remain_fail_closed() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create thir supplement tempdir");
    let emit = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "emit",
            "--family",
            "thir",
            "--function",
            "checked",
            "cargo-octet/tests/fixtures/hir-spike/checked.rs",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run THIR supplement emit");
    assert!(
        emit.status.success(),
        "THIR emit failed stdout={} stderr={}",
        String::from_utf8_lossy(&emit.stdout),
        String::from_utf8_lossy(&emit.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("THIR emit json");
    assert_eq!(receipt["family"], "thir");
    assert_eq!(receipt["boundary"]["claims_correctness"], false);
    assert_eq!(receipt["boundary"]["claims_semantic_equivalence"], false);
    assert_eq!(receipt["boundary"]["claims_pure_cache_eligibility"], false);
    assert_eq!(receipt["boundary"]["claims_cross_toolchain_stability"], false);
    let supplement =
        receipt["supplements"].as_array().expect("THIR supplement array").first().expect("THIR supplement");
    assert_eq!(supplement["schema"], "octet.function-object-thir-supplement.v1");
    assert_eq!(supplement["family"], "thir");
    assert_eq!(supplement["status"], "unsupported-planned");
    assert_eq!(supplement["boundary"]["claims_correctness"], false);
    assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);
    assert_eq!(supplement["observations"]["thir_context"]["status"], "unsupported-planned");
    assert_eq!(
        supplement["observations"]["thir_context"]["detail"],
        "THIR extraction is not implemented; this record preserves a fail-closed runtime family seam"
    );
    assert_eq!(supplement["observations"]["typed_hir_lowering"]["status"], "unsupported-planned");
    assert_eq!(
        supplement["observations"]["typed_hir_lowering"]["detail"],
        "typed HIR/THIR lowering observations require future extractor evidence before support"
    );
    assert!(
        supplement["observations"]["support_boundary"]["detail"]
            .as_str()
            .expect("THIR support boundary detail")
            .contains("unsupported-planned evidence only; no correctness"),
        "THIR support boundary must pin non-proof wording: {supplement:#?}"
    );
    assert!(
        supplement["extraction_caveats"]
            .as_array()
            .expect("THIR caveats")
            .iter()
            .any(|caveat| caveat["status"] == "unsupported-planned"),
        "THIR caveats must stay unsupported-planned: {supplement:#?}"
    );

    let receipt_path = temp.path().join("thir-supplement.json");
    fs::write(&receipt_path, &emit.stdout).expect("write THIR receipt");
    let check = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "check",
            "--family",
            "thir",
            "--receipt",
            receipt_path.to_str().expect("receipt path utf8"),
            "--function",
            "checked",
            "cargo-octet/tests/fixtures/hir-spike/checked.rs",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run THIR supplement check");
    assert!(
        check.status.success(),
        "THIR check failed stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("THIR check json");
    assert_eq!(report["family"], "thir");
    assert_eq!(report["status"], "pass");
    assert_eq!(report["boundary"]["claims_correctness"], false);
    assert_eq!(report["boundary"]["claims_semantic_equivalence"], false);
    assert!(
        report["comparisons"]
            .as_array()
            .expect("THIR check comparisons")
            .iter()
            .all(|comparison| comparison["status"] == "match"),
        "THIR check should only pass with matching fail-closed fields: {report:#?}"
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn thir_fixture_supplement_emit_check_and_negative_replay_are_bounded() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create thir fixture tempdir");

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
        .expect("run THIR fixture inventory");
    assert!(
        inventory.status.success(),
        "THIR inventory failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let inventory_json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("THIR inventory json");
    assert_eq!(inventory_json["schema"], "octet.thir-fixture-inventory.v1");
    assert_eq!(inventory_json["family"], "thir");
    assert_eq!(inventory_json["scope"], "rustc-thir-fixture-spike");
    assert_eq!(inventory_json["default_shape"], "checked-arithmetic");
    let shapes = inventory_json["shapes"].as_array().expect("THIR inventory shapes");
    assert_eq!(shapes.len(), 3, "THIR fixture support must stay closed to selected selectors: {inventory_json:#?}");
    let mut seen_shapes = BTreeSet::new();
    for shape in shapes {
        seen_shapes.insert(shape["shape"].as_str().unwrap().to_owned());
        assert_eq!(shape["support_status"], "pilot-supported");
        assert!(
            shape["support_boundary"].as_str().unwrap().contains("no general THIR extraction"),
            "THIR inventory must not overclaim: {inventory_json:#?}"
        );
    }
    assert_eq!(
        seen_shapes,
        BTreeSet::from([
            "branch-control-flow".to_owned(),
            "call-return".to_owned(),
            "checked-arithmetic".to_owned()
        ])
    );
    assert!(shapes.iter().any(|shape| {
        shape["shape"] == "checked-arithmetic"
            && shape["fixture_path"] == "cargo-octet/tests/fixtures/thir-spike/checked.rs"
            && shape["function"] == "checked"
    }));
    assert!(shapes.iter().any(|shape| {
        shape["shape"] == "call-return"
            && shape["fixture_path"] == "cargo-octet/tests/fixtures/thir-spike/call_return.rs"
            && shape["function"] == "call_then_return"
    }));
    assert!(shapes.iter().any(|shape| {
        shape["shape"] == "branch-control-flow"
            && shape["fixture_path"] == "cargo-octet/tests/fixtures/thir-spike/branch_control_flow.rs"
            && shape["function"] == "choose_branch"
    }));

    for fixture_shape in ["checked-arithmetic", "call-return", "branch-control-flow"] {
        let emit = Command::new(cargo_octet())
            .args([
                "object",
                "supplement",
                "emit",
                "--family",
                "thir",
                "--spike",
                "rustc-thir-fixture",
                "--fixture-shape",
                fixture_shape,
                "--output-format",
                "json",
            ])
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run THIR fixture emit {fixture_shape}: {error}"));
        assert!(
            emit.status.success(),
            "THIR fixture emit {fixture_shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&emit.stdout),
            String::from_utf8_lossy(&emit.stderr)
        );
        let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("THIR fixture emit json");
        assert_eq!(receipt["family"], "thir");
        assert_eq!(receipt["scope"], "rustc-thir-fixture-spike");
        let supplement = receipt["supplements"].as_array().unwrap().first().expect("THIR fixture supplement");
        assert_eq!(supplement["schema"], "octet.function-object-thir-supplement.v1");
        assert_eq!(supplement["family"], "thir");
        assert_eq!(supplement["status"], "pilot-supported");
        assert_eq!(supplement["extractor"]["name"], "octet-rustc-thir-fixture-contract");
        assert_eq!(supplement["observations"]["fixture_context"]["status"], "pilot-supported");
        assert!(
            supplement["observations"]["fixture_context"]["detail"]
                .as_str()
                .unwrap()
                .contains(&format!("fixture_shape={fixture_shape}")),
            "THIR fixture observation should bind selector: {supplement:#?}"
        );
        assert_eq!(supplement["observations"]["typed_expression_shape"]["status"], "pilot-supported");
        assert!(
            supplement["observations"]["typed_expression_shape"]["detail"]
                .as_str()
                .unwrap()
                .contains(fixture_shape),
            "THIR typed expression observation should name selector: {supplement:#?}"
        );
        assert!(
            supplement["observations"]["support_boundary"]["detail"]
                .as_str()
                .unwrap()
                .contains("selected-fixture-only"),
            "THIR fixture support boundary must be selected-only: {supplement:#?}"
        );
        assert_eq!(supplement["boundary"]["claims_correctness"], false);
        assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);
        assert_eq!(supplement["boundary"]["claims_pure_cache_eligibility"], false);
        assert_eq!(supplement["boundary"]["claims_cross_toolchain_stability"], false);

        let receipt_path = temp.path().join(format!("thir-fixture-{fixture_shape}.json"));
        fs::write(&receipt_path, &emit.stdout).expect("write THIR fixture receipt");
        let check = Command::new(cargo_octet())
            .args([
                "object",
                "supplement",
                "check",
                "--family",
                "thir",
                "--spike",
                "rustc-thir-fixture",
                "--fixture-shape",
                fixture_shape,
                "--receipt",
                receipt_path.to_str().unwrap(),
                "--output-format",
                "json",
            ])
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run THIR fixture check {fixture_shape}: {error}"));
        assert!(
            check.status.success(),
            "THIR fixture check {fixture_shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("THIR fixture check json");
        assert_eq!(report["status"], "pass");
        assert!(report["comparisons"].as_array().unwrap().iter().all(|comparison| comparison["status"] == "match"));

        let mut tampered = receipt.clone();
        tampered["supplements"][0]["observations"]["typed_expression_shape"]["detail"] =
            serde_json::Value::String("tampered selected fixture shape".to_owned());
        let tampered_path = temp.path().join(format!("thir-fixture-{fixture_shape}-tampered.json"));
        fs::write(&tampered_path, serde_json::to_vec_pretty(&tampered).unwrap()).expect("write tampered receipt");
        let tampered_check = Command::new(cargo_octet())
            .args([
                "object",
                "supplement",
                "check",
                "--family",
                "thir",
                "--spike",
                "rustc-thir-fixture",
                "--fixture-shape",
                fixture_shape,
                "--receipt",
                tampered_path.to_str().unwrap(),
                "--output-format",
                "json",
            ])
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run tampered THIR fixture check {fixture_shape}: {error}"));
        assert!(
            !tampered_check.status.success(),
            "tampered THIR fixture receipt {fixture_shape} must fail stdout={} stderr={}",
            String::from_utf8_lossy(&tampered_check.stdout),
            String::from_utf8_lossy(&tampered_check.stderr)
        );
        let tampered_report: serde_json::Value =
            serde_json::from_slice(&tampered_check.stdout).expect("tampered THIR fixture report json");
        assert_eq!(tampered_report["status"], "fail");
        assert!(
            tampered_report["comparisons"]
                .as_array()
                .unwrap()
                .iter()
                .any(|comparison| comparison["status"] == "mismatch"),
            "tampered THIR fixture receipt should report mismatch: {tampered_report:#?}"
        );
    }
}

#[test]
fn supplement_compatibility_receipts_fail_closed() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create supplement compatibility tempdir");

    let emit = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "emit",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "checked-arithmetic",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run MIR supplement compatibility emit");
    assert!(
        emit.status.success(),
        "MIR compatibility emit failed stdout={} stderr={}",
        String::from_utf8_lossy(&emit.stdout),
        String::from_utf8_lossy(&emit.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("MIR compatibility receipt json");
    let positive_path = temp.path().join("positive-mir-v1.json");
    fs::write(&positive_path, &emit.stdout).expect("write positive supplement receipt");
    let positive = supplement_check_fixture(&root, "mir", "rustc-mir-fixture", "checked-arithmetic", &positive_path);
    assert!(
        positive.status.success(),
        "positive supplement receipt should pass stdout={} stderr={}",
        String::from_utf8_lossy(&positive.stdout),
        String::from_utf8_lossy(&positive.stderr)
    );
    let positive_report: serde_json::Value = serde_json::from_slice(&positive.stdout).expect("positive report json");
    assert_eq!(positive_report["status"], "pass");
    assert_eq!(positive_report["errors"].as_array().unwrap().len(), 0);

    let mut missing_field = receipt.clone();
    missing_field["supplements"][0].as_object_mut().unwrap().remove("function_object_hash");
    assert_invalid_supplement_fixture(
        &root,
        temp.path(),
        "missing-function-object-hash.json",
        &missing_field,
        "missing-field",
        "function_object_hash",
    );

    let mut future_version = receipt.clone();
    future_version["supplements"][0]["schema_version"] = serde_json::Value::from(99);
    assert_invalid_supplement_fixture(
        &root,
        temp.path(),
        "future-schema-version.json",
        &future_version,
        "unsupported-schema-version",
        "schema_version",
    );

    let mut wrong_family = receipt.clone();
    wrong_family["supplements"][0]["family"] = serde_json::Value::String("thir".to_owned());
    assert_invalid_supplement_fixture(
        &root,
        temp.path(),
        "wrong-family.json",
        &wrong_family,
        "family-mismatch",
        "family",
    );

    let mut boundary_tamper = receipt.clone();
    boundary_tamper["supplements"][0]["boundary"]["claims_correctness"] = serde_json::Value::Bool(true);
    let boundary_path = temp.path().join("boundary-tamper.json");
    fs::write(&boundary_path, serde_json::to_vec_pretty(&boundary_tamper).unwrap()).expect("write boundary tamper");
    let boundary_check =
        supplement_check_fixture(&root, "mir", "rustc-mir-fixture", "checked-arithmetic", &boundary_path);
    assert!(!boundary_check.status.success(), "boundary tamper should fail closed");
    let boundary_report: serde_json::Value =
        serde_json::from_slice(&boundary_check.stdout).expect("boundary tamper report json");
    assert_eq!(boundary_report["status"], "fail");
    assert!(
        boundary_report["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|comparison| { comparison["field"] == "boundary" && comparison["status"] == "mismatch" })
    );

    let malformed_path = temp.path().join("malformed.json");
    fs::write(&malformed_path, b"{ not json").expect("write malformed supplement receipt");
    let malformed = supplement_check_fixture(&root, "mir", "rustc-mir-fixture", "checked-arithmetic", &malformed_path);
    assert!(!malformed.status.success(), "malformed supplement receipt should fail closed");
    let malformed_report: serde_json::Value = serde_json::from_slice(&malformed.stdout).expect("malformed report json");
    assert_eq!(malformed_report["status"], "invalid");
    assert_supplement_error(&malformed_report, "invalid-json", "receipt");
}

fn supplement_check_fixture(
    root: &Path,
    family: &str,
    spike: &str,
    fixture_shape: &str,
    receipt_path: &Path,
) -> std::process::Output {
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
            fixture_shape,
            "--receipt",
            receipt_path.to_str().unwrap(),
            "--output-format",
            "json",
        ])
        .current_dir(root)
        .output()
        .expect("run supplement compatibility check")
}

fn assert_invalid_supplement_fixture(
    root: &Path,
    temp: &Path,
    name: &str,
    receipt: &serde_json::Value,
    expected_kind: &str,
    expected_field: &str,
) {
    let path = temp.join(name);
    fs::write(&path, serde_json::to_vec_pretty(receipt).unwrap()).expect("write invalid supplement receipt");
    let check = supplement_check_fixture(root, "mir", "rustc-mir-fixture", "checked-arithmetic", &path);
    assert!(
        !check.status.success(),
        "invalid supplement receipt {name} should fail stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("invalid supplement report json");
    assert_eq!(report["status"], "invalid");
    assert_eq!(report["comparisons"].as_array().unwrap().len(), 0);
    assert_supplement_error(&report, expected_kind, expected_field);
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

#[allow(clippy::too_many_lines)]
#[test]
fn mir_fixture_supplement_emit_check_and_negative_replay_are_bounded() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create MIR fixture tempdir");

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
        .expect("run MIR fixture inventory");
    assert!(
        inventory.status.success(),
        "MIR inventory failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let inventory_json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("MIR inventory json");
    assert_eq!(inventory_json["schema"], "octet.mir-fixture-inventory.v1");
    assert_eq!(inventory_json["family"], "mir");
    assert_eq!(inventory_json["scope"], "rustc-mir-fixture-spike");
    assert_eq!(inventory_json["default_shape"], "checked-arithmetic");
    let shapes = inventory_json["shapes"].as_array().expect("MIR inventory shapes");
    assert_eq!(shapes.len(), 3, "MIR fixture support must stay closed to selected selectors: {inventory_json:#?}");
    assert_eq!(shapes[0]["shape"], "checked-arithmetic");
    assert_eq!(shapes[0]["fixture_path"], "cargo-octet/tests/fixtures/mir-spike/checked.rs");
    assert_eq!(shapes[0]["function"], "checked");
    assert_eq!(shapes[0]["support_status"], "pilot-supported");
    assert_eq!(shapes[1]["shape"], "call-return");
    assert_eq!(shapes[1]["fixture_path"], "cargo-octet/tests/fixtures/mir-spike/call_return.rs");
    assert_eq!(shapes[1]["function"], "call_then_return");
    assert_eq!(shapes[1]["support_status"], "pilot-supported");
    assert_eq!(shapes[2]["shape"], "branch-control-flow");
    assert_eq!(shapes[2]["fixture_path"], "cargo-octet/tests/fixtures/mir-spike/branch_control_flow.rs");
    assert_eq!(shapes[2]["function"], "choose_branch");
    assert_eq!(shapes[2]["support_status"], "pilot-supported");
    for shape in shapes {
        assert!(
            shape["support_boundary"].as_str().unwrap().contains("selected-fixture-only"),
            "MIR inventory must not overclaim: {inventory_json:#?}"
        );
    }

    let emit = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "emit",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "checked-arithmetic",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run MIR fixture emit");
    assert!(
        emit.status.success(),
        "MIR fixture emit failed stdout={} stderr={}",
        String::from_utf8_lossy(&emit.stdout),
        String::from_utf8_lossy(&emit.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("MIR fixture emit json");
    assert_eq!(receipt["family"], "mir");
    assert_eq!(receipt["scope"], "rustc-mir-fixture-spike");
    let supplement = receipt["supplements"].as_array().unwrap().first().expect("MIR fixture supplement");
    assert_eq!(supplement["schema"], "octet.function-object-mir-supplement.v1");
    assert_eq!(supplement["family"], "mir");
    assert_eq!(supplement["status"], "pilot-supported");
    assert_eq!(supplement["extractor"]["name"], "octet-rustc-mir-fixture-contract");
    assert_eq!(supplement["observations"]["fixture_context"]["status"], "pilot-supported");
    assert!(
        supplement["observations"]["fixture_context"]["detail"]
            .as_str()
            .unwrap()
            .contains("fixture_shape=checked-arithmetic"),
        "MIR fixture observation should bind selector: {supplement:#?}"
    );
    assert!(
        supplement["observations"]["mir_fixture_contract_hash"]["detail"].as_str().unwrap().contains("b3:"),
        "MIR fixture contract hash should be BLAKE3-addressed: {supplement:#?}"
    );
    assert!(
        supplement["observations"]["support_boundary"]["detail"]
            .as_str()
            .unwrap()
            .contains("selected-fixture-only"),
        "MIR fixture support boundary must be selected-only: {supplement:#?}"
    );
    assert_eq!(supplement["boundary"]["claims_correctness"], false);
    assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);
    assert_eq!(supplement["boundary"]["claims_pure_cache_eligibility"], false);
    assert_eq!(supplement["boundary"]["claims_cross_toolchain_stability"], false);

    let receipt_path = temp.path().join("mir-fixture-checked-arithmetic.json");
    fs::write(&receipt_path, &emit.stdout).expect("write MIR fixture receipt");
    let check = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "check",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "checked-arithmetic",
            "--receipt",
            receipt_path.to_str().unwrap(),
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run MIR fixture check");
    assert!(
        check.status.success(),
        "MIR fixture check failed stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("MIR fixture check json");
    assert_eq!(report["status"], "pass");
    assert!(report["comparisons"].as_array().unwrap().iter().all(|comparison| comparison["status"] == "match"));

    let mut tampered = receipt.clone();
    tampered["supplements"][0]["observations"]["mir_fixture_contract_hash"]["detail"] =
        serde_json::Value::String("tampered selected MIR fixture hash".to_owned());
    let tampered_path = temp.path().join("mir-fixture-checked-arithmetic-tampered.json");
    fs::write(&tampered_path, serde_json::to_vec_pretty(&tampered).unwrap()).expect("write tampered MIR receipt");
    let tampered_check = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "check",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "checked-arithmetic",
            "--receipt",
            tampered_path.to_str().unwrap(),
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run tampered MIR fixture check");
    assert!(
        !tampered_check.status.success(),
        "tampered MIR fixture receipt must fail stdout={} stderr={}",
        String::from_utf8_lossy(&tampered_check.stdout),
        String::from_utf8_lossy(&tampered_check.stderr)
    );
    let tampered_report: serde_json::Value =
        serde_json::from_slice(&tampered_check.stdout).expect("tampered MIR fixture report json");
    assert_eq!(tampered_report["status"], "fail");
    assert!(
        tampered_report["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|comparison| comparison["status"] == "mismatch"),
        "tampered MIR fixture receipt should report mismatch: {tampered_report:#?}"
    );

    let call_emit = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "emit",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "call-return",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run MIR call-return fixture emit");
    assert!(
        call_emit.status.success(),
        "MIR call-return fixture emit failed stdout={} stderr={}",
        String::from_utf8_lossy(&call_emit.stdout),
        String::from_utf8_lossy(&call_emit.stderr)
    );
    let call_receipt: serde_json::Value =
        serde_json::from_slice(&call_emit.stdout).expect("MIR call-return fixture emit json");
    let call_supplement = call_receipt["supplements"].as_array().unwrap().first().expect("MIR call-return supplement");
    assert_eq!(call_supplement["schema"], "octet.function-object-mir-supplement.v1");
    assert_eq!(call_supplement["family"], "mir");
    assert_eq!(call_supplement["status"], "pilot-supported");
    assert_eq!(call_supplement["source_path"], "cargo-octet/tests/fixtures/mir-spike/call_return.rs");
    assert_eq!(call_supplement["function_name"], "call_then_return");
    assert!(
        call_supplement["observations"]["fixture_context"]["detail"]
            .as_str()
            .unwrap()
            .contains("fixture_shape=call-return"),
        "MIR call-return fixture observation should bind selector: {call_supplement:#?}"
    );
    assert!(
        call_supplement["observations"]["mir_fixture_contract_hash"]["detail"]
            .as_str()
            .unwrap()
            .contains("fixture_shape=call-return"),
        "MIR call-return fixture contract hash should bind selector: {call_supplement:#?}"
    );
    assert_eq!(call_supplement["boundary"]["claims_correctness"], false);
    assert_eq!(call_supplement["boundary"]["claims_semantic_equivalence"], false);
    assert_eq!(call_supplement["boundary"]["claims_pure_cache_eligibility"], false);
    assert_eq!(call_supplement["boundary"]["claims_cross_toolchain_stability"], false);

    let call_receipt_path = temp.path().join("mir-fixture-call-return.json");
    fs::write(&call_receipt_path, &call_emit.stdout).expect("write MIR call-return receipt");
    let call_check = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "check",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "call-return",
            "--receipt",
            call_receipt_path.to_str().unwrap(),
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run MIR call-return fixture check");
    assert!(
        call_check.status.success(),
        "MIR call-return fixture check failed stdout={} stderr={}",
        String::from_utf8_lossy(&call_check.stdout),
        String::from_utf8_lossy(&call_check.stderr)
    );
    let call_report: serde_json::Value =
        serde_json::from_slice(&call_check.stdout).expect("MIR call-return fixture check json");
    assert_eq!(call_report["status"], "pass");
    assert!(
        call_report["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .all(|comparison| comparison["status"] == "match")
    );
}

#[test]
fn object_supplement_runtime_smoke_covers_nested_help_and_inventory() {
    let root = repo_root();

    for args in [
        &["object", "supplement", "emit", "--help"][..],
        &["object", "supplement", "check", "--help"][..],
        &["object", "supplement", "fixtures", "--help"][..],
    ] {
        let output = Command::new(cargo_octet())
            .args(args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run nested supplement help {args:?}: {error}"));
        assert!(
            output.status.success(),
            "nested supplement help {args:?} failed stdout={} stderr={}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8_lossy(&output.stdout);
        assert!(stdout.contains("cargo octet object"), "help missing object usage for {args:?}: {stdout}");
        assert!(
            stdout.contains("supplement fixtures --family mir --spike rustc-mir-fixture"),
            "help missing scoped fixture inventory command for {args:?}: {stdout}"
        );
        assert!(
            stdout.contains("--family hir|thir|mir"),
            "help missing THIR fail-closed supplement family for {args:?}: {stdout}"
        );
    }

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
        assert!(fixture["emit_command"].as_str().unwrap().contains(selector));
        assert!(fixture["check_command"].as_str().unwrap().contains(selector));
    }
}

#[test]
fn object_supplement_inventory_commands_roundtrip_for_every_shape() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create supplement roundtrip tempdir");
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
        .expect("run HIR fixture inventory json roundtrip source");
    assert!(
        inventory.status.success(),
        "inventory roundtrip source failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("inventory roundtrip json");
    let shapes = json["shapes"].as_array().expect("fixture inventory shapes array");
    let mut visited = BTreeSet::new();

    for fixture in shapes {
        let shape = fixture["shape"].as_str().expect("fixture shape");
        visited.insert(shape.to_owned());
        let emit_command = fixture["emit_command"].as_str().expect("emit command");
        let emit_args: Vec<&str> = emit_command
            .strip_prefix("cargo octet ")
            .unwrap_or_else(|| panic!("emit command should start with cargo octet: {emit_command}"))
            .split_whitespace()
            .collect();
        let emit = Command::new(cargo_octet())
            .args(&emit_args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run inventory emit command for {shape}: {error}"));
        assert!(
            emit.status.success(),
            "inventory emit command for {shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&emit.stdout),
            String::from_utf8_lossy(&emit.stderr)
        );
        let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("emit receipt json");
        assert_eq!(receipt["family"], "hir");
        let supplement = receipt["supplements"]
            .as_array()
            .expect("supplement array")
            .first()
            .unwrap_or_else(|| panic!("missing emitted supplement: {receipt:#?}"));
        assert_eq!(supplement["status"], "pilot-supported");
        assert!(
            supplement["observations"]["fixture_context"]["detail"]
                .as_str()
                .expect("fixture context detail")
                .contains(&format!("fixture_shape={shape}")),
            "receipt should name inventory shape {shape}: {receipt:#?}"
        );

        let receipt_path = temp.path().join(format!("{shape}.json"));
        fs::write(&receipt_path, &emit.stdout).expect("write emitted supplement receipt");
        let check_command = fixture["check_command"].as_str().expect("check command");
        let check_args: Vec<String> = check_command
            .strip_prefix("cargo octet ")
            .unwrap_or_else(|| panic!("check command should start with cargo octet: {check_command}"))
            .split_whitespace()
            .map(|arg| {
                if arg == "RECEIPT.json" {
                    receipt_path.to_string_lossy().into_owned()
                } else {
                    arg.to_owned()
                }
            })
            .collect();
        let check = Command::new(cargo_octet())
            .args(&check_args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run inventory check command for {shape}: {error}"));
        assert!(
            check.status.success(),
            "inventory check command for {shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check report json");
        assert_eq!(report["status"], "pass");
        assert!(
            report["comparisons"]
                .as_array()
                .expect("comparison array")
                .iter()
                .all(|comparison| comparison["status"] == "match"),
            "inventory check command for {shape} should match all compared fields: {report:#?}"
        );
    }

    assert_eq!(visited, BTreeSet::from(["branch-control-flow".to_owned(), "checked-arithmetic".to_owned()]));
}

#[allow(clippy::too_many_lines)]
#[test]
fn thir_inventory_commands_roundtrip_selected_fixture_evidence() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create THIR supplement roundtrip tempdir");
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
        .expect("run THIR fixture inventory json roundtrip source");
    assert!(
        inventory.status.success(),
        "THIR inventory roundtrip source failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("THIR inventory roundtrip json");
    assert_eq!(json["schema"], "octet.thir-fixture-inventory.v1");
    assert_eq!(json["family"], "thir");
    assert_eq!(json["scope"], "rustc-thir-fixture-spike");
    assert_eq!(json["default_shape"], "checked-arithmetic");
    let shapes = json["shapes"].as_array().expect("THIR fixture inventory shapes array");
    let mut visited = BTreeSet::new();

    for fixture in shapes {
        let shape = fixture["shape"].as_str().expect("THIR fixture shape");
        visited.insert(shape.to_owned());
        if shape == "branch-control-flow" {
            assert!(
                fixture["fixture_path"]
                    .as_str()
                    .expect("THIR branch fixture path")
                    .ends_with("thir-spike/branch_control_flow.rs"),
                "branch-control-flow must use the THIR-specific fixture path: {fixture:#?}"
            );
            assert_eq!(fixture["function"], "choose_branch");
        }
        assert_eq!(fixture["support_status"], "pilot-supported");
        assert!(
            fixture["support_boundary"]
                .as_str()
                .expect("support boundary")
                .contains("selected checked-in rustc THIR fixture contract only"),
            "THIR inventory should stay selected-fixture-only: {fixture:#?}"
        );
        let emit_command = fixture["emit_command"].as_str().expect("emit command");
        let emit_args: Vec<&str> = emit_command
            .strip_prefix("cargo octet ")
            .unwrap_or_else(|| panic!("emit command should start with cargo octet: {emit_command}"))
            .split_whitespace()
            .collect();
        assert!(emit_args.windows(2).any(|window| window == ["--family", "thir"]));
        assert!(emit_args.windows(2).any(|window| window == ["--spike", "rustc-thir-fixture"]));
        let emit = Command::new(cargo_octet())
            .args(&emit_args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run THIR inventory emit command for {shape}: {error}"));
        assert!(
            emit.status.success(),
            "THIR inventory emit command for {shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&emit.stdout),
            String::from_utf8_lossy(&emit.stderr)
        );
        let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("THIR emit receipt json");
        assert_eq!(receipt["family"], "thir");
        let supplement = receipt["supplements"]
            .as_array()
            .expect("THIR supplement array")
            .first()
            .unwrap_or_else(|| panic!("missing emitted THIR supplement: {receipt:#?}"));
        assert_eq!(supplement["status"], "pilot-supported");
        assert!(
            supplement["observations"]["fixture_context"]["detail"]
                .as_str()
                .expect("fixture context detail")
                .contains(&format!("fixture_shape={shape}")),
            "THIR receipt should name inventory shape {shape}: {receipt:#?}"
        );
        assert_eq!(supplement["boundary"]["claims_correctness"], false);
        assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);
        assert_eq!(supplement["boundary"]["claims_pure_cache_eligibility"], false);
        assert_eq!(supplement["boundary"]["claims_cross_toolchain_stability"], false);

        let receipt_path = temp.path().join(format!("{shape}.json"));
        fs::write(&receipt_path, &emit.stdout).expect("write emitted THIR supplement receipt");
        let check_command = fixture["check_command"].as_str().expect("check command");
        let check_args: Vec<String> = check_command
            .strip_prefix("cargo octet ")
            .unwrap_or_else(|| panic!("check command should start with cargo octet: {check_command}"))
            .split_whitespace()
            .map(|arg| {
                if arg == "RECEIPT.json" {
                    receipt_path.to_string_lossy().into_owned()
                } else {
                    arg.to_owned()
                }
            })
            .collect();
        let check = Command::new(cargo_octet())
            .args(&check_args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run THIR inventory check command for {shape}: {error}"));
        assert!(
            check.status.success(),
            "THIR inventory check command for {shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("THIR check report json");
        assert_eq!(report["status"], "pass");
        assert!(
            report["comparisons"]
                .as_array()
                .expect("comparison array")
                .iter()
                .all(|comparison| comparison["status"] == "match"),
            "THIR inventory check command for {shape} should match all compared fields: {report:#?}"
        );
    }

    assert_eq!(
        visited,
        BTreeSet::from([
            "branch-control-flow".to_owned(),
            "call-return".to_owned(),
            "checked-arithmetic".to_owned(),
        ])
    );
}

#[test]
fn thir_inventory_aggregate_gate_preserves_selected_fixture_boundaries() {
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
        assert!(
            fixture["emit_command"].as_str().expect("emit command").contains("--spike rustc-thir-fixture"),
            "aggregate THIR inventory must expose replayable emit commands: {fixture:#?}"
        );
        assert!(
            fixture["check_command"].as_str().expect("check command").contains("--spike rustc-thir-fixture"),
            "aggregate THIR inventory must expose replayable check commands: {fixture:#?}"
        );
        assert!(
            fixture["support_boundary"]
                .as_str()
                .expect("support boundary")
                .contains("selected checked-in rustc THIR fixture contract only"),
            "aggregate THIR inventory must remain selected-fixture-only: {fixture:#?}"
        );
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
    assert!(
        thir["unsupported_reason"]
            .as_str()
            .expect("THIR unsupported reason")
            .contains("default and non-fixture THIR requests remain unsupported-planned"),
        "aggregate THIR inventory must not broaden default/non-fixture THIR status: {thir:#?}"
    );
}

#[allow(clippy::too_many_lines)]
#[test]
fn mir_fixture_inventory_commands_roundtrip_selected_fixture_evidence() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create MIR supplement roundtrip tempdir");
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
        .expect("run MIR fixture inventory json roundtrip source");
    assert!(
        inventory.status.success(),
        "MIR inventory roundtrip source failed stdout={} stderr={}",
        String::from_utf8_lossy(&inventory.stdout),
        String::from_utf8_lossy(&inventory.stderr)
    );
    let json: serde_json::Value = serde_json::from_slice(&inventory.stdout).expect("MIR inventory roundtrip json");
    assert_eq!(json["schema"], "octet.mir-fixture-inventory.v1");
    assert_eq!(json["family"], "mir");
    assert_eq!(json["scope"], "rustc-mir-fixture-spike");
    assert_eq!(json["default_shape"], "checked-arithmetic");
    let shapes = json["shapes"].as_array().expect("MIR fixture inventory shapes array");
    let mut visited = BTreeSet::new();

    for fixture in shapes {
        let shape = fixture["shape"].as_str().expect("MIR fixture shape");
        visited.insert(shape.to_owned());
        assert_eq!(fixture["support_status"], "pilot-supported");
        assert!(
            fixture["support_boundary"].as_str().expect("support boundary").contains("selected-fixture-only"),
            "MIR inventory should stay selected-fixture-only: {fixture:#?}"
        );
        let emit_command = fixture["emit_command"].as_str().expect("emit command");
        let emit_args: Vec<&str> = emit_command
            .strip_prefix("cargo octet ")
            .unwrap_or_else(|| panic!("emit command should start with cargo octet: {emit_command}"))
            .split_whitespace()
            .collect();
        assert!(emit_args.windows(2).any(|window| window == ["--family", "mir"]));
        assert!(emit_args.windows(2).any(|window| window == ["--spike", "rustc-mir-fixture"]));
        assert!(emit_args.windows(2).any(|window| window == ["--fixture-shape", shape]));
        let emit = Command::new(cargo_octet())
            .args(&emit_args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run MIR inventory emit command for {shape}: {error}"));
        assert!(
            emit.status.success(),
            "MIR inventory emit command for {shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&emit.stdout),
            String::from_utf8_lossy(&emit.stderr)
        );
        let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("MIR emit receipt json");
        assert_eq!(receipt["family"], "mir");
        let supplement = receipt["supplements"]
            .as_array()
            .expect("MIR supplement array")
            .first()
            .unwrap_or_else(|| panic!("missing emitted MIR supplement: {receipt:#?}"));
        assert_eq!(supplement["status"], "pilot-supported");
        assert!(
            supplement["observations"]["fixture_context"]["detail"]
                .as_str()
                .expect("fixture context detail")
                .contains(&format!("fixture_shape={shape}")),
            "MIR receipt should name inventory shape {shape}: {receipt:#?}"
        );
        assert_eq!(supplement["boundary"]["claims_correctness"], false);
        assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);
        assert_eq!(supplement["boundary"]["claims_pure_cache_eligibility"], false);
        assert_eq!(supplement["boundary"]["claims_cross_toolchain_stability"], false);

        let receipt_path = temp.path().join(format!("{shape}.json"));
        fs::write(&receipt_path, &emit.stdout).expect("write emitted MIR supplement receipt");
        let check_command = fixture["check_command"].as_str().expect("check command");
        let check_args: Vec<String> = check_command
            .strip_prefix("cargo octet ")
            .unwrap_or_else(|| panic!("check command should start with cargo octet: {check_command}"))
            .split_whitespace()
            .map(|arg| {
                if arg == "RECEIPT.json" {
                    receipt_path.to_string_lossy().into_owned()
                } else {
                    arg.to_owned()
                }
            })
            .collect();
        let check = Command::new(cargo_octet())
            .args(&check_args)
            .current_dir(&root)
            .output()
            .unwrap_or_else(|error| panic!("run MIR inventory check command for {shape}: {error}"));
        assert!(
            check.status.success(),
            "MIR inventory check command for {shape} failed stdout={} stderr={}",
            String::from_utf8_lossy(&check.stdout),
            String::from_utf8_lossy(&check.stderr)
        );
        let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("MIR check report json");
        assert_eq!(report["status"], "pass");
        assert!(
            report["comparisons"]
                .as_array()
                .expect("comparison array")
                .iter()
                .all(|comparison| comparison["status"] == "match"),
            "MIR inventory check command for {shape} should match all compared fields: {report:#?}"
        );
    }

    assert_eq!(
        visited,
        BTreeSet::from([
            "branch-control-flow".to_owned(),
            "call-return".to_owned(),
            "checked-arithmetic".to_owned()
        ])
    );
}

#[test]
fn mir_fixture_inventory_aggregate_gate_preserves_selected_fixture_boundaries() {
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
        assert!(
            fixture["emit_command"].as_str().expect("emit command").contains("--spike rustc-mir-fixture"),
            "aggregate MIR inventory must expose replayable emit commands: {fixture:#?}"
        );
        assert!(
            fixture["check_command"].as_str().expect("check command").contains("--spike rustc-mir-fixture"),
            "aggregate MIR inventory must expose replayable check commands: {fixture:#?}"
        );
        assert!(
            fixture["support_boundary"].as_str().expect("support boundary").contains("selected-fixture-only"),
            "aggregate MIR inventory must remain selected-fixture-only: {fixture:#?}"
        );
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
    assert!(
        mir["unsupported_reason"]
            .as_str()
            .expect("MIR unsupported reason")
            .contains("default and non-fixture MIR requests remain unsupported-planned"),
        "aggregate MIR inventory must not broaden default/non-fixture MIR status: {mir:#?}"
    );
}
#[test]
fn mir_fixture_branch_control_flow_selector_emits_and_checks_bounded_receipt() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create MIR branch tempdir");
    let receipt_path = temp.path().join("mir-branch-control-flow.json");
    let emit = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "emit",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "branch-control-flow",
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run MIR branch-control-flow emit");
    assert!(
        emit.status.success(),
        "MIR branch-control-flow emit failed stdout={} stderr={}",
        String::from_utf8_lossy(&emit.stdout),
        String::from_utf8_lossy(&emit.stderr)
    );
    let receipt: serde_json::Value = serde_json::from_slice(&emit.stdout).expect("MIR branch receipt json");
    assert_eq!(receipt["family"], "mir");
    let supplement = receipt["supplements"].as_array().expect("supplements")[0].clone();
    assert_eq!(supplement["source_path"], "cargo-octet/tests/fixtures/mir-spike/branch_control_flow.rs");
    assert_eq!(supplement["function_name"], "choose_branch");
    assert!(
        supplement["observations"]["fixture_context"]["detail"]
            .as_str()
            .expect("fixture context")
            .contains("fixture_shape=branch-control-flow")
    );
    assert_eq!(supplement["boundary"]["claims_correctness"], false);
    assert_eq!(supplement["boundary"]["claims_semantic_equivalence"], false);
    assert_eq!(supplement["boundary"]["claims_pure_cache_eligibility"], false);
    assert_eq!(supplement["boundary"]["claims_cross_toolchain_stability"], false);

    fs::write(&receipt_path, &emit.stdout).expect("write MIR branch receipt");
    let check = Command::new(cargo_octet())
        .args([
            "object",
            "supplement",
            "check",
            "--family",
            "mir",
            "--spike",
            "rustc-mir-fixture",
            "--fixture-shape",
            "branch-control-flow",
            "--receipt",
            &receipt_path.to_string_lossy(),
            "--output-format",
            "json",
        ])
        .current_dir(&root)
        .output()
        .expect("run MIR branch-control-flow check");
    assert!(
        check.status.success(),
        "MIR branch-control-flow check failed stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("MIR branch check json");
    assert_eq!(report["status"], "pass");
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons")
            .iter()
            .all(|comparison| comparison["status"] == "match"),
        "MIR branch-control-flow check should match all compared fields: {report:#?}"
    );
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
    assert!(
        hir["fixture_replay_command"].as_str().unwrap().contains("--spike rustc-hir-fixture"),
        "HIR replay command should name fixture spike scope: {hir:#?}"
    );
    for fixture_shape in ["checked-arithmetic", "branch-control-flow"] {
        assert!(
            hir["selected_fixture_shapes"].as_array().unwrap().iter().any(|value| value == fixture_shape),
            "HIR contract should name selected fixture shape {fixture_shape}: {hir:#?}"
        );
        assert!(
            hir["fixture_replay_command"].as_str().unwrap().contains(fixture_shape)
                || hir["extractor_identity_expectations"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|value| { value.as_str().unwrap().contains(fixture_shape) }),
            "HIR contract should bind selected fixture shape {fixture_shape} to replay or extractor expectations: {hir:#?}"
        );
    }
    assert!(
        hir["unsupported_reason"]
            .as_str()
            .unwrap()
            .contains("non-fixture HIR requests remain unsupported-planned"),
        "HIR contract should reject broad/non-fixture HIR support claims: {hir:#?}"
    );
    assert!(
        hir["extractor_identity_expectations"]
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str().unwrap().contains("octet-rustc-hir-fixture-spike")),
        "HIR contract should name rustc HIR fixture spike extractor: {hir:#?}"
    );

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
        assert!(
            supplement["relationship_to_v1"].as_str().unwrap().contains("preserves-function-object-v1-identity"),
            "{family} supplement must preserve v1 identity boundary: {supplement:#?}"
        );
        assert!(
            supplement["toolchain_scope"].as_str().unwrap().contains("rustc-version"),
            "{family} supplement must be rustc/toolchain scoped: {supplement:#?}"
        );
        for field in [
            "schema",
            "schema_version",
            "function_object_hash",
            "rustc_version",
            "target_triple",
            "extraction_caveats",
            "boundary",
        ] {
            assert!(
                supplement["required_fields"].as_array().unwrap().iter().any(|value| value == field),
                "missing required {family} supplement field {field}: {supplement:#?}"
            );
        }
        assert!(
            supplement["unsupported_reason"].as_str().unwrap().contains("unsupported")
                || supplement["unsupported_reason"].as_str().unwrap().contains("not implemented"),
            "{family} supplement should document unsupported/bounded scope: {supplement:#?}"
        );
        assert!(
            supplement["required_evidence_before_support"].as_array().unwrap().iter().any(|value| {
                value.as_str().unwrap().contains("fixture") || value.as_str().unwrap().contains("smoke")
            }),
            "{family} supplement should require deterministic fixture or smoke evidence: {supplement:#?}"
        );
        assert!(
            supplement["extractor_identity_expectations"].as_array().unwrap().iter().any(|value| {
                value.as_str().unwrap().contains("extractor") || value.as_str().unwrap().contains("rustc_version")
            }),
            "{family} supplement should name extractor/toolchain identity expectations: {supplement:#?}"
        );
        assert!(
            supplement["fixture_replay_command"].as_str().unwrap().contains("supplement")
                || supplement["fixture_replay_command"].as_str().unwrap().contains("unsupported-planned"),
            "{family} supplement should name fixture replay or unsupported-plan command: {supplement:#?}"
        );
        assert_eq!(supplement["claims_correctness"], false);
        assert_eq!(supplement["claims_semantic_equivalence"], false);
        assert_eq!(supplement["claims_cross_toolchain_stability"], false);
    }
}

#[test]
fn object_corpus_contract_human_reports_hir_mir_fail_closed_boundaries() {
    let root = repo_root();
    let output = Command::new(cargo_octet())
        .args(["object", "corpus", "contract", "--output-format", "human"])
        .current_dir(&root)
        .output()
        .expect("object corpus contract human metadata");
    assert!(
        output.status.success(),
        "contract human stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    for needle in [
        "supplemental_evidence family=hir",
        "supplemental_evidence family=thir",
        "supplemental_evidence family=mir",
        "status=pilot-supported",
        "unsupported-planned until general extractor evidence is added",
        "optional-supplement-preserves-function-object-v1-identity",
        "toolchain_scope=rustc-version",
        "extractor_identity_expectations=",
        "selected_fixture_shapes=checked-arithmetic,branch-control-flow",
        "selected_fixture_shapes=checked-arithmetic,call-return",
        "fixture_replay_command=",
        "claims_correctness=false",
        "claims_semantic_equivalence=false",
        "claims_cross_toolchain_stability=false",
    ] {
        assert!(stdout.contains(needle), "human contract output missing {needle}: {stdout}");
    }
}

#[test]
fn object_internal_corpus_fixture_satisfies_contract_metadata() {
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
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    let receipt_path = temp.path().join("receipt.json");
    let mut receipt = write_corpus_receipt(temp.path(), &["lib.rs"], &receipt_path);
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
fn object_internal_corpus_receipt_is_deterministic_and_bounded() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let first_path = temp.path().join("internal-corpus-one.json");
    let second_path = temp.path().join("internal-corpus-two.json");
    let first = write_corpus_receipt(&root, INTERNAL_OBJECT_CORPUS, &first_path);
    let second = write_corpus_receipt(&root, INTERNAL_OBJECT_CORPUS, &second_path);
    assert_eq!(fs::read(&first_path).expect("first receipt"), fs::read(&second_path).expect("second receipt"));
    assert_eq!(first, second);

    let original = corpus_summary(&emit_real_code_corpus(&root));
    let object_count = first["object_count"].as_u64().expect("object count");
    assert!(
        object_count > original.object_count as u64,
        "expanded internal corpus should exceed original three-file corpus: receipt={first:#?} original={original:?}"
    );
    assert_eq!(first["source_paths"].as_array().expect("source paths").len(), INTERNAL_OBJECT_CORPUS.len());
    for path in INTERNAL_OBJECT_CORPUS {
        assert!(
            first["source_paths"].as_array().expect("source paths").iter().any(|value| value == path),
            "missing source path {path} in receipt {first:#?}"
        );
    }
    assert!(first["object_set_hash"].as_str().expect("object set hash").starts_with("b3:"));
}

#[test]
fn object_internal_corpus_receipt_is_stable_across_fresh_target_dirs() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let mut receipt_bytes = BTreeSet::new();
    let mut receipts = Vec::new();

    for run in 0..4 {
        let receipt_path = temp.path().join(format!("fresh-target-{run}.json"));
        let target_dir = temp.path().join(format!("cargo-target-{run}"));
        let receipt = write_corpus_receipt_with_fresh_target(&root, INTERNAL_OBJECT_CORPUS, &receipt_path, &target_dir);
        receipt_bytes.insert(fs::read(&receipt_path).expect("read fresh-target receipt bytes"));
        receipts.push(receipt);
    }

    assert_eq!(receipt_bytes.len(), 1, "fresh-target receipt bytes drifted across runs");
    let baseline = &receipts[0];
    for receipt in receipts.iter().skip(1) {
        for field in [
            "schema",
            "schema_version",
            "object_count",
            "source_paths",
            "object_set_hash",
            "caveat_counts",
            "pure_cache_blocked_count",
            "boundary",
        ] {
            assert_eq!(baseline[field], receipt[field], "fresh-target receipt field `{field}` drifted");
        }
    }

    println!("fresh_target.receipt_runs=4");
    println!("fresh_target.receipt_bytes_b3={}", blake3_hex(receipt_bytes.iter().next().unwrap()));
    println!("fresh_target.object_count={}", baseline["object_count"]);
    println!("fresh_target.object_set_hash={}", baseline["object_set_hash"].as_str().expect("object set hash"));
    println!("fresh_target.byte_stability: PASS");
    println!("fresh_target.compared_fields_stable: PASS");
    assert_eq!(baseline["boundary"]["claims_correctness"], false);
    assert_eq!(baseline["boundary"]["claims_semantic_equivalence"], false);
    println!("fresh_target.boundary_claims_false: PASS");
}

#[test]
fn object_internal_corpus_receipt_preserves_boundary_claims() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let receipt_path = temp.path().join("internal-corpus.json");
    let receipt = write_corpus_receipt(&root, INTERNAL_OBJECT_CORPUS, &receipt_path);

    assert_eq!(receipt["schema"], "octet.function-object-corpus-receipt.v1");
    assert_eq!(receipt["boundary"]["identity_scope"], "review-and-provenance-evidence");
    assert_eq!(receipt["boundary"]["claims_correctness"], false);
    assert_eq!(receipt["boundary"]["claims_semantic_equivalence"], false);
    let boundary = receipt["boundary"]["detail"].as_str().expect("boundary detail");
    assert!(boundary.contains("do not prove behavioral correctness"));
    assert!(boundary.contains("semantic equivalence"));
    assert!(receipt["pure_cache_blocked_count"].as_u64().expect("pure cache blocked") > 0);
    assert!(
        receipt["caveat_counts"]["dependency_status"]
            .as_object()
            .expect("dependency counts")
            .values()
            .any(|value| value.as_u64().unwrap_or(0) > 0)
    );
    assert!(
        receipt["caveat_counts"]["effect_kind"]
            .as_object()
            .expect("effect counts")
            .values()
            .any(|value| value.as_u64().unwrap_or(0) > 0)
    );
    assert!(
        receipt["replay"]["determinism_note"]
            .as_str()
            .expect("determinism note")
            .contains("not a correctness proof")
    );
}

#[test]
fn object_corpus_receipt_check_replays_cleanly() {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    let receipt_path = temp.path().join("receipt.json");
    write_corpus_receipt(temp.path(), &["lib.rs"], &receipt_path);

    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    assert!(
        check.status.success(),
        "check stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check report json");
    assert_eq!(report["status"], "match");
    assert!(report["comparisons"].as_array().unwrap().iter().all(|comparison| comparison["status"] == "match"));
}

#[test]
fn object_internal_corpus_gate_fixture_replays_cleanly() {
    let root = repo_root();
    let fixture = root.join(INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE);
    let check = check_corpus_receipt(&root, &fixture, INTERNAL_OBJECT_CORPUS);
    assert!(
        check.status.success(),
        "internal corpus gate stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("internal corpus check report json");
    assert_eq!(report["status"], "match");
    let comparisons = report["comparisons"].as_array().expect("comparisons array");
    assert!(comparisons.iter().all(|comparison| comparison["status"] == "match"), "report={report:#?}");
    assert!(
        comparisons.iter().any(|comparison| {
            comparison["field"] == "boundary"
                && comparison["expected"]["claims_correctness"] == false
                && comparison["current"]["claims_correctness"] == false
                && comparison["expected"]["claims_semantic_equivalence"] == false
                && comparison["current"]["claims_semantic_equivalence"] == false
        }),
        "boundary comparison missing no-correctness/no-semantic-equivalence claims: {report:#?}"
    );
}

#[test]
fn object_internal_corpus_gate_fixture_matches_fresh_receipt() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let fresh_path = temp.path().join("internal-corpus-receipt.json");
    write_corpus_receipt(&root, INTERNAL_OBJECT_CORPUS, &fresh_path);
    let fixture_bytes =
        fs::read(root.join(INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE)).expect("read checked-in receipt fixture");
    let fresh_bytes = fs::read(&fresh_path).expect("read fresh receipt");
    assert_eq!(fresh_bytes, fixture_bytes, "checked-in internal corpus receipt fixture is stale");
}

#[test]
fn object_internal_corpus_expanded_taxonomy_names_selected_source_buckets() {
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
fn object_internal_corpus_expanded_fixture_replays_cleanly() {
    let root = repo_root();
    let taxonomy = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE);
    let paths = expanded_internal_object_corpus_paths(&taxonomy);
    let check = check_corpus_receipt(&root, &root.join(EXPANDED_INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE), &paths);
    assert!(
        check.status.success(),
        "expanded internal corpus stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("expanded check report json");
    assert_eq!(report["status"], "match");
    assert!(report["comparisons"].as_array().unwrap().iter().all(|comparison| comparison["status"] == "match"));
}

#[test]
fn object_internal_corpus_expanded_fixture_matches_fresh_receipt_and_preserves_boundaries() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let taxonomy = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE);
    let paths = expanded_internal_object_corpus_paths(&taxonomy);
    let fresh_path = temp.path().join("expanded-internal-corpus-receipt.json");
    let fresh = write_corpus_receipt(&root, &paths, &fresh_path);
    let fixture_bytes = fs::read(root.join(EXPANDED_INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE))
        .expect("read expanded checked-in receipt fixture");
    let fresh_bytes = fs::read(&fresh_path).expect("read expanded fresh receipt");
    assert_eq!(fresh_bytes, fixture_bytes, "checked-in expanded corpus receipt fixture is stale");
    assert_eq!(fresh["boundary"]["claims_correctness"], false);
    assert_eq!(fresh["boundary"]["claims_semantic_equivalence"], false);
    assert!(fresh["object_count"].as_u64().expect("object count") > 400);
    assert!(fresh["pure_cache_blocked_count"].as_u64().expect("pure cache blocked") > 0);
    assert_eq!(taxonomy["boundary"]["claims_correctness"], false);
    assert_eq!(taxonomy["boundary"]["claims_semantic_equivalence"], false);
}

#[test]
fn object_internal_corpus_expanded_caveat_taxonomy_fixture_matches_fresh_report() {
    let root = repo_root();
    let taxonomy = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE);
    let paths = expanded_internal_object_corpus_paths(&taxonomy);
    let fresh_bytes = object_corpus_caveat_taxonomy(&root, &paths);
    let fixture_bytes = fs::read(root.join(EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE))
        .expect("read expanded caveat taxonomy fixture");
    assert_eq!(fresh_bytes, fixture_bytes, "checked-in caveat taxonomy fixture is stale");
}

#[test]
#[ignore = "explicit fixture refresh mutates checked-in object corpus receipts"]
fn update_internal_object_corpus_fixtures() {
    let root = repo_root();
    let taxonomy = read_json_fixture(&root, EXPANDED_INTERNAL_OBJECT_CORPUS_TAXONOMY_FIXTURE);
    let paths = expanded_internal_object_corpus_paths(&taxonomy);
    write_corpus_receipt(&root, INTERNAL_OBJECT_CORPUS, &root.join(INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE));
    write_corpus_receipt(&root, &paths, &root.join(EXPANDED_INTERNAL_OBJECT_CORPUS_RECEIPT_FIXTURE));
    fs::write(
        root.join(EXPANDED_INTERNAL_OBJECT_CORPUS_CAVEAT_TAXONOMY_FIXTURE),
        object_corpus_caveat_taxonomy(&root, &paths),
    )
    .expect("refresh expanded caveat taxonomy fixture");
}

#[test]
fn object_internal_corpus_caveat_taxonomy_names_per_source_blockers() {
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
fn object_internal_corpus_caveat_taxonomy_preserves_non_proof_boundaries() {
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
fn object_corpus_compatibility_positive_v1_fixture_replays_cleanly() {
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
        check.status.success(),
        "positive compatibility fixture stdout={} stderr={}",
        String::from_utf8_lossy(&check.stdout),
        String::from_utf8_lossy(&check.stderr)
    );
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("positive check report json");
    assert_eq!(report["status"], "match");
    assert!(report["errors"].as_array().expect("errors array").is_empty());
    assert!(
        report["comparisons"]
            .as_array()
            .expect("comparisons array")
            .iter()
            .all(|comparison| comparison["status"] == "match"),
        "positive compatibility comparisons did not all match: {report:#?}"
    );
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

#[test]
fn object_corpus_receipt_check_fails_on_source_drift() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(&fixture, "pub fn checked_value(input: u32) -> u32 { input + 1 }\n").expect("write fixture");
    let receipt_path = temp.path().join("receipt.json");
    write_corpus_receipt(temp.path(), &["lib.rs"], &receipt_path);
    fs::write(&fixture, "pub fn checked_value(input: u32) -> u32 { input + 2 }\n").expect("mutate fixture");

    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    assert!(!check.status.success(), "drift check unexpectedly passed");
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check report json");
    assert_eq!(report["status"], "mismatch");
    assert!(
        report["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|comparison| { comparison["field"] == "object_set_hash" && comparison["status"] == "mismatch" }),
        "expected object_set_hash mismatch in {report:#?}"
    );
}

#[test]
fn object_corpus_receipt_check_fails_on_boundary_tamper() {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    let receipt_path = temp.path().join("receipt.json");
    let mut receipt = write_corpus_receipt(temp.path(), &["lib.rs"], &receipt_path);
    receipt["boundary"]["claims_correctness"] = serde_json::Value::Bool(true);
    fs::write(
        &receipt_path,
        format!("{}\n", serde_json::to_string_pretty(&receipt).expect("tampered receipt json")),
    )
    .expect("write tampered receipt");

    let check = check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]);
    assert!(!check.status.success(), "boundary tamper check unexpectedly passed");
    let report: serde_json::Value = serde_json::from_slice(&check.stdout).expect("check report json");
    assert_eq!(report["status"], "mismatch");
    assert!(
        report["comparisons"]
            .as_array()
            .unwrap()
            .iter()
            .any(|comparison| { comparison["field"] == "boundary" && comparison["status"] == "mismatch" }),
        "expected boundary mismatch in {report:#?}"
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
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    let receipt_path = temp.path().join("receipt.json");
    let mut receipt = write_corpus_receipt(temp.path(), &["lib.rs"], &receipt_path);

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
fn object_corpus_receipt_check_rejects_missing_required_fields() {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    let receipt_path = temp.path().join("receipt.json");
    let mut receipt = write_corpus_receipt(temp.path(), &["lib.rs"], &receipt_path);
    receipt.as_object_mut().expect("receipt object").remove("object_set_hash");
    fs::write(&receipt_path, format!("{}\n", serde_json::to_string_pretty(&receipt).unwrap()))
        .expect("write missing field receipt");

    let report =
        assert_invalid_check_error(&check_corpus_receipt(temp.path(), &receipt_path, &["lib.rs"]), "missing-field");
    assert!(
        report["errors"]
            .as_array()
            .unwrap()
            .iter()
            .any(|error| { error["kind"] == "missing-field" && error["field"] == "object_set_hash" }),
        "missing object_set_hash field not named in {report:#?}"
    );
}

#[test]
fn object_corpus_receipt_check_rejects_invalid_source_selection() {
    let temp = tempfile::tempdir().expect("create tempdir");
    fs::write(temp.path().join("lib.rs"), "pub fn checked_value(input: u32) -> u32 { input + 1 }\n")
        .expect("write fixture");
    fs::write(temp.path().join("README.md"), "not rust\n").expect("write non-rust fixture");
    fs::write(temp.path().join("empty.rs"), "// no functions here\n").expect("write empty rust fixture");
    let receipt_path = temp.path().join("receipt.json");
    write_corpus_receipt(temp.path(), &["lib.rs"], &receipt_path);

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

#[test]
fn object_real_code_harness_emits_stable_summary() {
    let root = repo_root();
    let first = emit_real_code_corpus(&root);
    let second = emit_real_code_corpus(&root);
    assert_eq!(first, second);

    let summary = corpus_summary(&first);
    assert!(summary.object_count >= 30, "expected broad real-code corpus coverage, saw {summary:?}");
    assert_eq!(
        summary.source_paths,
        BTreeSet::from([
            "cargo-octet/src/cli.rs".to_owned(),
            "cargo-octet/src/fingerprint.rs".to_owned(),
            "cargo-octet/src/object.rs".to_owned(),
        ])
    );
    assert!(summary.macro_review_required > 0, "expected macro caveats in real corpus: {summary:?}");
    assert!(summary.panic_caveats > 0, "expected panic caveats in real corpus: {summary:?}");
    assert_eq!(summary.pure_cache_blocked, summary.object_count);
    assert!(
        first["objects"]
            .as_array()
            .unwrap()
            .iter()
            .all(|object| object["correctness_claim"]["claims_correctness"] == false)
    );
}

#[test]
fn object_harness_negative_cases_are_classified() {
    let temp = tempfile::tempdir().expect("create tempdir");
    let fixture = temp.path().join("lib.rs");
    fs::write(
        &fixture,
        r"pub fn changed_value(input: u32) -> u32 {
    input + 1
}
",
    )
    .expect("write initial fixture");
    let initial = emit_objects_json(temp.path(), &["lib.rs"]);
    let initial_hash = object_hash(&initial, "changed_value");

    fs::write(
        &fixture,
        r"// comment-only entropy should not affect object identity
pub fn changed_value( input: u32 ) -> u32 { input + 1 }
",
    )
    .expect("write formatted fixture");
    let formatted = emit_objects_json(temp.path(), &["lib.rs"]);
    assert_eq!(initial_hash, object_hash(&formatted, "changed_value"));

    fs::write(&fixture, "pub fn changed_value(input: u32) -> u32 { input + 2 }\n").expect("write body mutation");
    let changed = emit_objects_json(temp.path(), &["lib.rs"]);
    assert_ne!(initial_hash, object_hash(&changed, "changed_value"));

    fs::write(
        &fixture,
        r#"pub fn macro_heavy(input: Option<u32>) -> u32 {
    format!("{}", input.unwrap()).len() as u32
}
"#,
    )
    .expect("write macro-heavy fixture");
    let macro_json = emit_objects_json(temp.path(), &["lib.rs"]);
    let macro_object = &macro_json["objects"][0];
    assert_eq!(macro_object["macro_caveats"][0]["status"], "review-required");
    assert!(macro_object["effect_caveats"].as_array().unwrap().iter().any(|caveat| caveat["kind"] == "panic"));
    assert_eq!(macro_object["pure_cache_eligibility"]["status"], "blocked-by-caveats");
}

#[test]
fn object_harness_bundle_determinism_and_pointer_failures() {
    let root = repo_root();
    let temp = tempfile::tempdir().expect("create tempdir");
    let objects_path = temp.path().join("real-code-objects.json");
    let objects_json = write_objects(&root, REAL_CODE_CORPUS, &objects_path);
    let first_bundle = temp.path().join("real-code-bundle-one.json");
    let second_bundle = temp.path().join("real-code-bundle-two.json");

    for output_path in [&first_bundle, &second_bundle] {
        let bundle = export_bundle(&root, &objects_path, None, output_path);
        assert!(bundle.status.success(), "real-code bundle stderr: {}", String::from_utf8_lossy(&bundle.stderr));
    }
    let first_bundle_text = fs::read_to_string(&first_bundle).expect("read first bundle");
    let second_bundle_text = fs::read_to_string(&second_bundle).expect("read second bundle");
    assert_eq!(first_bundle_text, second_bundle_text);
    let bundle_json: serde_json::Value = serde_json::from_str(&first_bundle_text).expect("bundle json");
    assert_eq!(bundle_json["object_count"], objects_json["objects"].as_array().unwrap().len());
    assert_eq!(bundle_json["pointer_count"], 0);
    assert!(bundle_json["bundle_hash"].as_str().unwrap().starts_with("b3:"));
    assert!(bundle_json["replay"]["command"].as_str().unwrap().contains("cargo-octet/src/object.rs"));

    let pointers = temp.path().join("missing-pointer.json");
    fs::write(
        &pointers,
        r#"{"schema_version":1,"pointers":{"crate::missing":{"object_hash":"b3:0000000000000000000000000000000000000000000000000000000000000000","semantic_name":"crate::missing"}},"history":{}}"#,
    )
    .expect("write missing pointer manifest");
    let failed_bundle = export_bundle(&root, &objects_path, Some(&pointers), &temp.path().join("bad-bundle.json"));
    assert!(!failed_bundle.status.success(), "bundle with unresolved pointer unexpectedly passed");
    assert!(
        String::from_utf8_lossy(&failed_bundle.stderr).contains("pointer")
            || String::from_utf8_lossy(&failed_bundle.stdout).contains("missing-object"),
        "missing pointer failure was not classified: stdout={} stderr={}",
        String::from_utf8_lossy(&failed_bundle.stdout),
        String::from_utf8_lossy(&failed_bundle.stderr)
    );
}

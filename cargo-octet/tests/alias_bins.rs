use std::collections::BTreeSet;
use std::path::PathBuf;
use std::process::Command;
use std::process::Output;

const ALIAS_BINARY_COUNT: usize = 3;
const PACKAGE_MANIFEST_COUNT: usize = 2;
const BIN_SECTION: &str = "[[bin]]";
const PATH_FIELD_PREFIX: &str = "path = ";
const QUOTED_VALUE_DELIMITER: char = '"';
const EXPECTED_VERSION_OUTPUT: &str = concat!("cargo-octet ", env!("CARGO_PKG_VERSION"), "\n");
const ROOT_HELP_NEEDLE: &str = "Octet operator commands";
const DUPLICATE_TARGET_WARNING: &str = "found to be present in multiple build targets";
const DUPLICATE_TARGET_DIAGNOSTIC: &str = "duplicate-target warning text was present in Cargo output";
const DUPLICATE_SOURCE_PATH: &str = "src/main.rs";

#[test]
fn canonical_and_compatibility_bins_preserve_help_and_version() {
    for (name, binary) in cargo_octet_alias_bins() {
        let version = Command::new(binary).arg("--version").output().expect("version command should run");
        assert_success(name, &version);
        assert_eq!(String::from_utf8_lossy(&version.stdout), EXPECTED_VERSION_OUTPUT);

        let help = Command::new(binary).arg("--help").output().expect("help command should run");
        assert_success(name, &help);
        assert!(String::from_utf8_lossy(&help.stdout).contains(ROOT_HELP_NEEDLE));
    }
}

#[test]
fn owned_alias_bin_targets_use_distinct_wrapper_paths() {
    for manifest_path in package_manifests() {
        let manifest = std::fs::read_to_string(&manifest_path)
            .unwrap_or_else(|error| panic!("read {}: {error}", manifest_path.display()));
        let duplicates = duplicate_bin_paths(&manifest);
        assert!(duplicates.is_empty(), "{} has duplicate bin target paths: {duplicates:?}", manifest_path.display());
    }
}

#[test]
fn duplicate_bin_path_detector_rejects_reused_target_source() {
    let manifest = format!(
        "{BIN_SECTION}\nname = \"cargo-octet\"\npath = \"{DUPLICATE_SOURCE_PATH}\"\n\n{BIN_SECTION}\nname = \"cargo-slotcar\"\npath = \"{DUPLICATE_SOURCE_PATH}\"\n"
    );

    assert_eq!(duplicate_bin_paths(&manifest), vec![DUPLICATE_SOURCE_PATH.to_owned()]);
}

#[test]
fn duplicate_target_warning_detector_rejects_cargo_warning_text() {
    let output = format!("warning: file `{DUPLICATE_SOURCE_PATH}` {DUPLICATE_TARGET_WARNING}\n");

    assert_eq!(warning_clean_diagnostics(&output), vec![DUPLICATE_TARGET_DIAGNOSTIC]);
}

#[test]
fn duplicate_target_warning_detector_accepts_clean_cargo_output() {
    let output = "Finished `dev` profile [unoptimized + debuginfo] target(s)";

    assert!(warning_clean_diagnostics(output).is_empty());
}

fn cargo_octet_alias_bins() -> [(&'static str, &'static str); ALIAS_BINARY_COUNT] {
    [
        ("cargo-octet", env!("CARGO_BIN_EXE_cargo-octet")),
        ("cargo-slotcar", env!("CARGO_BIN_EXE_cargo-slotcar")),
        ("cargo-tigerstyle", env!("CARGO_BIN_EXE_cargo-tigerstyle")),
    ]
}

fn package_manifests() -> [PathBuf; PACKAGE_MANIFEST_COUNT] {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    [
        manifest_dir.join("Cargo.toml"),
        manifest_dir.join("../crates/octet-standards/Cargo.toml"),
    ]
}

fn duplicate_bin_paths(manifest: &str) -> Vec<String> {
    let mut paths = Vec::new();
    let mut in_bin_section = false;

    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed == BIN_SECTION {
            in_bin_section = true;
            continue;
        }
        if trimmed.starts_with('[') {
            in_bin_section = false;
        }
        if !in_bin_section || !trimmed.starts_with(PATH_FIELD_PREFIX) {
            continue;
        }
        if let Some(path) = quoted_value(trimmed) {
            paths.push(path.to_owned());
        }
    }

    duplicate_values(paths)
}

fn quoted_value(line: &str) -> Option<&str> {
    let mut parts = line.split(QUOTED_VALUE_DELIMITER);
    parts.next()?;
    parts.next()
}

fn duplicate_values(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut duplicates = BTreeSet::new();
    for value in values {
        if !seen.insert(value.clone()) {
            duplicates.insert(value);
        }
    }
    duplicates.into_iter().collect()
}

fn warning_clean_diagnostics(output: &str) -> Vec<&'static str> {
    if output.contains(DUPLICATE_TARGET_WARNING) {
        vec![DUPLICATE_TARGET_DIAGNOSTIC]
    } else {
        Vec::new()
    }
}

fn assert_success(name: &str, output: &Output) {
    assert!(
        output.status.success(),
        "{name} failed; stdout was: {}\nstderr was: {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

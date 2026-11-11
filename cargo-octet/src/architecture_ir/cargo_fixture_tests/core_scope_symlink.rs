use std::fs;
use std::path::Path;
use std::process::Command;

use super::TOOLCHAIN;
use super::required_artifact;

const STRICT_PROFILE: &str = octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV;
const FS_CALL: &str =
    "pub fn load(path: &std::path::Path) -> std::io::Result<String> {\n    std::fs::read_to_string(path)\n}\n";

fn write_fixture_crate(root: &Path, lib_rs: &str, scope: &str) {
    fs::create_dir_all(root.join("src")).expect("fixture source directory");
    fs::write(
        root.join("Cargo.toml"),
        "[package]\nname = \"core-scope-symlink-fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .expect("fixture manifest");
    fs::write(root.join("src/lib.rs"), lib_rs).expect("fixture source");
    fs::write(root.join("dylint.toml"), format!("[octet]\ncore_source_scopes = [\"{scope}\"]\n"))
        .expect("fixture config");
}

/// Compiler messages from `cargo check` through the Dylint driver.
fn check_messages(root: &Path, strict_profile: Option<&str>) -> Vec<serde_json::Value> {
    let library = required_artifact("OCTET_UI_LIBRARY");
    let driver = required_artifact("OCTET_UI_DRIVER");
    let mut command = Command::new("cargo");
    command
        .current_dir(root)
        .args(["check", "--message-format=json"])
        .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
        .env("CARGO_TARGET_DIR", root.join("target"))
        .env("RUSTC_WRAPPER", "")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("DYLINT_NO_DEPS", "1")
        .env("DYLINT_LIBS", serde_json::to_string(&[library]).expect("library selection"))
        .env_remove("DYLINT_RUSTFLAGS")
        .env_remove(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV)
        .env_remove(STRICT_PROFILE);
    if let Some(profile) = strict_profile {
        command.env(STRICT_PROFILE, profile);
    }
    let output = command.output().expect("check fixture through the Dylint driver");
    String::from_utf8(output.stdout)
        .expect("Cargo JSON is UTF-8")
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter(|message| message["reason"] == "compiler-message")
        .collect()
}

/// Level and file of every `impure_call_in_core` finding.
fn impure_calls(messages: &[serde_json::Value]) -> Vec<(&str, &str)> {
    messages
        .iter()
        .filter(|message| {
            message["message"]["code"]["code"].as_str().and_then(|code| code.rsplit("::").next())
                == Some("impure_call_in_core")
        })
        .map(|message| {
            let span = &message["message"]["spans"][0];
            (
                message["message"]["level"].as_str().expect("diagnostic level"),
                span["file_name"].as_str().expect("span file"),
            )
        })
        .collect()
}

fn config_warnings(messages: &[serde_json::Value]) -> Vec<&str> {
    messages
        .iter()
        .filter_map(|message| message["message"]["message"].as_str())
        .filter(|text| text.contains("core_source_scopes"))
        .collect()
}

// `cargo octet check --profile deterministic-core` accepts an in-root symlinked
// core scope (`linked -> src`), while rustc names the compiled file `src/lib.rs`.
// The lint library must resolve the scope the same way, so core lints still
// report through it.
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_symlinked_core_scope_lints_its_target() {
    let temporary = tempfile::TempDir::new().expect("fixture crate");
    let root = temporary.path();
    write_fixture_crate(root, FS_CALL, "linked");
    std::os::unix::fs::symlink(root.join("src"), root.join("linked")).expect("in-root scope symlink");

    let messages = check_messages(root, Some("deterministic-core"));

    assert_eq!(
        impure_calls(&messages),
        [("error", "src/lib.rs")],
        "the symlinked scope covers its target: {messages:#?}"
    );
    assert!(
        config_warnings(&messages).is_empty(),
        "an in-root symlinked scope is valid and needs no config warning: {messages:#?}"
    );
}

// rustc names a module reached through an in-root symlink by its in-root path
// (`src/domain/mod.rs`), even when the symlink leads outside the workspace, so
// the scope's own text must keep matching it.
#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_scope_through_an_outside_symlink_lints_its_in_root_path() {
    let temporary = tempfile::TempDir::new().expect("fixture parent");
    let root = temporary.path().join("workspace");
    let shared = temporary.path().join("shared/domain");
    fs::create_dir_all(&shared).expect("shared module directory");
    fs::write(shared.join("mod.rs"), FS_CALL).expect("shared module");
    write_fixture_crate(&root, "pub mod domain;\n", "src/domain");
    std::os::unix::fs::symlink(&shared, root.join("src/domain")).expect("outside-leading module symlink");

    let messages = check_messages(&root, None);

    assert_eq!(
        impure_calls(&messages),
        [("warning", "src/domain/mod.rs")],
        "the scope covers files compiled through its in-root path: {messages:#?}"
    );
    let warnings = config_warnings(&messages);
    assert_eq!(warnings.len(), 1, "the outside target is reported: {messages:#?}");
    assert!(warnings[0].contains("`src/domain`"), "{warnings:?}");
}

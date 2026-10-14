use std::collections::HashMap;
use std::env;
use std::fs;
#[cfg(unix)]
use std::os::unix::fs::symlink;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct UiFixtureSet {
    all_fixtures: Vec<String>,
    expected_failures: Vec<String>,
}

fn fixture_set() -> UiFixtureSet {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let path = manifest_dir.join("tests/fixtures/ui_fixture_set.json");
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("read UI fixture-set artifact {}: {error}", path.display()));
    serde_json::from_str(&raw)
        .unwrap_or_else(|error| panic!("parse UI fixture-set artifact {}: {error}", path.display()))
}

#[cfg(unix)]
fn create_library_alias(plain: &Path, dylint: &Path) {
    symlink(plain, dylint).expect("create dylint library alias");
}

#[cfg(not(unix))]
fn create_library_alias(plain: &Path, dylint: &Path) {
    fs::copy(plain, dylint).expect("copy dylint library alias");
}

fn temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now().duration_since(UNIX_EPOCH).expect("system time before unix epoch").as_nanos();
    env::temp_dir().join(format!("{prefix}-{}-{unique}", std::process::id()))
}

fn command_stdout(command: &mut Command, context: &str) -> String {
    let output = command.output().expect(context);
    assert!(
        output.status.success(),
        "{context} failed:\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).expect("utf8 stdout")
}

fn cargo_command() -> Command {
    let mut command = Command::new("cargo");
    command.env("CARGO_BUILD_RUSTC_WRAPPER", "");
    command
}

fn rustup_toolchain() -> String {
    if let Some(toolchain) = env::var_os("RUSTUP_TOOLCHAIN") {
        return toolchain.into_string().expect("utf8 RUSTUP_TOOLCHAIN for UI tests");
    }

    let stdout = command_stdout(Command::new("rustup").args(["show", "active-toolchain"]), "detect rustup toolchain");
    stdout.split_ascii_whitespace().next().expect("active rustup toolchain").to_owned()
}

fn host_triple() -> String {
    let stdout = command_stdout(Command::new("rustc").args(["-vV"]), "detect rustc host triple");
    stdout.lines().find_map(|line| line.strip_prefix("host: ")).expect("rustc host line").to_owned()
}

fn prebuilt_path(name: &str) -> Option<PathBuf> {
    env::var_os(name).map(PathBuf::from)
}

#[derive(Clone)]
struct FixtureSupport {
    dependency_search_path: PathBuf,
    externs: Vec<(String, PathBuf)>,
}

fn fixture_support_manifest(fixture_path: &Path) -> Option<PathBuf> {
    let manifest_path = fixture_path.parent()?.join("Cargo.toml");
    manifest_path.exists().then_some(manifest_path)
}

fn fixture_dependency_names(manifest_path: &Path) -> Vec<String> {
    let manifest = fs::read_to_string(manifest_path)
        .unwrap_or_else(|error| panic!("read fixture manifest {}: {error}", manifest_path.display()));
    let parsed = toml::from_str::<toml::Table>(&manifest)
        .unwrap_or_else(|error| panic!("parse fixture manifest {}: {error}", manifest_path.display()));

    parsed
        .get("dependencies")
        .and_then(toml::Value::as_table)
        .map(|deps| deps.keys().map(|name| name.replace('-', "_")).collect())
        .unwrap_or_default()
}

const RMETA_ARTIFACT_RANK: u8 = 0;
const RLIB_ARTIFACT_RANK: u8 = 1;
const DYNAMIC_LIBRARY_ARTIFACT_RANK: u8 = 2;

fn path_extension_matches(path: &Path, extension: &str) -> bool {
    path.extension()
        .and_then(|candidate| candidate.to_str())
        .is_some_and(|candidate| candidate.eq_ignore_ascii_case(extension))
}

fn extern_artifact_path(deps_dir: &Path, crate_name: &str) -> PathBuf {
    let prefix = format!("{}{}-", std::env::consts::DLL_PREFIX, crate_name);
    let dynamic_library_extension = std::env::consts::DLL_SUFFIX.trim_start_matches('.');
    let mut candidates = fs::read_dir(deps_dir)
        .unwrap_or_else(|error| panic!("read dependency dir {}: {error}", deps_dir.display()))
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let file_name = path.file_name()?.to_str()?;
            if !file_name.starts_with(&prefix) {
                return None;
            }

            let rank = if path_extension_matches(&path, "rmeta") {
                RMETA_ARTIFACT_RANK
            } else if path_extension_matches(&path, "rlib") {
                RLIB_ARTIFACT_RANK
            } else if path_extension_matches(&path, dynamic_library_extension) {
                DYNAMIC_LIBRARY_ARTIFACT_RANK
            } else {
                return None;
            };

            Some((rank, path))
        })
        .collect::<Vec<_>>();

    candidates.sort_by(|(left_rank, left_path), (right_rank, right_path)| {
        left_rank.cmp(right_rank).then_with(|| left_path.file_name().cmp(&right_path.file_name()))
    });

    candidates.into_iter().next().map_or_else(
        || panic!("missing extern artifact for crate `{crate_name}` under {}", deps_dir.display()),
        |(_, path)| path,
    )
}

fn build_fixture_support(toolchain: &str, manifest_path: &Path, dependency_names: &[String]) -> FixtureSupport {
    let target_dir = temp_path("octet-ui-fixture-deps");
    let status = cargo_command()
        .current_dir(manifest_path.parent().expect("fixture manifest parent"))
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["check", "--quiet", "--manifest-path"])
        .arg(manifest_path)
        .status()
        .expect("build fixture support dependencies");
    assert!(status.success(), "failed to build fixture support deps for {}", manifest_path.display());

    let dependency_search_path = target_dir.join("debug/deps");
    let externs = dependency_names
        .iter()
        .map(|name| (name.clone(), extern_artifact_path(&dependency_search_path, name)))
        .collect();

    FixtureSupport {
        dependency_search_path,
        externs,
    }
}

fn fixture_support_for<'a>(
    toolchain: &str,
    fixture_path: &Path,
    support_cache: &'a mut HashMap<PathBuf, FixtureSupport>,
) -> Option<&'a FixtureSupport> {
    let manifest_path = fixture_support_manifest(fixture_path)?;
    let dependency_names = fixture_dependency_names(&manifest_path);
    if dependency_names.is_empty() {
        return None;
    }

    if !support_cache.contains_key(&manifest_path) {
        let support = build_fixture_support(toolchain, &manifest_path, &dependency_names);
        support_cache.insert(manifest_path.clone(), support);
    }

    support_cache.get(&manifest_path)
}

fn build_library(toolchain: &str) -> PathBuf {
    if let Some(path) = prebuilt_path("OCTET_UI_LIBRARY").or_else(|| prebuilt_path("TIGERSTYLE_UI_LIBRARY")) {
        assert!(path.exists(), "missing prebuilt UI library: {}", path.display());
        return path;
    }

    let status = cargo_command()
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["build", "--lib"])
        .status()
        .expect("build UI test library");
    assert!(status.success(), "cargo build --lib failed for UI tests");

    let target_dir = env::var_os("CARGO_TARGET_DIR")
        .map_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("target"), PathBuf::from);
    let debug_dir = target_dir.join("debug");
    let lib_name = env!("CARGO_PKG_NAME").replace('-', "_");
    let plain = debug_dir.join(format!("{}{}{}", std::env::consts::DLL_PREFIX, lib_name, std::env::consts::DLL_SUFFIX));
    let dylint = debug_dir.join(format!(
        "{}{}@{}{}",
        std::env::consts::DLL_PREFIX,
        lib_name,
        toolchain,
        std::env::consts::DLL_SUFFIX
    ));

    if dylint.exists() {
        fs::remove_file(&dylint).expect("remove stale dylint library alias");
    }
    create_library_alias(&plain, &dylint);
    dylint
}

fn build_driver(toolchain: &str) -> PathBuf {
    if let Some(path) = prebuilt_path("OCTET_UI_DRIVER").or_else(|| prebuilt_path("TIGERSTYLE_UI_DRIVER")) {
        assert!(path.exists(), "missing prebuilt UI driver: {}", path.display());
        return path;
    }

    let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/support/ui-driver/Cargo.toml");
    let driver_target_dir = temp_path("octet-ui-driver-target");
    let status = cargo_command()
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CARGO_TARGET_DIR", &driver_target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .args(["build", "--quiet", "--locked", "--manifest-path"])
        .arg(&manifest_path)
        .status()
        .expect("build UI driver");
    assert!(status.success(), "failed to build UI driver");

    let debug_dir = driver_target_dir.join("debug");
    let candidates = [
        debug_dir.join(format!("octet-ui-driver{}", std::env::consts::EXE_SUFFIX)),
        debug_dir.join(format!("octet_ui_driver{}", std::env::consts::EXE_SUFFIX)),
    ];
    candidates
        .into_iter()
        .find(|path| path.exists())
        .unwrap_or_else(|| panic!("UI driver binary missing under {}", debug_dir.display()))
}

fn normalize_stderr(stderr: &str, fixture_path: &Path) -> String {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let ui_dir = manifest_dir.join("ui_tests");
    let relative = fixture_path.strip_prefix(&ui_dir).expect("fixture under ui_tests").to_string_lossy();
    stderr
        .replace("\r\n", "\n")
        .replace(fixture_path.to_string_lossy().as_ref(), &format!("$DIR/{relative}"))
        .replace(manifest_dir.to_string_lossy().as_ref(), "$ROOT")
}

fn extra_rustc_args(name: &str) -> &'static [&'static str] {
    match name {
        "no_panic"
        | "no_todo"
        | "no_unwrap"
        | "drop_must_use"
        | "fallible_int_fallback"
        | "magic_number"
        | "explicit_defaults/explicit_defaults"
        | "too_many_parameters"
        | "sentinel_fallback" => &["--cfg", "test"],
        _ => &[],
    }
}

fn bless_enabled() -> bool {
    env::var_os("BLESS").is_some()
}

fn run_fixture(
    driver: &Path,
    library: &Path,
    host: &str,
    toolchain: &str,
    name: &str,
    expect_success: bool,
    support_cache: &mut HashMap<PathBuf, FixtureSupport>,
) {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let fixture_path = ui_dir.join(format!("{name}.rs"));
    let expected_name = fixture_path.file_stem().expect("fixture file stem").to_string_lossy();
    let expected_path = ui_dir.join(format!("{expected_name}.stderr"));
    let work_dir = temp_path(&format!("octet-ui-case-{name}"));
    let aux_dir = work_dir.join("aux");
    fs::create_dir_all(&aux_dir).expect("create UI aux dir");

    let library = if library.is_absolute() {
        library.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(library)
    };
    let dylint_libs = serde_json::to_string(&vec![library]).expect("encode DYLINT_LIBS");
    let output_path = work_dir.join(name);
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).expect("create UI output dir");
    }

    let mut command = Command::new(driver);
    command
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .env("CLIPPY_DISABLE_DOCS_LINKS", "true")
        .env("DYLINT_LIBS", dylint_libs)
        .args(extra_rustc_args(name));

    if let Some(support) = fixture_support_for(toolchain, &fixture_path, support_cache) {
        command.arg("-L").arg(format!("dependency={}", support.dependency_search_path.display()));
        for (crate_name, artifact_path) in &support.externs {
            command.arg("--extern").arg(format!("{crate_name}={}", artifact_path.display()));
        }
    }

    let output = command
        .arg(&fixture_path)
        .arg("-L")
        .arg(&work_dir)
        .arg(format!("--target={host}"))
        .args(["--error-format", "human"])
        .args(["-C", "prefer-dynamic"])
        .arg("-o")
        .arg(&output_path)
        .args(["-A", "unused"])
        .arg("--edition=2024")
        .args(["--emit=metadata", "-Zui-testing"])
        .arg("-L")
        .arg(&aux_dir)
        .output()
        .expect("run UI fixture");

    assert!(
        output.stdout.is_empty(),
        "fixture {name} produced unexpected stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    assert_eq!(
        output.status.success(),
        expect_success,
        "fixture {name} exit-status mismatch: expected success={expect_success}, got status={}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    let actual = normalize_stderr(&String::from_utf8(output.stderr).expect("utf8 stderr"), &fixture_path);
    if bless_enabled() {
        fs::write(&expected_path, actual)
            .unwrap_or_else(|error| panic!("write expected stderr {}: {error}", expected_path.display()));
        return;
    }

    let expected = fs::read_to_string(&expected_path)
        .unwrap_or_else(|error| panic!("read expected stderr {}: {error}", expected_path.display()));
    assert_eq!(expected.trim_end_matches('\n'), actual.trim_end_matches('\n'), "stderr mismatch for fixture {name}");
}

fn fixed_fixture_names() -> Vec<String> {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let mut fixtures = fs::read_dir(&ui_dir)
        .unwrap_or_else(|error| panic!("read fixed fixture dir {}: {error}", ui_dir.display()))
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| panic!("collect fixed fixture dir {}: {error}", ui_dir.display()))
        .into_iter()
        .filter_map(|entry| {
            let path = entry.path();
            (path.extension().and_then(|ext| ext.to_str()) == Some("fixed"))
                .then(|| path.file_stem().expect("fixed fixture file stem").to_string_lossy().into_owned())
        })
        .collect::<Vec<_>>();
    fixtures.sort();
    fixtures
}

fn cargo_fix_autofix_fixture_command(crate_dir: &Path, driver: &Path, dylint_libs: &str, toolchain: &str) -> Command {
    let mut command = cargo_command();
    command
        .current_dir(crate_dir)
        .env("CLIPPY_DISABLE_DOCS_LINKS", "true")
        .env("DYLINT_LIBS", dylint_libs)
        .env("DYLINT_NO_DEPS", "1")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .env("CARGO_TARGET_DIR", crate_dir.join("target"))
        .args([
            "fix",
            "--allow-dirty",
            "--allow-staged",
            "--allow-no-vcs",
            "--bin",
            "autofix_fixture",
        ]);
    command
}

fn run_autofix_fixture(driver: &Path, library: &Path, toolchain: &str, name: &str) {
    let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
    let fixture_path = ui_dir.join(format!("{name}.rs"));
    let fixed_path = ui_dir.join(format!("{name}.fixed"));
    let expected = fs::read_to_string(&fixed_path)
        .unwrap_or_else(|error| panic!("read expected fixed output {}: {error}", fixed_path.display()));
    let source = fs::read_to_string(&fixture_path)
        .unwrap_or_else(|error| panic!("read autofix fixture {}: {error}", fixture_path.display()));

    let library = if library.is_absolute() {
        library.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join(library)
    };
    let dylint_libs = serde_json::to_string(&vec![library]).expect("encode DYLINT_LIBS for autofix fixture");

    let crate_dir = temp_path(&format!("octet-autofix-{name}"));
    let src_dir = crate_dir.join("src");
    fs::create_dir_all(&src_dir).expect("create autofix src dir");
    fs::write(
        crate_dir.join("Cargo.toml"),
        "[package]\nname = \"autofix_fixture\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[workspace]\n",
    )
    .expect("write autofix Cargo.toml");
    let main_rs = src_dir.join("main.rs");
    fs::write(&main_rs, source).expect("write autofix main.rs");

    let mut fix = cargo_fix_autofix_fixture_command(&crate_dir, driver, &dylint_libs, toolchain);

    eprintln!("[autofix fixture] {name}");
    eprintln!(
        "[autofix command] (cd {} && cargo fix --allow-dirty --allow-staged --allow-no-vcs --bin autofix_fixture)",
        crate_dir.display()
    );
    let fix_output = fix.output().expect("run cargo fix for autofix fixture");
    let fix_stdout = String::from_utf8(fix_output.stdout).expect("utf8 cargo fix stdout");
    let fix_stderr = String::from_utf8(fix_output.stderr).expect("utf8 cargo fix stderr");
    if !fix_stdout.is_empty() {
        eprintln!("[autofix stdout]\n{fix_stdout}");
    }
    if !fix_stderr.is_empty() {
        eprintln!("[autofix stderr]\n{fix_stderr}");
    }
    assert!(
        fix_output.status.success(),
        "cargo fix failed for fixture {name}:\nstdout:\n{fix_stdout}\nstderr:\n{fix_stderr}"
    );

    let actual = fs::read_to_string(&main_rs).expect("read autofixed main.rs");
    assert_eq!(expected, actual, "autofix output mismatch for fixture {name}");

    eprintln!(
        "[autofix idempotence command] (cd {} && cargo fix --allow-dirty --allow-staged --allow-no-vcs --bin autofix_fixture)",
        crate_dir.display()
    );
    let mut second_fix = cargo_fix_autofix_fixture_command(&crate_dir, driver, &dylint_libs, toolchain);
    let second_fix_output = second_fix.output().expect("rerun cargo fix for autofix fixture");
    let second_fix_stdout = String::from_utf8(second_fix_output.stdout).expect("utf8 second cargo fix stdout");
    let second_fix_stderr = String::from_utf8(second_fix_output.stderr).expect("utf8 second cargo fix stderr");
    if !second_fix_stdout.is_empty() {
        eprintln!("[autofix idempotence stdout]\n{second_fix_stdout}");
    }
    if !second_fix_stderr.is_empty() {
        eprintln!("[autofix idempotence stderr]\n{second_fix_stderr}");
    }
    assert!(
        second_fix_output.status.success(),
        "second cargo fix failed for fixture {name}:\nstdout:\n{second_fix_stdout}\nstderr:\n{second_fix_stderr}"
    );
    let second_actual = fs::read_to_string(&main_rs).expect("read twice-autofixed main.rs");
    assert_eq!(actual, second_actual, "autofix fixture {name} changed on second cargo fix run");

    eprintln!(
        "[autofix post-check command] (cd {} && cargo check --quiet --bin autofix_fixture)",
        crate_dir.display()
    );
    let mut check = cargo_command();
    check
        .current_dir(&crate_dir)
        .env("CLIPPY_DISABLE_DOCS_LINKS", "true")
        .env("DYLINT_LIBS", &dylint_libs)
        .env("DYLINT_NO_DEPS", "1")
        .env("RUSTC_WORKSPACE_WRAPPER", driver)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .env("CARGO_TARGET_DIR", crate_dir.join("target"))
        .args(["check", "--quiet", "--bin", "autofix_fixture"]);

    let check_output = check.output().expect("run cargo check for autofix fixture");
    let check_stdout = String::from_utf8(check_output.stdout).expect("utf8 cargo check stdout");
    let check_stderr = String::from_utf8(check_output.stderr).expect("utf8 cargo check stderr");
    if !check_stdout.is_empty() {
        eprintln!("[autofix post-check stdout]\n{check_stdout}");
    }
    if !check_stderr.is_empty() {
        eprintln!("[autofix post-check stderr]\n{check_stderr}");
    }
    assert!(
        check_output.status.success(),
        "cargo check failed for autofixed fixture {name}:\nstdout:\n{check_stdout}\nstderr:\n{check_stderr}"
    );
    let combined_check_output = format!("{check_stdout}{check_stderr}");
    assert!(
        !combined_check_output.contains("warning:"),
        "autofixed fixture {name} still emitted warnings:\n{combined_check_output}"
    );
}

#[test]
#[ignore = "run via nix flake check or cargo test --test ui -- --ignored --nocapture"]
fn ui() {
    let fixture_set = fixture_set();
    let expected_failures = fixture_set.expected_failures.into_iter().collect::<std::collections::HashSet<_>>();
    let toolchain = rustup_toolchain();
    let library = build_library(&toolchain);
    let host = host_triple();
    let driver = build_driver(&toolchain);
    let mut support_cache = HashMap::new();

    for fixture in fixture_set.all_fixtures {
        eprintln!("[ui fixture] {fixture}");
        let snapshot_stem = Path::new(&fixture).file_stem().expect("fixture file stem").to_string_lossy();
        eprintln!(
            "[ui snapshot] {}",
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("ui_tests")
                .join(format!("{snapshot_stem}.stderr"))
                .display()
        );
        run_fixture(
            &driver,
            &library,
            &host,
            &toolchain,
            &fixture,
            !expected_failures.contains(&fixture),
            &mut support_cache,
        );
    }
}

#[test]
#[ignore = "run via cargo test --test ui autofix_machine_applicable -- --ignored --nocapture"]
fn autofix_machine_applicable() {
    let toolchain = rustup_toolchain();
    let library = build_library(&toolchain);
    let driver = build_driver(&toolchain);

    for fixture in fixed_fixture_names() {
        run_autofix_fixture(&driver, &library, &toolchain, &fixture);
    }
}

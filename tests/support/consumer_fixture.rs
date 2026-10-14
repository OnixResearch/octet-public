use std::env;
use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::OnceLock;
use std::time::SystemTime;
use std::time::UNIX_EPOCH;

pub(crate) struct StandaloneCargoOctet {
    _tmp: TempDir,
    pub(crate) source_root: PathBuf,
    pub(crate) bin_path: PathBuf,
}

const COMPILED_CARGO_OCTET_BIN: Option<&str> = option_env!("CARGO_OCTET_BIN");
static BASH_PATH: OnceLock<Result<PathBuf, String>> = OnceLock::new();
static BUILT_CARGO_OCTET_BIN: OnceLock<Result<PathBuf, String>> = OnceLock::new();

pub(crate) struct TempDir {
    path: PathBuf,
}

impl TempDir {
    pub(crate) fn new(label: &str) -> Self {
        let nanos = SystemTime::now().duration_since(UNIX_EPOCH).expect("time went backwards").as_nanos();
        let path = env::temp_dir().join(format!("octet-consumer-integration-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).expect("create temp dir");
        Self { path }
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

pub(crate) fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("create parent dir");
    }
    fs::write(path, contents).expect("write file");
}

pub(crate) fn write_executable(path: &Path, contents: &str) {
    write_file(path, contents);
    let mut perms = fs::metadata(path).expect("metadata").permissions();
    perms.set_mode(0o755);
    fs::set_permissions(path, perms).expect("chmod +x");
}

fn copy_path(from: &Path, to: &Path) {
    let metadata = fs::symlink_metadata(from).expect("symlink metadata");
    if metadata.is_dir() {
        fs::create_dir_all(to).expect("create copied dir");
        for entry in fs::read_dir(from).expect("read copied dir") {
            let entry = entry.expect("dir entry");
            copy_path(&entry.path(), &to.join(entry.file_name()));
        }
    } else {
        if let Some(parent) = to.parent() {
            fs::create_dir_all(parent).expect("create copied parent dir");
        }
        fs::copy(from, to).expect("copy file");
    }
}

pub(crate) struct ConsumerFixture {
    _tmp: TempDir,
    pub(crate) repo: PathBuf,
    pub(crate) fakebin: PathBuf,
    pub(crate) log_path: PathBuf,
    pub(crate) original_lockfile: String,
    pub(crate) driver_path: PathBuf,
    pub(crate) lint_lib_path: PathBuf,
    pub(crate) target_dir: PathBuf,
}

impl ConsumerFixture {
    pub(crate) fn new(label: &str, metadata_block: &str) -> Self {
        let bash = bash_path().expect("find bash");
        let tmp = TempDir::new(label);
        let repo = tmp.path().join("consumer");
        let fakebin = tmp.path().join("fakebin");
        let log_path = tmp.path().join("cargo.log");
        let driver_path = tmp.path().join("bin/dylint-driver");
        let lint_lib_path = tmp.path().join("lib/liboctet.so");
        let target_dir = tmp.path().join("target");

        fs::create_dir_all(&repo).expect("create repo dir");
        fs::create_dir_all(&fakebin).expect("create fakebin");
        fs::create_dir_all(target_dir.as_path()).expect("create target dir");

        write_file(
            &repo.join("Cargo.toml"),
            &format!(
                r#"[package]
name = "consumer"
version = "0.1.0"
edition = "2024"

[workspace]
{metadata_block}
"#
            ),
        );
        write_file(&repo.join("src/lib.rs"), "pub fn noop() {}\n");
        write_file(&repo.join("dylint.toml"), "[octet]\n");
        write_file(&repo.join("Cargo.lock"), "# original lockfile\n");
        write_executable(
            &fakebin.join("cargo"),
            &format!(
                r#"#!{bash}
set -euo pipefail
printf 'ARGS:%s\n' "$*" >> "$TS_LOG"
printf 'RUSTC_WORKSPACE_WRAPPER=%s\n' "${{RUSTC_WORKSPACE_WRAPPER:-}}" >> "$TS_LOG"
printf 'DYLINT_LIBS=%s\n' "${{DYLINT_LIBS:-}}" >> "$TS_LOG"
printf 'DYLINT_NO_DEPS=%s\n' "${{DYLINT_NO_DEPS:-}}" >> "$TS_LOG"
printf 'OCTET_STRICT_PROFILE=%s\n' "${{OCTET_STRICT_PROFILE:-}}" >> "$TS_LOG"
printf 'RUSTUP_TOOLCHAIN=%s\n' "${{RUSTUP_TOOLCHAIN:-}}" >> "$TS_LOG"
case "${{1:-}}" in
  build)
    out_dir="${{CARGO_TARGET_DIR:?}}/release"
    mkdir -p "$out_dir"
    if [[ "${{TS_MUTATE_LOCKFILE_ON_BUILD:-0}}" == "1" ]]; then
      cat >> "${{TS_CONSUMER_LOCKFILE:?}}" <<'EOF'
# mutated by fake cargo build
EOF
    fi
    if [[ " $* " == *" -p octet "* ]]; then
      cat > "$out_dir/liboctet.so" <<'EOF'
fake built Octet dylint library
EOF
      toolchain="${{RUSTUP_TOOLCHAIN:?}}"
      ln -sf "$out_dir/liboctet.so" "$out_dir/liboctet@${{toolchain}}.so"
    else
      cat > "$out_dir/tigerstyle-dylint-driver" <<'EOF'
#!{bash}
echo bootstrapped fake dylint driver
EOF
      chmod +x "$out_dir/tigerstyle-dylint-driver"
    fi
    exit 0
    ;;
  check)
    if [[ "${{TS_MUTATE_LOCKFILE:-0}}" == "1" ]]; then
      cat >> Cargo.lock <<'EOF'
# mutated by fake cargo
EOF
    fi
    if [[ -n "${{TS_STDOUT_FILE:-}}" ]]; then
      cat "$TS_STDOUT_FILE"
    fi
    if [[ -n "${{TS_STDERR:-}}" ]]; then
      printf '%s\n' "$TS_STDERR" >&2
    fi
    if [[ "${{TS_KILL_SELF:-0}}" == "1" ]]; then
      kill -9 $$
    fi
    exit "${{TS_CARGO_EXIT_STATUS:-0}}"
    ;;
  *)
    echo "unexpected cargo invocation: $*" >&2
    exit 97
    ;;
esac
"#,
                bash = bash.display(),
            ),
        );
        write_executable(&driver_path, &format!("#!{}\necho fake dylint driver\n", bash.display()));
        write_file(&lint_lib_path, "fake Octet dylint library\n");

        let original_lockfile = fs::read_to_string(repo.join("Cargo.lock")).expect("read original lockfile");

        Self {
            _tmp: tmp,
            repo,
            fakebin,
            log_path,
            original_lockfile,
            driver_path,
            lint_lib_path,
            target_dir,
        }
    }

    pub(crate) fn run(
        &self,
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            None,
            None,
            &[],
            args,
            cargo_exit_status,
            mutate_lockfile,
            false,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    pub(crate) fn run_raw_os(
        &self,
        args: Vec<OsString>,
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        let path = format!("{}:{}", self.fakebin.display(), env::var("PATH").expect("PATH"));

        let mut command = match env::var("CARGO_OCTET_BIN") {
            Ok(bin) if !bin.is_empty() => Command::new(bin),
            _ => match COMPILED_CARGO_OCTET_BIN.filter(|bin| !bin.is_empty()) {
                Some(bin) => Command::new(bin),
                None => Command::new(built_cargo_octet_bin().expect("build cargo-octet binary")),
            },
        };
        command
            .args(args)
            .current_dir(&self.repo)
            .env("PATH", &path)
            .env("TS_LOG", &self.log_path)
            .env("TS_CARGO_EXIT_STATUS", cargo_exit_status.to_string())
            .env("TS_MUTATE_LOCKFILE", if mutate_lockfile { "1" } else { "0" })
            .env("TS_MUTATE_LOCKFILE_ON_BUILD", "0")
            .env("TS_CONSUMER_LOCKFILE", self.repo.join("Cargo.lock"))
            .env("CARGO_TARGET_DIR", &self.target_dir);

        fs::write(&self.log_path, format!("LAUNCHER:{}\nPATH:{}\n", command.get_program().to_string_lossy(), path))
            .expect("seed cargo log");

        match lint_lib_env {
            Some(value) => {
                command.env("TIGERSTYLE_LINT_LIB", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_LINT_LIB");
            }
        }

        match driver_env {
            Some(value) => {
                command.env("TIGERSTYLE_DRIVER_PATH", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DRIVER_PATH");
            }
        }

        match toolchain_env {
            Some(value) => {
                command.env("TIGERSTYLE_TOOLCHAIN", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_TOOLCHAIN");
            }
        }

        match dylint_rev_env {
            Some(value) => {
                command.env("TIGERSTYLE_DYLINT_REV", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DYLINT_REV");
            }
        }

        command.output().expect("run cargo-octet")
    }

    pub(crate) fn run_with_launcher(
        &self,
        launcher: Option<&Path>,
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_with_launcher_and_env(
            launcher,
            &[],
            args,
            cargo_exit_status,
            mutate_lockfile,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    pub(crate) fn run_with_launcher_and_env(
        &self,
        launcher: Option<&Path>,
        extra_env: &[(&str, &str)],
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            None,
            launcher,
            extra_env,
            args,
            cargo_exit_status,
            mutate_lockfile,
            false,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "test harness exposes independent CLI/env knobs"
    )]
    pub(crate) fn run_from_dir_with_launcher_and_env(
        &self,
        current_dir: &Path,
        launcher: Option<&Path>,
        extra_env: &[(&str, &str)],
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            Some(current_dir),
            launcher,
            extra_env,
            args,
            cargo_exit_status,
            mutate_lockfile,
            false,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    pub(crate) fn run_with_bootstrap_lockfile_mutation(
        &self,
        args: &[&str],
        cargo_exit_status: i32,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        self.run_internal(
            None,
            None,
            &[],
            args,
            cargo_exit_status,
            false,
            true,
            toolchain_env,
            driver_env,
            lint_lib_env,
            dylint_rev_env,
        )
    }

    #[expect(
        clippy::too_many_arguments,
        reason = "test harness centralizes independent CLI/env knobs"
    )]
    fn run_internal(
        &self,
        current_dir: Option<&Path>,
        launcher: Option<&Path>,
        extra_env: &[(&str, &str)],
        args: &[&str],
        cargo_exit_status: i32,
        mutate_lockfile: bool,
        mutate_lockfile_on_build: bool,
        toolchain_env: Option<&str>,
        driver_env: Option<&Path>,
        lint_lib_env: Option<&Path>,
        dylint_rev_env: Option<&str>,
    ) -> std::process::Output {
        let path = format!("{}:{}", self.fakebin.display(), env::var("PATH").expect("PATH"));

        let mut command = match launcher {
            Some(bin) => Command::new(bin),
            None => match env::var("CARGO_OCTET_BIN") {
                Ok(bin) if !bin.is_empty() => Command::new(bin),
                _ => match COMPILED_CARGO_OCTET_BIN.filter(|bin| !bin.is_empty()) {
                    Some(bin) => Command::new(bin),
                    None => Command::new(built_cargo_octet_bin().expect("build cargo-octet binary")),
                },
            },
        };
        command
            .arg("check")
            .args(args)
            .current_dir(current_dir.unwrap_or(self.repo.as_path()))
            .env("PATH", &path)
            .env("TS_LOG", &self.log_path)
            .env("TS_CARGO_EXIT_STATUS", cargo_exit_status.to_string())
            .env("TS_MUTATE_LOCKFILE", if mutate_lockfile { "1" } else { "0" })
            .env("TS_MUTATE_LOCKFILE_ON_BUILD", if mutate_lockfile_on_build { "1" } else { "0" })
            .env("TS_CONSUMER_LOCKFILE", self.repo.join("Cargo.lock"))
            .env("CARGO_TARGET_DIR", &self.target_dir);

        fs::write(&self.log_path, format!("LAUNCHER:{}\nPATH:{}\n", command.get_program().to_string_lossy(), path))
            .expect("seed cargo log");

        match lint_lib_env {
            Some(value) => {
                command.env("TIGERSTYLE_LINT_LIB", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_LINT_LIB");
            }
        }

        match driver_env {
            Some(value) => {
                command.env("TIGERSTYLE_DRIVER_PATH", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DRIVER_PATH");
            }
        }

        match toolchain_env {
            Some(value) => {
                command.env("TIGERSTYLE_TOOLCHAIN", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_TOOLCHAIN");
            }
        }

        match dylint_rev_env {
            Some(value) => {
                command.env("TIGERSTYLE_DYLINT_REV", value);
            }
            None => {
                command.env_remove("TIGERSTYLE_DYLINT_REV");
            }
        }

        for (key, value) in extra_env {
            command.env(key, value);
        }

        command.output().expect("run cargo-octet")
    }

    pub(crate) fn log(&self) -> String {
        fs::read_to_string(&self.log_path).unwrap_or_default()
    }

    pub(crate) fn lockfile(&self) -> String {
        fs::read_to_string(self.repo.join("Cargo.lock")).expect("read lockfile")
    }
}

pub(crate) fn metadata(default_scope: &str, cargo_check_args: &str) -> String {
    format!(
        r"[workspace.metadata.octet]
default_scope = {default_scope}
cargo_check_args = {cargo_check_args}
"
    )
}

pub(crate) fn bash_path() -> Result<PathBuf, String> {
    BASH_PATH
        .get_or_init(|| {
            if let Some(path) = env::var_os("BASH").map(PathBuf::from).filter(|path| path.is_file()) {
                return Ok(path);
            }

            let path = env::var_os("PATH").ok_or_else(|| "missing `PATH` env var".to_owned())?;
            env::split_paths(&path)
                .map(|dir| dir.join("bash"))
                .find(|candidate| candidate.is_file())
                .ok_or_else(|| "could not find `bash` on PATH".to_owned())
        })
        .clone()
}

pub(crate) fn built_cargo_octet_bin() -> Result<PathBuf, String> {
    BUILT_CARGO_OCTET_BIN
        .get_or_init(|| {
            let cargo = env::var("CARGO").map_err(|_| "missing `CARGO` env var".to_owned())?;
            let manifest_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("cargo-octet/Cargo.toml");
            let target_dir = env::temp_dir().join("octet-consumer-integration-cli-target");

            let status = Command::new(cargo)
                .arg("build")
                .arg("-q")
                .arg("--locked")
                .arg("--manifest-path")
                .arg(&manifest_path)
                .arg("--bin")
                .arg("cargo-octet")
                .arg("--target-dir")
                .arg(&target_dir)
                .status()
                .map_err(|error| format!("build cargo-octet binary: {error}"))?;
            if !status.success() {
                return Err("cargo-octet build failed".to_owned());
            }

            let binary_path = target_dir.join("debug").join(format!("cargo-octet{}", env::consts::EXE_SUFFIX));
            if !binary_path.is_file() {
                return Err(format!("built cargo-octet binary missing at {}", binary_path.display()));
            }

            Ok(binary_path)
        })
        .clone()
}

pub(crate) fn build_standalone_cargo_octet() -> StandaloneCargoOctet {
    let tmp = TempDir::new("installed-cargo-octet");
    let source_root = tmp.path().join("source");
    let target_dir = tmp.path().join("target");
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"));

    for relative in [
        "Cargo.toml",
        "Cargo.lock",
        "rust-toolchain.toml",
        "src",
        "cargo-octet",
        "crates/octet-lint-metadata",
        "crates/octet-standards",
        "crates/valence-core",
        "standards/effect-closure/generated",
        "standards/proof-imports/generated",
        "standards/strict-profiles/generated",
    ] {
        copy_path(&repo_root.join(relative), &source_root.join(relative));
    }

    let cargo = env::var("CARGO").expect("CARGO env var");
    let manifest_path = source_root.join("cargo-octet/Cargo.toml");
    let status = Command::new(cargo)
        .arg("build")
        .arg("-q")
        .arg("--locked")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--bin")
        .arg("cargo-octet")
        .arg("--target-dir")
        .arg(&target_dir)
        .status()
        .expect("build standalone cargo-octet");
    assert!(status.success(), "standalone cargo-octet build failed: {status}");

    let bin_path = target_dir.join("debug").join(format!("cargo-octet{}", env::consts::EXE_SUFFIX));
    assert!(bin_path.is_file(), "standalone cargo-octet binary missing at {}", bin_path.display());

    fs::remove_dir_all(&source_root).expect("remove standalone source root");
    assert!(!source_root.exists(), "standalone source root should be gone: {}", source_root.display());

    StandaloneCargoOctet {
        _tmp: tmp,
        source_root,
        bin_path,
    }
}

pub(crate) fn expected_repo_toolchain() -> String {
    let rust_toolchain = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("rust-toolchain.toml"))
        .expect("read repo rust-toolchain.toml");
    let document = rust_toolchain.parse::<toml::Table>().expect("parse repo rust-toolchain.toml");
    let channel = document
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("channel"))
        .and_then(toml::Value::as_str)
        .expect("toolchain.channel");

    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(rustc).arg("-vV").output().expect("run rustc -vV for host triple");
    assert!(output.status.success(), "rustc -vV failed: {}", String::from_utf8_lossy(&output.stderr));
    let stdout = String::from_utf8(output.stdout).expect("rustc -vV stdout utf8");
    let host = stdout.lines().find_map(|line| line.strip_prefix("host: ")).expect("rustc host triple");

    if channel.ends_with(host) {
        channel.to_owned()
    } else {
        format!("{channel}-{host}")
    }
}

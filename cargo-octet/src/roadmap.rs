use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::ffi::OsString;
use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;

use crate::determinism;

const SCHEMA_VERSION: u32 = 1;
const BLAKE3_IDENTITY_PREFIX: &str = "b3:";
const BLAKE3_HEX_LENGTH: usize = 64;
/// Stands in for the workspace root (the current directory) at the start of a path string.
const WORKSPACE_PLACEHOLDER: &str = "<workspace>";
/// Stands in for `HOME` at the start of a path string.
const HOME_PLACEHOLDER: &str = "<home>";
/// Stands in for the temporary directory at the start of a path string.
const TEMP_PLACEHOLDER: &str = "<temp>";
/// What [`normalize_value`] guarantees about stored artifacts and capsules.
const REDACTION_STATUS: &str = "secret-like-and-local-fields-dropped-local-roots-normalized";
/// Evidence mode of the compatible `--observed` path: caller-supplied run metadata is compared,
/// and the recorded job is never executed.
const SUPPLIED_OBSERVATION_EVIDENCE_MODE: &str = "supplied_observation";
const SUPPLIED_OBSERVATION_EVIDENCE_LABEL: &str = "supplied observation comparison; the capsule job was not executed";
/// Evidence mode of `--execute`: a reviewed profile is launched under bounded execution.
const EXECUTED_EVIDENCE_MODE: &str = "executed";
const EXECUTED_EVIDENCE_LABEL: &str = "independent bounded execution of a reviewed native profile";
/// Stands in for the isolated output directory in the profile-canonical argv recorded in a capsule
/// and in the execution receipt.
const OUTPUT_PLACEHOLDER: &str = "<output>";
/// Isolated output directory, relative to the workspace root.
const EXECUTION_OUTPUT_DIRECTORY: &str = ".octet/replay-execution";
const EXECUTION_RECEIPT_KIND: &str = "octet-execution-receipt";
const EXECUTION_POLL_INTERVAL_MS: u64 = 10;
const EXECUTION_TEARDOWN_TIMEOUT_MS: u64 = 5_000;
const EXECUTION_STDIN_MAX_BYTES: usize = 1;
const EXECUTION_INPUT_FILE_MAX_BYTES: u64 = 64 * 1024 * 1024;
const EXECUTION_OUTPUT_FILE_MAX_BYTES: u64 = 4 * 1024 * 1024;
/// Characters that would make a capsule-supplied argument a shell fragment rather than a single
/// token. Capsule command text is never executed, and it must not look like it could be. Angle
/// brackets stay allowed because the canonical argv spells the isolated output directory as
/// [`OUTPUT_PLACEHOLDER`].
const SHELL_FRAGMENT_CHARACTERS: [char; 14] =
    [';', '|', '&', '`', '$', '(', ')', '{', '}', '\n', '\r', '\0', '\\', '"'];
const PURE_CACHE_KEY_FIELDS: [&str; 5] = [
    "command_identity",
    "input_hashes",
    "config_hash",
    "tool_revision",
    "environment_allowlist",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayOutputFormat {
    Human,
    Json,
}

impl ReplayOutputFormat {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "human" => Ok(Self::Human),
            "json" => Ok(Self::Json),
            other => Err(format!("unsupported capsule replay output format `{other}`; expected `human` or `json`")),
        }
    }
}

#[derive(Debug)]
pub enum RoadmapCommand {
    ArtifactStore {
        kind: String,
        input: PathBuf,
        store_dir: PathBuf,
    },
    ArtifactInspect {
        hash: String,
        store_dir: PathBuf,
    },
    ArtifactReplay {
        hash: String,
        store_dir: PathBuf,
    },
    CapsuleCapture {
        input: PathBuf,
        capsule: PathBuf,
    },
    CapsuleInspect {
        capsule: PathBuf,
    },
    CapsuleReplay {
        capsule: PathBuf,
        observed: Option<PathBuf>,
        execute: bool,
        inputs: Option<PathBuf>,
        receipt: Option<PathBuf>,
        output_format: ReplayOutputFormat,
    },
    AbilityRegistry,
    AbilitySummary {
        findings: PathBuf,
    },
    AbilityEnforce {
        manifest: PathBuf,
        trace: PathBuf,
    },
    PointerSet {
        registry: PathBuf,
        name: String,
        kind: String,
        hash: String,
        receipt: PathBuf,
    },
    PointerInspect {
        registry: PathBuf,
        name: String,
    },
    PointerRollback {
        registry: PathBuf,
        receipt: PathBuf,
    },
    SessionCheck {
        input: PathBuf,
    },
    SessionMigrate {
        input: PathBuf,
        output: PathBuf,
    },
    SessionTodo {
        input: PathBuf,
    },
    PureCacheCheck {
        key: PathBuf,
        store_dir: PathBuf,
        output_hash: String,
    },
}

pub fn parse(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("artifact") => parse_artifact(&args[1..]),
        Some("capsule") => parse_capsule(&args[1..]),
        Some("ability") => parse_ability(&args[1..]),
        Some("pointer") => parse_pointer(&args[1..]),
        Some("session") => parse_session(&args[1..]),
        Some("pure-cache") => parse_pure_cache(&args[1..]),
        Some(other) => Err(format!("unknown roadmap subcommand `{other}`")),
        None => Err("missing roadmap subcommand".to_owned()),
    }
}

pub fn run(command: RoadmapCommand) -> Result<i32, String> {
    match command {
        RoadmapCommand::ArtifactStore { kind, input, store_dir } => {
            artifact_store(&kind, &input, &store_dir, &local_roots_from_environment())
        }
        RoadmapCommand::ArtifactInspect { hash, store_dir } => artifact_inspect(&hash, &store_dir),
        RoadmapCommand::ArtifactReplay { hash, store_dir } => artifact_replay(&hash, &store_dir),
        RoadmapCommand::CapsuleCapture { input, capsule } => {
            capsule_capture(&input, &capsule, &local_roots_from_environment())
        }
        RoadmapCommand::CapsuleInspect { capsule } => capsule_inspect(&capsule),
        RoadmapCommand::CapsuleReplay {
            capsule,
            observed,
            execute,
            inputs,
            receipt,
            output_format,
        } => {
            let local_roots = local_roots_from_environment();
            if execute {
                capsule_replay_executed(&capsule, inputs.as_deref(), receipt.as_deref(), &local_roots, output_format)
            } else {
                let observed =
                    observed.ok_or_else(|| "capsule replay requires `--observed PATH` or `--execute`".to_owned())?;
                capsule_replay_observed(&capsule, &observed, &local_roots, output_format)
            }
        }
        RoadmapCommand::AbilityRegistry => ability_registry(),
        RoadmapCommand::AbilitySummary { findings } => ability_summary(&findings),
        RoadmapCommand::AbilityEnforce { manifest, trace } => ability_enforce(&manifest, &trace),
        RoadmapCommand::PointerSet {
            registry,
            name,
            kind,
            hash,
            receipt,
        } => pointer_set(&registry, &name, &kind, &hash, &receipt),
        RoadmapCommand::PointerInspect { registry, name } => pointer_inspect(&registry, &name),
        RoadmapCommand::PointerRollback { registry, receipt } => pointer_rollback(&registry, &receipt),
        RoadmapCommand::SessionCheck { input } => session_check(&input),
        RoadmapCommand::SessionMigrate { input, output } => session_migrate(&input, &output),
        RoadmapCommand::SessionTodo { input } => session_todo(&input),
        RoadmapCommand::PureCacheCheck {
            key,
            store_dir,
            output_hash,
        } => pure_cache_check(&key, &store_dir, &output_hash),
    }
}

fn parse_artifact(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("store") => {
            let mut flags = RoadmapFlags::parse("artifact", args, &["--kind", "--input", "--store-dir"])?;
            Ok(RoadmapCommand::ArtifactStore {
                kind: flags.take("--kind")?,
                input: flags.take_path("--input")?,
                store_dir: flags.take_path("--store-dir")?,
            })
        }
        Some("inspect") => {
            let mut flags = RoadmapFlags::parse("artifact", args, &["--hash", "--store-dir"])?;
            Ok(RoadmapCommand::ArtifactInspect {
                hash: flags.take("--hash")?,
                store_dir: flags.take_path("--store-dir")?,
            })
        }
        Some("replay") => {
            let mut flags = RoadmapFlags::parse("artifact", args, &["--hash", "--store-dir"])?;
            Ok(RoadmapCommand::ArtifactReplay {
                hash: flags.take("--hash")?,
                store_dir: flags.take_path("--store-dir")?,
            })
        }
        Some(other) => Err(format!("unknown artifact action `{other}`")),
        None => Err("missing artifact action".to_owned()),
    }
}

fn parse_capsule(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("capture") => {
            let mut flags = RoadmapFlags::parse("capsule", args, &["--input", "--capsule"])?;
            Ok(RoadmapCommand::CapsuleCapture {
                input: flags.take_path("--input")?,
                capsule: flags.take_path("--capsule")?,
            })
        }
        Some("inspect") => {
            let mut flags = RoadmapFlags::parse("capsule", args, &["--capsule"])?;
            Ok(RoadmapCommand::CapsuleInspect {
                capsule: flags.take_path("--capsule")?,
            })
        }
        Some("replay") => {
            let mut flags = RoadmapFlags::parse_with_switches(
                "capsule",
                args,
                &["--capsule", "--observed", "--inputs", "--receipt", "--output-format"],
                &["--execute"],
            )?;
            let execute = flags.switch("--execute");
            let observed = flags.take_optional("--observed").map(PathBuf::from);
            let inputs = flags.take_optional("--inputs").map(PathBuf::from);
            let receipt = flags.take_optional("--receipt").map(PathBuf::from);
            let output_format = flags
                .take_optional("--output-format")
                .map(|format| ReplayOutputFormat::parse(&format))
                .transpose()?
                .unwrap_or(ReplayOutputFormat::Json);
            if execute && observed.is_some() {
                return Err(
                    "`--execute` and `--observed` are mutually exclusive; executed replay collects its own observations"
                        .to_owned(),
                );
            }
            if !execute && observed.is_none() {
                return Err("missing `--observed`; pass `--observed PATH` to compare supplied observations or `--execute` to run the reviewed profile".to_owned());
            }
            if !execute && inputs.is_some() {
                return Err("`--inputs` requires `--execute`".to_owned());
            }
            if !execute && receipt.is_some() {
                return Err("`--receipt` requires `--execute`".to_owned());
            }
            Ok(RoadmapCommand::CapsuleReplay {
                capsule: flags.take_path("--capsule")?,
                observed,
                execute,
                inputs,
                receipt,
                output_format,
            })
        }
        Some(other) => Err(format!("unknown capsule action `{other}`")),
        None => Err("missing capsule action".to_owned()),
    }
}

fn parse_ability(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("registry") => {
            RoadmapFlags::parse("ability", args, &[])?;
            Ok(RoadmapCommand::AbilityRegistry)
        }
        Some("summary") => {
            let mut flags = RoadmapFlags::parse("ability", args, &["--findings"])?;
            Ok(RoadmapCommand::AbilitySummary {
                findings: flags.take_path("--findings")?,
            })
        }
        Some("enforce") => {
            let mut flags = RoadmapFlags::parse("ability", args, &["--manifest", "--trace"])?;
            Ok(RoadmapCommand::AbilityEnforce {
                manifest: flags.take_path("--manifest")?,
                trace: flags.take_path("--trace")?,
            })
        }
        Some(other) => Err(format!("unknown ability action `{other}`")),
        None => Err("missing ability action".to_owned()),
    }
}

fn parse_pointer(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("set") => {
            let mut flags =
                RoadmapFlags::parse("pointer", args, &["--registry", "--name", "--kind", "--hash", "--receipt"])?;
            Ok(RoadmapCommand::PointerSet {
                registry: flags.take_path("--registry")?,
                name: flags.take("--name")?,
                kind: flags.take("--kind")?,
                hash: flags.take("--hash")?,
                receipt: flags.take_path("--receipt")?,
            })
        }
        Some("inspect") => {
            let mut flags = RoadmapFlags::parse("pointer", args, &["--registry", "--name"])?;
            Ok(RoadmapCommand::PointerInspect {
                registry: flags.take_path("--registry")?,
                name: flags.take("--name")?,
            })
        }
        Some("rollback") => {
            let mut flags = RoadmapFlags::parse("pointer", args, &["--registry", "--receipt"])?;
            Ok(RoadmapCommand::PointerRollback {
                registry: flags.take_path("--registry")?,
                receipt: flags.take_path("--receipt")?,
            })
        }
        Some(other) => Err(format!("unknown pointer action `{other}`")),
        None => Err("missing pointer action".to_owned()),
    }
}

fn parse_session(args: &[String]) -> Result<RoadmapCommand, String> {
    match args.first().map(String::as_str) {
        Some("check") => {
            let mut flags = RoadmapFlags::parse("session", args, &["--input"])?;
            Ok(RoadmapCommand::SessionCheck {
                input: flags.take_path("--input")?,
            })
        }
        Some("migrate") => {
            let mut flags = RoadmapFlags::parse("session", args, &["--input", "--output"])?;
            Ok(RoadmapCommand::SessionMigrate {
                input: flags.take_path("--input")?,
                output: flags.take_path("--output")?,
            })
        }
        Some("todo") => {
            let mut flags = RoadmapFlags::parse("session", args, &["--input"])?;
            Ok(RoadmapCommand::SessionTodo {
                input: flags.take_path("--input")?,
            })
        }
        Some(other) => Err(format!("unknown session action `{other}`")),
        None => Err("missing session action".to_owned()),
    }
}

fn parse_pure_cache(args: &[String]) -> Result<RoadmapCommand, String> {
    if !matches!(args.first().map(String::as_str), Some("check")) {
        return Err("pure-cache expects `check`".to_owned());
    }
    let mut flags = RoadmapFlags::parse("pure-cache", args, &["--key", "--store-dir", "--output-hash"])?;
    Ok(RoadmapCommand::PureCacheCheck {
        key: flags.take_path("--key")?,
        store_dir: flags.take_path("--store-dir")?,
        output_hash: flags.take("--output-hash")?,
    })
}

/// `--flag VALUE` / `--flag=VALUE` options of one roadmap action, each given at most once.
struct RoadmapFlags {
    values: BTreeMap<&'static str, String>,
    switches: BTreeSet<&'static str>,
}

impl RoadmapFlags {
    /// Parses `args` (the action name followed by its options), rejecting flags outside `allowed`,
    /// stray positional arguments, repeated flags, and missing values (including a following
    /// `--flag`).
    fn parse(surface: &str, args: &[String], allowed: &[&'static str]) -> Result<Self, String> {
        Self::parse_with_switches(surface, args, allowed, &[])
    }

    /// Like [`Self::parse`], with `switches` as value-less flags that may appear once.
    fn parse_with_switches(
        surface: &str,
        args: &[String],
        allowed: &[&'static str],
        switches: &[&'static str],
    ) -> Result<Self, String> {
        let action = args.first().map_or("", String::as_str);
        let mut values = BTreeMap::new();
        let mut switches_seen = BTreeSet::new();
        let mut index = 1;
        while let Some(arg) = args.get(index) {
            let (name, inline_value) = match arg.split_once('=') {
                Some((name, value)) if name.starts_with("--") => (name, Some(value)),
                _ => (arg.as_str(), None),
            };
            if let Some(switch) = switches.iter().copied().find(|switch| *switch == name) {
                if inline_value.is_some() {
                    return Err(format!("`{switch}` takes no value"));
                }
                if !switches_seen.insert(switch) {
                    return Err(format!("`{switch}` may be provided only once"));
                }
                index += 1;
                continue;
            }
            let Some(flag) = allowed.iter().copied().find(|flag| *flag == name) else {
                return Err(if arg.starts_with('-') {
                    format!("unsupported roadmap {surface} {action} argument `{arg}`")
                } else {
                    format!("unexpected roadmap {surface} {action} argument `{arg}`")
                });
            };
            let value = if let Some(value) = inline_value {
                index += 1;
                value
            } else {
                let value = args.get(index + 1).map_or("", String::as_str);
                index += 2;
                if value.starts_with("--") { "" } else { value }
            };
            if value.is_empty() {
                return Err(format!("missing value after `{flag}`"));
            }
            if values.insert(flag, value.to_owned()).is_some() {
                return Err(format!("`{flag}` may be provided only once"));
            }
        }
        Ok(Self {
            values,
            switches: switches_seen,
        })
    }

    fn switch(&self, flag: &str) -> bool {
        self.switches.contains(flag)
    }

    fn take(&mut self, flag: &str) -> Result<String, String> {
        self.values.remove(flag).ok_or_else(|| format!("missing `{flag}`"))
    }

    fn take_optional(&mut self, flag: &str) -> Option<String> {
        self.values.remove(flag)
    }

    fn take_path(&mut self, flag: &str) -> Result<PathBuf, String> {
        self.take(flag).map(PathBuf::from)
    }
}

#[derive(Serialize)]
struct ArtifactRecord {
    schema_version: u32,
    artifact_kind: String,
    hash: String,
    normalized: Value,
    dependencies: Vec<String>,
    redaction_status: &'static str,
}

fn artifact_store(kind: &str, input: &Path, store_dir: &Path, local_roots: &LocalRoots) -> Result<i32, String> {
    let normalized = normalize_value(read_json(input)?, local_roots);
    let dependencies = artifact_dependencies(&normalized)?;
    let hash = artifact_hash(&Value::from(SCHEMA_VERSION), kind, &normalized)?;
    let record = ArtifactRecord {
        schema_version: SCHEMA_VERSION,
        artifact_kind: kind.to_owned(),
        hash: hash.clone(),
        normalized,
        dependencies,
        redaction_status: REDACTION_STATUS,
    };
    fs::create_dir_all(store_dir).map_err(|error| format!("create {}: {error}", store_dir.display()))?;
    write_json(&store_dir.join(hash_file_name(&hash)), &record)?;
    print_json(&record)?;
    Ok(0)
}

fn artifact_hash(schema_version: &Value, kind: &str, normalized: &Value) -> Result<String, String> {
    let canonical = canonical_json(&serde_json::json!({
        "schema_version": schema_version,
        "artifact_kind": kind,
        "normalized": normalized,
    }))?;
    Ok(determinism::full_hash(&canonical))
}

/// The `dependencies` of a normalized artifact: absent means none, otherwise an array of
/// well-formed artifact hashes.
fn artifact_dependencies(normalized: &Value) -> Result<Vec<String>, String> {
    let Some(dependencies) = normalized.get("dependencies") else {
        return Ok(Vec::new());
    };
    let items = dependencies
        .as_array()
        .ok_or_else(|| "artifact dependencies must be an array of artifact hashes".to_owned())?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            let dependency = item
                .as_str()
                .ok_or_else(|| format!("artifact dependencies[{index}] must be an artifact hash string"))?;
            validate_artifact_hash(dependency)?;
            Ok(dependency.to_owned())
        })
        .collect()
}

/// A stored artifact record whose content re-hashes to the hash it was requested by.
struct VerifiedArtifact {
    record: Value,
    dependencies: Vec<String>,
}

fn load_artifact(hash: &str, store_dir: &Path) -> Result<VerifiedArtifact, String> {
    validate_artifact_hash(hash)?;
    let path = store_dir.join(hash_file_name(hash));
    let record = read_json(&path)?;
    let (Some(schema_version), Some(kind), Some(normalized)) = (
        record.get("schema_version"),
        record.get("artifact_kind").and_then(Value::as_str),
        record.get("normalized"),
    ) else {
        return Err(format!(
            "artifact record {} must contain schema_version, string artifact_kind, and normalized",
            path.display()
        ));
    };
    let recomputed = artifact_hash(schema_version, kind, normalized)?;
    let recorded = record.get("hash").and_then(Value::as_str);
    if recomputed != hash || recorded != Some(hash) {
        return Err(format!(
            "artifact record {} was requested as `{hash}` and records hash {} but its content hashes to `{recomputed}`; the record was modified or misplaced",
            path.display(),
            record.get("hash").unwrap_or(&Value::Null)
        ));
    }
    let dependencies = artifact_dependencies(normalized).map_err(|error| format!("{}: {error}", path.display()))?;
    let recorded_dependencies_match = record
        .get("dependencies")
        .and_then(Value::as_array)
        .is_some_and(|items| items.iter().map(Value::as_str).eq(dependencies.iter().map(|item| Some(item.as_str()))));
    if !recorded_dependencies_match {
        return Err(format!("artifact record {} dependencies differ from its hashed content", path.display()));
    }
    Ok(VerifiedArtifact { record, dependencies })
}

fn artifact_report(artifact: &VerifiedArtifact, operation: &'static str) -> Value {
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "operation": operation,
        "artifact_kind": artifact.record["artifact_kind"],
        "hash": artifact.record["hash"],
        "dependencies": artifact.dependencies,
        "replay_prerequisites": artifact.dependencies,
    })
}

fn artifact_inspect(hash: &str, store_dir: &Path) -> Result<i32, String> {
    let artifact = load_artifact(hash, store_dir)?;
    print_json(&artifact_report(&artifact, "inspect"))?;
    Ok(0)
}

fn artifact_replay(hash: &str, store_dir: &Path) -> Result<i32, String> {
    let artifact = load_artifact(hash, store_dir)?;
    if let Some(missing) = missing_dependency(hash, &artifact.dependencies, store_dir)? {
        let report = serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "operation": "replay",
            "hash": hash,
            "status": "failed-closed",
            "missing_dependency": missing,
        });
        print_json(&report)?;
        return Ok(1);
    }
    print_json(&artifact_report(&artifact, "replay"))?;
    Ok(0)
}

/// Walks the transitive dependency closure of `root`, verifying every present record like the
/// root, and returns the first dependency hash missing from the store.
fn missing_dependency(root: &str, dependencies: &[String], store_dir: &Path) -> Result<Option<String>, String> {
    let mut visited = BTreeSet::from([root.to_owned()]);
    let mut pending: Vec<String> = dependencies.iter().rev().cloned().collect();
    while let Some(dependency) = pending.pop() {
        if !visited.insert(dependency.clone()) {
            continue;
        }
        if !store_dir.join(hash_file_name(&dependency)).exists() {
            return Ok(Some(dependency));
        }
        let artifact =
            load_artifact(&dependency, store_dir).map_err(|error| format!("dependency `{dependency}`: {error}"))?;
        pending.extend(artifact.dependencies.into_iter().rev());
    }
    Ok(None)
}

fn capsule_capture(input: &Path, capsule_path: &Path, local_roots: &LocalRoots) -> Result<i32, String> {
    let mut manifest = normalize_value(read_json(input)?, local_roots);
    for field in [
        "command_argv",
        "run_kind",
        "config_hash",
        "input_hashes",
        "output_hashes",
        "expected_exit_code",
    ] {
        if manifest.get(field).is_none() {
            return Err(format!("capsule input missing {field}"));
        }
    }
    if manifest.get("expected_exit_code").and_then(Value::as_i64).is_none() {
        return Err("capsule input expected_exit_code must be an integer".to_owned());
    }
    if manifest.get("octet_binary_identity").is_none() {
        let Some(legacy_identity) = manifest.get("slotcar_binary_identity").cloned() else {
            return Err("capsule input missing octet_binary_identity".to_owned());
        };
        manifest["octet_binary_identity"] = legacy_identity;
    }
    if let Some(object) = manifest.as_object_mut() {
        object.remove("slotcar_binary_identity");
    }
    let canonical = canonical_json(&manifest)?;
    let capsule_hash = determinism::full_hash(&canonical);
    let capsule = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "capsule_hash": capsule_hash,
        "capsule_kind": "octet-replay-capsule",
        "manifest": manifest,
        "replay_contract": {
            "input_match": "required",
            "output_match": "required",
            "expected_failure_allowed": true,
            "drift_behavior": "fail-closed"
        },
        "redaction_status": REDACTION_STATUS
    });
    write_json(capsule_path, &capsule)?;
    print_json(&capsule)?;
    Ok(0)
}

fn capsule_inspect(capsule_path: &Path) -> Result<i32, String> {
    let capsule = read_json(capsule_path)?;
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "operation": "inspect",
        "capsule_hash": capsule["capsule_hash"],
        "run_kind": capsule["manifest"]["run_kind"],
        "command_argv": capsule["manifest"]["command_argv"],
        "input_hashes": capsule["manifest"]["input_hashes"],
        "output_hashes": capsule["manifest"]["output_hashes"],
        "expected_exit_code": capsule["manifest"]["expected_exit_code"],
        "redaction_status": capsule["redaction_status"],
    }))?;
    Ok(0)
}

/// Reads and verifies a replay capsule, returning the capsule document and its manifest. Rejects
/// a capsule whose manifest is not an object, lacks a compared field, or no longer hashes to its
/// recorded `capsule_hash`, so an edited capsule is never replayed.
fn load_capsule(capsule_path: &Path) -> Result<(Value, Value), String> {
    let capsule = read_json(capsule_path)?;
    let manifest = capsule
        .get("manifest")
        .filter(|manifest| manifest.is_object())
        .ok_or_else(|| format!("capsule {} manifest must be a JSON object", capsule_path.display()))?
        .clone();
    if let Some(field) = CAPSULE_COMPARED_FIELDS.iter().find(|field| manifest.get(**field).is_none()) {
        return Err(format!("capsule manifest missing {field}"));
    }
    if manifest.get("expected_exit_code").and_then(Value::as_i64).is_none() {
        return Err("capsule manifest expected_exit_code must be an integer".to_owned());
    }
    let recorded_hash = capsule
        .get("capsule_hash")
        .and_then(Value::as_str)
        .ok_or_else(|| "capsule missing capsule_hash".to_owned())?;
    let recomputed_hash = determinism::full_hash(&canonical_json(&manifest)?);
    if recorded_hash != recomputed_hash {
        return Err(format!(
            "capsule_hash mismatch: capsule records `{recorded_hash}` but its manifest hashes to `{recomputed_hash}`; the capsule was modified after capture"
        ));
    }
    Ok((capsule, manifest))
}

/// The compatible comparison path: `--observed` metadata is compared with the capsule and the
/// recorded job is never launched. The report names its evidence mode so it cannot be mistaken for
/// executed replay.
fn capsule_replay_observed(
    capsule_path: &Path,
    observed_path: &Path,
    local_roots: &LocalRoots,
    output_format: ReplayOutputFormat,
) -> Result<i32, String> {
    let (code, report) = capsule_replay_observed_report(capsule_path, observed_path, local_roots)?;
    emit_replay(&report, output_format, &observed_human_lines(&report))?;
    Ok(code)
}

fn capsule_replay_observed_report(
    capsule_path: &Path,
    observed_path: &Path,
    local_roots: &LocalRoots,
) -> Result<(i32, Value), String> {
    let (capsule, manifest) = load_capsule(capsule_path)?;
    let mut observed = normalize_value(read_json(observed_path)?, local_roots);
    if observed.get("octet_binary_identity").is_none()
        && let Some(legacy_identity) = observed.get("slotcar_binary_identity").cloned()
    {
        observed["octet_binary_identity"] = legacy_identity;
    }
    if let Some(object) = observed.as_object_mut() {
        object.remove("slotcar_binary_identity");
    }
    let expected_exit = manifest
        .get("expected_exit_code")
        .and_then(Value::as_i64)
        .ok_or_else(|| "capsule manifest expected_exit_code must be an integer".to_owned())?;
    let mismatches = capsule_mismatches(&manifest, &observed);
    let observed_exit = observed["exit_code"].as_i64().unwrap_or(i64::MIN);
    let status = if mismatches.is_empty() && expected_exit == observed_exit {
        if expected_exit == 0 {
            "replayed"
        } else {
            "expected-failure-replayed"
        }
    } else {
        "drifted"
    };
    let report = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "operation": "replay",
        "evidence_mode": SUPPLIED_OBSERVATION_EVIDENCE_MODE,
        "evidence_mode_label": SUPPLIED_OBSERVATION_EVIDENCE_LABEL,
        "capsule_hash": capsule["capsule_hash"],
        "status": status,
        "expected_exit_code": expected_exit,
        "observed_exit_code": observed_exit,
        "mismatches": mismatches,
        "drift_behavior": "fail-closed",
    });
    Ok((i32::from(status == "drifted"), report))
}

/// A reviewed executable source. Capsule text selects a profile; it never selects a program.
#[derive(Clone, Copy, PartialEq, Eq)]
enum ProfileExecutable {
    /// The running Octet binary performs the reviewed job itself.
    SelfOctet,
    /// The caller declares a workspace-relative executable with the reviewed name.
    DeclaredWorkspaceExecutable,
}

/// One profile-owned argv token.
#[derive(Clone, Copy)]
enum ArgTemplate {
    Literal(&'static str),
    OutputDirectory,
    InputPath,
    InputBinding,
}

/// One output identity the reviewed profile produces and a capsule may expect.
#[derive(Clone, Copy)]
enum ProfileOutput {
    Stdout,
    Stderr,
    File(&'static str),
}

#[derive(Clone, Copy)]
enum ContractStream {
    Stdout,
    Stderr,
}

/// A reviewed named failure contract. An expected failure is only replayed when the executed run
/// satisfies this contract; an arbitrary nonzero exit is not enough.
#[derive(Clone, Copy)]
struct FailureContract {
    name: &'static str,
    stream: ContractStream,
    marker: &'static str,
}

/// A closed, reviewed native job profile. Everything the child may do comes from this table, not
/// from the capsule.
struct ReviewedProfile {
    id: &'static str,
    run_kind: &'static str,
    executable_name: &'static str,
    executable: ProfileExecutable,
    /// Spellings of argv[0] that a recorded command line may use for this profile. A recorded
    /// program outside this set is an imported executable and is rejected.
    argv0_aliases: &'static [&'static str],
    argv_prefix: &'static [ArgTemplate],
    argv_per_input: &'static [ArgTemplate],
    environment_allowlist: &'static [&'static str],
    fixed_environment: &'static [(&'static str, &'static str)],
    capabilities: &'static [&'static str],
    timeout_ms: u64,
    stdout_max_bytes: usize,
    stderr_max_bytes: usize,
    outputs: &'static [ProfileOutput],
    failure_contracts: &'static [FailureContract],
}

const FINGERPRINT_PRODUCTION_PROFILE: ReviewedProfile = ReviewedProfile {
    id: "fingerprint-production-v1",
    run_kind: "fingerprint",
    executable_name: "octet",
    executable: ProfileExecutable::SelfOctet,
    argv0_aliases: &["octet", "cargo-octet", "cargo"],
    argv_prefix: &[ArgTemplate::Literal("fingerprint"), ArgTemplate::Literal("--check")],
    argv_per_input: &[ArgTemplate::InputPath],
    environment_allowlist: &["PATH", "TMPDIR", "HOME", "LANG", "LC_ALL"],
    fixed_environment: &[("NO_COLOR", "1"), ("RUST_BACKTRACE", "0")],
    capabilities: &[
        "process-execute:self-octet-binary",
        "filesystem-read:declared-inputs",
        "filesystem-write:isolated-output-directory",
        "environment:allowlisted-only",
        "network:not-restricted-not-claimed",
        "cargo-build-scripts:not-applicable-no-cargo-invocation",
        "cargo-procedural-macros:not-applicable-no-cargo-invocation",
    ],
    timeout_ms: 120_000,
    stdout_max_bytes: 1024 * 1024,
    stderr_max_bytes: 1024 * 1024,
    outputs: &[ProfileOutput::Stdout, ProfileOutput::Stderr],
    failure_contracts: &[FailureContract {
        name: "fingerprint-drift",
        stream: ContractStream::Stdout,
        marker: "fingerprint-drift:",
    }],
};

const CHECK_PROFILE: ReviewedProfile = ReviewedProfile {
    id: "check-v1",
    run_kind: "check",
    executable_name: "octet-check-probe",
    executable: ProfileExecutable::DeclaredWorkspaceExecutable,
    argv0_aliases: &["octet-check-probe"],
    argv_prefix: &[ArgTemplate::Literal("--output-dir"), ArgTemplate::OutputDirectory],
    argv_per_input: &[ArgTemplate::Literal("--input"), ArgTemplate::InputBinding],
    environment_allowlist: &["PATH", "TMPDIR", "HOME", "LANG", "LC_ALL"],
    fixed_environment: &[("NO_COLOR", "1"), ("RUST_BACKTRACE", "0")],
    capabilities: &[
        "process-execute:declared-workspace-executable",
        "filesystem-read:declared-inputs-and-executable",
        "filesystem-write:isolated-output-directory",
        "environment:allowlisted-only",
        "network:not-restricted-not-claimed",
        "cargo-build-scripts:not-applicable-no-cargo-invocation",
        "cargo-procedural-macros:not-applicable-no-cargo-invocation",
    ],
    timeout_ms: 60_000,
    stdout_max_bytes: 64 * 1024,
    stderr_max_bytes: 64 * 1024,
    outputs: &[ProfileOutput::File("check-report.json"), ProfileOutput::Stderr],
    failure_contracts: &[FailureContract {
        name: "check-diagnostic-failure",
        stream: ContractStream::Stderr,
        marker: "octet-check-failure:",
    }],
};

const REVIEWED_PROFILES: [&ReviewedProfile; 2] = [&CHECK_PROFILE, &FINGERPRINT_PRODUCTION_PROFILE];

/// The caller-owned half of executed replay: which local files stand in for the capsule inputs and
/// which reviewed executable to run. Capsule text cannot supply any of these.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ExecutionInputsDocument {
    #[serde(default)]
    schema_version: Option<u64>,
    #[serde(default)]
    profile: Option<String>,
    #[serde(default)]
    executable: Option<String>,
    #[serde(default)]
    inputs: BTreeMap<String, String>,
    #[serde(default)]
    config_hash: Option<String>,
    #[serde(default)]
    nix_closure_identity: Option<String>,
    #[serde(default)]
    environment: BTreeMap<String, String>,
}

#[derive(Clone, Copy, Debug)]
struct EffectiveLimits {
    timeout_ms: u64,
    stdout_max_bytes: usize,
    stderr_max_bytes: usize,
}

struct BoundInput {
    name: String,
    path: PathBuf,
    normalized_path: String,
    hash: String,
}

/// One precise difference between a capsule expectation and an independently collected fact.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
struct Drift {
    kind: &'static str,
    field: String,
    expected: String,
    actual: String,
}

struct PreparedExecution {
    profile: &'static ReviewedProfile,
    executable: PathBuf,
    executable_hash: String,
    octet_identity: String,
    inputs: Vec<BoundInput>,
    config_hash: String,
    nix_closure: Option<String>,
    environment: Vec<(OsString, OsString)>,
    environment_names: Vec<String>,
    environment_overrides: Vec<String>,
    limits: EffectiveLimits,
    failure_contract: Option<&'static FailureContract>,
    comparison_argv: Vec<String>,
    expected_outputs: BTreeMap<String, String>,
    expected_exit_code: i32,
    output_dir: PathBuf,
    pre_execution_drifts: Vec<Drift>,
}

enum Preparation {
    Ready(Box<PreparedExecution>),
    Failed { reason: &'static str, detail: String },
}

enum DeclaredPath {
    Present(PathBuf),
    Missing(PathBuf),
}

enum OutputFile {
    Present(Vec<u8>),
    Missing,
    Oversized(u64),
}

/// Whether a reviewed loader may appear in a caller-declared executable path. This is stricter
/// than an input path because the executable is spawned directly.
fn is_executable_path_character(character: char) -> bool {
    character.is_ascii_alphanumeric() || matches!(character, '-' | '_' | '.' | '/')
}

fn contains_shell_fragment(text: &str) -> bool {
    text.chars().any(|character| SHELL_FRAGMENT_CHARACTERS.contains(&character))
}

fn byte_identity(bytes: &[u8]) -> String {
    format!("{BLAKE3_IDENTITY_PREFIX}{}", crate::evidence::artifact_blake3_hex(bytes))
}

fn file_identity(path: &Path) -> Result<String, String> {
    let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    Ok(byte_identity(&bytes))
}

#[cfg(unix)]
fn is_executable_file(path: &Path) -> bool {
    use std::os::unix::fs::PermissionsExt as _;

    fs::metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.permissions().mode() & 0o111 != 0)
}

#[cfg(not(unix))]
fn is_executable_file(path: &Path) -> bool {
    path.is_file()
}

/// Binds a caller-declared path to the workspace. Parent components are rejected before touching
/// the filesystem, and a path whose resolved form escapes the resolved workspace root is refused,
/// so neither `..` nor a symlink can leave the reviewed root.
fn bind_declared_path(raw: &str, workspace: &Path, role: &str, executable: bool) -> Result<DeclaredPath, String> {
    if raw.is_empty() {
        return Err(format!("{role} path is empty"));
    }
    if raw.contains('\0') {
        return Err(format!("{role} path `{raw}` contains a NUL byte"));
    }
    if raw.starts_with('-') {
        return Err(format!("{role} path `{raw}` may not start with `-`"));
    }
    if executable && !raw.chars().all(is_executable_path_character) {
        return Err(format!("{role} path `{raw}` contains characters outside the reviewed executable path set"));
    }
    let declared = Path::new(raw);
    if declared
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::Prefix(_)))
    {
        return Err(format!("path escape: {role} path `{raw}` contains a parent or prefix component"));
    }
    let joined = if declared.is_absolute() {
        declared.to_path_buf()
    } else {
        workspace.join(declared)
    };
    if !joined.exists() {
        return Ok(DeclaredPath::Missing(joined));
    }
    let canonical_workspace =
        fs::canonicalize(workspace).map_err(|error| format!("resolve workspace {}: {error}", workspace.display()))?;
    let canonical = fs::canonicalize(&joined).map_err(|error| format!("resolve {role} path `{raw}`: {error}"))?;
    if !canonical.starts_with(&canonical_workspace) {
        return Err(format!("path escape: {role} path `{raw}` resolves outside the workspace root"));
    }
    Ok(DeclaredPath::Present(joined))
}

/// Normalizes a bound path to the same placeholder spelling `capsule capture` records, falling back
/// to the workspace root this replay was given.
fn normalize_bound_path(path: &Path, workspace: &Path, local_roots: &LocalRoots) -> String {
    let text = path.to_str().unwrap_or_default();
    if let Some(normalized) = local_roots.normalize(text) {
        return normalized;
    }
    let workspace_text = workspace.to_str().unwrap_or_default().trim_end_matches(std::path::is_separator);
    if !workspace_text.is_empty()
        && let Some(remainder) = text.strip_prefix(workspace_text)
        && (remainder.is_empty() || remainder.starts_with(std::path::is_separator))
    {
        return format!("{WORKSPACE_PLACEHOLDER}{remainder}");
    }
    text.to_owned()
}

/// Reads a capsule's named identities, which executed replay requires as a name-to-`b3:`-identity
/// object so every input and output can be bound individually.
fn named_hash_map(value: Option<&Value>, field: &str) -> Result<BTreeMap<String, String>, String> {
    let Some(Value::Object(map)) = value else {
        return Err(format!(
            "capsule {field} must be a JSON object mapping names to `{BLAKE3_IDENTITY_PREFIX}` identities for executed replay"
        ));
    };
    let mut identities = BTreeMap::new();
    for (name, value) in map {
        let Some(identity) = value.as_str() else {
            return Err(format!("capsule {field}.{name} must be a string identity"));
        };
        validate_artifact_hash(identity).map_err(|error| format!("capsule {field}.{name}: {error}"))?;
        identities.insert(name.clone(), identity.to_owned());
    }
    Ok(identities)
}

fn capsule_is_import_only(manifest: &Value) -> bool {
    ["imported", "import_only"]
        .iter()
        .any(|field| manifest.get(*field).and_then(Value::as_bool) == Some(true))
        || manifest.get("run_kind").and_then(Value::as_str).is_some_and(|kind| kind.starts_with("imported"))
        || manifest.get("capsule_kind").and_then(Value::as_str).is_some_and(|kind| kind.contains("import"))
}

/// Rejects capsule command text that could confer execution authority: shell fragments,
/// non-reviewed programs, and a recorded argv that is not a single-token vector.
fn validate_capsule_command_argv(profile: &ReviewedProfile, argv: &[String]) -> Result<(), String> {
    let Some(program) = argv.first() else {
        return Err("capsule manifest command_argv must not be empty".to_owned());
    };
    if argv.iter().any(|argument| contains_shell_fragment(argument)) {
        return Err(format!(
            "capsule command text contains a shell fragment; executed replay accepts single tokens only: {argv:?}"
        ));
    }
    if !profile.argv0_aliases.contains(&program.as_str()) {
        return Err(format!(
            "imported executable `{program}` is not the reviewed program for profile `{}` (expected one of {:?})",
            profile.id, profile.argv0_aliases
        ));
    }
    Ok(())
}

fn render_argv_template(template: ArgTemplate, input: Option<&BoundInput>) -> Result<String, String> {
    match (template, input) {
        (ArgTemplate::Literal(literal), _) => Ok(literal.to_owned()),
        (ArgTemplate::OutputDirectory, _) => Ok(OUTPUT_PLACEHOLDER.to_owned()),
        (ArgTemplate::InputPath, Some(input)) => Ok(input.normalized_path.clone()),
        (ArgTemplate::InputBinding, Some(input)) => Ok(format!("{}={}", input.name, input.normalized_path)),
        (ArgTemplate::InputPath | ArgTemplate::InputBinding, None) => {
            Err("reviewed profile uses a per-input argument outside the per-input section".to_owned())
        }
    }
}

fn render_process_argument(
    template: ArgTemplate,
    input: Option<&BoundInput>,
    output_dir: &Path,
) -> Result<String, String> {
    match (template, input) {
        (ArgTemplate::Literal(literal), _) => Ok(literal.to_owned()),
        (ArgTemplate::OutputDirectory, _) => output_dir
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("isolated output directory {} is not valid UTF-8", output_dir.display())),
        (ArgTemplate::InputPath, Some(input)) => input
            .path
            .to_str()
            .map(str::to_owned)
            .ok_or_else(|| format!("input `{}` is not valid UTF-8", input.name)),
        (ArgTemplate::InputBinding, Some(input)) => {
            let path = input.path.to_str().ok_or_else(|| format!("input `{}` is not valid UTF-8", input.name))?;
            Ok(format!("{}={path}", input.name))
        }
        (ArgTemplate::InputPath | ArgTemplate::InputBinding, None) => {
            Err("reviewed profile uses a per-input argument outside the per-input section".to_owned())
        }
    }
}

fn comparison_argv(profile: &ReviewedProfile, inputs: &[BoundInput]) -> Result<Vec<String>, String> {
    let mut argv = vec![profile.executable_name.to_owned()];
    for template in profile.argv_prefix {
        argv.push(render_argv_template(*template, None)?);
    }
    for input in inputs {
        for template in profile.argv_per_input {
            argv.push(render_argv_template(*template, Some(input))?);
        }
    }
    Ok(argv)
}

fn process_arguments(
    profile: &ReviewedProfile,
    inputs: &[BoundInput],
    output_dir: &Path,
) -> Result<Vec<OsString>, String> {
    let mut arguments = Vec::new();
    for template in profile.argv_prefix {
        arguments.push(OsString::from(render_process_argument(*template, None, output_dir)?));
    }
    for input in inputs {
        for template in profile.argv_per_input {
            arguments.push(OsString::from(render_process_argument(*template, Some(input), output_dir)?));
        }
    }
    Ok(arguments)
}

fn default_inputs_path(capsule_path: &Path) -> PathBuf {
    let file_name = capsule_path.file_name().and_then(|name| name.to_str()).unwrap_or("capsule.json");
    let stem = file_name.strip_suffix(".json").unwrap_or(file_name);
    let mut path = capsule_path.to_path_buf();
    path.set_file_name(format!("{stem}.inputs.json"));
    path
}

fn read_execution_inputs_document(path: &Path) -> Result<Option<ExecutionInputsDocument>, String> {
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text)
        .map(Some)
        .map_err(|error| format!("parse execution inputs {}: {error}", path.display()))
}

/// Applies a capsule's optional `execution.resource_limits`, which can only tighten the reviewed
/// profile limits and never loosen them.
fn effective_limits(
    execution: Option<&Value>,
    profile: &ReviewedProfile,
) -> Result<EffectiveLimits, (&'static str, String)> {
    let mut limits = EffectiveLimits {
        timeout_ms: profile.timeout_ms,
        stdout_max_bytes: profile.stdout_max_bytes,
        stderr_max_bytes: profile.stderr_max_bytes,
    };
    let Some(declared) = execution.and_then(|execution| execution.get("resource_limits")) else {
        return Ok(limits);
    };
    let Value::Object(declared) = declared else {
        return Err((
            "unsupported-resource-limits",
            "capsule execution.resource_limits must be a JSON object".to_owned(),
        ));
    };
    for (name, value) in declared {
        let Some(value) = value.as_u64() else {
            return Err((
                "unsupported-resource-limits",
                format!("capsule execution.resource_limits.{name} must be a positive integer"),
            ));
        };
        match name.as_str() {
            "timeout_ms" => {
                if value == 0 || value > profile.timeout_ms {
                    return Err((
                        "unsupported-resource-limits",
                        format!(
                            "capsule execution.resource_limits.timeout_ms {value} must be in 1..={}",
                            profile.timeout_ms
                        ),
                    ));
                }
                limits.timeout_ms = value;
            }
            "stdout_max_bytes" => {
                let value = usize::try_from(value)
                    .map_err(|_| ("unsupported-resource-limits", "stdout_max_bytes is too large".to_owned()))?;
                if value == 0 || value > profile.stdout_max_bytes {
                    return Err((
                        "unsupported-resource-limits",
                        format!(
                            "capsule execution.resource_limits.stdout_max_bytes must be in 1..={}",
                            profile.stdout_max_bytes
                        ),
                    ));
                }
                limits.stdout_max_bytes = value;
            }
            "stderr_max_bytes" => {
                let value = usize::try_from(value)
                    .map_err(|_| ("unsupported-resource-limits", "stderr_max_bytes is too large".to_owned()))?;
                if value == 0 || value > profile.stderr_max_bytes {
                    return Err((
                        "unsupported-resource-limits",
                        format!(
                            "capsule execution.resource_limits.stderr_max_bytes must be in 1..={}",
                            profile.stderr_max_bytes
                        ),
                    ));
                }
                limits.stderr_max_bytes = value;
            }
            other => {
                return Err((
                    "unsupported-resource-limits",
                    format!("capsule execution.resource_limits.{other} is not a reviewed limit"),
                ));
            }
        }
    }
    Ok(limits)
}

fn prepare_execution(
    profile: &'static ReviewedProfile,
    manifest: &Value,
    inputs_path: &Path,
    workspace: &Path,
    local_roots: &LocalRoots,
) -> Result<Preparation, String> {
    let Some(document) = read_execution_inputs_document(inputs_path)? else {
        return Ok(Preparation::Failed {
            reason: "missing-inputs",
            detail: format!("execution inputs document {} does not exist", inputs_path.display()),
        });
    };
    if let Some(schema_version) = document.schema_version
        && schema_version != u64::from(SCHEMA_VERSION)
    {
        return Ok(Preparation::Failed {
            reason: "unsupported-inputs-schema",
            detail: format!("execution inputs schema_version {schema_version} is unsupported"),
        });
    }
    if let Some(declared_profile) = document.profile.as_deref()
        && declared_profile != profile.id
    {
        return Ok(Preparation::Failed {
            reason: "profile-mismatch",
            detail: format!(
                "execution inputs name profile `{declared_profile}` but the capsule selects `{}`",
                profile.id
            ),
        });
    }
    let expected_exit = manifest.get("expected_exit_code").and_then(Value::as_i64).unwrap_or_default();
    let Ok(expected_exit_code) = i32::try_from(expected_exit) else {
        return Ok(Preparation::Failed {
            reason: "unsupported-exit-code",
            detail: format!("capsule expected_exit_code {expected_exit} is not a portable exit code"),
        });
    };
    if !(0..=255).contains(&expected_exit) {
        return Ok(Preparation::Failed {
            reason: "unsupported-exit-code",
            detail: format!("capsule expected_exit_code {expected_exit} is not a portable exit code"),
        });
    }
    let execution = match manifest.get("execution") {
        None => None,
        Some(value) if value.is_object() => Some(value),
        Some(_) => {
            return Ok(Preparation::Failed {
                reason: "unsupported-execution-binding",
                detail: "capsule execution must be a JSON object".to_owned(),
            });
        }
    };
    let Some(capsule_config) = manifest.get("config_hash").and_then(Value::as_str) else {
        return Ok(Preparation::Failed {
            reason: "unsupported-config-binding",
            detail: "capsule config_hash must be a string for executed replay".to_owned(),
        });
    };
    let Some(capsule_identity) = manifest.get("octet_binary_identity").and_then(Value::as_str) else {
        return Ok(Preparation::Failed {
            reason: "unsupported-toolchain-binding",
            detail: "capsule octet_binary_identity must be a string for executed replay".to_owned(),
        });
    };
    let Some(config_hash) = document.config_hash.clone() else {
        return Ok(Preparation::Failed {
            reason: "missing-config",
            detail: "execution inputs must bind config_hash".to_owned(),
        });
    };
    let expected_outputs = match named_hash_map(manifest.get("output_hashes"), "output_hashes") {
        Ok(outputs) => outputs,
        Err(detail) => {
            return Ok(Preparation::Failed {
                reason: "unsupported-output-binding",
                detail,
            });
        }
    };
    let capsule_inputs = match named_hash_map(manifest.get("input_hashes"), "input_hashes") {
        Ok(inputs) => inputs,
        Err(detail) => {
            return Ok(Preparation::Failed {
                reason: "unsupported-input-binding",
                detail,
            });
        }
    };

    let current_exe =
        std::env::current_exe().map_err(|error| format!("resolve the running Octet executable: {error}"))?;
    let octet_identity = file_identity(&current_exe)?;
    let executable = match profile.executable {
        ProfileExecutable::SelfOctet => {
            if document.executable.is_some() {
                return Ok(Preparation::Failed {
                    reason: "unexpected-executable",
                    detail: format!(
                        "profile `{}` runs the Octet binary itself and does not accept a declared executable",
                        profile.id
                    ),
                });
            }
            current_exe.clone()
        }
        ProfileExecutable::DeclaredWorkspaceExecutable => {
            let Some(declared) = document.executable.as_deref() else {
                return Ok(Preparation::Failed {
                    reason: "missing-executable",
                    detail: format!("profile `{}` requires `executable` in the execution inputs", profile.id),
                });
            };
            match bind_declared_path(declared, workspace, "executable", true)? {
                DeclaredPath::Missing(path) => {
                    return Ok(Preparation::Failed {
                        reason: "missing-executable",
                        detail: format!("declared executable {} does not exist", path.display()),
                    });
                }
                DeclaredPath::Present(path) => {
                    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or_default();
                    if name != profile.executable_name {
                        return Err(format!(
                            "imported executable `{declared}` does not have the reviewed name `{}`",
                            profile.executable_name
                        ));
                    }
                    if !is_executable_file(&path) {
                        return Err(format!("declared executable {} is not an executable file", path.display()));
                    }
                    path
                }
            }
        }
    };
    let executable_hash = if profile.executable == ProfileExecutable::SelfOctet {
        octet_identity.clone()
    } else {
        file_identity(&executable)?
    };

    let missing_inputs: Vec<String> =
        capsule_inputs.keys().filter(|name| !document.inputs.contains_key(*name)).cloned().collect();
    if !missing_inputs.is_empty() {
        return Ok(Preparation::Failed {
            reason: "missing-input",
            detail: format!("execution inputs do not bind capsule inputs {missing_inputs:?}"),
        });
    }
    let undeclared_inputs: Vec<String> =
        document.inputs.keys().filter(|name| !capsule_inputs.contains_key(*name)).cloned().collect();
    if !undeclared_inputs.is_empty() {
        return Ok(Preparation::Failed {
            reason: "undeclared-input",
            detail: format!("execution inputs declare inputs the capsule does not record: {undeclared_inputs:?}"),
        });
    }

    let mut pre_execution_drifts = Vec::new();
    let mut inputs = Vec::with_capacity(capsule_inputs.len());
    for (name, expected_hash) in &capsule_inputs {
        let declared = document.inputs.get(name).map(String::as_str).unwrap_or_default();
        let path = match bind_declared_path(declared, workspace, "input", false)? {
            DeclaredPath::Missing(path) => {
                return Ok(Preparation::Failed {
                    reason: "missing-input",
                    detail: format!("input `{name}` at {} does not exist", path.display()),
                });
            }
            DeclaredPath::Present(path) => path,
        };
        if !path.is_file() {
            return Ok(Preparation::Failed {
                reason: "unsupported-input",
                detail: format!("input `{name}` at {} is not a regular file", path.display()),
            });
        }
        let size = fs::metadata(&path).map_err(|error| format!("stat {}: {error}", path.display()))?.len();
        if size > EXECUTION_INPUT_FILE_MAX_BYTES {
            return Ok(Preparation::Failed {
                reason: "unsupported-input",
                detail: format!(
                    "input `{name}` is {size} bytes, above the {EXECUTION_INPUT_FILE_MAX_BYTES} byte limit"
                ),
            });
        }
        let hash = file_identity(&path)?;
        if &hash != expected_hash {
            pre_execution_drifts.push(Drift {
                kind: "source",
                field: name.clone(),
                expected: expected_hash.clone(),
                actual: hash.clone(),
            });
        }
        inputs.push(BoundInput {
            name: name.clone(),
            normalized_path: normalize_bound_path(&path, workspace, local_roots),
            path,
            hash,
        });
    }

    if capsule_config != config_hash {
        pre_execution_drifts.push(Drift {
            kind: "config",
            field: "config_hash".to_owned(),
            expected: capsule_config.to_owned(),
            actual: config_hash.clone(),
        });
    }
    if capsule_identity != octet_identity {
        pre_execution_drifts.push(Drift {
            kind: "toolchain",
            field: "octet_binary_identity".to_owned(),
            expected: capsule_identity.to_owned(),
            actual: octet_identity.clone(),
        });
    }
    if let Some(toolchain) = execution.and_then(|execution| execution.get("toolchain")) {
        if !toolchain.is_object() {
            return Ok(Preparation::Failed {
                reason: "unsupported-toolchain-binding",
                detail: "capsule execution.toolchain must be a JSON object".to_owned(),
            });
        }
        if let Some(declared_hash) = toolchain.get("executable_hash") {
            let Some(declared_hash) = declared_hash.as_str() else {
                return Ok(Preparation::Failed {
                    reason: "unsupported-toolchain-binding",
                    detail: "capsule execution.toolchain.executable_hash must be a string".to_owned(),
                });
            };
            if declared_hash != executable_hash {
                pre_execution_drifts.push(Drift {
                    kind: "toolchain",
                    field: "executable_hash".to_owned(),
                    expected: declared_hash.to_owned(),
                    actual: executable_hash.clone(),
                });
            }
        }
        if let Some(declared_closure) = toolchain.get("nix_closure_identity") {
            let Some(declared_closure) = declared_closure.as_str() else {
                return Ok(Preparation::Failed {
                    reason: "unsupported-toolchain-binding",
                    detail: "capsule execution.toolchain.nix_closure_identity must be a string".to_owned(),
                });
            };
            let Some(actual_closure) = document.nix_closure_identity.as_deref() else {
                return Ok(Preparation::Failed {
                    reason: "missing-tool-closure",
                    detail: format!(
                        "capsule binds Nix closure `{declared_closure}` but the execution inputs provide no nix_closure_identity"
                    ),
                });
            };
            if declared_closure != actual_closure {
                pre_execution_drifts.push(Drift {
                    kind: "toolchain",
                    field: "nix_closure_identity".to_owned(),
                    expected: declared_closure.to_owned(),
                    actual: actual_closure.to_owned(),
                });
            }
        }
    }

    let recorded_argv = match manifest.get("command_argv") {
        Some(Value::Array(items)) => {
            let mut argv = Vec::with_capacity(items.len());
            for (index, item) in items.iter().enumerate() {
                let Some(argument) = item.as_str() else {
                    return Err(format!("capsule manifest command_argv[{index}] must be a string"));
                };
                argv.push(argument.to_owned());
            }
            argv
        }
        _ => return Err("capsule manifest command_argv must be an array of strings".to_owned()),
    };
    validate_capsule_command_argv(profile, &recorded_argv)?;
    let comparison_argv = comparison_argv(profile, &inputs)?;
    let mut recorded_argv = recorded_argv;
    if let Some(program) = recorded_argv.first_mut() {
        *program = profile.executable_name.to_owned();
    }
    if recorded_argv != comparison_argv {
        pre_execution_drifts.push(Drift {
            kind: "argv",
            field: "command_argv".to_owned(),
            expected: recorded_argv.join(" "),
            actual: comparison_argv.join(" "),
        });
    }

    let declared_contract = execution.and_then(|execution| execution.get("failure_contract"));
    let failure_contract = if expected_exit != 0 {
        let Some(name) = declared_contract.and_then(Value::as_str) else {
            return Ok(Preparation::Failed {
                reason: "missing-failure-contract",
                detail: format!(
                    "expected exit {expected_exit} requires a named failure contract reviewed for profile `{}`",
                    profile.id
                ),
            });
        };
        match profile.failure_contracts.iter().find(|contract| contract.name == name) {
            Some(contract) => Some(contract),
            None => {
                return Ok(Preparation::Failed {
                    reason: "unsupported-failure-contract",
                    detail: format!("failure contract `{name}` is not reviewed for profile `{}`", profile.id),
                });
            }
        }
    } else {
        if declared_contract.is_some() {
            return Ok(Preparation::Failed {
                reason: "unexpected-failure-contract",
                detail: "a capsule failure contract requires a non-zero expected_exit_code".to_owned(),
            });
        }
        None
    };
    let limits = match effective_limits(execution, profile) {
        Ok(limits) => limits,
        Err((reason, detail)) => return Ok(Preparation::Failed { reason, detail }),
    };

    let fixed_names: BTreeSet<&str> = profile.fixed_environment.iter().map(|(name, _)| *name).collect();
    for name in document.environment.keys() {
        if !profile.environment_allowlist.contains(&name.as_str()) {
            return Err(format!(
                "undeclared environment `{name}` is not in the profile `{}` allowlist {:?}",
                profile.id, profile.environment_allowlist
            ));
        }
        if fixed_names.contains(name.as_str()) {
            return Err(format!("environment `{name}` is fixed by profile `{}` and cannot be overridden", profile.id));
        }
    }
    let mut environment: BTreeMap<OsString, OsString> = BTreeMap::new();
    for name in profile.environment_allowlist {
        if let Some(value) = std::env::var_os(name) {
            environment.insert(OsString::from(*name), value);
        }
    }
    for (name, value) in profile.fixed_environment {
        environment.insert(OsString::from(*name), OsString::from(*value));
    }
    for (name, value) in &document.environment {
        environment.insert(OsString::from(name), OsString::from(value));
    }
    let environment: Vec<(OsString, OsString)> = environment.into_iter().collect();
    let environment_names: Vec<String> =
        environment.iter().filter_map(|(name, _)| name.to_str().map(str::to_owned)).collect();
    let environment_overrides: Vec<String> = document.environment.keys().cloned().collect();

    let output_dir = workspace.join(EXECUTION_OUTPUT_DIRECTORY).join(capsule_hash_file_name(manifest, profile)?);

    Ok(Preparation::Ready(Box::new(PreparedExecution {
        profile,
        executable,
        executable_hash,
        octet_identity,
        inputs,
        config_hash,
        nix_closure: document.nix_closure_identity,
        environment,
        environment_names,
        environment_overrides,
        limits,
        failure_contract,
        comparison_argv,
        expected_outputs,
        expected_exit_code,
        output_dir,
        pre_execution_drifts,
    })))
}

/// The isolated output directory name is derived from the verified capsule manifest, so a stale
/// directory is always cleared rather than silently reused.
fn capsule_hash_file_name(manifest: &Value, profile: &ReviewedProfile) -> Result<String, String> {
    let identity = determinism::full_hash(&canonical_json(manifest)?);
    Ok(format!("{}-{}", profile.id, identity.replace(':', "_")))
}

fn run_prepared_execution(
    prepared: &PreparedExecution,
    workspace: &Path,
) -> Result<bounded_exec::ExecutionOutput, String> {
    if prepared.output_dir.exists() {
        fs::remove_dir_all(&prepared.output_dir)
            .map_err(|error| format!("clear isolated output directory {}: {error}", prepared.output_dir.display()))?;
    }
    fs::create_dir_all(&prepared.output_dir)
        .map_err(|error| format!("create isolated output directory {}: {error}", prepared.output_dir.display()))?;
    let arguments = process_arguments(prepared.profile, &prepared.inputs, &prepared.output_dir)?;
    let current_dir =
        fs::canonicalize(workspace).map_err(|error| format!("resolve workspace {}: {error}", workspace.display()))?;
    let outcome_policy = bounded_exec::OutcomePolicy::new(vec![prepared.expected_exit_code], false, false)
        .map_err(|error| format!("construct profile outcome policy: {error:?}"))?;
    let request = bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: prepared.executable.clone(),
            args: arguments,
            current_dir,
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: prepared.environment.clone(),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: prepared.limits.timeout_ms,
            stdin_max_bytes: EXECUTION_STDIN_MAX_BYTES,
            stdout_max_bytes: prepared.limits.stdout_max_bytes,
            stderr_max_bytes: prepared.limits.stderr_max_bytes,
            poll_interval_ms: EXECUTION_POLL_INTERVAL_MS.min(prepared.limits.timeout_ms),
            teardown_timeout_ms: EXECUTION_TEARDOWN_TIMEOUT_MS,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy,
    };
    bounded_exec::run(request).map_err(|error| format!("run reviewed profile `{}`: {error}", prepared.profile.id))
}

fn read_output_file(output_dir: &Path, name: &str) -> Result<OutputFile, String> {
    if name.is_empty()
        || name == "."
        || name == ".."
        || name.contains('/')
        || name.contains('\\')
        || name.contains('\0')
    {
        return Err(format!("reviewed profile output `{name}` must be a simple file name"));
    }
    let path = output_dir.join(name);
    if !path.exists() {
        return Ok(OutputFile::Missing);
    }
    let metadata = fs::symlink_metadata(&path).map_err(|error| format!("stat output {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Ok(OutputFile::Missing);
    }
    if metadata.len() > EXECUTION_OUTPUT_FILE_MAX_BYTES {
        return Ok(OutputFile::Oversized(metadata.len()));
    }
    fs::read(&path)
        .map(OutputFile::Present)
        .map_err(|error| format!("read output {}: {error}", path.display()))
}

fn completion_name(completion: bounded_exec::Completion) -> &'static str {
    match completion {
        bounded_exec::Completion::Exited => "exited",
        bounded_exec::Completion::TimedOut => "timed-out",
        bounded_exec::Completion::Cancelled => "cancelled",
    }
}

fn contract_satisfied(contract: &FailureContract, output: &bounded_exec::ExecutionOutput) -> bool {
    let bytes = match contract.stream {
        ContractStream::Stdout => &output.stdout.bytes,
        ContractStream::Stderr => &output.stderr.bytes,
    };
    String::from_utf8_lossy(bytes).contains(contract.marker)
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptProfile {
    id: String,
    run_kind: String,
    executable_name: String,
    executable_source: String,
    executable_hash: String,
    argv: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptInput {
    path: String,
    hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptSource {
    inputs: BTreeMap<String, ReceiptInput>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptPolicy {
    config_hash: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptToolchain {
    octet_binary_identity: String,
    executable_hash: String,
    nix_closure_identity: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptEnvironment {
    allowlist: Vec<String>,
    fixed: BTreeMap<String, String>,
    present: Vec<String>,
    overrides: Vec<String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptLimits {
    timeout_ms: u64,
    stdout_max_bytes: usize,
    stderr_max_bytes: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptExpected {
    exit_code: i64,
    outputs: BTreeMap<String, String>,
    failure_contract: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptStream {
    hash: String,
    bytes: u64,
    truncated: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptOutput {
    hash: String,
    bytes: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ReceiptOutcome {
    completion: String,
    exit_code: Option<i32>,
    signal: Option<i32>,
    timed_out: bool,
    stdout: ReceiptStream,
    stderr: ReceiptStream,
    outputs: BTreeMap<String, ReceiptOutput>,
    output_directory: String,
}

/// Versioned, Rust-owned execution receipt. The `receipt_hash` covers every other field, so a
/// fabricated or edited receipt cannot pass [`verify_execution_receipt`].
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ExecutionReceiptBody {
    schema_version: u32,
    receipt_kind: String,
    capsule_hash: String,
    evidence_mode: String,
    profile: ReceiptProfile,
    source: ReceiptSource,
    policy: ReceiptPolicy,
    toolchain: ReceiptToolchain,
    capabilities: Vec<String>,
    environment: ReceiptEnvironment,
    limits: ReceiptLimits,
    expected: ReceiptExpected,
    outcome: ReceiptOutcome,
    status: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
struct ExecutionReceipt {
    #[serde(flatten)]
    body: ExecutionReceiptBody,
    receipt_hash: String,
}

fn verify_execution_receipt(path: &Path, capsule_hash: &str) -> Result<ExecutionReceipt, String> {
    let receipt: ExecutionReceipt = serde_json::from_value(read_json(path)?)
        .map_err(|error| format!("parse execution receipt {}: {error}", path.display()))?;
    if receipt.body.schema_version != SCHEMA_VERSION {
        return Err(format!("execution receipt {} schema_version must be {SCHEMA_VERSION}", path.display()));
    }
    if receipt.body.receipt_kind != EXECUTION_RECEIPT_KIND {
        return Err(format!("execution receipt {} receipt_kind must be `{EXECUTION_RECEIPT_KIND}`", path.display()));
    }
    let body = serde_json::to_value(&receipt.body)
        .map_err(|error| format!("serialize execution receipt {} body: {error}", path.display()))?;
    let recomputed = determinism::full_hash(&canonical_json(&body)?);
    if recomputed != receipt.receipt_hash {
        return Err(format!(
            "execution receipt {} was modified: recorded receipt_hash `{}` does not match the recomputed `{recomputed}`",
            path.display(),
            receipt.receipt_hash
        ));
    }
    if receipt.body.capsule_hash != capsule_hash {
        return Err(format!(
            "execution receipt {} is linked to capsule `{}`, not `{capsule_hash}`",
            path.display(),
            receipt.body.capsule_hash
        ));
    }
    Ok(receipt)
}

fn capsule_field_for_kind(kind: &str) -> &'static str {
    match kind {
        "source" => "input_hashes",
        "config" => "config_hash",
        "toolchain" => "octet_binary_identity",
        "argv" => "command_argv",
        "output" => "output_hashes",
        "exit" => "expected_exit_code",
        "failure_contract" => "execution.failure_contract",
        "receipt" => "receipt",
        _ => "execution",
    }
}

struct ExecutedReport<'a> {
    capsule_hash: &'a str,
    profile_id: &'a str,
    run_kind: &'a str,
    status: &'a str,
    reason: Option<&'a str>,
    detail: Option<&'a str>,
    expected_exit: i64,
    observed_exit: Option<i64>,
    drifts: &'a [Drift],
    failure_contract: Option<(&'a str, bool)>,
    limits: Option<&'a EffectiveLimits>,
    executed: bool,
    receipt: Option<(&'a Path, &'a str)>,
}

fn executed_report(input: &ExecutedReport<'_>) -> Value {
    let mismatches: Vec<&'static str> = input
        .drifts
        .iter()
        .map(|drift| capsule_field_for_kind(drift.kind))
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect();
    serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "operation": "replay",
        "evidence_mode": EXECUTED_EVIDENCE_MODE,
        "evidence_mode_label": EXECUTED_EVIDENCE_LABEL,
        "capsule_hash": input.capsule_hash,
        "profile": input.profile_id,
        "run_kind": input.run_kind,
        "status": input.status,
        "reason": input.reason,
        "detail": input.detail,
        "expected_exit_code": input.expected_exit,
        "observed_exit_code": input.observed_exit,
        "drifts": input.drifts,
        "mismatches": mismatches,
        "failure_contract": input.failure_contract.map(|(name, satisfied)| serde_json::json!({
            "name": name,
            "satisfied": satisfied,
        })),
        "limits": input.limits.map(|limits| serde_json::json!({
            "timeout_ms": limits.timeout_ms,
            "stdout_max_bytes": limits.stdout_max_bytes,
            "stderr_max_bytes": limits.stderr_max_bytes,
        })),
        "executed": input.executed,
        "receipt": input.receipt.map(|(path, receipt_hash)| serde_json::json!({
            "path": path.display().to_string(),
            "receipt_hash": receipt_hash,
        })),
        "drift_behavior": "fail-closed",
    })
}

fn report_string(report: &Value, field: &str) -> String {
    report.get(field).and_then(Value::as_str).unwrap_or("-").to_owned()
}

fn observed_human_lines(report: &Value) -> Vec<String> {
    let mut lines = vec![
        format!("capsule replay ({SUPPLIED_OBSERVATION_EVIDENCE_MODE})"),
        format!("  evidence mode: {SUPPLIED_OBSERVATION_EVIDENCE_MODE} ({SUPPLIED_OBSERVATION_EVIDENCE_LABEL})"),
        format!("  capsule: {}", report_string(report, "capsule_hash")),
        format!("  status: {}", report_string(report, "status")),
        format!("  expected exit: {}", report.get("expected_exit_code").map(Value::to_string).unwrap_or_default()),
        format!("  observed exit: {}", report.get("observed_exit_code").map(Value::to_string).unwrap_or_default()),
    ];
    let mismatches = report
        .get("mismatches")
        .and_then(Value::as_array)
        .map(|fields| fields.iter().filter_map(Value::as_str).collect::<Vec<_>>().join(", "))
        .unwrap_or_default();
    lines.push(format!(
        "  mismatches: {}",
        if mismatches.is_empty() {
            "none".to_owned()
        } else {
            mismatches
        }
    ));
    lines
}

fn executed_human_lines(report: &Value) -> Vec<String> {
    let mut lines = vec![
        format!("capsule replay ({EXECUTED_EVIDENCE_MODE})"),
        format!("  evidence mode: {EXECUTED_EVIDENCE_MODE} ({EXECUTED_EVIDENCE_LABEL})"),
        format!("  capsule: {}", report_string(report, "capsule_hash")),
        format!("  profile: {}", report_string(report, "profile")),
        format!("  status: {}", report_string(report, "status")),
    ];
    if let Some(reason) = report.get("reason").and_then(Value::as_str) {
        lines.push(format!("  reason: {reason}"));
    }
    if let Some(detail) = report.get("detail").and_then(Value::as_str) {
        lines.push(format!("  detail: {detail}"));
    }
    lines.push(format!(
        "  expected exit: {}",
        report.get("expected_exit_code").map(Value::to_string).unwrap_or_default()
    ));
    lines.push(format!(
        "  observed exit: {}",
        report.get("observed_exit_code").map(Value::to_string).unwrap_or_default()
    ));
    let drifts = report.get("drifts").and_then(Value::as_array).map_or(0, Vec::len);
    lines.push(format!("  drifts: {drifts}"));
    if let Some(contract) = report.get("failure_contract") {
        lines.push(format!(
            "  failure contract: {} (satisfied: {})",
            contract.get("name").and_then(Value::as_str).unwrap_or("-"),
            contract.get("satisfied").and_then(Value::as_bool).unwrap_or(false)
        ));
    }
    if let Some(receipt) = report.get("receipt") {
        lines.push(format!(
            "  receipt: {} ({})",
            receipt.get("path").and_then(Value::as_str).unwrap_or("-"),
            receipt.get("receipt_hash").and_then(Value::as_str).unwrap_or("-")
        ));
    }
    lines
}

fn emit_replay(report: &Value, format: ReplayOutputFormat, human_lines: &[String]) -> Result<(), String> {
    match format {
        ReplayOutputFormat::Json => print_json(report),
        ReplayOutputFormat::Human => {
            for line in human_lines {
                println!("{line}");
            }
            Ok(())
        }
    }
}

fn capsule_replay_executed(
    capsule_path: &Path,
    inputs_path: Option<&Path>,
    receipt_path: Option<&Path>,
    local_roots: &LocalRoots,
    output_format: ReplayOutputFormat,
) -> Result<i32, String> {
    let workspace = std::env::current_dir().map_err(|error| format!("resolve the workspace root: {error}"))?;
    let (code, report) =
        capsule_replay_executed_report(capsule_path, inputs_path, receipt_path, &workspace, local_roots)?;
    emit_replay(&report, output_format, &executed_human_lines(&report))?;
    Ok(code)
}

fn capsule_replay_executed_report(
    capsule_path: &Path,
    inputs_path: Option<&Path>,
    receipt_path: Option<&Path>,
    workspace: &Path,
    local_roots: &LocalRoots,
) -> Result<(i32, Value), String> {
    let (capsule, manifest) = load_capsule(capsule_path)?;
    let capsule_hash = capsule.get("capsule_hash").and_then(Value::as_str).unwrap_or_default().to_owned();
    let run_kind = manifest.get("run_kind").and_then(Value::as_str).unwrap_or_default().to_owned();
    let expected_exit = manifest.get("expected_exit_code").and_then(Value::as_i64).unwrap_or_default();
    if capsule_is_import_only(&manifest) {
        let report = executed_report(&ExecutedReport {
            capsule_hash: &capsule_hash,
            profile_id: "",
            run_kind: &run_kind,
            status: "failed-closed",
            reason: Some("import-only-capsule"),
            detail: Some("an imported capsule has no reviewed execution profile"),
            expected_exit,
            observed_exit: None,
            drifts: &[],
            failure_contract: None,
            limits: None,
            executed: false,
            receipt: None,
        });
        return Ok((1, report));
    }
    let execution = match manifest.get("execution") {
        None => None,
        Some(value) if value.is_object() => Some(value),
        Some(_) => {
            let report = executed_report(&ExecutedReport {
                capsule_hash: &capsule_hash,
                profile_id: "",
                run_kind: &run_kind,
                status: "failed-closed",
                reason: Some("unsupported-execution-binding"),
                detail: Some("capsule execution must be a JSON object"),
                expected_exit,
                observed_exit: None,
                drifts: &[],
                failure_contract: None,
                limits: None,
                executed: false,
                receipt: None,
            });
            return Ok((1, report));
        }
    };
    let declared_profile = execution.and_then(|execution| execution.get("profile")).and_then(Value::as_str);
    let profile = match declared_profile {
        Some(id) => REVIEWED_PROFILES.iter().copied().find(|profile| profile.id == id),
        None => REVIEWED_PROFILES.iter().copied().find(|profile| profile.run_kind == run_kind),
    };
    let Some(profile) = profile else {
        let (reason, detail) = match declared_profile {
            Some(id) => (
                "unsupported-profile",
                format!("declared execution profile `{id}` is not reviewed for executed replay"),
            ),
            None => ("unsupported-run-kind", format!("no reviewed execution profile covers run_kind `{run_kind}`")),
        };
        let report = executed_report(&ExecutedReport {
            capsule_hash: &capsule_hash,
            profile_id: "unsupported",
            run_kind: &run_kind,
            status: "failed-closed",
            reason: Some(reason),
            detail: Some(detail.as_str()),
            expected_exit,
            observed_exit: None,
            drifts: &[],
            failure_contract: None,
            limits: None,
            executed: false,
            receipt: None,
        });
        return Ok((1, report));
    };
    if profile.run_kind != run_kind {
        let detail = format!("profile `{}` covers run_kind `{}`, not `{run_kind}`", profile.id, profile.run_kind);
        let report = executed_report(&ExecutedReport {
            capsule_hash: &capsule_hash,
            profile_id: profile.id,
            run_kind: &run_kind,
            status: "failed-closed",
            reason: Some("profile-run-kind-mismatch"),
            detail: Some(detail.as_str()),
            expected_exit,
            observed_exit: None,
            drifts: &[],
            failure_contract: None,
            limits: None,
            executed: false,
            receipt: None,
        });
        return Ok((1, report));
    }

    let inputs_path = inputs_path.map_or_else(|| default_inputs_path(capsule_path), Path::to_path_buf);
    let prepared = match prepare_execution(profile, &manifest, &inputs_path, workspace, local_roots)? {
        Preparation::Ready(prepared) => prepared,
        Preparation::Failed { reason, detail } => {
            let report = executed_report(&ExecutedReport {
                capsule_hash: &capsule_hash,
                profile_id: profile.id,
                run_kind: &run_kind,
                status: "failed-closed",
                reason: Some(reason),
                detail: Some(detail.as_str()),
                expected_exit,
                observed_exit: None,
                drifts: &[],
                failure_contract: None,
                limits: None,
                executed: false,
                receipt: None,
            });
            return Ok((1, report));
        }
    };

    if !prepared.pre_execution_drifts.is_empty() {
        let report = executed_report(&ExecutedReport {
            capsule_hash: &capsule_hash,
            profile_id: profile.id,
            run_kind: &run_kind,
            status: "drifted",
            reason: None,
            detail: Some("bindings changed before execution, so the reviewed profile was not launched"),
            expected_exit,
            observed_exit: None,
            drifts: &prepared.pre_execution_drifts,
            failure_contract: None,
            limits: Some(&prepared.limits),
            executed: false,
            receipt: None,
        });
        return Ok((1, report));
    }

    let existing_receipt = match receipt_path {
        Some(path) if path.exists() => Some(verify_execution_receipt(path, &capsule_hash)?),
        _ => None,
    };

    let output = run_prepared_execution(&prepared, workspace)?;
    let stdout = ReceiptStream {
        hash: byte_identity(&output.stdout.bytes),
        bytes: u64::try_from(output.stdout.observed_bytes).unwrap_or(u64::MAX),
        truncated: output.stdout.truncated,
    };
    let stderr = ReceiptStream {
        hash: byte_identity(&output.stderr.bytes),
        bytes: u64::try_from(output.stderr.observed_bytes).unwrap_or(u64::MAX),
        truncated: output.stderr.truncated,
    };

    let mut drifts = Vec::new();
    let mut failure: Option<(&'static str, String)> = None;
    let mut contract_result: Option<(&'static str, bool)> = None;
    let mut produced = BTreeMap::new();
    let mut produced_hashes = BTreeMap::new();
    if output.completion == bounded_exec::Completion::TimedOut {
        failure = Some(("timeout", format!("profile execution exceeded {} ms", prepared.limits.timeout_ms)));
    } else if output.completion == bounded_exec::Completion::Cancelled {
        failure = Some(("cancelled", "profile execution was cancelled".to_owned()));
    } else if output.stdout.truncated || output.stderr.truncated {
        failure = Some((
            "output-overflow",
            format!(
                "captured output exceeded the {} / {} byte limits",
                prepared.limits.stdout_max_bytes, prepared.limits.stderr_max_bytes
            ),
        ));
    }
    for declared in profile.outputs {
        if failure.is_some() {
            break;
        }
        match declared {
            ProfileOutput::Stdout => {
                produced.insert("stdout".to_owned(), ReceiptOutput {
                    hash: stdout.hash.clone(),
                    bytes: stdout.bytes,
                });
                produced_hashes.insert("stdout".to_owned(), stdout.hash.clone());
            }
            ProfileOutput::Stderr => {
                produced.insert("stderr".to_owned(), ReceiptOutput {
                    hash: stderr.hash.clone(),
                    bytes: stderr.bytes,
                });
                produced_hashes.insert("stderr".to_owned(), stderr.hash.clone());
            }
            ProfileOutput::File(name) => match read_output_file(&prepared.output_dir, name)? {
                OutputFile::Present(bytes) => {
                    let hash = byte_identity(&bytes);
                    produced.insert((*name).to_owned(), ReceiptOutput {
                        hash: hash.clone(),
                        bytes: u64::try_from(bytes.len()).unwrap_or(u64::MAX),
                    });
                    produced_hashes.insert((*name).to_owned(), hash);
                }
                OutputFile::Missing => {
                    failure = Some(("missing-output", format!("reviewed profile output `{name}` was not produced")));
                }
                OutputFile::Oversized(size) => {
                    failure = Some((
                        "output-overflow",
                        format!(
                            "reviewed profile output `{name}` is {size} bytes, above the {EXECUTION_OUTPUT_FILE_MAX_BYTES} byte limit"
                        ),
                    ));
                }
            },
        }
    }

    if failure.is_none() {
        let names: BTreeSet<String> = produced_hashes.keys().chain(prepared.expected_outputs.keys()).cloned().collect();
        for name in &names {
            match (prepared.expected_outputs.get(name), produced_hashes.get(name)) {
                (Some(expected), Some(actual)) if expected == actual => {}
                (expected, actual) => drifts.push(Drift {
                    kind: "output",
                    field: name.clone(),
                    expected: expected.cloned().unwrap_or_else(|| "<absent>".to_owned()),
                    actual: actual.cloned().unwrap_or_else(|| "<absent>".to_owned()),
                }),
            }
        }
        match output.exit_code {
            None => {
                failure = Some(("missing-exit-status", "the child produced no portable exit code".to_owned()));
            }
            Some(exit_code) => {
                if i64::from(exit_code) != expected_exit {
                    drifts.push(Drift {
                        kind: "exit",
                        field: "expected_exit_code".to_owned(),
                        expected: expected_exit.to_string(),
                        actual: exit_code.to_string(),
                    });
                } else if expected_exit != 0
                    && let Some(contract) = prepared.failure_contract
                {
                    let satisfied = contract_satisfied(contract, &output);
                    contract_result = Some((contract.name, satisfied));
                    if !satisfied {
                        drifts.push(Drift {
                            kind: "failure_contract",
                            field: contract.name.to_owned(),
                            expected: format!("named failure contract `{}` was satisfied", contract.name),
                            actual: "contract marker missing from the executed run".to_owned(),
                        });
                    }
                }
            }
        }
    }

    let mut body = ExecutionReceiptBody {
        schema_version: SCHEMA_VERSION,
        receipt_kind: EXECUTION_RECEIPT_KIND.to_owned(),
        capsule_hash: capsule_hash.clone(),
        evidence_mode: EXECUTED_EVIDENCE_MODE.to_owned(),
        profile: ReceiptProfile {
            id: profile.id.to_owned(),
            run_kind: profile.run_kind.to_owned(),
            executable_name: profile.executable_name.to_owned(),
            executable_source: match profile.executable {
                ProfileExecutable::SelfOctet => "self-octet".to_owned(),
                ProfileExecutable::DeclaredWorkspaceExecutable => "declared-workspace".to_owned(),
            },
            executable_hash: prepared.executable_hash.clone(),
            argv: prepared.comparison_argv.clone(),
        },
        source: ReceiptSource {
            inputs: prepared
                .inputs
                .iter()
                .map(|input| {
                    (input.name.clone(), ReceiptInput {
                        path: input.normalized_path.clone(),
                        hash: input.hash.clone(),
                    })
                })
                .collect(),
        },
        policy: ReceiptPolicy {
            config_hash: prepared.config_hash.clone(),
        },
        toolchain: ReceiptToolchain {
            octet_binary_identity: prepared.octet_identity.clone(),
            executable_hash: prepared.executable_hash.clone(),
            nix_closure_identity: prepared.nix_closure.clone(),
        },
        capabilities: profile.capabilities.iter().map(|capability| (*capability).to_owned()).collect(),
        environment: ReceiptEnvironment {
            allowlist: profile.environment_allowlist.iter().map(|name| (*name).to_owned()).collect(),
            fixed: profile
                .fixed_environment
                .iter()
                .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
                .collect(),
            present: prepared.environment_names.clone(),
            overrides: prepared.environment_overrides.clone(),
        },
        limits: ReceiptLimits {
            timeout_ms: prepared.limits.timeout_ms,
            stdout_max_bytes: prepared.limits.stdout_max_bytes,
            stderr_max_bytes: prepared.limits.stderr_max_bytes,
        },
        expected: ReceiptExpected {
            exit_code: expected_exit,
            outputs: prepared.expected_outputs.clone(),
            failure_contract: prepared.failure_contract.map(|contract| contract.name.to_owned()),
        },
        outcome: ReceiptOutcome {
            completion: completion_name(output.completion).to_owned(),
            exit_code: output.exit_code,
            signal: output.signal,
            timed_out: output.completion == bounded_exec::Completion::TimedOut,
            stdout,
            stderr,
            outputs: produced,
            output_directory: normalize_bound_path(&prepared.output_dir, workspace, local_roots),
        },
        status: String::new(),
    };

    if let Some(existing) = &existing_receipt {
        let mut comparable_existing = existing.body.clone();
        comparable_existing.status = String::new();
        if comparable_existing != body {
            drifts.push(Drift {
                kind: "receipt",
                field: existing.receipt_hash.clone(),
                expected: "execution receipt matching the recomputed outcome".to_owned(),
                actual: "recomputed outcome differs from the retained receipt".to_owned(),
            });
        }
    }

    let status = if failure.is_some() {
        "failed-closed"
    } else if drifts.is_empty() {
        if expected_exit == 0 {
            "replayed"
        } else {
            "expected-failure-replayed"
        }
    } else {
        "drifted"
    };
    body.status = status.to_owned();
    let receipt_hash = {
        let body_value =
            serde_json::to_value(&body).map_err(|error| format!("serialize execution receipt body: {error}"))?;
        determinism::full_hash(&canonical_json(&body_value)?)
    };
    let receipt = ExecutionReceipt { body, receipt_hash };
    if let Some(path) = receipt_path {
        write_json(path, &receipt)?;
    }

    let failure_contract = contract_result
        .or_else(|| prepared.failure_contract.map(|contract| (contract.name, status == "expected-failure-replayed")));
    let report = executed_report(&ExecutedReport {
        capsule_hash: &capsule_hash,
        profile_id: profile.id,
        run_kind: &run_kind,
        status,
        reason: failure.as_ref().map(|(reason, _)| *reason),
        detail: failure.as_ref().map(|(_, detail)| detail.as_str()),
        expected_exit,
        observed_exit: output.exit_code.map(i64::from),
        drifts: &drifts,
        failure_contract,
        limits: Some(&prepared.limits),
        executed: true,
        receipt: receipt_path.map(|path| (path, receipt.receipt_hash.as_str())),
    });
    let code = i32::from(status == "drifted" || status == "failed-closed");
    Ok((code, report))
}

const CAPSULE_COMPARED_FIELDS: [&str; 6] = [
    "command_argv",
    "run_kind",
    "octet_binary_identity",
    "config_hash",
    "input_hashes",
    "output_hashes",
];

fn capsule_mismatches(manifest: &Value, observed: &Value) -> Vec<String> {
    let mut mismatches = Vec::new();
    for field in CAPSULE_COMPARED_FIELDS {
        match (manifest.get(field), observed.get(field)) {
            (Some(expected), Some(actual)) if expected == actual => {}
            _ => mismatches.push(field.to_owned()),
        }
    }
    mismatches
}

fn ability_registry() -> Result<i32, String> {
    let families = [
        ("time", "ambient time or clock authority"),
        ("randomness", "non-deterministic random data"),
        ("filesystem", "filesystem reads, writes, metadata, and paths"),
        ("network", "network sockets, DNS, and HTTP-like authority"),
        ("process", "process spawning and subprocess control"),
        ("environment", "environment variables and host configuration"),
        ("secrets", "secret material or credential access"),
        ("remote-execution", "remote machine or service execution"),
        ("unsafe-code", "unsafe Rust or unchecked low-level authority"),
        ("global-state", "mutable static or ambient global state"),
        ("concurrency", "threading, task, channel, or synchronization authority"),
        ("unknown", "explicit unmapped capability status"),
    ];
    let entries: Vec<_> = families
        .iter()
        .map(|(family, description)| {
            serde_json::json!({
                "capability_family": family,
                "description": description,
                "fcis_core_policy": if *family == "unknown" { "must-be-made-explicit" } else { "core-prohibited-unless-explicit-data-or-named-capability" },
                "shell_policy": "interpreter-responsibility",
            })
        })
        .collect();
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "approved-capability-vocabulary",
        "families": entries,
    }))?;
    Ok(0)
}

fn ability_summary(findings_path: &Path) -> Result<i32, String> {
    let findings = read_json(findings_path)?;
    let array = findings.as_array().ok_or_else(|| "findings must be a JSON array".to_owned())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut advisory: BTreeMap<String, usize> = BTreeMap::new();
    for finding in array {
        let family = finding.get("effect_family").and_then(Value::as_str).unwrap_or("unknown");
        let status = finding.get("capability_status").and_then(Value::as_str).unwrap_or(if family == "unknown" {
            "unknown"
        } else {
            "known"
        });
        let key = if status == "unknown" { "unknown" } else { family };
        *counts.entry(key.to_owned()).or_default() += 1;
        if finding.get("enforcement").and_then(Value::as_str) == Some("advisory") {
            *advisory.entry(key.to_owned()).or_default() += 1;
        }
    }
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "capability-summary",
        "counts_by_capability": counts,
        "advisory_counts_by_capability": advisory,
        "unknown_capability_visible": counts.contains_key("unknown"),
    }))?;
    Ok(0)
}

fn ability_enforce(manifest_path: &Path, trace_path: &Path) -> Result<i32, String> {
    let manifest = read_json(manifest_path)?;
    let trace = read_json(trace_path)?;
    let allowed = string_array(&manifest, "allowed_effects", manifest_path)?;
    let effects = string_array(&trace, "effects", trace_path)?;
    let denied: Vec<String> = effects.iter().filter(|effect| !allowed.contains(effect)).cloned().collect();
    let status = if denied.is_empty() { "allowed" } else { "denied" };
    let report = serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": status,
        "allowed_effects": allowed,
        "observed_effects": effects,
        "denied_effects": denied,
        "redaction_status": "no-secret-values-or-absolute-paths",
    });
    print_json(&report)?;
    Ok(i32::from(status != "allowed"))
}

/// `document[field]` as strings; a missing field, a non-array, or a non-string entry is an error.
fn string_array(document: &Value, field: &str, path: &Path) -> Result<Vec<String>, String> {
    let items = document
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| format!("{}: {field} must be an array of strings", path.display()))?;
    items
        .iter()
        .enumerate()
        .map(|(index, item)| {
            item.as_str()
                .map(str::to_owned)
                .ok_or_else(|| format!("{}: {field}[{index}] must be a string", path.display()))
        })
        .collect()
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
struct PointerRegistry {
    pointers: BTreeMap<String, PointerTarget>,
    #[serde(default)]
    history: BTreeMap<String, Vec<PointerTarget>>,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize, Default)]
struct PointerTarget {
    artifact_kind: String,
    hash: String,
}

#[derive(Debug, Clone, Serialize, serde::Deserialize)]
struct PointerReceipt {
    schema_version: u32,
    name: String,
    target_kind: String,
    /// Artifact kind of the replaced target; absent in receipts written before it was recorded.
    #[serde(default)]
    old_kind: Option<String>,
    old_hash: Option<String>,
    new_hash: String,
    rollback_hint: String,
}

fn pointer_set(registry_path: &Path, name: &str, kind: &str, hash: &str, receipt_path: &Path) -> Result<i32, String> {
    let mut registry = read_registry(registry_path)?;
    let old_target = registry.pointers.get(name).cloned();
    let old_kind = old_target.as_ref().map(|target| target.artifact_kind.clone());
    let old_hash = old_target.as_ref().map(|target| target.hash.clone());
    if let Some(target) = old_target {
        registry.history.entry(name.to_owned()).or_default().push(target);
    }
    registry.pointers.insert(name.to_owned(), PointerTarget {
        artifact_kind: kind.to_owned(),
        hash: hash.to_owned(),
    });
    write_json(registry_path, &registry)?;
    let receipt = PointerReceipt {
        schema_version: SCHEMA_VERSION,
        name: name.to_owned(),
        target_kind: kind.to_owned(),
        old_kind,
        old_hash,
        new_hash: hash.to_owned(),
        rollback_hint: format!(
            "cargo octet roadmap pointer rollback --registry {} --receipt {}",
            registry_path.display(),
            receipt_path.display()
        ),
    };
    write_json(receipt_path, &receipt)?;
    print_json(&receipt)?;
    Ok(0)
}

fn pointer_inspect(registry_path: &Path, name: &str) -> Result<i32, String> {
    let registry = read_registry(registry_path)?;
    let target = registry.pointers.get(name).ok_or_else(|| format!("name `{name}` is not registered"))?;
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "name": name,
        "target_hash": target.hash,
        "target_artifact_kind": target.artifact_kind,
        "identity_source": "immutable-hash",
        "human_label": name,
        "previous_targets": registry.history.get(name).cloned().unwrap_or_default(),
        "review_identity_status": "matched-identity-not-correctness-proof",
        "correctness_claim": "none",
    }))?;
    Ok(0)
}

fn pointer_rollback(registry_path: &Path, receipt_path: &Path) -> Result<i32, String> {
    let receipt: PointerReceipt =
        serde_json::from_value(read_json(receipt_path)?).map_err(|error| format!("parse receipt: {error}"))?;
    let Some(old_hash) = receipt.old_hash.clone() else {
        return Err("receipt has no old_hash; cannot rollback initial pointer creation".to_owned());
    };
    let mut registry = read_registry(registry_path)?;
    let name = receipt.name.clone();
    let current = registry
        .pointers
        .get(&name)
        .cloned()
        .ok_or_else(|| format!("name `{name}` is not registered; cannot roll back receipt"))?;
    if current.hash != receipt.new_hash || current.artifact_kind != receipt.target_kind {
        return Err(format!(
            "pointer `{name}` currently targets {} `{}`, not the receipt's {} `{}`; refusing to roll back a receipt superseded by a later update",
            current.artifact_kind, current.hash, receipt.target_kind, receipt.new_hash
        ));
    }
    let old_kind = match receipt.old_kind {
        Some(old_kind) => old_kind,
        None => registry
            .history
            .get(&name)
            .and_then(|history| history.iter().rev().find(|target| target.hash == old_hash))
            .map(|target| target.artifact_kind.clone())
            .ok_or_else(|| {
                format!(
                    "receipt has no old_kind and history for `{name}` has no `{old_hash}` target; cannot restore the previous artifact kind"
                )
            })?,
    };
    registry.history.entry(name.clone()).or_default().push(current);
    registry.pointers.insert(name.clone(), PointerTarget {
        artifact_kind: old_kind.clone(),
        hash: old_hash.clone(),
    });
    write_json(registry_path, &registry)?;
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "rolled-back",
        "name": name,
        "restored_hash": old_hash,
        "restored_artifact_kind": old_kind,
    }))?;
    Ok(0)
}

fn session_check(input: &Path) -> Result<i32, String> {
    let session = read_json(input)?;
    let records = session["records"].as_array().ok_or_else(|| "session missing records array".to_owned())?;
    let mut kinds = Vec::new();
    for record in records {
        if record.get("kind").and_then(Value::as_str).is_none()
            || record.get("schema_version").and_then(Value::as_u64).is_none()
            || record.get("identity").and_then(Value::as_str).is_none()
            || record.get("redaction_status").and_then(Value::as_str).is_none()
        {
            return Err("session record missing kind, schema_version, identity, or redaction_status".to_owned());
        }
        kinds.push(record["kind"].as_str().unwrap().to_owned());
    }
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "typed-versioned",
        "record_count": records.len(),
        "record_kinds": kinds,
    }))?;
    Ok(0)
}

fn session_migrate(input: &Path, output: &Path) -> Result<i32, String> {
    let mut session = read_json(input)?;
    let object = session.as_object_mut().ok_or_else(|| "session must be a JSON object".to_owned())?;
    let original_identity = object
        .get("identity")
        .and_then(Value::as_str)
        .ok_or_else(|| "session identity must be a string".to_owned())?
        .to_owned();
    let from_schema_version = session_schema_version(object)?;
    if from_schema_version > u64::from(SCHEMA_VERSION) {
        return Err(format!(
            "session schema_version {from_schema_version} is newer than supported schema_version {SCHEMA_VERSION}"
        ));
    }
    let migration_receipt = serde_json::json!({
        "from_schema_version": from_schema_version,
        "to_schema_version": SCHEMA_VERSION,
        "original_identity": original_identity,
        "identity_change": false,
    });
    object.insert("schema_version".to_owned(), Value::from(SCHEMA_VERSION));
    object.insert("migration_receipt".to_owned(), migration_receipt.clone());
    write_json(output, &session)?;
    print_json(&migration_receipt)?;
    Ok(0)
}

/// The session's current schema version: `schema_version`, else the legacy
/// `original_schema_version`, else 0 for an unversioned legacy session.
fn session_schema_version(session: &serde_json::Map<String, Value>) -> Result<u64, String> {
    for field in ["schema_version", "original_schema_version"] {
        if let Some(value) = session.get(field) {
            return value.as_u64().ok_or_else(|| format!("session {field} must be a non-negative integer"));
        }
    }
    Ok(0)
}

fn session_todo(input: &Path) -> Result<i32, String> {
    let session = read_json(input)?;
    let todos = session["todos"].as_array().ok_or_else(|| "session missing todos array".to_owned())?;
    let mut counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut items = Vec::new();
    for todo in todos {
        let kind = todo["kind"].as_str().ok_or_else(|| "todo missing kind".to_owned())?;
        let status = todo["status"].as_str().ok_or_else(|| "todo missing status".to_owned())?;
        let source = todo["source"].as_str().unwrap_or("unknown");
        let next_action = todo["next_action"].as_str().unwrap_or("none");
        *counts.entry(status.to_owned()).or_default() += 1;
        items.push(serde_json::json!({"kind": kind, "status": status, "source": source, "next_action": next_action, "blocker": todo.get("blocker")}));
    }
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "structured",
        "counts_by_status": counts,
        "items": items,
    }))?;
    Ok(0)
}

fn pure_cache_check(key_path: &Path, store_dir: &Path, output_hash: &str) -> Result<i32, String> {
    let key = redact_secret_fields(read_json(key_path)?);
    for field in PURE_CACHE_KEY_FIELDS {
        if key.get(field).is_none() {
            return Err(format!("cache key missing {field}"));
        }
    }
    let canonical = canonical_json(&key)?;
    let cache_key = determinism::full_hash(&canonical);
    fs::create_dir_all(store_dir).map_err(|error| format!("create {}: {error}", store_dir.display()))?;
    let entry_path = store_dir.join(hash_file_name(&cache_key));
    if entry_path.exists() {
        let entry = read_json(&entry_path)?;
        let stored_output_hash = entry
            .get("output_hash")
            .and_then(Value::as_str)
            .ok_or_else(|| format!("cache entry {} missing string output_hash", entry_path.display()))?;
        if stored_output_hash != output_hash {
            print_json(&serde_json::json!({
                "schema_version": SCHEMA_VERSION,
                "status": "output-mismatch",
                "cache_key": cache_key,
                "stored_output_hash": stored_output_hash,
                "observed_output_hash": output_hash,
                "key_components": key,
            }))?;
            return Ok(1);
        }
        print_json(&serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "status": "hit",
            "cache_key": cache_key,
            "stored_output_hash": stored_output_hash,
            "key_components": key,
        }))?;
        return Ok(0);
    }
    let changed = changed_key_components(&key, &stored_cache_keys(store_dir)?);
    let entry = serde_json::json!({"cache_key": cache_key, "output_hash": output_hash, "key_components": key});
    write_json(&entry_path, &entry)?;
    print_json(&serde_json::json!({
        "schema_version": SCHEMA_VERSION,
        "status": "miss",
        "cache_key": cache_key,
        "miss_reason": if changed.is_empty() { "new-key" } else { "changed-key-components" },
        "changed_key_components": changed,
    }))?;
    Ok(0)
}

/// Key components of every entry in the cache store, in file-name order. Only files named like
/// entries are read; other files, such as the key itself, are skipped.
fn stored_cache_keys(store_dir: &Path) -> Result<Vec<Value>, String> {
    let mut paths = Vec::new();
    for entry in fs::read_dir(store_dir).map_err(|error| format!("read {}: {error}", store_dir.display()))? {
        let path = entry.map_err(|error| format!("read {}: {error}", store_dir.display()))?.path();
        if path.file_name().and_then(std::ffi::OsStr::to_str).is_some_and(is_hash_file_name) {
            paths.push(path);
        }
    }
    paths.sort();
    let mut keys = Vec::new();
    for path in &paths {
        if let Some(components) = cache_entry_key_components(read_json(path)?, path)? {
            keys.push(components);
        }
    }
    Ok(keys)
}

/// The key components of a store file named like an entry: `None` for an object without a
/// `cache_key`, such as an artifact record sharing the store, and an error for a non-object or an
/// entry without a `key_components` object.
fn cache_entry_key_components(mut entry: Value, path: &Path) -> Result<Option<Value>, String> {
    let object = entry
        .as_object_mut()
        .ok_or_else(|| format!("cache store file {} must be a JSON object", path.display()))?;
    if !object.contains_key("cache_key") {
        return Ok(None);
    }
    object
        .remove("key_components")
        .filter(Value::is_object)
        .map(Some)
        .ok_or_else(|| format!("cache entry {} missing key_components object", path.display()))
}

/// Names the key fields that differ from the closest stored key with the same command identity,
/// without their values; empty when no stored key shares the command identity.
fn changed_key_components(key: &Value, stored_keys: &[Value]) -> Vec<String> {
    stored_keys
        .iter()
        .filter(|stored| stored.get("command_identity") == key.get("command_identity"))
        .map(|stored| differing_fields(key, stored))
        .min_by_key(Vec::len)
        .unwrap_or_default()
}

fn differing_fields(left: &Value, right: &Value) -> Vec<String> {
    let fields: BTreeSet<&String> =
        left.as_object().into_iter().chain(right.as_object()).flat_map(serde_json::Map::keys).collect();
    fields
        .into_iter()
        .filter(|field| left.get(field.as_str()) != right.get(field.as_str()))
        .cloned()
        .collect()
}

/// Local directories whose location must not affect content identity, each paired with the
/// placeholder that replaces it, longest first so the most specific root wins.
struct LocalRoots {
    roots: Vec<(String, &'static str)>,
}

impl LocalRoots {
    /// Keeps each spelling of each root that is absolute, valid UTF-8, and not a filesystem root,
    /// without trailing separators; on equal length the workspace wins over home, and home over
    /// temp.
    fn new(workspace: &[PathBuf], home: &[PathBuf], temp: &[PathBuf]) -> Self {
        let mut roots: Vec<(String, &'static str)> = [
            (workspace, WORKSPACE_PLACEHOLDER),
            (home, HOME_PLACEHOLDER),
            (temp, TEMP_PLACEHOLDER),
        ]
        .into_iter()
        .flat_map(|(spellings, placeholder)| spellings.iter().map(move |root| (root.as_path(), placeholder)))
        .filter(|(root, _)| root.is_absolute() && root.parent().is_some())
        .filter_map(|(root, placeholder)| {
            Some((root.to_str()?.trim_end_matches(std::path::is_separator).to_owned(), placeholder))
        })
        .collect();
        roots.sort_by_key(|(root, _)| std::cmp::Reverse(root.len()));
        Self { roots }
    }

    /// `text` with a leading local root replaced by its placeholder, keeping the relative
    /// remainder; `None` when `text` does not start with a local root at a path boundary.
    fn normalize(&self, text: &str) -> Option<String> {
        self.roots.iter().find_map(|(root, placeholder)| {
            let remainder = text.strip_prefix(root.as_str())?;
            (remainder.is_empty() || remainder.starts_with(std::path::is_separator))
                .then(|| format!("{placeholder}{remainder}"))
        })
    }
}

/// Normalizes a DTO for content identity: drops secret-like and local-only fields and replaces a
/// leading local root in string values with its placeholder, except under identity fields.
fn normalize_value(value: Value, local_roots: &LocalRoots) -> Value {
    normalize_json(value, Some(local_roots))
}

/// Drops secret-like and local-only fields but keeps every other value, for keys whose path
/// values must keep distinguishing entries.
fn redact_secret_fields(value: Value) -> Value {
    normalize_json(value, None)
}

fn normalize_json(value: Value, local_roots: Option<&LocalRoots>) -> Value {
    match value {
        Value::Object(map) => {
            let mut out = serde_json::Map::new();
            for (key, value) in map {
                if is_redacted_key(&key) {
                    continue;
                }
                let value_roots = local_roots.filter(|_| !is_identity_key(&key));
                out.insert(key, normalize_json(value, value_roots));
            }
            Value::Object(out)
        }
        Value::Array(items) => Value::Array(items.into_iter().map(|item| normalize_json(item, local_roots)).collect()),
        Value::String(text) => {
            Value::String(local_roots.and_then(|local_roots| local_roots.normalize(&text)).unwrap_or(text))
        }
        other => other,
    }
}

/// Whether a field names an identity, such as `octet_binary_identity`, whose value is compared
/// verbatim and therefore never path-normalized.
fn is_identity_key(key: &str) -> bool {
    key_words(key).last().is_some_and(|word| word == "identity")
}

/// Words that, right before a trailing `token` or `tokens`, mark it as a vocabulary, position,
/// pagination, or coordination token rather than a credential (`eos_token`, `stop_token`,
/// `next_page_token`, `PaginationToken`, `cancellation_token`). Any other trailing `token` is
/// treated as a credential, so unlisted names such as `VAULT_TOKEN` or
/// `CARGO_REGISTRIES_<NAME>_TOKEN` fail closed.
const NON_CREDENTIAL_TOKEN_QUALIFIERS: [&str; 26] = [
    "eos",
    "bos",
    "eot",
    "eod",
    "pad",
    "unk",
    "sep",
    "cls",
    "mask",
    "start",
    "stop",
    "end",
    "special",
    "first",
    "last",
    "prev",
    "previous",
    "page",
    "pagination",
    "next",
    "continuation",
    "resume",
    "cursor",
    "cancellation",
    "sync",
    "count",
];
/// Words that, right before a trailing plural `tokens`, mark it as a usage count or a list of
/// token types or vocabulary words (`prompt_tokens`, `input_tokens`, `capability_tokens`, and
/// the Cairn policy's `normative_tokens` and `placeholder_tokens`). A singular `token` after them
/// stays a credential: `INPUT_TOKEN` is how GitHub Actions passes a `token` input, and
/// `capability_token` or `cached_token` can hold one.
const TOKEN_LIST_QUALIFIERS: [&str; 9] = [
    "prompt",
    "completion",
    "input",
    "output",
    "cached",
    "reasoning",
    "capability",
    "normative",
    "placeholder",
];
/// Words that make a `token` or `tokens` field a count or limit wherever they appear
/// (`max_tokens`, `max_new_tokens`, `num_tokens`).
const TOKEN_COUNT_WORDS: [&str; 4] = ["max", "min", "num", "total"];
/// Trailing words that only number or place a credential, skipped before looking for a trailing
/// `token` (`GITHUB_TOKEN_2`, `NPM_TOKEN_PROD`). An all-digit word is skipped too.
const CREDENTIAL_SUFFIX_WORDS: [&str; 20] = [
    "prod",
    "production",
    "dev",
    "development",
    "staging",
    "stage",
    "test",
    "testing",
    "qa",
    "uat",
    "ci",
    "local",
    "live",
    "sandbox",
    "preview",
    "old",
    "new",
    "primary",
    "secondary",
    "backup",
];
/// Words that hold or carry a credential wherever they appear, unless a secret metadata word
/// marks the field as describing it. Any word containing `password` (`PGPASSWORD`) counts too.
const CREDENTIAL_WORDS: [&str; 13] = [
    "secret",
    "secrets",
    "passwd",
    "passphrase",
    "credential",
    "credentials",
    "authorization",
    "auth",
    "bearer",
    "cookie",
    "cookies",
    "jwt",
    "sessionid",
];
/// Abbreviated password words that are credentials unless the next word describes them
/// (`DB_PASS`, `DB_PASS_B64`, `MYSQL_PWD` are credentials; `pass_signal`, `pass_rate` are not).
const ABBREVIATED_PASSWORD_WORDS: [&str; 2] = ["pass", "pwd"];
/// Words that, right after a qualified `key` or after `pass` or `pwd`, make the field describe
/// or locate the secret instead of holding it (`changed_key_components`, `signing_key_path`,
/// `pass_signal`, `pass_rate`). Any other following word, such as an encoding, version, region,
/// or environment (`SIGNING_KEY_PEM`, `STRIPE_KEY_V2`, `DB_PASS_PROD`), keeps the secret.
const DESCRIPTIVE_FOLLOW_WORDS: [&str; 20] = [
    "component",
    "components",
    "count",
    "name",
    "names",
    "path",
    "paths",
    "dir",
    "type",
    "kind",
    "len",
    "length",
    "size",
    "format",
    "algorithm",
    "hint",
    "rate",
    "signal",
    "through",
    "fail",
];
/// Words that, right before `key`, name a lookup, schema, or public key rather than key material
/// (`primary_key`, `cache_key`, `public_key`). Any other qualified `key` is key material unless
/// a [`DESCRIPTIVE_FOLLOW_WORDS`] word follows it, so `STRIPE_KEY`, `SIGNING_KEY_PEM`, and
/// `AWS_ACCESS_KEY_ID` fail closed while a bare `key` names nothing secret.
const NON_SECRET_KEY_QUALIFIERS: [&str; 17] = [
    "primary",
    "foreign",
    "composite",
    "unique",
    "sort",
    "partition",
    "shard",
    "index",
    "row",
    "group",
    "dedup",
    "cache",
    "map",
    "dict",
    "hash",
    "lookup",
    "public",
];
/// Words that make a credential-word field describe the credential instead of holding it
/// (`secret_count`, `password_min_length`, `credential_helper`, `cookie_name`).
const SECRET_METADATA_WORDS: [&str; 15] = [
    "count", "has", "is", "min", "max", "length", "len", "policy", "helper", "enabled", "required", "name", "expiry",
    "expires", "rotation",
];

/// Whether a field name is secret-like or local-machine-only, compared case-insensitively over
/// its `snake_case`, `kebab-case`, or `camelCase` words.
fn is_redacted_key(key: &str) -> bool {
    let words = key_words(key);
    let words: Vec<&str> = words.iter().map(String::as_str).collect();
    let local_only = matches!(words.as_slice(), ["timestamp" | "username"] | ["absolute", "path"] | ["env", "dump"]);
    let names_secret = words.iter().any(|word| CREDENTIAL_WORDS.contains(word) || word.contains("password"))
        || words.windows(2).any(|pair| pair == ["session", "id"])
        || words
            .iter()
            .enumerate()
            .any(|(index, word)| ABBREVIATED_PASSWORD_WORDS.contains(word) && !is_described(&words, index));
    let describes_secret = words.iter().any(|word| SECRET_METADATA_WORDS.contains(word));
    let key_material = words.iter().any(|word| matches!(*word, "apikey" | "privatekey"))
        || words.windows(2).any(|pair| pair[1] == "key" && (pair[0] == "private" || pair[0].ends_with("api")))
        || words.windows(2).enumerate().any(|(index, pair)| {
            pair[1] == "key" && !NON_SECRET_KEY_QUALIFIERS.contains(&pair[0]) && !is_described(&words, index + 1)
        });
    local_only
        || (names_secret && !describes_secret)
        || is_credential_token(without_credential_suffixes(&words))
        || key_material
}

/// Whether the word after `words[index]` is a [`DESCRIPTIVE_FOLLOW_WORDS`] word.
fn is_described(words: &[&str], index: usize) -> bool {
    words.get(index + 1).is_some_and(|next| DESCRIPTIVE_FOLLOW_WORDS.contains(next))
}

/// The words before any trailing numbering or environment suffix words (`GITHUB_TOKEN_2`,
/// `NPM_TOKEN_PROD`); an all-digit word counts as a suffix.
fn without_credential_suffixes<'a>(words: &'a [&'a str]) -> &'a [&'a str] {
    let suffix_start = words
        .iter()
        .rposition(|word| !(CREDENTIAL_SUFFIX_WORDS.contains(word) || word.bytes().all(|byte| byte.is_ascii_digit())))
        .map_or(0, |last_core| last_core + 1);
    &words[..suffix_start]
}

/// Whether the last core word is `token` or `tokens` (optionally with trailing digits) and
/// neither its qualifier nor a count word marks it as a non-credential token.
fn is_credential_token(core: &[&str]) -> bool {
    let Some((last, qualifiers)) = core.split_last() else {
        return false;
    };
    let plural = match last.trim_end_matches(|character: char| character.is_ascii_digit()) {
        "token" => false,
        "tokens" => true,
        _ => return false,
    };
    let counted = core.iter().any(|word| TOKEN_COUNT_WORDS.contains(word));
    let non_credential = qualifiers.last().is_some_and(|qualifier| {
        NON_CREDENTIAL_TOKEN_QUALIFIERS.contains(qualifier) || (plural && TOKEN_LIST_QUALIFIERS.contains(qualifier))
    });
    !counted && !non_credential
}

/// Lowercase words of a field name, split at non-alphanumeric characters, at `camelCase` humps,
/// and before the last capital of an acronym run that starts a capitalized word
/// (`AWSSecret` → `aws`, `secret`).
fn key_words(key: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut current = String::new();
    let mut previous: Option<char> = None;
    let mut characters = key.chars().peekable();
    while let Some(character) = characters.next() {
        if !character.is_alphanumeric() {
            if !current.is_empty() {
                words.push(std::mem::take(&mut current));
            }
            previous = None;
            continue;
        }
        let next = characters.peek().copied();
        let starts_word = character.is_uppercase()
            && previous.is_some_and(|previous| {
                previous.is_lowercase()
                    || previous.is_numeric()
                    || (previous.is_uppercase() && next.is_some_and(char::is_lowercase))
            });
        if starts_word {
            words.push(std::mem::take(&mut current));
        }
        previous = Some(character);
        current.extend(character.to_lowercase());
    }
    if !current.is_empty() {
        words.push(current);
    }
    words
}

fn canonical_json(value: &Value) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| format!("serialize canonical json: {error}"))
}

/// Rejects anything but `b3:` plus 64 lowercase hex digits, so a hash can be joined to a store
/// directory without escaping it.
fn validate_artifact_hash(hash: &str) -> Result<(), String> {
    if hash.strip_prefix(BLAKE3_IDENTITY_PREFIX).is_some_and(is_blake3_hex) {
        Ok(())
    } else {
        Err(format!("artifact hash `{hash}` must be `b3:` followed by {BLAKE3_HEX_LENGTH} lowercase hex digits"))
    }
}

fn is_blake3_hex(hex: &str) -> bool {
    hex.len() == BLAKE3_HEX_LENGTH && hex.bytes().all(|byte| matches!(byte, b'0'..=b'9' | b'a'..=b'f'))
}

fn hash_file_name(hash: &str) -> String {
    format!("{}.json", hash.replace(':', "_"))
}

/// Whether `name` is what [`hash_file_name`] produces for a well-formed hash: `b3_`, 64
/// lowercase hex digits, and `.json`.
fn is_hash_file_name(name: &str) -> bool {
    name.strip_suffix(".json").and_then(|stem| stem.strip_prefix("b3_")).is_some_and(is_blake3_hex)
}

/// The local roots of this process: the current directory as the workspace, `HOME`, and the
/// temporary directory.
fn local_roots_from_environment() -> LocalRoots {
    local_roots_from(
        std::env::current_dir().ok(),
        std::env::var_os("PWD").map(PathBuf::from),
        std::env::var_os("HOME").map(PathBuf::from),
        std::env::temp_dir(),
    )
}

/// Local roots with every spelling a recorded path is likely to use. `current_dir` is already
/// resolved, while argv usually spells the checkout as the shell's `$PWD`, through symlinks, so
/// `pwd` is also a workspace root when it resolves to the same directory; a stale `pwd` naming
/// another directory is ignored. `HOME` and the temporary directory are registered both as given
/// and resolved.
fn local_roots_from(
    current_dir: Option<PathBuf>,
    pwd: Option<PathBuf>,
    home: Option<PathBuf>,
    temp: PathBuf,
) -> LocalRoots {
    let resolved_current = current_dir.as_deref().and_then(|current| fs::canonicalize(current).ok());
    let pwd = pwd.filter(|pwd| resolved_current.is_some() && fs::canonicalize(pwd).ok() == resolved_current);
    let workspace: Vec<PathBuf> = current_dir.into_iter().chain(pwd).collect();
    let home = with_resolved_spelling(home);
    let temp = with_resolved_spelling(Some(temp));
    LocalRoots::new(&workspace, &home, &temp)
}

/// `path` as given, followed by its symlink-resolved form when it exists.
fn with_resolved_spelling(path: Option<PathBuf>) -> Vec<PathBuf> {
    let resolved = path.as_deref().and_then(|path| fs::canonicalize(path).ok());
    path.into_iter().chain(resolved).collect()
}

fn read_json(path: &Path) -> Result<Value, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn read_registry(path: &Path) -> Result<PointerRegistry, String> {
    if !path.exists() {
        return Ok(PointerRegistry::default());
    }
    serde_json::from_value(read_json(path)?).map_err(|error| format!("parse registry: {error}"))
}

fn write_json<T: Serialize + ?Sized>(path: &Path, value: &T) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    let mut text = serde_json::to_string_pretty(value).map_err(|error| format!("serialize json: {error}"))?;
    text.push('\n');
    fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))
}

fn print_json<T: Serialize + ?Sized>(value: &T) -> Result<(), String> {
    println!("{}", serde_json::to_string_pretty(value).map_err(|error| format!("serialize output: {error}"))?);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capsule_mismatches_treat_absent_fields_as_mismatches() {
        let manifest = serde_json::json!({"command_argv": ["cargo", "octet"], "run_kind": "check"});
        let observed = serde_json::json!({"command_argv": ["cargo", "octet"], "run_kind": "check"});
        assert_eq!(capsule_mismatches(&manifest, &observed), vec![
            "octet_binary_identity",
            "config_hash",
            "input_hashes",
            "output_hashes",
        ]);
        assert_eq!(capsule_mismatches(&Value::Null, &Value::Null).len(), 6);
    }

    #[test]
    fn redacted_keys_cover_secret_like_names_case_insensitively() {
        for key in [
            "secret",
            "raw_secret",
            "Client-Secret",
            "token",
            "access_token",
            "accessToken",
            "GITHUB_TOKEN",
            "password",
            "dbPassword",
            "passwd",
            "credential",
            "aws_credentials",
            "api_key",
            "API_KEY",
            "apiKey",
            "apikey",
            "x-api-key",
            "private_key",
            "privateKey",
            "absolute_path",
            "timestamp",
            "username",
            "env_dump",
        ] {
            assert!(is_redacted_key(key), "{key} must be redacted");
        }
        for key in [
            "prompt",
            "max_tokens",
            "prompt_tokens",
            "token_count",
            "tokenizer",
            "key",
            "public_key",
            "keyboard",
            "path",
            "file",
            "name",
        ] {
            assert!(!is_redacted_key(key), "{key} must stay in the identity");
        }
    }

    #[test]
    fn redacted_keys_treat_trailing_tokens_as_credentials_unless_known_and_skip_secret_metadata() {
        for key in [
            "token",
            "refresh_token",
            "id_token",
            "bearerToken",
            "csrf-token",
            "oauth_token",
            "session_token",
            "personal_access_token",
            "client_token",
            "secret_token",
            "GH_TOKEN",
            "GITLAB_TOKEN",
            "slack_token",
            "NPM_TOKEN",
            "pypi_token",
            "registry_token",
            "VAULT_TOKEN",
            "HF_TOKEN",
            "CI_JOB_TOKEN",
            "CARGO_REGISTRIES_MY_REGISTRY_TOKEN",
            "CARGO_REGISTRY_TOKEN",
            "deploy_token",
            "client_secret",
            "db_password",
            "secrets",
            "aws_credentials",
        ] {
            assert!(is_redacted_key(key), "{key} must be redacted");
        }
        for key in [
            "eos_token",
            "pad_token",
            "bos_token",
            "unk_token",
            "sep_token",
            "cls_token",
            "mask_token",
            "decoder_start_token",
            "end_token",
            "special_token",
            "next_page_token",
            "pageToken",
            "next_token",
            "continuation_token",
            "cancellation_token",
            "sync_token",
            "max_token",
            "num_token",
            "count_token",
            "max_tokens",
            "token_count",
            "token_limit",
            "secret_count",
            "has_secrets",
            "password_min_length",
            "maxPasswordLen",
            "credential_helper",
            "secret_name",
            "password_policy",
            "is_password_required",
            "secretRotationEnabled",
            "credentials_expires",
        ] {
            assert!(!is_redacted_key(key), "{key} must stay in the identity");
        }
    }

    #[test]
    fn redacted_keys_cover_credential_spellings() {
        for key in [
            "tokens",
            "api_tokens",
            "GITHUB_TOKEN_2",
            "GITHUB_TOKEN2",
            "githubToken2",
            "NPM_TOKEN_PROD",
            "slack_token_staging",
            "PGPASSWORD",
            "MYSQL_PWD",
            "DB_PASS",
            "SMTP_PASS",
            "DB_PASS_PROD",
            "db_pwd_2",
            "INPUT_TOKEN",
            "input_token",
            "output_token",
            "cached_token",
            "prompt_token",
            "completion_token",
            "reasoning_token",
            "capability_token",
            "STRIPE_KEY_PROD",
            "private_key_path",
            "api_key_name",
            "Authorization",
            "proxy_authorization",
            "bearer",
            "cookie",
            "set_cookies",
            "session_id",
            "sessionId",
            "SESSIONID",
            "jwt",
            "auth",
            "basic_auth",
            "AWS_ACCESS_KEY_ID",
            "STRIPE_KEY",
            "SIGNING_KEY",
            "encryption_key",
            "OpenAIAPIKey",
            "SIGNING_KEY_PEM",
            "ENCRYPTION_KEY_B64",
            "master_key_hex",
            "STRIPE_KEY_V2",
            "SIGNING_KEY_EU",
            "SSH_KEY_PASSPHRASE",
            "passphrase",
            "DB_PASS_B64",
            "SMTP_PASS_ENC",
            "lint_pass",
            "pwd_value",
        ] {
            assert!(is_redacted_key(key), "{key} must be redacted");
        }
    }

    #[test]
    fn redacted_keys_keep_vocabulary_usage_lookup_and_identity_names() {
        for key in [
            "stop_token",
            "eot_token",
            "eod_token",
            "first_token",
            "last_token",
            "prev_token",
            "previous_token",
            "resume_token",
            "cursor_token",
            "PaginationToken",
            "pagination_token",
            "max_tokens",
            "max_new_tokens",
            "min_tokens",
            "num_tokens",
            "token_count",
            "token_limit",
            "tokens_used_count",
            "prompt_tokens",
            "completion_tokens",
            "input_tokens",
            "output_tokens",
            "total_tokens",
            "cached_tokens",
            "reasoning_tokens",
            "capability_tokens",
            "eos_token_2",
            "primary_key",
            "foreign_key",
            "composite_key",
            "unique_key",
            "sort_key",
            "partition_key",
            "shard_key",
            "index_key",
            "row_key",
            "group_key",
            "dedup_key",
            "cache_key",
            "map_key",
            "dict_key",
            "hash_key",
            "lookup_key",
            "public_key",
            "public_key_id",
            "key",
            "key_id",
            "auth_required",
            "cookie_name",
            "jwt_expiry",
            "pass_count",
            "session",
            "passes",
            "authority",
            "currently_authorized",
            "pass_signal",
            "pass_rate",
            "pwd_hint_count",
            "stop_tokens",
            "pagination_tokens",
            "normative_tokens",
            "placeholder_tokens",
            "changed_key_components",
            "changed_key_component",
            "signing_key_path",
            "ssh_key_paths",
            "signing_key_type",
            "encryption_key_len",
            "pass_through",
            "pass_fail",
            "pass_signal_count",
            "pwd_hint",
            "passphrase_required",
        ] {
            assert!(!is_redacted_key(key), "{key} must stay in the identity");
        }
    }

    #[test]
    fn key_words_split_acronym_runs_before_a_capitalized_word() {
        for (key, words) in [
            ("AWSSecretAccessKey", &["aws", "secret", "access", "key"][..]),
            ("DBPassword", &["db", "password"]),
            ("GITHUBToken", &["github", "token"]),
            ("OpenAIAPIKey", &["open", "aiapi", "key"]),
            ("HTTPServer", &["http", "server"]),
            ("accessToken", &["access", "token"]),
            ("API_KEY", &["api", "key"]),
            ("sha256Digest", &["sha256", "digest"]),
        ] {
            assert_eq!(key_words(key), words, "{key}");
        }
        for key in [
            "AWSSecretAccessKey",
            "DBPassword",
            "GITHUBToken",
            "JWTSecret",
            "SSHPrivateKey",
            "XApiKey",
            "OpenAIAPIKey",
        ] {
            assert!(is_redacted_key(key), "{key} must be redacted");
        }
        for key in ["HTTPServer", "URLPath", "IOError", "JSONSchema"] {
            assert!(!is_redacted_key(key), "{key} must stay in the identity");
        }
    }

    #[test]
    fn normalize_value_replaces_local_root_prefixes_only() {
        let roots = LocalRoots::new(&["/home/alice/repo/".into()], &["/home/alice".into()], &["/tmp".into()]);
        let normalized = normalize_value(
            serde_json::json!({
                "workspace": "/home/alice/repo",
                "source": "/home/alice/repo/src/lib.rs",
                "config": "/home/alice/.config/octet.toml",
                "scratch": "/tmp/run-1/out.json",
                "argv": ["cargo", "--manifest-path", "/home/alice/repo/crates/a/Cargo.toml"],
                "octet_binary_identity": "/home/alice/.cargo/bin/cargo-octet",
                "nested_identity": {"binary": "/tmp/cargo-octet"},
                "sibling": "/home/alice/repository",
                "outside": "/srv/shared/repo",
                "route": "/v1/chat/completions",
                "pointer": "/definitions/finding",
                "root": "/",
                "relative": "src/lib.rs",
                "embedded": "--manifest-path=/home/alice/repo/Cargo.toml",
                "number": 3
            }),
            &roots,
        );
        for (field, expected) in [
            ("workspace", "<workspace>"),
            ("source", "<workspace>/src/lib.rs"),
            ("config", "<home>/.config/octet.toml"),
            ("scratch", "<temp>/run-1/out.json"),
            ("octet_binary_identity", "/home/alice/.cargo/bin/cargo-octet"),
            ("sibling", "<home>/repository"),
            ("outside", "/srv/shared/repo"),
            ("route", "/v1/chat/completions"),
            ("pointer", "/definitions/finding"),
            ("root", "/"),
            ("relative", "src/lib.rs"),
            ("embedded", "--manifest-path=/home/alice/repo/Cargo.toml"),
        ] {
            assert_eq!(normalized[field], expected, "{field}");
        }
        assert_eq!(normalized["argv"][2], "<workspace>/crates/a/Cargo.toml");
        assert_eq!(normalized["nested_identity"]["binary"], "/tmp/cargo-octet");
        assert_eq!(normalized["number"], 3);
    }

    #[test]
    fn local_roots_skip_relative_and_filesystem_roots() {
        let roots = LocalRoots::new(&["/".into()], &["relative/home".into()], &[]);
        assert_eq!(roots.normalize("/home/alice/repo"), None);
        assert_eq!(roots.normalize("relative/home/x"), None);
    }

    #[test]
    fn local_roots_register_symlinked_and_resolved_spellings() {
        let temp = tempfile::tempdir().expect("tempdir");
        let base = std::fs::canonicalize(temp.path()).expect("canonical tempdir");
        let real = base.join("real");
        for directory in ["checkout", "home", "tmp", "elsewhere"] {
            std::fs::create_dir_all(real.join(directory)).expect("create local root");
        }
        let link = base.join("link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink checkout parent");

        let roots = local_roots_from(
            Some(real.join("checkout")),
            Some(link.join("checkout")),
            Some(link.join("home")),
            link.join("tmp"),
        );
        for (spelling, parent) in [("symlinked", &link), ("resolved", &real)] {
            for (root, rest, expected) in [
                ("checkout", "Cargo.toml", "<workspace>/Cargo.toml"),
                ("home", ".config/octet.toml", "<home>/.config/octet.toml"),
                ("tmp", "run-1/out.json", "<temp>/run-1/out.json"),
            ] {
                let path = parent.join(root).join(rest);
                let text = path.to_str().expect("utf-8 path");
                assert_eq!(roots.normalize(text).as_deref(), Some(expected), "{spelling} {text}");
            }
        }

        let stale = local_roots_from(Some(real.join("checkout")), Some(real.join("elsewhere")), None, link.join("tmp"));
        let elsewhere = real.join("elsewhere/Cargo.toml");
        assert_eq!(stale.normalize(elsewhere.to_str().expect("utf-8 path")), None, "stale PWD is not the workspace");
        let relative =
            local_roots_from(Some(real.join("checkout")), Some(PathBuf::from("checkout")), None, link.join("tmp"));
        assert_eq!(relative.normalize("checkout/Cargo.toml"), None, "relative PWD is not a root");
    }

    fn args(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_owned()).collect()
    }

    #[test]
    fn roadmap_parse_rejects_flag_in_value_position() {
        let error = parse(&args(&[
            "pointer",
            "set",
            "--registry",
            "registry.json",
            "--name",
            "--kind",
            "prompt",
            "--hash",
            "b3:x",
            "--receipt",
            "receipt.json",
        ]))
        .expect_err("flag in value position fails");
        assert!(error.contains("missing value after `--name`"), "{error}");

        let error = parse(&args(&["session", "check", "--input"])).expect_err("trailing flag fails");
        assert!(error.contains("missing value after `--input`"), "{error}");

        let error = parse(&args(&["session", "check", "--input="])).expect_err("empty inline value fails");
        assert!(error.contains("missing value after `--input`"), "{error}");
    }

    #[test]
    fn roadmap_parse_rejects_unknown_flags_and_stray_arguments() {
        let error = parse(&args(&[
            "artifact",
            "store",
            "--kind",
            "prompt",
            "--input",
            "in.json",
            "--store-dir",
            "store",
            "--bogus",
            "x",
        ]))
        .expect_err("unknown flag fails");
        assert!(error.contains("unsupported roadmap artifact store argument `--bogus`"), "{error}");

        let error = parse(&args(&["ability", "registry", "--verbose"])).expect_err("registry takes no flags");
        assert!(error.contains("unsupported roadmap ability registry argument `--verbose`"), "{error}");

        let error = parse(&args(&["session", "check", "--input", "session.json", "extra"]))
            .expect_err("stray positional fails");
        assert!(error.contains("unexpected roadmap session check argument `extra`"), "{error}");

        let error = parse(&args(&["capsule", "inspect", "--capsule", "c.json", "--observed", "o.json"]))
            .expect_err("flag from another action fails");
        assert!(error.contains("unsupported roadmap capsule inspect argument `--observed`"), "{error}");
    }

    #[test]
    fn roadmap_parse_rejects_repeated_flags() {
        let error = parse(&args(&["session", "check", "--input", "a.json", "--input", "b.json"]))
            .expect_err("repeated flag fails");
        assert!(error.contains("`--input` may be provided only once"), "{error}");

        let error = parse(&args(&["session", "check", "--input=a.json", "--input", "b.json"]))
            .expect_err("repeated mixed-form flag fails");
        assert!(error.contains("`--input` may be provided only once"), "{error}");
    }

    #[test]
    fn roadmap_parse_accepts_split_and_inline_values() {
        let command = parse(&args(&["capsule", "replay", "--capsule=c.json", "--observed", "o.json"]))
            .expect("valid capsule replay args");
        assert!(
            matches!(
                &command,
                RoadmapCommand::CapsuleReplay {
                    capsule,
                    observed: Some(observed),
                    execute: false,
                    inputs: None,
                    receipt: None,
                    output_format: ReplayOutputFormat::Json,
                } if capsule == Path::new("c.json") && observed == Path::new("o.json")
            ),
            "{command:?}"
        );
        let command = parse(&args(&[
            "capsule",
            "replay",
            "--capsule",
            "c.json",
            "--execute",
            "--inputs=inputs.json",
            "--receipt",
            "receipt.json",
            "--output-format",
            "human",
        ]))
        .expect("valid executed capsule replay args");
        assert!(
            matches!(
                &command,
                RoadmapCommand::CapsuleReplay {
                    capsule,
                    observed: None,
                    execute: true,
                    inputs: Some(inputs),
                    receipt: Some(receipt),
                    output_format: ReplayOutputFormat::Human,
                } if capsule == Path::new("c.json")
                    && inputs == Path::new("inputs.json")
                    && receipt == Path::new("receipt.json")
            ),
            "{command:?}"
        );
        let command = parse(&args(&[
            "pure-cache",
            "check",
            "--output-hash",
            "b3:out",
            "--store-dir",
            "cache",
            "--key",
            "key.json",
        ]))
        .expect("valid pure-cache args");
        assert!(
            matches!(&command, RoadmapCommand::PureCacheCheck { key, store_dir, output_hash }
                if key == Path::new("key.json") && store_dir == Path::new("cache") && output_hash == "b3:out"),
            "{command:?}"
        );
        assert!(matches!(parse(&args(&["ability", "registry"])), Ok(RoadmapCommand::AbilityRegistry)));
    }

    #[test]
    fn roadmap_parse_rejects_conflicting_executed_replay_flags() {
        let error = parse(&args(&[
            "capsule",
            "replay",
            "--capsule",
            "c.json",
            "--execute",
            "--observed",
            "o.json",
        ]))
        .expect_err("executed replay must not also compare supplied observations");
        assert!(error.contains("mutually exclusive"), "{error}");

        let error = parse(&args(&["capsule", "replay", "--capsule", "c.json"]))
            .expect_err("capsule replay needs an evidence mode");
        assert!(error.contains("missing `--observed`"), "{error}");

        let error = parse(&args(&[
            "capsule",
            "replay",
            "--capsule",
            "c.json",
            "--observed",
            "o.json",
            "--inputs",
            "i.json",
        ]))
        .expect_err("--inputs requires --execute");
        assert!(error.contains("`--inputs` requires `--execute`"), "{error}");

        let error = parse(&args(&[
            "capsule",
            "replay",
            "--capsule",
            "c.json",
            "--observed",
            "o.json",
            "--receipt",
            "r.json",
        ]))
        .expect_err("--receipt requires --execute");
        assert!(error.contains("`--receipt` requires `--execute`"), "{error}");

        let error = parse(&args(&["capsule", "replay", "--capsule", "c.json", "--execute", "--execute"]))
            .expect_err("repeated switch fails");
        assert!(error.contains("`--execute` may be provided only once"), "{error}");

        let error = parse(&args(&["capsule", "replay", "--capsule", "c.json", "--execute=true"]))
            .expect_err("switches take no value");
        assert!(error.contains("`--execute` takes no value"), "{error}");

        let error = parse(&args(&[
            "capsule",
            "replay",
            "--capsule",
            "c.json",
            "--execute",
            "--output-format",
            "yaml",
        ]))
        .expect_err("unknown output format fails");
        assert!(error.contains("unsupported capsule replay output format `yaml`"), "{error}");
    }

    const SUCCESS_REPORT: &str = "{\"findings\":0}";
    const PROBE_PROLOGUE: &str = "out=\"\"\nprevious=\"\"\nfor argument in \"$@\"; do\n  if [ \"$previous\" = \"--output-dir\" ]; then out=\"$argument\"; fi\n  previous=\"$argument\"\ndone\n";

    fn test_workspace() -> (tempfile::TempDir, PathBuf) {
        let temp = tempfile::tempdir().expect("tempdir");
        let workspace = std::fs::canonicalize(temp.path()).expect("canonical workspace");
        (temp, workspace)
    }

    fn test_local_roots(workspace: &Path) -> LocalRoots {
        local_roots_from(Some(workspace.to_path_buf()), None, None, std::env::temp_dir())
    }

    fn write_test_json(path: &Path, value: &Value) {
        std::fs::write(path, serde_json::to_string_pretty(value).expect("serialize json")).expect("write json");
    }

    fn write_test_input(workspace: &Path) -> String {
        let directory = workspace.join("inputs");
        std::fs::create_dir_all(&directory).expect("create inputs directory");
        std::fs::write(directory.join("src.rs"), "pub fn f() {}\n").expect("write input");
        byte_identity(b"pub fn f() {}\n")
    }

    fn current_octet_identity() -> String {
        file_identity(&std::env::current_exe().expect("current executable")).expect("octet identity")
    }

    fn write_probe(workspace: &Path, body: &str) -> String {
        let path = workspace.join("octet-check-probe");
        crate::test_support::write_executable_script(&path, &format!("#!/bin/sh\n{PROBE_PROLOGUE}{body}\n"));
        file_identity(&path).expect("probe identity")
    }

    fn executed_capsule(manifest: &Value) -> Value {
        let capsule_hash = determinism::full_hash(&canonical_json(manifest).expect("canonical manifest"));
        serde_json::json!({
            "schema_version": SCHEMA_VERSION,
            "capsule_hash": capsule_hash,
            "capsule_kind": "octet-replay-capsule",
            "manifest": manifest,
            "redaction_status": REDACTION_STATUS,
        })
    }

    fn check_manifest(
        input_hash: &str,
        expected_report: &str,
        expected_stderr: &str,
        executable_hash: &str,
        expected_exit: i64,
        extra_execution: Value,
    ) -> Value {
        let mut execution = serde_json::json!({
            "profile": "check-v1",
            "toolchain": {"executable_hash": executable_hash},
        });
        if let (Value::Object(execution), Value::Object(extra)) = (&mut execution, extra_execution) {
            for (key, value) in extra {
                execution.insert(key, value);
            }
        }
        serde_json::json!({
            "command_argv": [
                "octet-check-probe",
                "--output-dir",
                OUTPUT_PLACEHOLDER,
                "--input",
                format!("src/lib.rs={WORKSPACE_PLACEHOLDER}/inputs/src.rs"),
            ],
            "run_kind": "check",
            "octet_binary_identity": current_octet_identity(),
            "config_hash": "b3:cfg",
            "input_hashes": {"src/lib.rs": input_hash},
            "output_hashes": {
                "check-report.json": byte_identity(expected_report.as_bytes()),
                "stderr": byte_identity(expected_stderr.as_bytes()),
            },
            "expected_exit_code": expected_exit,
            "execution": execution,
        })
    }

    fn check_inputs() -> Value {
        serde_json::json!({
            "schema_version": 1,
            "profile": "check-v1",
            "executable": "octet-check-probe",
            "inputs": {"src/lib.rs": "inputs/src.rs"},
            "config_hash": "b3:cfg",
        })
    }

    fn try_run_executed(
        workspace: &Path,
        manifest: &Value,
        inputs: &Value,
        receipt: Option<&Path>,
    ) -> Result<(i32, Value), String> {
        let capsule_path = workspace.join("capsule.json");
        let inputs_path = workspace.join("execution-inputs.json");
        write_test_json(&capsule_path, &executed_capsule(manifest));
        write_test_json(&inputs_path, inputs);
        let roots = test_local_roots(workspace);
        capsule_replay_executed_report(&capsule_path, Some(&inputs_path), receipt, workspace, &roots)
    }

    fn run_executed(workspace: &Path, manifest: &Value, inputs: &Value, receipt: Option<&Path>) -> (i32, Value) {
        try_run_executed(workspace, manifest, inputs, receipt).expect("executed replay report")
    }

    fn drift_kinds(report: &Value) -> Vec<String> {
        report["drifts"]
            .as_array()
            .map(|drifts| drifts.iter().filter_map(|drift| drift["kind"].as_str().map(str::to_owned)).collect())
            .unwrap_or_default()
    }

    #[test]
    fn executed_replay_selects_the_fingerprint_profile_and_requires_local_inputs() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let manifest = serde_json::json!({
            "command_argv": [
                "octet",
                "fingerprint",
                "--check",
                format!("{WORKSPACE_PLACEHOLDER}/inputs/src.rs"),
            ],
            "run_kind": "fingerprint",
            "octet_binary_identity": current_octet_identity(),
            "config_hash": "b3:cfg",
            "input_hashes": {"src/lib.rs": input_hash},
            "output_hashes": {"stdout": byte_identity(b""), "stderr": byte_identity(b"")},
            "expected_exit_code": 0,
        });
        let capsule_path = workspace.join("capsule.json");
        write_test_json(&capsule_path, &executed_capsule(&manifest));
        let roots = test_local_roots(&workspace);
        let (code, report) = capsule_replay_executed_report(
            &capsule_path,
            Some(&workspace.join("missing-execution-inputs.json")),
            None,
            &workspace,
            &roots,
        )
        .expect("executed replay report");
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["profile"], FINGERPRINT_PRODUCTION_PROFILE.id);
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "missing-inputs");
        assert_eq!(report["executed"], false);
    }

    #[test]
    fn executed_replay_launches_reviewed_profile_and_retains_receipt() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let executable_hash =
            write_probe(&workspace, &format!("printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\n"));
        let manifest = check_manifest(&input_hash, SUCCESS_REPORT, "", &executable_hash, 0, Value::Null);
        let receipt_path = workspace.join("receipt.json");

        let (code, report) = run_executed(&workspace, &manifest, &check_inputs(), Some(&receipt_path));
        assert_eq!(code, 0, "{report}");
        assert_eq!(report["status"], "replayed");
        assert_eq!(report["evidence_mode"], EXECUTED_EVIDENCE_MODE);
        assert_eq!(report["executed"], true);
        assert_eq!(report["profile"], "check-v1");
        assert_eq!(report["mismatches"], serde_json::json!([]));
        assert!(workspace.join(EXECUTION_OUTPUT_DIRECTORY).exists(), "isolated output directory is used");

        let receipt: Value = serde_json::from_str(&std::fs::read_to_string(&receipt_path).expect("read receipt"))
            .expect("parse receipt");
        assert_eq!(receipt["receipt_kind"], EXECUTION_RECEIPT_KIND);
        assert_eq!(receipt["capsule_hash"], executed_capsule(&manifest)["capsule_hash"]);
        assert_eq!(receipt["evidence_mode"], EXECUTED_EVIDENCE_MODE);
        assert_eq!(receipt["outcome"]["exit_code"], serde_json::json!(0));
        assert_eq!(receipt["source"]["inputs"]["src/lib.rs"]["path"], "<workspace>/inputs/src.rs");
        assert!(receipt["receipt_hash"].as_str().expect("receipt hash").starts_with("b3:"));
        assert_eq!(report["receipt"]["receipt_hash"], receipt["receipt_hash"]);

        let human = executed_human_lines(&report).join("\n");
        assert!(human.contains("evidence mode: executed"), "{human}");

        let (replay_code, replay) = run_executed(&workspace, &manifest, &check_inputs(), Some(&receipt_path));
        assert_eq!(replay_code, 0, "{replay}");
        assert_eq!(replay["status"], "replayed");
        assert!(drift_kinds(&replay).is_empty(), "{replay}");
    }

    #[test]
    fn executed_replay_cannot_be_satisfied_by_a_fabricated_observation() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let executable_hash =
            write_probe(&workspace, &format!("printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\n"));
        let manifest = check_manifest(&input_hash, "{\"findings\":7}", "", &executable_hash, 0, Value::Null);
        let capsule = executed_capsule(&manifest);
        let capsule_path = workspace.join("capsule.json");
        write_test_json(&capsule_path, &capsule);

        let observed_path = workspace.join("observed.json");
        write_test_json(
            &observed_path,
            &serde_json::json!({
                "command_argv": manifest["command_argv"],
                "run_kind": "check",
                "octet_binary_identity": manifest["octet_binary_identity"],
                "config_hash": "b3:cfg",
                "input_hashes": manifest["input_hashes"],
                "output_hashes": manifest["output_hashes"],
                "exit_code": 0,
            }),
        );
        let roots = test_local_roots(&workspace);
        let (supplied_code, supplied) =
            capsule_replay_observed_report(&capsule_path, &observed_path, &roots).expect("supplied replay");
        assert_eq!(supplied_code, 0);
        assert_eq!(supplied["status"], "replayed");
        assert_eq!(supplied["evidence_mode"], SUPPLIED_OBSERVATION_EVIDENCE_MODE);
        let human = observed_human_lines(&supplied).join("\n");
        assert!(human.contains("evidence mode: supplied_observation"), "{human}");

        let (code, report) = run_executed(&workspace, &manifest, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(report["executed"], true);
        assert!(drift_kinds(&report).contains(&"output".to_owned()), "{report}");
    }

    #[test]
    fn executed_replay_reports_source_argv_output_and_exit_drift() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let executable_hash =
            write_probe(&workspace, &format!("printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\n"));

        let manifest = check_manifest(&input_hash, SUCCESS_REPORT, "", &executable_hash, 0, Value::Null);
        std::fs::write(workspace.join("inputs/src.rs"), "pub fn f() { changed(); }\n").expect("change input");
        let (code, report) = run_executed(&workspace, &manifest, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(report["executed"], false);
        assert_eq!(drift_kinds(&report), vec!["source".to_owned()]);
        assert_eq!(report["mismatches"], serde_json::json!(["input_hashes"]));

        std::fs::write(workspace.join("inputs/src.rs"), "pub fn f() {}\n").expect("restore input");
        let mut argv_manifest = check_manifest(&input_hash, SUCCESS_REPORT, "", &executable_hash, 0, Value::Null);
        argv_manifest["command_argv"] = serde_json::json!([
            "octet-check-probe",
            "--verbose",
            "--output-dir",
            OUTPUT_PLACEHOLDER,
            "--input",
            format!("src/lib.rs={WORKSPACE_PLACEHOLDER}/inputs/src.rs"),
        ]);
        let (code, report) = run_executed(&workspace, &argv_manifest, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(report["executed"], false);
        assert_eq!(drift_kinds(&report), vec!["argv".to_owned()]);

        let mut changed_config = check_inputs();
        changed_config["config_hash"] = serde_json::json!("b3:changed-config");
        let config_manifest = check_manifest(&input_hash, SUCCESS_REPORT, "", &executable_hash, 0, Value::Null);
        let (code, report) = run_executed(&workspace, &config_manifest, &changed_config, None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(report["executed"], false);
        assert_eq!(drift_kinds(&report), vec!["config".to_owned()]);
        assert_eq!(report["mismatches"], serde_json::json!(["config_hash"]));

        let mut changed_toolchain = check_manifest(&input_hash, SUCCESS_REPORT, "", &executable_hash, 0, Value::Null);
        changed_toolchain["execution"]["toolchain"]["executable_hash"] = serde_json::json!(byte_identity(b"other"));
        let (code, report) = run_executed(&workspace, &changed_toolchain, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(report["executed"], false);
        assert_eq!(drift_kinds(&report), vec!["toolchain".to_owned()]);
        assert_eq!(report["mismatches"], serde_json::json!(["octet_binary_identity"]));

        let output_manifest = check_manifest(&input_hash, "{\"findings\":9}", "", &executable_hash, 0, Value::Null);
        let (code, report) = run_executed(&workspace, &output_manifest, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(report["executed"], true);
        assert_eq!(drift_kinds(&report), vec!["output".to_owned()]);

        let failing_executable =
            write_probe(&workspace, &format!("printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\nexit 4\n"));
        let exit_manifest = check_manifest(&input_hash, SUCCESS_REPORT, "", &failing_executable, 0, Value::Null);
        let (code, report) = run_executed(&workspace, &exit_manifest, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(report["observed_exit_code"], serde_json::json!(4));
        assert_eq!(drift_kinds(&report), vec!["exit".to_owned()]);
        assert_eq!(report["mismatches"], serde_json::json!(["expected_exit_code"]));
    }

    #[test]
    fn executed_replay_fails_closed_on_timeout_and_output_overflow() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let spinner = write_probe(&workspace, "while :; do :; done\n");
        let timeout_manifest = check_manifest(
            &input_hash,
            SUCCESS_REPORT,
            "",
            &spinner,
            0,
            serde_json::json!({"resource_limits": {"timeout_ms": 250}}),
        );
        let (code, report) = run_executed(&workspace, &timeout_manifest, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "timeout");
        assert_eq!(report["executed"], true);

        let noisy = write_probe(
            &workspace,
            &format!(
                "count=0\nwhile [ \"$count\" -lt 200 ]; do printf 'xxxxxxxxxx'; count=$((count + 1)); done\nprintf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\n"
            ),
        );
        let overflow_manifest = check_manifest(
            &input_hash,
            SUCCESS_REPORT,
            "",
            &noisy,
            0,
            serde_json::json!({"resource_limits": {"stdout_max_bytes": 64}}),
        );
        let (code, report) = run_executed(&workspace, &overflow_manifest, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "output-overflow");
    }

    #[test]
    fn executed_replay_fails_closed_on_unsupported_profiles_and_import_only_capsules() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let executable_hash =
            write_probe(&workspace, &format!("printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\n"));
        let manifest = check_manifest(&input_hash, SUCCESS_REPORT, "", &executable_hash, 0, Value::Null);

        let mut unsupported = manifest.clone();
        unsupported.as_object_mut().expect("manifest object").remove("execution");
        unsupported["run_kind"] = serde_json::json!("coverage");
        let (code, report) = run_executed(&workspace, &unsupported, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "unsupported-run-kind");

        let mut imported = manifest.clone();
        imported.as_object_mut().expect("manifest object").remove("execution");
        imported["run_kind"] = serde_json::json!("imported-tool-evidence");
        let (code, report) = run_executed(&workspace, &imported, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "import-only-capsule");

        let mut foreign_program = manifest.clone();
        foreign_program["command_argv"][0] = serde_json::json!("sh");
        let error = try_run_executed(&workspace, &foreign_program, &check_inputs(), None)
            .expect_err("an imported executable is rejected");
        assert!(error.contains("imported executable"), "{error}");

        let mut shell_fragment = manifest.clone();
        shell_fragment["command_argv"] =
            serde_json::json!(["octet-check-probe", "--input", "src/lib.rs; touch pwned",]);
        let error = try_run_executed(&workspace, &shell_fragment, &check_inputs(), None)
            .expect_err("shell fragments are rejected");
        assert!(error.contains("shell fragment"), "{error}");

        let mut escaped = check_inputs();
        escaped["executable"] = serde_json::json!("../outside-probe");
        let error =
            try_run_executed(&workspace, &manifest, &escaped, None).expect_err("executable path escapes are rejected");
        assert!(error.contains("path escape"), "{error}");

        let mut escaped_input = check_inputs();
        escaped_input["inputs"] = serde_json::json!({"src/lib.rs": "../escape.rs"});
        let error =
            try_run_executed(&workspace, &manifest, &escaped_input, None).expect_err("input path escapes are rejected");
        assert!(error.contains("path escape"), "{error}");

        let mut undeclared_environment = check_inputs();
        undeclared_environment["environment"] = serde_json::json!({"LD_PRELOAD": "evil.so"});
        let error = try_run_executed(&workspace, &manifest, &undeclared_environment, None)
            .expect_err("undeclared environment is rejected");
        assert!(error.contains("undeclared environment"), "{error}");

        let foreign_path = workspace.join("other-probe");
        crate::test_support::write_executable_script(&foreign_path, "#!/bin/sh\nexit 0\n");
        let mut imported_name = check_inputs();
        imported_name["executable"] = serde_json::json!("other-probe");
        let error = try_run_executed(&workspace, &manifest, &imported_name, None)
            .expect_err("an imported executable name is rejected");
        assert!(error.contains("imported executable"), "{error}");

        let mut missing_inputs = check_inputs();
        missing_inputs["inputs"] = serde_json::json!({});
        let (code, report) = run_executed(&workspace, &manifest, &missing_inputs, None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "missing-input");
        assert_eq!(report["executed"], false);

        let mut closure_bound = manifest.clone();
        closure_bound["execution"]["toolchain"]["nix_closure_identity"] = serde_json::json!("nix:fixture-closure");
        let (code, report) = run_executed(&workspace, &closure_bound, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "missing-tool-closure");
    }

    #[test]
    fn executed_replay_requires_the_named_failure_contract() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let contract_stderr = "octet-check-failure: source drift";

        let marker_probe = write_probe(
            &workspace,
            &format!(
                "printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\nprintf '%s' '{contract_stderr}' >&2\nexit 3\n"
            ),
        );
        let satisfying = check_manifest(
            &input_hash,
            SUCCESS_REPORT,
            contract_stderr,
            &marker_probe,
            3,
            serde_json::json!({"failure_contract": "check-diagnostic-failure"}),
        );
        let (code, report) = run_executed(&workspace, &satisfying, &check_inputs(), None);
        assert_eq!(code, 0, "{report}");
        assert_eq!(report["status"], "expected-failure-replayed");
        assert_eq!(report["failure_contract"]["name"], "check-diagnostic-failure");
        assert_eq!(report["failure_contract"]["satisfied"], true);

        let silent_probe =
            write_probe(&workspace, &format!("printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\nexit 3\n"));
        let unsatisfied = check_manifest(
            &input_hash,
            SUCCESS_REPORT,
            "",
            &silent_probe,
            3,
            serde_json::json!({"failure_contract": "check-diagnostic-failure"}),
        );
        let (code, report) = run_executed(&workspace, &unsatisfied, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "drifted");
        assert_eq!(drift_kinds(&report), vec!["failure_contract".to_owned()]);
        assert_eq!(report["failure_contract"]["satisfied"], false);

        let unnamed = check_manifest(&input_hash, SUCCESS_REPORT, "", &silent_probe, 3, Value::Null);
        let (code, report) = run_executed(&workspace, &unnamed, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "missing-failure-contract");

        let unknown = check_manifest(
            &input_hash,
            SUCCESS_REPORT,
            "",
            &silent_probe,
            3,
            serde_json::json!({"failure_contract": "unreviewed-contract"}),
        );
        let (code, report) = run_executed(&workspace, &unknown, &check_inputs(), None);
        assert_eq!(code, 1, "{report}");
        assert_eq!(report["status"], "failed-closed");
        assert_eq!(report["reason"], "unsupported-failure-contract");
    }

    #[test]
    fn executed_replay_rejects_a_tampered_retained_receipt() {
        let (_temp, workspace) = test_workspace();
        let input_hash = write_test_input(&workspace);
        let executable_hash =
            write_probe(&workspace, &format!("printf '%s' '{SUCCESS_REPORT}' > \"$out/check-report.json\"\n"));
        let manifest = check_manifest(&input_hash, SUCCESS_REPORT, "", &executable_hash, 0, Value::Null);
        let receipt_path = workspace.join("receipt.json");
        let (code, _report) = run_executed(&workspace, &manifest, &check_inputs(), Some(&receipt_path));
        assert_eq!(code, 0);

        let mut receipt: Value = serde_json::from_str(&std::fs::read_to_string(&receipt_path).expect("read receipt"))
            .expect("parse receipt");
        receipt["outcome"]["exit_code"] = serde_json::json!(9);
        write_test_json(&receipt_path, &receipt);
        let error = try_run_executed(&workspace, &manifest, &check_inputs(), Some(&receipt_path))
            .expect_err("a modified receipt cannot be verified");
        assert!(error.contains("does not match the recomputed"), "{error}");
    }
}

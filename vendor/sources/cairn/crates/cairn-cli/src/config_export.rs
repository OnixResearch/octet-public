use std::{
    collections::BTreeSet,
    error::Error,
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

use cairn_core::{
    admit_agent_guidance_declaration, admit_lifecycle_store_identity, blake3_hex, canonical_json,
    parse_policy_json, AgentArtifactGuidance, AgentContextReference, AgentGuidanceDeclaration,
    AgentOperationGuidance, LifecycleStoreIdentityProjection, LIFECYCLE_STORE_IDENTITY_NON_CLAIMS,
    LIFECYCLE_STORE_IDENTITY_SCHEMA_V1, LIFECYCLE_STORE_NON_CLAIMS_MAX_COUNT,
    LIFECYCLE_STORE_NON_CLAIM_MAX_BYTES,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    nickel_export_boundary::export_json_dual_run_with_contract,
    support::{print_json, with_receipt_hash},
};

const CONFIG_EXPORT_SCHEMA_V1: &str = "cairn.lifecycle-config-export-plan.v1";
const STORE_SOURCE_SCHEMA_V1: &str = "cairn.lifecycle-store-source.v1";
const PROJECT_SOURCE_SCHEMA_V1: &str = "cairn.lifecycle-store-project-source.v1";
const PROJECT_PROJECTION_SCHEMA_V1: &str = "cairn.lifecycle-store-project.v1";
const STORE_SOURCE_PATH: &str = ".cairn-store/store.ncl";
const STORE_PROJECTION_PATH: &str = ".cairn-store/store.json";
const PROJECT_SOURCE_PATH: &str = ".cairn/project.ncl";
const PROJECT_PROJECTION_PATH: &str = ".cairn/project.json";
const SOURCE_FILE_MAX_BYTES: u64 = 1_048_576;
const STAGING_ROOT: &str = "target/cairn-config-export-stage";
const STAGED_SOURCE: &str = "source.ncl";
const STAGED_CONTRACTS: &str = "lifecycle-store-contracts.ncl";
const STAGED_CONTRACT: &str = "selected-contract.ncl";
const STAGED_OUTPUT: &str = "output.json";
const CONTRACTS_SOURCE: &str = include_str!("../../../cairn-policy/lifecycle-store-contracts.ncl");
const GENERATED_POLICY_JSON: &str =
    include_str!("../../../cairn-policy/generated/cairn-policy.json");
const STORE_CONTRACT_FIELD: &str = "StoreIdentitySource";
const PROJECT_CONTRACT_FIELD: &str = "ProjectDeclarationSource";
const PROJECT_REQUIRED_NON_CLAIMS: &[&str] = &[
    "a project pointer selects one declared store but does not grant lifecycle authority",
    "a direct reference provides read-only discovery and does not satisfy dependencies, gates, evidence, or requirements",
    "reference indexing does not prove truth, freshness, ownership, implementation, or satisfaction",
    "project configuration does not authorize transitive reference traversal or transport",
    "project facts and advisory guidance do not override generated policy or command authorization",
    "project context checks do not prove truth, secrecy, semantic consistency, agent compliance, or release eligibility",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ConfigKind {
    Store,
    Project,
}

impl ConfigKind {
    fn parse(value: &str) -> Result<Self, Box<dyn Error>> {
        match value {
            "store" => Ok(Self::Store),
            "project" => Ok(Self::Project),
            _ => {
                Err(format!("config export --kind must be store or project, got {value:?}").into())
            }
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Store => "store",
            Self::Project => "project",
        }
    }

    const fn source_path(self) -> &'static str {
        match self {
            Self::Store => STORE_SOURCE_PATH,
            Self::Project => PROJECT_SOURCE_PATH,
        }
    }

    const fn projection_path(self) -> &'static str {
        match self {
            Self::Store => STORE_PROJECTION_PATH,
            Self::Project => PROJECT_PROJECTION_PATH,
        }
    }

    const fn contract_field(self) -> &'static str {
        match self {
            Self::Store => STORE_CONTRACT_FIELD,
            Self::Project => PROJECT_CONTRACT_FIELD,
        }
    }
}

#[derive(Debug)]
struct ConfigExportOptions {
    kind: ConfigKind,
    root: PathBuf,
    check: bool,
    execute: bool,
}

#[derive(Clone, Debug)]
struct ExportPlan {
    kind: ConfigKind,
    root: PathBuf,
    source_relative: PathBuf,
    projection_relative: PathBuf,
    source_bytes: Vec<u8>,
    prior_projection: Option<Vec<u8>>,
    projection_bytes: Vec<u8>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct StoreSource {
    schema: String,
    store_id: String,
    remote_hint: Option<String>,
    non_claims: Vec<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct StoreReference {
    store_id: String,
    remote_hint: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ProjectSource {
    schema: String,
    lifecycle_store: Option<StoreReference>,
    references: Vec<StoreReference>,
    #[serde(default)]
    context_refs: Vec<AgentContextReference>,
    #[serde(default)]
    artifact_guidance: Vec<AgentArtifactGuidance>,
    #[serde(default)]
    operation_guidance: Vec<AgentOperationGuidance>,
    non_claims: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ProjectProjection {
    schema: String,
    lifecycle_store: Option<StoreReference>,
    references: Vec<StoreReference>,
    context_refs: Vec<AgentContextReference>,
    artifact_guidance: Vec<AgentArtifactGuidance>,
    operation_guidance: Vec<AgentOperationGuidance>,
    source_blake3: String,
    non_claims: Vec<String>,
}

pub(crate) fn run_config(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = parse_options(&args)?;
    let plan = build_export_plan(&options)?;
    let current = plan
        .prior_projection
        .as_ref()
        .is_some_and(|bytes| bytes == &plan.projection_bytes);

    if options.check {
        let receipt = plan_receipt(&plan, true, false, current, current);
        print_json(with_receipt_hash(receipt))?;
        if !current {
            return Err(format!(
                "{} projection is stale: {}",
                plan.kind.name(),
                plan.root.join(&plan.projection_relative).display()
            )
            .into());
        }
        return Ok(());
    }

    let mut mutated = false;
    if options.execute && !current {
        execute_plan(&plan)?;
        mutated = true;
    }
    let receipt = plan_receipt(&plan, false, options.execute, true, mutated);
    print_json(with_receipt_hash(receipt))
}

fn parse_options(args: &[String]) -> Result<ConfigExportOptions, Box<dyn Error>> {
    if args.first().map(String::as_str) != Some("export") {
        return Err("config requires export".into());
    }
    let mut kind = None;
    let mut root = None;
    let mut check = false;
    let mut execute = false;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--kind" => {
                if kind.is_some() {
                    return Err("config export received --kind more than once".into());
                }
                let value = args
                    .get(index + 1)
                    .ok_or("config export --kind requires a value")?;
                kind = Some(ConfigKind::parse(value)?);
                index += 2;
            }
            "--root" => {
                if root.is_some() {
                    return Err("config export received --root more than once".into());
                }
                let value = args
                    .get(index + 1)
                    .ok_or("config export --root requires a path")?;
                root = Some(PathBuf::from(value));
                index += 2;
            }
            "--check" => {
                check = true;
                index += 1;
            }
            "--execute" => {
                execute = true;
                index += 1;
            }
            unknown => return Err(format!("unknown config export argument: {unknown}").into()),
        }
    }
    if check && execute {
        return Err("config export --check and --execute are mutually exclusive".into());
    }
    Ok(ConfigExportOptions {
        kind: kind.ok_or("config export requires --kind store|project")?,
        root: root.ok_or("config export requires --root PATH")?,
        check,
        execute,
    })
}

fn build_export_plan(options: &ConfigExportOptions) -> Result<ExportPlan, Box<dyn Error>> {
    let root = normalize_root(&options.root)?;
    let source_relative = PathBuf::from(options.kind.source_path());
    let projection_relative = PathBuf::from(options.kind.projection_path());
    let source_path = root.join(&source_relative);
    reject_symlink_file(&source_path, "configuration source")?;
    let source_bytes = read_bounded(&source_path, SOURCE_FILE_MAX_BYTES)?;
    let evaluated = evaluate_source(options.kind, &source_bytes)?;
    let projection_bytes = build_projection(options.kind, &source_bytes, evaluated)?;
    let projection_path = root.join(&projection_relative);
    let prior_projection = read_optional_regular_file(&projection_path)?;
    Ok(ExportPlan {
        kind: options.kind,
        root,
        source_relative,
        projection_relative,
        source_bytes,
        prior_projection,
        projection_bytes,
    })
}

fn evaluate_source(kind: ConfigKind, source_bytes: &[u8]) -> Result<Value, Box<dyn Error>> {
    let stage = stage_root(kind)?;
    let result = (|| {
        fs::create_dir_all(&stage)?;
        fs::write(stage.join(STAGED_SOURCE), source_bytes)?;
        fs::write(stage.join(STAGED_CONTRACTS), CONTRACTS_SOURCE.as_bytes())?;
        let contract = format!(
            "let contracts = import \"{STAGED_CONTRACTS}\" in contracts.{}\n",
            kind.contract_field()
        );
        fs::write(stage.join(STAGED_CONTRACT), contract)?;
        let export = export_json_dual_run_with_contract(
            &format!("cairn.lifecycle-config.{}", kind.name()),
            &stage.join(STAGED_SOURCE),
            &stage.join(STAGED_OUTPUT),
            &stage.join(STAGED_CONTRACT),
            false,
        )?;
        Ok::<Value, Box<dyn Error>>(serde_json::from_slice(&export.bytes)?)
    })();
    let cleanup = fs::remove_dir_all(&stage);
    match (result, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(error), _) => Err(error),
        (Ok(_), Err(error)) => Err(error.into()),
    }
}

fn build_projection(
    kind: ConfigKind,
    source_bytes: &[u8],
    evaluated: Value,
) -> Result<Vec<u8>, Box<dyn Error>> {
    let source_blake3 = blake3_hex(source_bytes);
    let value = match kind {
        ConfigKind::Store => build_store_projection(evaluated, &source_blake3)?,
        ConfigKind::Project => build_project_projection(evaluated, &source_blake3)?,
    };
    Ok((canonical_json(&value) + "\n").into_bytes())
}

fn build_store_projection(evaluated: Value, source_blake3: &str) -> Result<Value, Box<dyn Error>> {
    let source: StoreSource = serde_json::from_value(evaluated)?;
    if source.schema != STORE_SOURCE_SCHEMA_V1 {
        return Err(format!("unsupported store source schema: {}", source.schema).into());
    }
    let projection = LifecycleStoreIdentityProjection {
        schema: LIFECYCLE_STORE_IDENTITY_SCHEMA_V1.to_string(),
        store_id: source.store_id,
        remote_hint: source.remote_hint,
        non_claims: source.non_claims,
        source_identity: source_blake3.to_string(),
    };
    let admission = admit_lifecycle_store_identity(&projection);
    let identity = admission.identity.ok_or_else(|| {
        admission
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join("; ")
    })?;
    serde_json::to_value(LifecycleStoreIdentityProjection {
        schema: identity.schema,
        store_id: identity.store_id.as_str().to_string(),
        remote_hint: identity.remote_hint.map(|value| value.as_str().to_string()),
        non_claims: identity
            .non_claims
            .iter()
            .map(|value| value.as_str().to_string())
            .collect(),
        source_identity: identity.source_identity.as_str().to_string(),
    })
    .map_err(Into::into)
}

// r[impl cairn.agent_guidance_context.project_config.valid]
// r[impl cairn.agent_guidance_context.project_config.invalid]
fn build_project_projection(
    evaluated: Value,
    source_blake3: &str,
) -> Result<Value, Box<dyn Error>> {
    let source: ProjectSource = serde_json::from_value(evaluated)?;
    if source.schema != PROJECT_SOURCE_SCHEMA_V1 {
        return Err(format!("unsupported project source schema: {}", source.schema).into());
    }
    let non_claims = validate_non_claims(source.non_claims, PROJECT_REQUIRED_NON_CLAIMS)?;
    if let Some(pointer) = &source.lifecycle_store {
        validate_reference(pointer, source_blake3)?;
    }
    let declaration = AgentGuidanceDeclaration {
        context_refs: source.context_refs,
        artifact_guidance: source.artifact_guidance,
        operation_guidance: source.operation_guidance,
    };
    let generated_policy = parse_policy_json(GENERATED_POLICY_JSON)?;
    let admitted =
        admit_agent_guidance_declaration(&declaration, &generated_policy.agent_guidance_policy)
            .map_err(|issues| {
                let diagnostics = issues
                    .iter()
                    .map(|issue| format!("{}:{}: {}", issue.code, issue.field_path, issue.message))
                    .collect::<Vec<_>>()
                    .join("; ");
                format!("project guidance declaration is invalid: {diagnostics}")
            })?;
    let mut references = source.references;
    for reference in &references {
        validate_reference(reference, source_blake3)?;
    }
    references.sort_by(|left, right| {
        left.store_id
            .cmp(&right.store_id)
            .then(left.remote_hint.cmp(&right.remote_hint))
    });
    let mut seen = BTreeSet::new();
    for reference in &references {
        if !seen.insert(reference.store_id.as_str()) {
            return Err(format!("duplicate project reference: {}", reference.store_id).into());
        }
        if source
            .lifecycle_store
            .as_ref()
            .is_some_and(|pointer| pointer.store_id == reference.store_id)
        {
            return Err(format!(
                "project store cannot reference itself: {}",
                reference.store_id
            )
            .into());
        }
    }
    serde_json::to_value(ProjectProjection {
        schema: PROJECT_PROJECTION_SCHEMA_V1.to_string(),
        lifecycle_store: source.lifecycle_store,
        references,
        context_refs: admitted.declaration.context_refs,
        artifact_guidance: admitted.declaration.artifact_guidance,
        operation_guidance: admitted.declaration.operation_guidance,
        source_blake3: source_blake3.to_string(),
        non_claims,
    })
    .map_err(Into::into)
}

fn validate_reference(
    reference: &StoreReference,
    source_blake3: &str,
) -> Result<(), Box<dyn Error>> {
    let projection = LifecycleStoreIdentityProjection {
        schema: LIFECYCLE_STORE_IDENTITY_SCHEMA_V1.to_string(),
        store_id: reference.store_id.clone(),
        remote_hint: reference.remote_hint.clone(),
        non_claims: LIFECYCLE_STORE_IDENTITY_NON_CLAIMS
            .iter()
            .map(|value| (*value).to_string())
            .collect(),
        source_identity: source_blake3.to_string(),
    };
    let admission = admit_lifecycle_store_identity(&projection);
    if admission.valid {
        Ok(())
    } else {
        Err(admission
            .diagnostics
            .iter()
            .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message))
            .collect::<Vec<_>>()
            .join("; ")
            .into())
    }
}

fn validate_non_claims(
    mut values: Vec<String>,
    required: &[&str],
) -> Result<Vec<String>, Box<dyn Error>> {
    if values.len() > LIFECYCLE_STORE_NON_CLAIMS_MAX_COUNT {
        return Err("project non-claim count exceeds the reviewed bound".into());
    }
    values.sort();
    let mut seen = BTreeSet::new();
    for value in &values {
        if value.is_empty()
            || value.len() > LIFECYCLE_STORE_NON_CLAIM_MAX_BYTES
            || value.trim() != value
            || value.chars().any(char::is_control)
        {
            return Err("project non-claim contains unsafe text".into());
        }
        if !seen.insert(value.as_str()) {
            return Err("project non-claims contain a duplicate".into());
        }
    }
    for required_value in required {
        if !seen.contains(required_value) {
            return Err(format!(
                "project configuration is missing required non-claim: {required_value}"
            )
            .into());
        }
    }
    Ok(values)
}

fn plan_receipt(
    plan: &ExportPlan,
    check: bool,
    execute: bool,
    valid: bool,
    mutated: bool,
) -> Value {
    let current = plan
        .prior_projection
        .as_ref()
        .is_some_and(|bytes| bytes == &plan.projection_bytes);
    json!({
        "schema": CONFIG_EXPORT_SCHEMA_V1,
        "command": "config export",
        "kind": plan.kind.name(),
        "root": plan.root,
        "source": plan.source_relative,
        "projection": plan.projection_relative,
        "source_blake3": blake3_hex(&plan.source_bytes),
        "projection_blake3": blake3_hex(&plan.projection_bytes),
        "prior_projection_blake3": plan.prior_projection.as_ref().map(|bytes| blake3_hex(bytes)),
        "check": check,
        "dry_run": !execute,
        "execute": execute,
        "current": current || mutated,
        "mutated": mutated,
        "actions": if current { Vec::<Value>::new() } else { vec![json!({
            "kind": "replace_projection",
            "path": plan.projection_relative,
            "description": "atomically replace the selected checked projection after input recheck",
        })] },
        "valid": valid,
        "verdict": if valid { "PASS" } else { "FAIL" },
    })
}

fn execute_plan(plan: &ExportPlan) -> Result<(), Box<dyn Error>> {
    let destination = plan.root.join(&plan.projection_relative);
    let directory = destination
        .parent()
        .ok_or("config projection requires one parent directory")?;
    let file_name = destination
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or("config projection requires a UTF-8 file name")?;
    let lock_path = directory.join(format!(".{file_name}.cairn-lock"));
    let temporary_path = directory.join(format!(".{file_name}.cairn-tmp"));
    let _lock = FileGuard::create_lock(&lock_path)?;
    let mut temporary_guard = FileGuard::for_prepared(&temporary_path);

    let current_source = read_bounded(
        &plan.root.join(&plan.source_relative),
        SOURCE_FILE_MAX_BYTES,
    )?;
    if current_source != plan.source_bytes {
        return Err("configuration source changed after export planning".into());
    }
    let current_projection = read_optional_regular_file(&destination)?;
    if current_projection != plan.prior_projection {
        return Err("configuration projection changed after export planning".into());
    }
    reject_existing_path(&temporary_path)?;
    let mut temporary = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary_path)?;
    temporary.write_all(&plan.projection_bytes)?;
    temporary.sync_all()?;
    drop(temporary);
    if fs::read(&temporary_path)? != plan.projection_bytes {
        return Err("prepared configuration projection failed byte recheck".into());
    }
    fs::rename(&temporary_path, &destination)?;
    temporary_guard.disarm();
    if fs::read(&destination)? != plan.projection_bytes {
        return Err("configuration projection failed final byte recheck".into());
    }
    Ok(())
}

fn normalize_root(root: &Path) -> Result<PathBuf, Box<dyn Error>> {
    let metadata = fs::symlink_metadata(root)?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(format!(
            "config export root must be a non-symlink directory: {}",
            root.display()
        )
        .into());
    }
    Ok(fs::canonicalize(root)?)
}

fn read_bounded(path: &Path, max_bytes: u64) -> Result<Vec<u8>, Box<dyn Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err(format!(
            "configuration source must be a regular non-symlink file: {}",
            path.display()
        )
        .into());
    }
    if metadata.len() > max_bytes {
        return Err(format!(
            "configuration source exceeds the byte bound: {}",
            path.display()
        )
        .into());
    }
    Ok(fs::read(path)?)
}

fn read_optional_regular_file(path: &Path) -> Result<Option<Vec<u8>>, Box<dyn Error>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() || !metadata.is_file() => Err(format!(
            "configuration projection must be a regular non-symlink file: {}",
            path.display()
        )
        .into()),
        Ok(_) => Ok(Some(fs::read(path)?)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn reject_symlink_file(path: &Path, label: &str) -> Result<(), Box<dyn Error>> {
    let metadata = fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(format!("{label} must not be a symlink: {}", path.display()).into());
    }
    Ok(())
}

fn reject_existing_path(path: &Path) -> Result<(), Box<dyn Error>> {
    match fs::symlink_metadata(path) {
        Ok(_) => Err(format!(
            "prepared configuration path already exists: {}",
            path.display()
        )
        .into()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn stage_root(kind: ConfigKind) -> Result<PathBuf, Box<dyn Error>> {
    let stage = PathBuf::from(STAGING_ROOT).join(format!("{}-{}", kind.name(), std::process::id()));
    match fs::remove_dir_all(&stage) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    Ok(stage)
}

struct FileGuard {
    path: PathBuf,
    remove: bool,
}

impl FileGuard {
    fn create_lock(path: &Path) -> Result<Self, Box<dyn Error>> {
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(b"cairn config export\n")?;
        file.sync_all()?;
        Ok(Self {
            path: path.to_path_buf(),
            remove: true,
        })
    }

    fn for_prepared(path: &Path) -> Self {
        Self {
            path: path.to_path_buf(),
            remove: true,
        }
    }

    fn disarm(&mut self) {
        self.remove = false;
    }
}

impl Drop for FileGuard {
    fn drop(&mut self) {
        if self.remove {
            let _ = fs::remove_file(&self.path);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PRIOR_PROJECTION: &[u8] = b"prior projection\n";
    const CHANGED_PROJECTION: &[u8] = b"changed projection\n";
    const NEXT_PROJECTION: &[u8] = b"next projection\n";
    const SOURCE_BYTES: &[u8] = b"{}\n";

    #[test]
    fn overwrite_recheck_rejects_drift_without_partial_mutation() -> Result<(), Box<dyn Error>> {
        let root = std::env::temp_dir().join(format!(
            "cairn-config-export-overwrite-recheck-{}",
            std::process::id()
        ));
        match fs::remove_dir_all(&root) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
        fs::create_dir_all(root.join(".cairn-store"))?;
        fs::write(root.join(STORE_SOURCE_PATH), SOURCE_BYTES)?;
        fs::write(root.join(STORE_PROJECTION_PATH), PRIOR_PROJECTION)?;
        let plan = ExportPlan {
            kind: ConfigKind::Store,
            root: root.clone(),
            source_relative: PathBuf::from(STORE_SOURCE_PATH),
            projection_relative: PathBuf::from(STORE_PROJECTION_PATH),
            source_bytes: SOURCE_BYTES.to_vec(),
            prior_projection: Some(PRIOR_PROJECTION.to_vec()),
            projection_bytes: NEXT_PROJECTION.to_vec(),
        };

        fs::write(root.join(STORE_PROJECTION_PATH), CHANGED_PROJECTION)?;
        let error = execute_plan(&plan).expect_err("projection drift must fail the overwrite");
        assert!(error
            .to_string()
            .contains("projection changed after export planning"));
        assert_eq!(
            fs::read(root.join(STORE_PROJECTION_PATH))?,
            CHANGED_PROJECTION
        );
        assert!(!root.join(".cairn-store/.store.json.cairn-tmp").exists());
        assert!(!root.join(".cairn-store/.store.json.cairn-lock").exists());

        fs::remove_dir_all(root)?;
        Ok(())
    }
}

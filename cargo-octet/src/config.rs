use std::fs;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use octet_standards::nickel_export::NamedText;
use octet_standards::nickel_export::NickelExportInput;
use octet_standards::nickel_export::NickelExportManifest;
use octet_standards::nickel_export::NickelExportSpec;
use octet_standards::nickel_export::manifest_for_exports;
use octet_standards::project_architecture_policy::ProjectArchitecturePolicy;
use octet_standards::project_architecture_policy::parse_project_architecture_policy;

const ARCHITECTURE_POLICY_FIELD: &str = "architecture_policy";
const ARCHITECTURE_POLICY_FAMILY_ID: &str = "octet.project-architecture-policy";
const ARCHITECTURE_POLICY_CONTRACT: &str = "ProjectArchitecturePolicy and ProjectArchitecturePolicyRules";
const ARCHITECTURE_POLICY_SOURCE_EXTENSION: &str = "ncl";
const ARCHITECTURE_POLICY_OUTPUT_EXTENSION: &str = "json";
const ARCHITECTURE_POLICY_CONTRACT_SUFFIX: &str = ".contracts.ncl";
const ARCHITECTURE_POLICY_MANIFEST_SUFFIX: &str = ".manifest.json";
const JSON_OUTPUT_FORMAT: &str = "json";
const HEX_CHARS_PER_BYTE: usize = 2;
const BLAKE3_HEX_LENGTH: usize = blake3::OUT_LEN * HEX_CHARS_PER_BYTE;

#[derive(Debug)]
pub struct WorkspaceConfig {
    pub workspace_root: PathBuf,
    pub default_scope: Vec<String>,
    pub cargo_check_args: Vec<String>,
    pub architecture_policy: Option<SelectedProjectArchitecturePolicy>,
}

#[derive(Debug)]
pub struct SelectedProjectArchitecturePolicy {
    pub source_path: PathBuf,
    pub export_path: PathBuf,
    pub manifest_path: PathBuf,
    pub source_blake3: String,
    pub policy_blake3: String,
    pub policy: ProjectArchitecturePolicy,
}

impl SelectedProjectArchitecturePolicy {
    pub(crate) fn validate_loaded_binding(&self) -> Result<(), String> {
        if self.source_path.as_os_str().is_empty()
            || self.export_path.as_os_str().is_empty()
            || self.manifest_path.as_os_str().is_empty()
        {
            return Err("activated architecture policy paths must not be empty".to_owned());
        }
        if self.source_blake3.len() != BLAKE3_HEX_LENGTH || self.policy_blake3.len() != BLAKE3_HEX_LENGTH {
            return Err("activated architecture policy identities must be BLAKE3 hex values".to_owned());
        }
        if self.policy.project_id.trim().is_empty() {
            return Err("activated architecture policy project identity must not be empty".to_owned());
        }
        Ok(())
    }
}

// r[impl octet.project_architecture_policy.activation]
// r[impl octet.project_architecture_policy.compatibility]
pub fn load(start_dir: &Path) -> Result<WorkspaceConfig, String> {
    let manifest_path = find_workspace_manifest(start_dir)?;
    let workspace_root = manifest_path
        .parent()
        .ok_or_else(|| format!("workspace manifest has no parent: {}", manifest_path.display()))?
        .to_path_buf();
    let source =
        fs::read_to_string(&manifest_path).map_err(|error| format!("read {}: {error}", manifest_path.display()))?;
    let document = source
        .parse::<toml::Table>()
        .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;

    let workspace = table_field(&document, "workspace", manifest_path.as_path())?;
    let metadata = table_field(workspace, "metadata", manifest_path.as_path())?;
    let octet = workspace_metadata_table(metadata, manifest_path.as_path())?;
    let architecture_policy = metadata
        .get("octet")
        .and_then(toml::Value::as_table)
        .map(|current| load_architecture_policy(&workspace_root, current, manifest_path.as_path()))
        .transpose()?
        .flatten();

    Ok(WorkspaceConfig {
        workspace_root,
        default_scope: string_array_field(octet, "default_scope", manifest_path.as_path())?,
        cargo_check_args: string_array_field(octet, "cargo_check_args", manifest_path.as_path())?,
        architecture_policy,
    })
}

fn load_architecture_policy(
    workspace_root: &Path,
    octet: &toml::Table,
    manifest_path: &Path,
) -> Result<Option<SelectedProjectArchitecturePolicy>, String> {
    let Some(raw_pointer) = optional_string_field(octet, ARCHITECTURE_POLICY_FIELD, manifest_path)? else {
        return Ok(None);
    };
    let source_path = normalize_workspace_relative_path(&raw_pointer, ARCHITECTURE_POLICY_FIELD)?;
    if source_path.extension().and_then(|extension| extension.to_str()) != Some(ARCHITECTURE_POLICY_SOURCE_EXTENSION) {
        return Err(format!(
            "`{ARCHITECTURE_POLICY_FIELD}` must name a Nickel .{ARCHITECTURE_POLICY_SOURCE_EXTENSION} file"
        ));
    }
    let (export_path, contract_path, export_manifest_path) = companion_paths(&source_path)?;
    let source_bytes = read_workspace_file(workspace_root, &source_path, "architecture policy source")?;
    let contract_bytes = read_workspace_file(workspace_root, &contract_path, "architecture policy contract")?;
    let export_bytes = read_workspace_file(workspace_root, &export_path, "architecture policy export")?;
    let export_manifest_bytes =
        read_workspace_file(workspace_root, &export_manifest_path, "architecture policy export manifest")?;
    let policy = parse_project_architecture_policy(&export_bytes).map_err(|diagnostics| {
        let messages = diagnostics
            .iter()
            .map(|diagnostic| format!("{}:{}: {}", diagnostic.code, diagnostic.field, diagnostic.message))
            .collect::<Vec<_>>()
            .join("; ");
        format!("invalid activated project architecture policy: {messages}")
    })?;
    let manifest = serde_json::from_slice::<NickelExportManifest>(&export_manifest_bytes)
        .map_err(|error| format!("parse {}: {error}", export_manifest_path.display()))?;
    let source_text = utf8_text(&source_path, &source_bytes)?;
    let contract_text = utf8_text(&contract_path, &contract_bytes)?;
    let export_text = utf8_text(&export_path, &export_bytes)?;
    let source_name = workspace_path_text(&source_path)?;
    let contract_name = workspace_path_text(&contract_path)?;
    let export_name = workspace_path_text(&export_path)?;
    let spec = NickelExportSpec {
        schema_version: octet_standards::nickel_export::NICKEL_EXPORT_SPEC_SCHEMA_VERSION.to_owned(),
        family_id: ARCHITECTURE_POLICY_FAMILY_ID.to_owned(),
        source: source_name,
        dependencies: vec![contract_name.clone()],
        selector: String::new(),
        contract: ARCHITECTURE_POLICY_CONTRACT.to_owned(),
        output_format: JSON_OUTPUT_FORMAT.to_owned(),
        output_path: export_name,
        allow_secret_material: false,
    };
    let input = NickelExportInput {
        spec: &spec,
        source_text,
        dependencies: vec![NamedText {
            path: &contract_name,
            text: contract_text,
        }],
        output_text: export_text,
        nickel_identity: &manifest.nickel_identity,
        contract_diagnostics: Vec::new(),
    };
    let expected = manifest_for_exports(&[input])
        .map_err(|diagnostics| format!("invalid activated architecture export inputs: {}", diagnostics.join("; ")))?;
    if manifest != expected {
        return Err(format!(
            "stale architecture policy export manifest `{}`; regenerate the deterministic Nickel export and BLAKE3 manifest",
            export_manifest_path.display()
        ));
    }

    let selected = SelectedProjectArchitecturePolicy {
        source_path,
        export_path,
        manifest_path: export_manifest_path,
        source_blake3: blake3::hash(&source_bytes).to_hex().to_string(),
        policy_blake3: blake3::hash(&export_bytes).to_hex().to_string(),
        policy,
    };
    selected.validate_loaded_binding()?;
    Ok(Some(selected))
}

fn companion_paths(source_path: &Path) -> Result<(PathBuf, PathBuf, PathBuf), String> {
    let stem = source_path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| format!("`{ARCHITECTURE_POLICY_FIELD}` must have a UTF-8 file stem"))?;
    let export_path = source_path.with_extension(ARCHITECTURE_POLICY_OUTPUT_EXTENSION);
    let contract_path = source_path.with_file_name(format!("{stem}{ARCHITECTURE_POLICY_CONTRACT_SUFFIX}"));
    let manifest_path = source_path.with_file_name(format!("{stem}{ARCHITECTURE_POLICY_MANIFEST_SUFFIX}"));
    Ok((export_path, contract_path, manifest_path))
}

fn read_workspace_file(workspace_root: &Path, relative_path: &Path, label: &str) -> Result<Vec<u8>, String> {
    let canonical_root = workspace_root
        .canonicalize()
        .map_err(|error| format!("canonicalize workspace {}: {error}", workspace_root.display()))?;
    let candidate = workspace_root.join(relative_path);
    let canonical_candidate =
        candidate.canonicalize().map_err(|error| format!("read {label} {}: {error}", candidate.display()))?;
    if !canonical_candidate.starts_with(&canonical_root) {
        return Err(format!(
            "{label} `{}` resolves outside workspace `{}`",
            relative_path.display(),
            workspace_root.display()
        ));
    }
    fs::read(&canonical_candidate).map_err(|error| format!("read {label} {}: {error}", candidate.display()))
}

fn normalize_workspace_relative_path(raw: &str, field: &str) -> Result<PathBuf, String> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return Err(format!("`{field}` must not be empty"));
    }
    let path = Path::new(trimmed);
    if path.is_absolute() {
        return Err(format!("`{field}` must be workspace-relative and must not be absolute"));
    }
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(value) => normalized.push(value),
            Component::CurDir => {}
            Component::ParentDir => {
                return Err(format!("`{field}` must stay within the workspace and must not contain `..`"));
            }
            Component::RootDir | Component::Prefix(_) => {
                return Err(format!("`{field}` must be workspace-relative"));
            }
        }
    }
    if normalized.as_os_str().is_empty() {
        return Err(format!("`{field}` must name a file below the workspace root"));
    }
    Ok(normalized)
}

fn workspace_path_text(path: &Path) -> Result<String, String> {
    let parts = path
        .components()
        .map(|component| match component {
            Component::Normal(value) => value
                .to_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("workspace path is not UTF-8: {}", path.display())),
            _ => Err(format!("workspace path is not normalized: {}", path.display())),
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(parts.join("/"))
}

fn utf8_text<'a>(path: &Path, bytes: &'a [u8]) -> Result<&'a str, String> {
    std::str::from_utf8(bytes).map_err(|error| format!("{} is not UTF-8: {error}", path.display()))
}

fn optional_string_field(table: &toml::Table, key: &str, manifest_path: &Path) -> Result<Option<String>, String> {
    table
        .get(key)
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("`{key}` must be a string in {}", manifest_path.display()))
        })
        .transpose()
}

fn workspace_metadata_table<'a>(metadata: &'a toml::Table, manifest_path: &Path) -> Result<&'a toml::Table, String> {
    metadata
        .get("octet")
        .and_then(toml::Value::as_table)
        .or_else(|| metadata.get("slotcar").and_then(toml::Value::as_table))
        .or_else(|| metadata.get("tigerstyle").and_then(toml::Value::as_table))
        .ok_or_else(|| format!("missing `[workspace.metadata.octet]` table in {}", manifest_path.display()))
}

fn find_workspace_manifest(start_dir: &Path) -> Result<PathBuf, String> {
    let mut dir = start_dir.canonicalize().map_err(|error| format!("canonicalize {}: {error}", start_dir.display()))?;

    loop {
        let manifest_path = dir.join("Cargo.toml");
        if manifest_path.is_file() {
            let source = fs::read_to_string(&manifest_path)
                .map_err(|error| format!("read {}: {error}", manifest_path.display()))?;
            let document = source
                .parse::<toml::Table>()
                .map_err(|error| format!("parse {}: {error}", manifest_path.display()))?;
            if document.contains_key("workspace") {
                return Ok(manifest_path);
            }
        }

        if !dir.pop() {
            break;
        }
    }

    Err(format!("could not find workspace Cargo.toml starting from {}", start_dir.display()))
}

fn table_field<'a>(table: &'a toml::Table, key: &str, manifest_path: &Path) -> Result<&'a toml::Table, String> {
    table
        .get(key)
        .and_then(toml::Value::as_table)
        .ok_or_else(|| format!("missing `[{}]` table in {}", key, manifest_path.display()))
}

fn string_array_field(table: &toml::Table, key: &str, manifest_path: &Path) -> Result<Vec<String>, String> {
    let values = table
        .get(key)
        .and_then(toml::Value::as_array)
        .ok_or_else(|| format!("missing `{key}` array in {}", manifest_path.display()))?;

    values
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| format!("`{key}` must contain only strings in {}", manifest_path.display()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::fs;

    use octet_standards::nickel_export::NamedText;
    use octet_standards::nickel_export::NickelExportInput;
    use octet_standards::nickel_export::NickelExportSpec;
    use octet_standards::nickel_export::manifest_for_exports;
    use tempfile::TempDir;

    use super::*;

    const POLICY_SOURCE: &str = "let contracts = import \"./octet-architecture.contracts.ncl\" in {}\n";
    const CONTRACT_SOURCE: &str = "{ ProjectArchitecturePolicy = Dyn, ProjectArchitecturePolicyRules = Dyn }\n";
    const POLICY_EXPORT: &str = include_str!("../../standards/project-architecture/octet-self-architecture.json");
    const NICKEL_IDENTITY: &str = "nickel-test-identity";

    fn metadata_table(source: &str) -> toml::Table {
        let document = source.parse::<toml::Table>().expect("parse Cargo.toml");
        document
            .get("workspace")
            .and_then(toml::Value::as_table)
            .and_then(|workspace| workspace.get("metadata"))
            .and_then(toml::Value::as_table)
            .cloned()
            .expect("workspace metadata table")
    }

    #[test]
    fn octet_config_and_command_paths() {
        let metadata = metadata_table(
            r#"
[workspace.metadata.octet]
default_scope = ["-p", "octet"]
cargo_check_args = ["--lib"]

[workspace.metadata.slotcar]
default_scope = ["-p", "slotcar"]
cargo_check_args = ["--tests"]
"#,
        );

        let selected =
            workspace_metadata_table(&metadata, std::path::Path::new("Cargo.toml")).expect("select Octet metadata");
        assert_eq!(selected["default_scope"].as_array().expect("scope array")[1].as_str(), Some("octet"));

        let missing = toml::Table::new();
        let error = workspace_metadata_table(&missing, std::path::Path::new("Cargo.toml"))
            .expect_err("missing current and compatibility config should fail");
        assert!(error.contains("missing `[workspace.metadata.octet]`"));
    }

    #[test]
    fn absent_architecture_pointer_is_inactive() {
        let workspace = workspace_fixture(false);
        let config = load(workspace.path()).expect("load unconfigured workspace");
        assert!(config.architecture_policy.is_none());
    }

    #[test]
    fn active_architecture_pointer_loads_fresh_policy() {
        let workspace = workspace_fixture(true);
        write_architecture_policy(workspace.path());
        let config = load(workspace.path()).expect("load configured workspace");
        let selected = config.architecture_policy.expect("selected architecture policy");
        assert_eq!(selected.policy.project_id, "octet");
        assert_eq!(selected.source_path, PathBuf::from("octet-architecture.ncl"));
        assert_eq!(selected.policy_blake3.len(), BLAKE3_HEX_LENGTH);
        assert_eq!(selected.source_blake3.len(), BLAKE3_HEX_LENGTH);
    }

    #[test]
    fn architecture_pointer_rejects_escape_and_stale_manifest() {
        let escaping = workspace_fixture_with_pointer("../outside.ncl");
        let escape_error = load(escaping.path()).expect_err("escaping pointer must fail");
        assert!(escape_error.contains("must stay within the workspace"));

        let stale = workspace_fixture(true);
        write_architecture_policy(stale.path());
        fs::write(stale.path().join("octet-architecture.json"), "{}\n").expect("tamper export");
        let stale_error = load(stale.path()).expect_err("stale policy manifest must fail");
        assert!(
            stale_error.contains("invalid activated project architecture policy")
                || stale_error.contains("stale architecture policy export manifest")
        );
    }

    #[test]
    fn legacy_metadata_cannot_activate_project_architecture() {
        let workspace = tempfile::tempdir().expect("create legacy workspace fixture");
        let manifest = r#"
[workspace]
resolver = "2"

[workspace.metadata.slotcar]
default_scope = ["--workspace"]
cargo_check_args = ["--lib"]
architecture_policy = "octet-architecture.ncl"
"#;
        fs::write(workspace.path().join("Cargo.toml"), manifest).expect("write legacy workspace manifest");

        let config = load(workspace.path()).expect("load legacy workspace");

        assert!(config.architecture_policy.is_none());
    }

    #[test]
    fn octet_metadata_wins_over_legacy_sections() {
        let metadata = metadata_table(
            r#"
[workspace.metadata.octet]
default_scope = ["-p", "octet"]
cargo_check_args = ["--lib"]

[workspace.metadata.slotcar]
default_scope = ["-p", "slotcar"]
cargo_check_args = ["--tests"]

[workspace.metadata.tigerstyle]
default_scope = ["-p", "tigerstyle"]
cargo_check_args = ["--bins"]
"#,
        );

        let selected =
            workspace_metadata_table(&metadata, std::path::Path::new("Cargo.toml")).expect("select metadata");

        assert_eq!(selected["default_scope"].as_array().expect("scope array")[1].as_str(), Some("octet"));
    }

    #[test]
    fn slotcar_metadata_remains_legacy_fallback() {
        let metadata = metadata_table(
            r#"
[workspace.metadata.slotcar]
default_scope = ["-p", "slotcar"]
cargo_check_args = ["--lib"]
"#,
        );

        let selected =
            workspace_metadata_table(&metadata, std::path::Path::new("Cargo.toml")).expect("select metadata");

        assert_eq!(selected["default_scope"].as_array().expect("scope array")[1].as_str(), Some("slotcar"));
    }

    fn workspace_fixture(with_policy: bool) -> TempDir {
        if with_policy {
            workspace_fixture_with_pointer("octet-architecture.ncl")
        } else {
            workspace_fixture_with_pointer("")
        }
    }

    fn workspace_fixture_with_pointer(pointer: &str) -> TempDir {
        let workspace = tempfile::tempdir().expect("create workspace fixture");
        let architecture_line = if pointer.is_empty() {
            String::new()
        } else {
            format!("architecture_policy = \"{pointer}\"\n")
        };
        let manifest = format!(
            "[workspace]\nresolver = \"2\"\n\n[workspace.metadata.octet]\ndefault_scope = [\"--workspace\"]\ncargo_check_args = [\"--lib\"]\n{architecture_line}"
        );
        fs::write(workspace.path().join("Cargo.toml"), manifest).expect("write workspace manifest");
        workspace
    }

    fn write_architecture_policy(workspace: &Path) {
        let source_path = workspace.join("octet-architecture.ncl");
        let contract_path = workspace.join("octet-architecture.contracts.ncl");
        let export_path = workspace.join("octet-architecture.json");
        let manifest_path = workspace.join("octet-architecture.manifest.json");
        fs::write(&source_path, POLICY_SOURCE).expect("write policy source");
        fs::write(&contract_path, CONTRACT_SOURCE).expect("write policy contract");
        fs::write(&export_path, POLICY_EXPORT).expect("write policy export");
        let spec = NickelExportSpec {
            schema_version: octet_standards::nickel_export::NICKEL_EXPORT_SPEC_SCHEMA_VERSION.to_owned(),
            family_id: ARCHITECTURE_POLICY_FAMILY_ID.to_owned(),
            source: "octet-architecture.ncl".to_owned(),
            dependencies: vec!["octet-architecture.contracts.ncl".to_owned()],
            selector: String::new(),
            contract: ARCHITECTURE_POLICY_CONTRACT.to_owned(),
            output_format: JSON_OUTPUT_FORMAT.to_owned(),
            output_path: "octet-architecture.json".to_owned(),
            allow_secret_material: false,
        };
        let input = NickelExportInput {
            spec: &spec,
            source_text: POLICY_SOURCE,
            dependencies: vec![NamedText {
                path: "octet-architecture.contracts.ncl",
                text: CONTRACT_SOURCE,
            }],
            output_text: POLICY_EXPORT,
            nickel_identity: NICKEL_IDENTITY,
            contract_diagnostics: Vec::new(),
        };
        let manifest = manifest_for_exports(&[input]).expect("build policy manifest");
        let text = serde_json::to_string_pretty(&manifest).expect("serialize policy manifest");
        fs::write(manifest_path, format!("{text}\n")).expect("write policy manifest");
    }
}

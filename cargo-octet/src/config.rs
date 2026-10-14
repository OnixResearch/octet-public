use std::fs;
use std::path::Path;
use std::path::PathBuf;

#[derive(Debug)]
pub struct WorkspaceConfig {
    pub workspace_root: PathBuf,
    pub default_scope: Vec<String>,
    pub cargo_check_args: Vec<String>,
}

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

    Ok(WorkspaceConfig {
        workspace_root,
        default_scope: string_array_field(octet, "default_scope", manifest_path.as_path())?,
        cargo_check_args: string_array_field(octet, "cargo_check_args", manifest_path.as_path())?,
    })
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
    use super::workspace_metadata_table;

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
        assert_eq!(selected["default_scope"].as_array().unwrap()[1].as_str(), Some("octet"));

        let missing = toml::Table::new();
        let error = workspace_metadata_table(&missing, std::path::Path::new("Cargo.toml"))
            .expect_err("missing current and compatibility config should fail");
        assert!(error.contains("missing `[workspace.metadata.octet]`"));
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

        assert_eq!(selected["default_scope"].as_array().unwrap()[1].as_str(), Some("octet"));
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

        assert_eq!(selected["default_scope"].as_array().unwrap()[1].as_str(), Some("slotcar"));
    }
}

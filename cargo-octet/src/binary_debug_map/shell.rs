//! Capability-rooted collection shell for the binary debug-map rail.
//!
//! The shell opens each supplied file through its parent directory capability,
//! enforces profile byte limits before parsing, and returns a deterministic
//! envelope. Bundle publication remains the generic evidence-rail writer's job.

use std::path::Path;
use std::path::PathBuf;

use super::core;
use super::identity;
use super::linkage;
use super::parse_registry;
use super::parse_requirement_refs;
use super::selected_profile;
use super::validate_registry;
use crate::cap_fs::RootRelativePath;
use crate::cap_fs::SourceRoot;

/// Profile registries are small Nickel exports; reject oversized input before parsing.
pub(crate) const MAX_PROFILE_BYTES: u64 = 1_048_576;
/// Function-address and requirement-reference handoffs are bounded like debug data.
const MAX_LINKAGE_ARTIFACT_BYTES: u64 = 8_388_608;

pub(crate) struct ShellRequest<'a> {
    pub(crate) workspace_root: &'a Path,
    pub(crate) binary: &'a Path,
    pub(crate) companion: Option<&'a Path>,
    pub(crate) config: &'a Path,
    pub(crate) profile_id: &'a str,
    pub(crate) function_addresses: Option<&'a Path>,
    pub(crate) requirement_refs: Option<&'a Path>,
}

#[derive(Debug)]
pub(crate) struct ShellOutput {
    pub(crate) envelope_json: String,
    pub(crate) registry_identity: String,
}

pub(crate) fn run(request: &ShellRequest<'_>) -> Result<ShellOutput, String> {
    let config_bytes =
        read_capability_bounded(request.workspace_root, request.config, MAX_PROFILE_BYTES, "binary debug-map profile")?;
    let config_text = std::str::from_utf8(&config_bytes)
        .map_err(|error| format!("binary debug-map profile registry is not UTF-8: {error}"))?;
    let registry = parse_registry(config_text)?;
    let diagnostics = validate_registry(&registry);
    if !diagnostics.is_empty() {
        return Err(format!("invalid binary debug-map profile registry:\n- {}", diagnostics.join("\n- ")));
    }
    let profile = selected_profile(&registry, request.profile_id)?;
    let limits = core::byte_limits(profile);
    let binary_bytes =
        read_capability_bounded(request.workspace_root, request.binary, limits.max_binary_bytes, "binary")?;
    let companion_bytes = match request.companion {
        Some(companion) => Some(read_capability_bounded(
            request.workspace_root,
            companion,
            limits.max_debug_bytes,
            "debug companion",
        )?),
        None => None,
    };
    let function_addresses = match request.function_addresses {
        Some(path) => {
            let bytes = read_capability_bounded(
                request.workspace_root,
                path,
                MAX_LINKAGE_ARTIFACT_BYTES,
                "function-address artifact",
            )?;
            Some(linkage::parse_function_address_artifact(&bytes)?)
        }
        None => None,
    };
    let requirement_refs = match request.requirement_refs {
        Some(path) => {
            let bytes = read_capability_bounded(
                request.workspace_root,
                path,
                MAX_LINKAGE_ARTIFACT_BYTES,
                "requirement-reference handoff",
            )?;
            let text = std::str::from_utf8(&bytes)
                .map_err(|error| format!("requirement-reference handoff is not UTF-8: {error}"))?;
            let references = parse_requirement_refs(text)?;
            linkage::validate_requirement_ref_set(&references)?;
            Some((references, identity(&bytes)))
        }
        None => None,
    };
    let registry_identity = identity(&config_bytes);
    let envelope = core::analyze(core::AnalysisInput {
        binary: &binary_bytes,
        companion: companion_bytes.as_deref(),
        profile,
        registry_identity: registry_identity.clone(),
        function_addresses: function_addresses.as_ref(),
        requirement_refs: requirement_refs.as_ref().map(|(references, _)| references),
        requirement_refs_identity: requirement_refs.as_ref().map(|(_, identity)| identity.clone()),
    });
    let mut envelope_json =
        serde_json::to_string(&envelope).map_err(|error| format!("serialize binary debug-map envelope: {error}"))?;
    envelope_json.push('\n');
    Ok(ShellOutput {
        envelope_json,
        registry_identity,
    })
}

/// Reads one file through its parent-directory capability under an exact byte
/// limit. Relative paths resolve against the supplied workspace root.
fn read_capability_bounded(workspace_root: &Path, path: &Path, max_bytes: u64, role: &str) -> Result<Vec<u8>, String> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        workspace_root.join(path)
    };
    let parent = absolute.parent().unwrap_or_else(|| Path::new("."));
    let name = absolute.file_name().ok_or_else(|| format!("{role} path {} has no file name", absolute.display()))?;
    let root = SourceRoot::open_existing(parent)?;
    let relative = RootRelativePath::new(PathBuf::from(name))?;
    root.read_bounded(&relative, max_bytes)
}

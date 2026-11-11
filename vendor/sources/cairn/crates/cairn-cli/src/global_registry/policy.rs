use std::{collections::BTreeMap, error::Error};

use cairn_core::{Policy, RegistryCorePolicy, RegistryGraphProfile, GLOBAL_REGISTRY_SCHEMA};
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub(super) struct RegistryAdapterProfile {
    pub id: String,
    pub kind: String,
    pub schema: String,
    pub capabilities: Vec<String>,
    pub record_families: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
struct TagNamespace {
    id: String,
    authority: String,
}

#[derive(Debug, Clone, Deserialize)]
struct RawGlobalRegistryPolicy {
    schema: String,
    storage_schema: String,
    max_artifact_bytes: u64,
    adapters: Vec<RegistryAdapterProfile>,
    tag_namespaces: Vec<TagNamespace>,
    graph_profiles: Vec<RegistryGraphProfile>,
    allowed_vocabulary_owners: Vec<String>,
    required_non_claims: Vec<String>,
}

#[derive(Debug, Clone)]
pub(super) struct GlobalRegistryPolicy {
    pub schema: String,
    pub storage_schema: String,
    pub max_artifact_bytes: u64,
    pub adapters: BTreeMap<String, RegistryAdapterProfile>,
    pub graph_profiles: BTreeMap<String, RegistryGraphProfile>,
    pub core: RegistryCorePolicy,
}

pub(super) fn global_registry_policy(
    policy: &Policy,
) -> Result<GlobalRegistryPolicy, Box<dyn Error>> {
    let raw: RawGlobalRegistryPolicy = serde_json::from_value(
        policy
            .raw
            .get("global_registry_policy")
            .cloned()
            .ok_or("generated policy missing global_registry_policy")?,
    )?;
    let reserved_tag_namespaces = raw
        .tag_namespaces
        .iter()
        .filter(|namespace| namespace.authority.starts_with("reserved-"))
        .map(|namespace| namespace.id.clone())
        .collect();
    let adapters = raw
        .adapters
        .into_iter()
        .map(|adapter| (adapter.id.clone(), adapter))
        .collect();
    let graph_profiles = raw
        .graph_profiles
        .into_iter()
        .map(|profile| (profile.id.clone(), profile))
        .collect();
    Ok(GlobalRegistryPolicy {
        schema: raw.schema,
        storage_schema: raw.storage_schema,
        max_artifact_bytes: raw.max_artifact_bytes,
        adapters,
        graph_profiles,
        core: RegistryCorePolicy {
            reserved_tag_namespaces,
            allowed_vocabulary_owners: raw.allowed_vocabulary_owners,
            required_non_claims: raw.required_non_claims,
        },
    })
}

pub(super) fn validate_global_registry_policy(
    policy: &GlobalRegistryPolicy,
) -> Result<(), Box<dyn Error>> {
    if policy.schema != GLOBAL_REGISTRY_SCHEMA {
        return Err(format!("unsupported global registry schema: {}", policy.schema).into());
    }
    if policy.storage_schema != "cairn.global-registry-storage.v1" {
        return Err(format!(
            "unsupported global registry storage schema: {}",
            policy.storage_schema
        )
        .into());
    }
    Ok(())
}

pub(super) fn require_adapter<'a>(
    policy: &'a GlobalRegistryPolicy,
    adapter_id: &str,
    kind: &str,
) -> Result<&'a RegistryAdapterProfile, Box<dyn Error>> {
    let adapter = policy
        .adapters
        .get(adapter_id)
        .ok_or_else(|| format!("unsupported registry adapter: {adapter_id}"))?;
    if adapter.kind != kind {
        return Err(format!(
            "registry adapter {adapter_id} has kind {}, expected {kind}",
            adapter.kind
        )
        .into());
    }
    Ok(adapter)
}

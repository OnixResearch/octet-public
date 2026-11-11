use super::model::ExperimentProfile;
use super::model::ExperimentRegistry;
use super::model::PROFILES_SCHEMA;
use crate::benchmark::prefixed_identity;

const PROFILE_JSON: &str = include_str!("../../../../standards/benchmark/experiment-profiles.json");

pub const PROFILE_IDENTITY: &str = "b3:85644c90d25fd73f231f6deb1c8351d6cfda65e4461fe5984d3b9be40af36476";

pub fn selected_profile(profile_id: &str) -> Result<ExperimentProfile, String> {
    if prefixed_identity(PROFILE_JSON.as_bytes()) != PROFILE_IDENTITY {
        return Err("embedded experiment profile identity is stale".to_owned());
    }
    let registry: ExperimentRegistry = serde_json::from_str(PROFILE_JSON)
        .map_err(|error| format!("parse embedded experiment profile registry: {error}"))?;
    if registry.schema_version != PROFILES_SCHEMA {
        return Err("embedded experiment profile registry schema is unsupported".to_owned());
    }
    registry
        .profiles
        .into_iter()
        .find(|profile| profile.profile_id == profile_id)
        .ok_or_else(|| format!("unknown experiment profile `{profile_id}`"))
}

#[cfg(test)]
pub fn embedded_profile_identity() -> &'static str {
    PROFILE_IDENTITY
}

#[cfg(test)]
pub fn profile_bytes_identity() -> String {
    prefixed_identity(PROFILE_JSON.as_bytes())
}

use std::path::Path;

use super::EXPECTED_BLOCKER;
use super::model::PilotProfile;
use crate::cap_fs::RootRelativePath;
use crate::cap_fs::SourceRoot;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProviderPlan {
    pub command_argv: Vec<String>,
    pub environment: Vec<String>,
    pub clear_environment: bool,
    pub network_allowed: bool,
    pub timeout_ms: u64,
    pub max_log_bytes: u64,
}

pub fn reviewed_plan(profile: &PilotProfile) -> Result<ProviderPlan, String> {
    let diagnostics = super::policy::validate(profile);
    if !diagnostics.is_empty() {
        return Err(format!("MIRAI pilot preflight rejected profile: {}", diagnostics.join("; ")));
    }
    Ok(ProviderPlan {
        command_argv: profile.provider.command_argv.clone(),
        environment: profile.provider.environment.clone(),
        clear_environment: true,
        network_allowed: false,
        timeout_ms: profile.limits.timeout_ms,
        max_log_bytes: profile.limits.max_log_bytes,
    })
}

pub fn execute(_plan: &ProviderPlan) -> Result<(), String> {
    Err(format!(
        "MIRAI pilot execution blocked before spawn: {EXPECTED_BLOCKER}; no stable machine-readable output contract"
    ))
}

pub fn verify_corpus(workspace: &Path, profile: &PilotProfile) -> Vec<String> {
    let root = match SourceRoot::open_existing(workspace) {
        Ok(root) => root,
        Err(error) => return vec![error],
    };
    let mut diagnostics = Vec::new();
    for fixture in &profile.corpus {
        let relative = match RootRelativePath::new(&fixture.path) {
            Ok(relative) => relative,
            Err(error) => {
                diagnostics.push(error);
                continue;
            }
        };
        match root.read_bounded(&relative, profile.limits.max_source_bytes) {
            Ok(bytes)
                if u64::try_from(bytes.len()).unwrap_or(u64::MAX) == fixture.bytes
                    && super::prefixed_identity(&bytes) == fixture.source_identity => {}
            Ok(_) => diagnostics.push(format!("MIRAI pilot fixture `{}` identity or size drifted", fixture.fixture_id)),
            Err(error) => diagnostics.push(error),
        }
    }
    diagnostics.sort();
    diagnostics.dedup();
    diagnostics
}

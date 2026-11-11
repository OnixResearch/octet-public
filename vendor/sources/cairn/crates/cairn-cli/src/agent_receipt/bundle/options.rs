use std::{error::Error, path::PathBuf};

use crate::support::read_option;

pub(super) struct AgentEvidenceBundleOptions {
    pub(super) root: PathBuf,
    pub(super) path: PathBuf,
}

pub(super) fn agent_evidence_bundle_options(
    args: &[String],
) -> Result<AgentEvidenceBundleOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let path = args
        .iter()
        .find(|arg| !arg.starts_with("--") && *arg != &root)
        .ok_or("agent-receipt verify-bundle requires <bundle.json>")?;
    Ok(AgentEvidenceBundleOptions {
        root,
        path: PathBuf::from(path),
    })
}

use std::{error::Error, path::PathBuf};

use crate::support::{read_option, read_string_option};

pub(super) struct ReleaseEvidencePublishOptions {
    pub(super) root: PathBuf,
    pub(super) index: PathBuf,
    pub(super) artifact_dir: PathBuf,
    pub(super) publish_dir: PathBuf,
    pub(super) channel: Option<String>,
    pub(super) release: Option<String>,
    pub(super) retention: Option<String>,
}

pub(super) fn release_evidence_publish_options(
    args: &[String],
) -> Result<ReleaseEvidencePublishOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let Some(index) = read_string_option(args, "--index").map(PathBuf::from) else {
        return Err("release-evidence publish requires --index <file>".into());
    };
    let Some(artifact_dir) = read_string_option(args, "--artifact-dir").map(PathBuf::from) else {
        return Err("release-evidence publish requires --artifact-dir <dir>".into());
    };
    let Some(publish_dir) = read_string_option(args, "--publish-dir").map(PathBuf::from) else {
        return Err("release-evidence publish requires --publish-dir <dir>".into());
    };
    Ok(ReleaseEvidencePublishOptions {
        root,
        index,
        artifact_dir,
        publish_dir,
        channel: read_string_option(args, "--channel"),
        release: read_string_option(args, "--release"),
        retention: read_string_option(args, "--retention"),
    })
}

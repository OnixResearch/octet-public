// r[impl cli_maintainability.evidence_manifest_cli_source.focused_modules]
// r[impl cli_maintainability.evidence_manifest_cli_source.focused_modules.visible_boundaries]
// r[impl cli_maintainability.evidence_manifest_cli_source.focused_modules.narrow_bridges]
// r[impl cli_maintainability.evidence_manifest_cli_source.behavior_preserved]
// r[verify cli_maintainability.evidence_manifest_cli_source.behavior_preserved.focused_checks]
// r[verify cli_maintainability.evidence_manifest_cli_source.behavior_preserved.repository_checks]
use std::error::Error;

mod artifact;
mod generate;
mod issues;
mod metadata;
mod roles;
mod verify;

pub(crate) use generate::{evidence_manifest_generate_manifest, EvidenceManifestGenerateOptions};
pub(crate) use verify::evidence_manifest_verification_receipt;

pub(crate) fn run_evidence_manifest(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence-manifest requires a subcommand: verify".into());
    };
    match subcommand.as_str() {
        "generate" => generate::run_evidence_manifest_generate(iter.collect()),
        "verify" => verify::run_evidence_manifest_verify(iter.collect()),
        unknown => Err(format!("unknown evidence-manifest subcommand: {unknown}").into()),
    }
}

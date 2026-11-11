use crate::support::{read_option, read_string_option};
use std::{error::Error, path::PathBuf};

pub(crate) struct EvidenceRepublishOptions {
    pub(crate) root: PathBuf,
    pub(crate) imported: Option<PathBuf>,
    pub(crate) out: Option<PathBuf>,
}

pub(super) fn evidence_republish_options(
    args: &[String],
) -> Result<EvidenceRepublishOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let imported = read_string_option(args, "--imported").map(PathBuf::from);
    let out = read_string_option(args, "--out").map(PathBuf::from);
    if imported.is_none() {
        return Err("evidence republish requires --imported <dir>".into());
    }
    if out.is_none() {
        return Err("evidence republish requires --out <dir>".into());
    }
    Ok(EvidenceRepublishOptions {
        root,
        imported,
        out,
    })
}

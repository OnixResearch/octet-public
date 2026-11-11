use crate::support::{read_option, read_string_option};
use std::{error::Error, path::PathBuf};

pub(crate) struct EvidenceIndexOptions {
    pub(crate) root: PathBuf,
    pub(crate) bundle: Option<PathBuf>,
    pub(crate) out: Option<PathBuf>,
}

#[derive(Default)]
pub(crate) struct EvidenceIndexVerifyOptions {
    pub(crate) root: PathBuf,
    pub(crate) index: Option<PathBuf>,
}

pub(super) fn evidence_index_options(
    args: &[String],
) -> Result<EvidenceIndexOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let bundle = read_string_option(args, "--bundle").map(PathBuf::from);
    let out = read_string_option(args, "--out").map(PathBuf::from);
    if bundle.is_none() {
        return Err("evidence index requires --bundle <dir>".into());
    }
    if out.is_none() {
        return Err("evidence index requires --out <file>".into());
    }
    Ok(EvidenceIndexOptions { root, bundle, out })
}

pub(super) fn evidence_index_verify_options(
    args: &[String],
) -> Result<EvidenceIndexVerifyOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let index = read_string_option(args, "--index").map(PathBuf::from);
    if index.is_none() {
        return Err("evidence index verify requires --index <file>".into());
    }
    Ok(EvidenceIndexVerifyOptions { root, index })
}

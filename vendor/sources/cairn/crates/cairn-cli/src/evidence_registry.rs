use std::{error::Error, path::PathBuf};

use serde_json::Value;

use crate::support::{print_json, read_option, read_string_option};

mod build_verify;
mod common;
mod consume;
mod fetch_execute;
mod labels;
mod mirror;

use build_verify::evidence_registry_receipt;
pub(crate) use build_verify::evidence_registry_verify_receipt;
use consume::{
    evidence_registry_consume_selectors, evidence_registry_fetch_plan_receipt,
    evidence_registry_select_receipt, EvidenceRegistryConsumeSelectors,
};
use fetch_execute::evidence_registry_fetch_execute_receipt;
use labels::{evidence_registry_labels_from_args, EvidenceRegistryLabels};
use mirror::evidence_registry_mirror_verify_receipt;

#[derive(Default)]
pub(super) struct EvidenceRegistryOptions {
    pub(super) root: PathBuf,
    pub(super) indexes: Vec<PathBuf>,
    pub(super) out: Option<PathBuf>,
    pub(super) labels: EvidenceRegistryLabels,
}

#[derive(Default)]
pub(crate) struct EvidenceRegistryVerifyOptions {
    pub(crate) root: PathBuf,
    pub(crate) registry: Option<PathBuf>,
}

#[derive(Default)]
pub(super) struct EvidenceRegistryMirrorVerifyOptions {
    pub(super) root: PathBuf,
    pub(super) registry: Option<PathBuf>,
    pub(super) mirror: Option<PathBuf>,
}

#[derive(Default)]
pub(super) struct EvidenceRegistrySelectOptions {
    pub(super) root: PathBuf,
    pub(super) registry: Option<PathBuf>,
    pub(super) out: Option<PathBuf>,
    pub(super) selectors: EvidenceRegistryConsumeSelectors,
}

#[derive(Default)]
pub(super) struct EvidenceRegistryFetchPlanOptions {
    pub(super) root: PathBuf,
    pub(super) registry: Option<PathBuf>,
    pub(super) out: Option<PathBuf>,
    pub(super) selectors: EvidenceRegistryConsumeSelectors,
}

#[derive(Default)]
pub(super) struct EvidenceRegistryFetchExecuteOptions {
    pub(super) root: PathBuf,
    pub(super) plan: Option<PathBuf>,
    pub(super) source: Option<PathBuf>,
    pub(super) out: Option<PathBuf>,
}

pub(crate) fn run_evidence_registry(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("evidence registry requires build or verify".into());
    };
    let receipt = match subcommand.as_str() {
        "build" => {
            let options = evidence_registry_options(&iter.collect::<Vec<_>>())?;
            evidence_registry_receipt(&options)
        }
        "verify" => {
            let options = evidence_registry_verify_options(&iter.collect::<Vec<_>>())?;
            evidence_registry_verify_receipt(&options)
        }
        "select" => {
            let options = evidence_registry_select_options(&iter.collect::<Vec<_>>())?;
            evidence_registry_select_receipt(&options)
        }
        "fetch-plan" => {
            let options = evidence_registry_fetch_plan_options(&iter.collect::<Vec<_>>())?;
            evidence_registry_fetch_plan_receipt(&options)
        }
        "fetch-execute" => {
            let options = evidence_registry_fetch_execute_options(&iter.collect::<Vec<_>>())?;
            evidence_registry_fetch_execute_receipt(&options)
        }
        "mirror-verify" => {
            let options = evidence_registry_mirror_verify_options(&iter.collect::<Vec<_>>())?;
            evidence_registry_mirror_verify_receipt(&options)
        }
        unknown => return Err(format!("unknown evidence registry subcommand: {unknown}").into()),
    };
    let valid = receipt
        .get("valid")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    print_json(receipt)?;
    if !valid {
        std::process::exit(1);
    }
    Ok(())
}

fn evidence_registry_options(args: &[String]) -> Result<EvidenceRegistryOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let indexes = read_repeated_path_options(args, "--index");
    let out = read_string_option(args, "--out").map(PathBuf::from);
    let labels = evidence_registry_labels_from_args(args);
    if indexes.is_empty() {
        return Err("evidence registry build requires at least one --index <file>".into());
    }
    if out.is_none() {
        return Err("evidence registry build requires --out <file>".into());
    }
    Ok(EvidenceRegistryOptions {
        root,
        indexes,
        out,
        labels,
    })
}

fn evidence_registry_verify_options(
    args: &[String],
) -> Result<EvidenceRegistryVerifyOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let registry = read_string_option(args, "--registry").map(PathBuf::from);
    if registry.is_none() {
        return Err("evidence registry verify requires --registry <file>".into());
    }
    Ok(EvidenceRegistryVerifyOptions { root, registry })
}

fn evidence_registry_mirror_verify_options(
    args: &[String],
) -> Result<EvidenceRegistryMirrorVerifyOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let registry = read_string_option(args, "--registry").map(PathBuf::from);
    let mirror = read_string_option(args, "--mirror").map(PathBuf::from);
    if registry.is_none() {
        return Err("evidence registry mirror-verify requires --registry <file>".into());
    }
    if mirror.is_none() {
        return Err("evidence registry mirror-verify requires --mirror <dir>".into());
    }
    Ok(EvidenceRegistryMirrorVerifyOptions {
        root,
        registry,
        mirror,
    })
}

fn evidence_registry_select_options(
    args: &[String],
) -> Result<EvidenceRegistrySelectOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let registry = read_string_option(args, "--registry").map(PathBuf::from);
    let out = read_string_option(args, "--out").map(PathBuf::from);
    if registry.is_none() {
        return Err("evidence registry select requires --registry <file>".into());
    }
    if out.is_none() {
        return Err("evidence registry select requires --out <file>".into());
    }
    Ok(EvidenceRegistrySelectOptions {
        root,
        registry,
        out,
        selectors: evidence_registry_consume_selectors(args),
    })
}

fn evidence_registry_fetch_plan_options(
    args: &[String],
) -> Result<EvidenceRegistryFetchPlanOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let registry = read_string_option(args, "--registry").map(PathBuf::from);
    let out = read_string_option(args, "--out").map(PathBuf::from);
    if registry.is_none() {
        return Err("evidence registry fetch-plan requires --registry <file>".into());
    }
    if out.is_none() {
        return Err("evidence registry fetch-plan requires --out <file>".into());
    }
    Ok(EvidenceRegistryFetchPlanOptions {
        root,
        registry,
        out,
        selectors: evidence_registry_consume_selectors(args),
    })
}

fn evidence_registry_fetch_execute_options(
    args: &[String],
) -> Result<EvidenceRegistryFetchExecuteOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let plan = read_string_option(args, "--plan").map(PathBuf::from);
    let source = read_string_option(args, "--source").map(PathBuf::from);
    let out = read_string_option(args, "--out").map(PathBuf::from);
    if plan.is_none() {
        return Err("evidence registry fetch-execute requires --plan <file>".into());
    }
    if source.is_none() {
        return Err("evidence registry fetch-execute requires --source <dir>".into());
    }
    if out.is_none() {
        return Err("evidence registry fetch-execute requires --out <dir>".into());
    }
    Ok(EvidenceRegistryFetchExecuteOptions {
        root,
        plan,
        source,
        out,
    })
}

fn read_repeated_path_options(args: &[String], flag: &str) -> Vec<PathBuf> {
    let mut values = Vec::new();
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        if arg == flag {
            if let Some(value) = iter.next() {
                values.push(PathBuf::from(value));
            }
        }
    }
    values
}

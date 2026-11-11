use std::{error::Error, path::PathBuf};

use crate::support::default_policy_path;

const ROOT_FLAG: &str = "--root";
const POLICY_FLAG: &str = "--policy";
const SCHEMA_FLAG: &str = "--schema";
const CHANGE_FLAG: &str = "--change";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum WorkflowRequestKind {
    Status { change: String },
    Instructions { change: String, artifact: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct WorkflowRequest {
    pub kind: WorkflowRequestKind,
    pub root: PathBuf,
    pub policy: PathBuf,
    pub schema: Option<String>,
}

pub(super) fn parse_workflow_request(args: &[String]) -> Result<WorkflowRequest, Box<dyn Error>> {
    let mut values = args.iter();
    let subcommand = values
        .next()
        .map(String::as_str)
        .ok_or("workflow requires status or instructions")?;
    match subcommand {
        "status" => parse_status_request(&mut values),
        "instructions" => parse_instructions_request(&mut values),
        unknown => Err(format!("unknown workflow subcommand: {unknown}").into()),
    }
}

fn parse_status_request<'a>(
    values: &mut impl Iterator<Item = &'a String>,
) -> Result<WorkflowRequest, Box<dyn Error>> {
    let change = required_positional(values, "workflow status requires a change name")?;
    let options = parse_options(values, false)?;
    Ok(WorkflowRequest {
        kind: WorkflowRequestKind::Status { change },
        root: options.root,
        policy: options.policy,
        schema: options.schema,
    })
}

fn parse_instructions_request<'a>(
    values: &mut impl Iterator<Item = &'a String>,
) -> Result<WorkflowRequest, Box<dyn Error>> {
    let artifact = required_positional(values, "workflow instructions requires an artifact id")?;
    let options = parse_options(values, true)?;
    let change = options
        .change
        .ok_or("workflow instructions requires --change CHANGE")?;
    Ok(WorkflowRequest {
        kind: WorkflowRequestKind::Instructions { change, artifact },
        root: options.root,
        policy: options.policy,
        schema: options.schema,
    })
}

fn required_positional<'a>(
    values: &mut impl Iterator<Item = &'a String>,
    message: &'static str,
) -> Result<String, Box<dyn Error>> {
    let value = values.next().ok_or(message)?;
    if value.starts_with('-') {
        return Err(message.into());
    }
    Ok(value.clone())
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ParsedOptions {
    root: PathBuf,
    policy: PathBuf,
    schema: Option<String>,
    change: Option<String>,
}

fn parse_options<'a>(
    values: &mut impl Iterator<Item = &'a String>,
    change_allowed: bool,
) -> Result<ParsedOptions, Box<dyn Error>> {
    let mut options = ParsedOptions {
        root: PathBuf::from("."),
        policy: PathBuf::from(default_policy_path()),
        schema: None,
        change: None,
    };
    while let Some(flag) = values.next() {
        let value = values
            .next()
            .ok_or_else(|| format!("workflow argument {flag} requires a value"))?;
        match flag.as_str() {
            ROOT_FLAG => set_once_path(&mut options.root, value, ROOT_FLAG, PathBuf::from("."))?,
            POLICY_FLAG => set_once_path(
                &mut options.policy,
                value,
                POLICY_FLAG,
                PathBuf::from(default_policy_path()),
            )?,
            SCHEMA_FLAG => set_once_string(&mut options.schema, value, SCHEMA_FLAG)?,
            CHANGE_FLAG if change_allowed => {
                set_once_string(&mut options.change, value, CHANGE_FLAG)?
            }
            unknown => return Err(format!("unknown workflow argument: {unknown}").into()),
        }
    }
    Ok(options)
}

fn set_once_path(
    target: &mut PathBuf,
    value: &str,
    flag: &str,
    default: PathBuf,
) -> Result<(), Box<dyn Error>> {
    if *target != default {
        return Err(format!("duplicate workflow argument: {flag}").into());
    }
    if value.is_empty() {
        return Err(format!("workflow argument {flag} must not be empty").into());
    }
    *target = PathBuf::from(value);
    Ok(())
}

fn set_once_string(
    target: &mut Option<String>,
    value: &str,
    flag: &str,
) -> Result<(), Box<dyn Error>> {
    if target.is_some() {
        return Err(format!("duplicate workflow argument: {flag}").into());
    }
    if value.is_empty() {
        return Err(format!("workflow argument {flag} must not be empty").into());
    }
    *target = Some(value.to_string());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_status_and_instructions_requests() {
        let status = parse_workflow_request(&[
            "status".to_string(),
            "demo".to_string(),
            "--schema".to_string(),
            "spec-driven".to_string(),
        ])
        .expect("status request");
        assert_eq!(status.schema.as_deref(), Some("spec-driven"));
        assert!(matches!(
            status.kind,
            WorkflowRequestKind::Status { ref change } if change == "demo"
        ));

        let instructions = parse_workflow_request(&[
            "instructions".to_string(),
            "design".to_string(),
            "--change".to_string(),
            "demo".to_string(),
        ])
        .expect("instructions request");
        assert!(matches!(
            instructions.kind,
            WorkflowRequestKind::Instructions {
                ref change,
                ref artifact,
            } if change == "demo" && artifact == "design"
        ));
    }

    #[test]
    fn rejects_unknown_duplicate_and_missing_arguments() {
        assert!(parse_workflow_request(&["status".to_string()]).is_err());
        assert!(parse_workflow_request(&[
            "status".to_string(),
            "demo".to_string(),
            "--execute".to_string(),
            "yes".to_string(),
        ])
        .is_err());
        assert!(
            parse_workflow_request(&["instructions".to_string(), "tasks".to_string(),]).is_err()
        );
        assert!(parse_workflow_request(&[
            "status".to_string(),
            "demo".to_string(),
            "--schema".to_string(),
            "first".to_string(),
            "--schema".to_string(),
            "second".to_string(),
        ])
        .is_err());
    }
}

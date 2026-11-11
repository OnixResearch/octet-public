use std::{error::Error, path::PathBuf};

use crate::support::default_policy_path;

const OPTION_WIDTH: usize = 2;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum AgentWorkflowCommand {
    List,
    Show,
    Plan,
    Check,
    Install,
}

impl AgentWorkflowCommand {
    pub(super) fn name(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Show => "show",
            Self::Plan => "plan",
            Self::Check => "check",
            Self::Install => "install",
        }
    }

    fn requires_destination(self) -> bool {
        matches!(self, Self::Plan | Self::Check | Self::Install)
    }
}

#[derive(Debug)]
pub(super) struct AgentWorkflowOptions {
    pub(super) command: AgentWorkflowCommand,
    pub(super) workflow: Option<String>,
    pub(super) root: Option<PathBuf>,
    pub(super) policy: PathBuf,
    pub(super) adapter: Option<String>,
    pub(super) destination: Option<String>,
    pub(super) execute: bool,
    pub(super) plan_hash: Option<String>,
}

pub(super) fn parse_agent_workflow_options(
    args: &[String],
) -> Result<AgentWorkflowOptions, Box<dyn Error>> {
    let Some(command) = args.first().map(String::as_str) else {
        return Err("agent workflow requires list, show, plan, check, or install".into());
    };
    let command = match command {
        "list" => AgentWorkflowCommand::List,
        "show" => AgentWorkflowCommand::Show,
        "plan" => AgentWorkflowCommand::Plan,
        "check" => AgentWorkflowCommand::Check,
        "install" => AgentWorkflowCommand::Install,
        _ => return Err(format!("unknown agent workflow command: {command}").into()),
    };
    let mut workflow = None;
    let mut root = None;
    let mut policy = None;
    let mut adapter = None;
    let mut destination = None;
    let mut execute = false;
    let mut plan_hash = None;
    let mut index = 1;
    if command == AgentWorkflowCommand::Show
        && args.get(index).is_some_and(|value| !value.starts_with('-'))
    {
        workflow = args.get(index).cloned();
        index += 1;
    }
    while index < args.len() {
        match args[index].as_str() {
            "--root" => set_path(&mut root, args, index, "--root")?,
            "--policy" => set_path(&mut policy, args, index, "--policy")?,
            "--adapter" => set_string(&mut adapter, args, index, "--adapter")?,
            "--destination" => set_string(&mut destination, args, index, "--destination")?,
            "--plan-hash" => set_string(&mut plan_hash, args, index, "--plan-hash")?,
            "--execute" if !execute => {
                execute = true;
                index += 1;
                continue;
            }
            "--execute" => return Err("--execute must not be repeated".into()),
            unknown => return Err(format!("unknown agent workflow option: {unknown}").into()),
        }
        index += OPTION_WIDTH;
    }
    validate_options(
        command,
        workflow.as_deref(),
        root.as_ref(),
        adapter.as_deref(),
        destination.as_deref(),
        execute,
        plan_hash.as_deref(),
    )?;
    Ok(AgentWorkflowOptions {
        command,
        workflow,
        root,
        policy: policy.unwrap_or_else(|| PathBuf::from(default_policy_path())),
        adapter,
        destination,
        execute,
        plan_hash,
    })
}

fn validate_options(
    command: AgentWorkflowCommand,
    workflow: Option<&str>,
    root: Option<&PathBuf>,
    adapter: Option<&str>,
    destination: Option<&str>,
    execute: bool,
    plan_hash: Option<&str>,
) -> Result<(), Box<dyn Error>> {
    if command == AgentWorkflowCommand::Show && workflow.is_none() {
        return Err("agent workflow show requires a workflow ID".into());
    }
    if command.requires_destination()
        && (root.is_none() || adapter.is_none() || destination.is_none())
    {
        return Err(format!(
            "agent workflow {} requires --root, --adapter, and --destination",
            command.name()
        )
        .into());
    }
    if matches!(
        command,
        AgentWorkflowCommand::List | AgentWorkflowCommand::Show
    ) && (root.is_some() || adapter.is_some() || destination.is_some())
    {
        return Err("list and show accept only --policy".into());
    }
    if execute && command != AgentWorkflowCommand::Install {
        return Err("--execute is available only for install".into());
    }
    if execute && plan_hash.is_none() {
        return Err("--execute requires --plan-hash from a current preview".into());
    }
    if !execute && plan_hash.is_some() {
        return Err("--plan-hash requires --execute".into());
    }
    Ok(())
}

fn set_string(
    target: &mut Option<String>,
    args: &[String],
    index: usize,
    name: &'static str,
) -> Result<(), Box<dyn Error>> {
    if target.is_some() {
        return Err(format!("{name} must not be repeated").into());
    }
    let value = args
        .get(index + 1)
        .filter(|value| !value.is_empty() && !value.starts_with("--"))
        .ok_or_else(|| format!("{name} requires a value"))?;
    *target = Some(value.clone());
    Ok(())
}

fn set_path(
    target: &mut Option<PathBuf>,
    args: &[String],
    index: usize,
    name: &'static str,
) -> Result<(), Box<dyn Error>> {
    if target.is_some() {
        return Err(format!("{name} must not be repeated").into());
    }
    let mut value = None;
    set_string(&mut value, args, index, name)?;
    *target = value.map(PathBuf::from);
    Ok(())
}

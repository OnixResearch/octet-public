use std::error::Error;

// r[impl cairn.developer_experience.cli_modules.focused_modules]
// r[impl cairn.developer_experience.cli_modules.focused_modules.review]
// r[impl cairn.developer_experience.cli_modules.facade_boundary]
// r[impl cairn.developer_experience.cli_modules.facade_boundary.dispatch]
// r[impl cairn.developer_experience.cli_modules.behavior_preservation]
// r[impl cairn.developer_experience.cli_modules.archived_design]
// r[impl cairn.developer_experience.cli_modules.archived_design.review]
mod common;
mod discovery;
mod scaffold;
mod template;

pub(crate) fn run_template(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("template requires a subcommand: list or apply".into());
    };
    let rest = iter.collect::<Vec<_>>();
    match subcommand.as_str() {
        "list" => template::run_list(rest),
        "apply" => {
            let Some(template) = rest.first() else {
                return Err("template apply requires a template id".into());
            };
            template::run_apply(rest[1..].to_vec(), template)
        }
        unknown => Err(format!("unknown template subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_scaffold(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let mut iter = args.into_iter();
    let Some(subcommand) = iter.next() else {
        return Err("scaffold requires a subcommand: change".into());
    };
    match subcommand.as_str() {
        "change" => {
            let Some(change) = iter.next() else {
                return Err("scaffold change requires a change name".into());
            };
            scaffold::run_change(iter.collect(), &change)
        }
        unknown => Err(format!("unknown scaffold subcommand: {unknown}").into()),
    }
}

pub(crate) fn run_discover_specs(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    discovery::run(args)
}

use super::support::*;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum ReplayOutputFormat {
    Json,
    Jsonl,
    Summary,
}

pub(super) struct ReplayOptions {
    pub(super) root: PathBuf,
    pub(super) selected_case: Option<String>,
    pub(super) selected_group: Option<String>,
    pub(super) format: ReplayOutputFormat,
    pub(super) out: Option<PathBuf>,
}

pub(super) fn replay_options(args: &[String]) -> Result<ReplayOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", "fixtures/basic");
    let selected_case = read_string_option(args, "--case");
    let selected_group = read_string_option(args, "--group");
    let format = match read_string_option(args, "--format")
        .as_deref()
        .unwrap_or("json")
    {
        "json" => ReplayOutputFormat::Json,
        "jsonl" => ReplayOutputFormat::Jsonl,
        "summary" => ReplayOutputFormat::Summary,
        other => return Err(format!("unknown replay format: {other}").into()),
    };
    Ok(ReplayOptions {
        root,
        selected_case,
        selected_group,
        format,
        out: read_string_option(args, "--out").map(PathBuf::from),
    })
}

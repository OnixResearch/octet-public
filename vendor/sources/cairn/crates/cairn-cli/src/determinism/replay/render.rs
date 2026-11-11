use super::options::ReplayOutputFormat;
use super::support::*;

pub(super) fn render_replay_receipt(
    receipt: &Value,
    format: ReplayOutputFormat,
    out: Option<&Path>,
) -> Result<(), Box<dyn Error>> {
    // r[impl cairn.supersede.determinism_replay.ci_output_modes]
    if let Some(path) = out {
        let parent = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        let file_name = path.file_name().ok_or("replay output must name a file")?;
        let output_root = CapabilityRoot::open_or_create(CapabilityRootKind::Replay, parent)?;
        output_root.write(
            Path::new(file_name),
            format!(
                "{}
",
                serde_json::to_string_pretty(receipt)?
            )
            .as_bytes(),
        )?;
    }
    match format {
        ReplayOutputFormat::Json => print_json(receipt.clone()),
        ReplayOutputFormat::Jsonl => print_replay_jsonl(receipt, out),
        ReplayOutputFormat::Summary => print_replay_summary(receipt, out),
    }
}

fn print_replay_jsonl(receipt: &Value, out: Option<&Path>) -> Result<(), Box<dyn Error>> {
    println!(
        "{}",
        serde_json::to_string(&json!({
            "event": "summary",
            "valid": receipt["valid"],
            "verdict": receipt["verdict"],
            "summary": receipt["summary"],
            "receipt_hash": receipt["receipt_hash"],
        }))?
    );
    if let Some(path) = out {
        println!(
            "{}",
            serde_json::to_string(&json!({
                "event": "artifact",
                "path": path_string(path),
                "receipt_hash": receipt["receipt_hash"],
            }))?
        );
    }
    for case in receipt["selected_cases"].as_array().into_iter().flatten() {
        println!(
            "{}",
            serde_json::to_string(&json!({"event": "case", "id": case}))?
        );
    }
    for mismatch in receipt["mismatches"].as_array().into_iter().flatten() {
        println!(
            "{}",
            serde_json::to_string(&json!({"event": "mismatch", "mismatch": mismatch}))?
        );
    }
    for failure in receipt["nondeterminism_failures"]
        .as_array()
        .into_iter()
        .flatten()
    {
        println!(
            "{}",
            serde_json::to_string(&json!({"event": "nondeterminism_failure", "failure": failure}))?
        );
    }
    Ok(())
}

fn print_replay_summary(receipt: &Value, out: Option<&Path>) -> Result<(), Box<dyn Error>> {
    let summary = &receipt["summary"];
    println!(
        "determinism replay: verdict={} total={} failed={} mismatches={} entropy_failures={} receipt_hash={}",
        receipt["verdict"].as_str().unwrap_or("unknown"),
        summary["total_cases"].as_u64().unwrap_or(0),
        summary["failed_cases"].as_array().map(|items| items.len()).unwrap_or(0),
        summary["mismatch_count"].as_u64().unwrap_or(0),
        summary["nondeterminism_failure_count"].as_u64().unwrap_or(0),
        receipt["receipt_hash"].as_str().unwrap_or(""),
    );
    if let Some(path) = out {
        println!("artifact={}", path_string(path));
    }
    for case in summary["failed_cases"].as_array().into_iter().flatten() {
        println!("failed_case={}", case.as_str().unwrap_or(""));
    }
    Ok(())
}

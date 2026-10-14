use std::fs;
use std::path::Path;

use serde::Serialize;

use crate::cli::OutputFormat;
use crate::cli::TranscriptArgs;
use crate::determinism;
use crate::evidence;

#[derive(Debug, Serialize)]
struct TranscriptReport {
    schema_version: u32,
    path: String,
    transcript_hash: String,
    command_blocks: usize,
    expected_output_blocks: usize,
    status: &'static str,
    failures: Vec<String>,
    provenance: evidence::ProvenanceRecord,
}

pub fn run(args: &TranscriptArgs) -> Result<i32, String> {
    let report = verify(&args.path)?;
    match args.output_format {
        OutputFormat::Human => {
            println!(
                "octet transcript {} command_blocks={} expected_output_blocks={} status={}",
                report.path, report.command_blocks, report.expected_output_blocks, report.status,
            );
            for failure in &report.failures {
                println!("failure: {failure}");
            }
        }
        OutputFormat::Json => println!(
            "{}",
            serde_json::to_string_pretty(&report).map_err(|error| format!("serialize transcript report: {error}"))?
        ),
        OutputFormat::Sarif => return Err("transcript does not support SARIF output; use human or json".to_owned()),
    }
    Ok(i32::from(!report.failures.is_empty()))
}

fn verify(path: &Path) -> Result<TranscriptReport, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    let mut command_blocks = 0;
    let mut expected_output_blocks = 0;
    let mut failures = Vec::new();
    let mut in_fence = false;
    let mut fence_lang = String::new();
    let mut fence_start = 0usize;
    let mut saw_command_in_current_fence = false;

    for (idx, line) in text.lines().enumerate() {
        let line_no = idx + 1;
        if let Some(lang) = line.strip_prefix("```") {
            if in_fence {
                if saw_command_in_current_fence
                    && fence_lang != "octet-transcript"
                    && fence_lang != "slotcar-transcript"
                    && fence_lang != "console"
                    && fence_lang != "sh"
                {
                    failures
                        .push(format!("line {fence_start}: command fence must use octet-transcript, slotcar-transcript, console, or sh"));
                }
                in_fence = false;
                fence_lang.clear();
                saw_command_in_current_fence = false;
            } else {
                in_fence = true;
                fence_lang.clear();
                fence_lang.push_str(lang.trim());
                fence_start = line_no;
            }
            continue;
        }
        if !in_fence {
            continue;
        }
        if line.trim_start().starts_with("$ ") {
            command_blocks += 1;
            saw_command_in_current_fence = true;
        }
        if line.trim_start().starts_with("=> ") {
            expected_output_blocks += 1;
        }
    }

    if in_fence {
        failures.push(format!("line {fence_start}: unclosed fenced transcript block"));
    }
    if command_blocks == 0 {
        failures.push("transcript contains no `$ ` command lines".to_owned());
    }
    if expected_output_blocks == 0 {
        failures.push("transcript contains no `=> ` expected-output lines".to_owned());
    }

    let transcript_hash = determinism::full_hash(&text);
    let path_display = path.display().to_string();
    Ok(TranscriptReport {
        schema_version: 1,
        path: path_display.clone(),
        transcript_hash,
        command_blocks,
        expected_output_blocks,
        status: if failures.is_empty() { "passed" } else { "failed" },
        failures,
        provenance: transcript_provenance_record(&path_display, text.as_bytes()),
    })
}

fn transcript_provenance_record(path: &str, contents: &[u8]) -> evidence::ProvenanceRecord {
    evidence::ProvenanceRecord {
        schema_version: evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: "octet.transcript.assertion".to_owned(),
        checker_id: evidence::CHECKER_OCTET_TRANSCRIPT.to_owned(),
        evidence_tier: evidence::EvidenceTier::TranscriptAssertion,
        artifact_locator: path.to_owned(),
        blake3: evidence::artifact_blake3_hex(contents),
        checked_scope: format!("transcript:{path}"),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: None,
        parent_artifacts: Vec::new(),
    }
}

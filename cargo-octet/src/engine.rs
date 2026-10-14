use std::env;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::IsTerminal;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::process::Stdio;

use crate::baseline;
use crate::baseline::BaselineEvaluation;
use crate::cap_fs::ArtifactRoot;
use crate::cap_fs::CacheRoot;
use crate::cap_fs::RootRelativePath;
use crate::cli::CheckArgs;
use crate::cli::OutputFormat;
use crate::config;
use crate::determinism;
use crate::determinism::RunMetadata;
use crate::output;
use crate::output::Finding;

mod embedded_source {
    include!(concat!(env!("OUT_DIR"), "/embedded_source.rs"));
}

const SOURCE_HASH_MARKER: &str = ".tigerstyle-source-blake3";
const CARGO_COMMAND: &str = "cargo";
const CARGO_CHECK_SUBCOMMAND: &str = "check";
const CARGO_MESSAGE_FORMAT_JSON: &str = "--message-format=json";
const COLOR_ALWAYS_FLAG: &str = "--color=always";
const COMMAND_ARTIFACT_NAME: &str = "command.txt";
const STATUS_ARTIFACT_NAME: &str = "status.json";
const SUMMARY_ARTIFACT_NAME: &str = "summary.txt";
const JSON_RESULTS_ARTIFACT_NAME: &str = "results.jsonl";
const SARIF_RESULTS_ARTIFACT_NAME: &str = "results.sarif";
const PROVENANCE_ARTIFACT_NAME: &str = "provenance.jsonl";
const RUNNER_PROVENANCE_SOURCE_ID: &str = "octet.runner.artifact-bundle";
const RUNNER_STATIC_FINDINGS_SOURCE_ID: &str = "octet.runner.static-findings";
const FUNCTION_ADDRESS_PROVENANCE_SOURCE_ID: &str = "octet.function_address.receipt";
const RUNNER_CHECKER_ID: &str = "cargo-octet";
const RUNNER_MANUAL_REVIEW_ACTION: &str =
    "review runner status, command, and summary before using this artifact as release evidence";
const TERMINATED_WITHOUT_EXIT_CODE_ERROR: &str = "cargo check terminated without an exit code";
const LINT_CACHE_DIR: &str = ".octet/cache";
const LINT_CACHE_FILE: &str = "findings.json";

#[derive(Debug, Clone)]
struct StructuredArtifact {
    file_name: &'static str,
    contents: String,
}

#[derive(Debug, Clone)]
struct RunArtifacts {
    summary: String,
    status_json: String,
    provenance_jsonl: String,
    structured_output: Option<StructuredArtifact>,
    additional_artifacts: Vec<StructuredArtifact>,
}

#[derive(Debug, Clone)]
struct FinalizedRun {
    outcome: output::RunOutcome,
    run_artifacts: RunArtifacts,
    error: Option<String>,
}

fn apply_baseline_gate(
    outcome: output::RunOutcome,
    baseline_evaluation: Option<&BaselineEvaluation>,
) -> output::RunOutcome {
    if outcome.cargo_process_exit.is_success() && baseline_evaluation.is_some_and(BaselineEvaluation::has_new_findings)
    {
        return outcome.force_lint_failure();
    }

    outcome
}

#[allow(clippy::too_many_lines)]
pub fn run(check: &CheckArgs) -> Result<i32, String> {
    let cwd = env::current_dir().map_err(|error| format!("current_dir: {error}"))?;
    let cfg = config::load(&cwd)?;
    let scope_args = if check.scope_args.is_empty() {
        cfg.default_scope.clone()
    } else {
        check.scope_args.clone()
    };
    let cargo_check_args = check.cargo_check_args.clone().unwrap_or(cfg.cargo_check_args.clone());
    let selected_profile =
        crate::strict_profile::prepare_selected_profile(check.profile.as_deref(), &cfg.workspace_root)?;
    let effective_function_addresses =
        crate::strict_profile::effective_function_address_check(selected_profile.as_ref(), &check.function_addresses);

    if cargo_check_args.iter().any(|arg| arg.starts_with("--message-format")) {
        return Err("`--message-format` is managed by `--output-format`; remove it from passthrough args".to_owned());
    }

    let lockfile_path = cfg.workspace_root.join("Cargo.lock");
    let skip_lockfile_guard =
        env::var("TIGERSTYLE_SKIP_LOCKFILE_CHECK").is_ok_and(|value| !value.is_empty() && value != "0");
    let lockfile_before = if skip_lockfile_guard {
        LockfileSnapshot::Missing
    } else {
        snapshot_lockfile(&lockfile_path)?
    };

    let result = (|| {
        let toolchain = resolve_toolchain()?;
        let strict_profile_identity = selected_profile.as_ref().map(|selected| determinism::StrictProfileIdentity {
            profile_id: &selected.policy.profile_id,
            policy_hash: &selected.policy_hash,
        });
        let run_metadata = determinism::build_run_metadata(&determinism::MetadataInput {
            workspace_root: &cfg.workspace_root,
            toolchain: &toolchain,
            scope_args: &scope_args,
            cargo_check_args: &cargo_check_args,
            output_format: check.output_format.as_str(),
            strict_profile: strict_profile_identity,
        });
        let driver = resolve_driver(&toolchain)?;
        let lint_lib = resolve_lint_lib(&toolchain)?;
        let dylint_libs = serde_json::to_string(&vec![lint_lib.path().to_owned()])
            .map_err(|error| format!("encode DYLINT_LIBS: {error}"))?;

        let mut cmd = Command::new(CARGO_COMMAND);
        cmd.arg(CARGO_CHECK_SUBCOMMAND).arg(CARGO_MESSAGE_FORMAT_JSON);

        if check.output_format == OutputFormat::Human
            && std::io::stderr().is_terminal()
            && !cargo_check_args.iter().any(|argument| argument.starts_with("--color"))
        {
            cmd.arg(COLOR_ALWAYS_FLAG);
        }

        cmd.args(&scope_args)
            .args(&cargo_check_args)
            .current_dir(&cfg.workspace_root)
            .env("RUSTC_WORKSPACE_WRAPPER", driver.path())
            .env("DYLINT_LIBS", &dylint_libs)
            .env("DYLINT_NO_DEPS", "1")
            .env("RUSTUP_TOOLCHAIN", &toolchain)
            .stdout(Stdio::piped());
        cmd.env_remove(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV);
        if let Some(selected) = &selected_profile {
            cmd.env(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV, &selected.policy.profile_id);
        }

        if check.output_format != OutputFormat::Human {
            cmd.stderr(Stdio::null());
        }

        let mut child = cmd.spawn().map_err(|error| format!("run cargo check: {error}"))?;
        let child_stdout = child.stdout.take().expect("stdout piped");

        let meta_map = output::lint_meta_map();
        let mut findings: Vec<Finding> = Vec::new();
        let mut finding_counter: usize = 0;
        let mut stdout_lock = std::io::stdout().lock();
        let mut stderr_lock = std::io::stderr().lock();

        for line in BufReader::new(child_stdout).lines() {
            let line = line.map_err(|error| format!("read cargo stdout: {error}"))?;
            let json: serde_json::Value =
                serde_json::from_str(&line).map_err(|error| format!("parse cargo JSON: {error}"))?;

            let reason = json["reason"].as_str().unwrap_or("");

            match reason {
                "compiler-message" => {
                    let message = &json["message"];

                    if check.output_format == OutputFormat::Human
                        && let Some(rendered) = message["rendered"].as_str()
                    {
                        let _ = write!(stderr_lock, "{rendered}");
                    }

                    let code = message["code"]["code"].as_str().unwrap_or("");
                    let lint_name = normalize_lint_name(code);

                    let meta = meta_map.get(lint_name.as_str()).unwrap_or(&output::UNKNOWN_LINT_META);
                    finding_counter += 1;
                    findings.push(build_finding(
                        finding_counter,
                        &json,
                        meta,
                        &lint_name,
                        &run_metadata.config_hash,
                        check.cache,
                    ));
                }
                "compiler-artifact" if check.output_format == OutputFormat::Human => {
                    if let Some(banner) = format_compiling_banner(&json) {
                        let _ = writeln!(stderr_lock, "{banner}");
                    }
                }
                "build-finished" if check.output_format == OutputFormat::Human => {
                    let success = json["success"].as_bool().unwrap_or(false);
                    if success {
                        let _ = writeln!(stderr_lock, "    Finished checking");
                    }
                }
                _ => {}
            }
        }

        let cargo_process_exit =
            classify_cargo_process_exit(child.wait().map_err(|error| format!("wait for cargo check: {error}"))?);
        let function_address_artifact = if cargo_process_exit.is_success() {
            match crate::function_address::collect_run(
                &effective_function_addresses,
                &cfg.workspace_root,
                &run_metadata.config_hash,
            )? {
                Some(run) => {
                    findings.extend(run.findings);
                    Some(StructuredArtifact {
                        file_name: crate::function_address::FUNCTION_ADDRESS_ARTIFACT_NAME,
                        contents: run.artifact_json,
                    })
                }
                None => None,
            }
        } else {
            None
        };
        canonicalize_findings(&mut findings);
        if check.cache {
            update_lint_cache(&cfg.workspace_root, &mut findings)?;
        }
        let cargo_process_succeeded = cargo_process_exit.is_success();
        let base_outcome = output::RunOutcome::classify(&findings, cargo_process_exit);
        let baseline_evaluation = match check.baseline.as_deref() {
            Some(path) => Some(baseline::evaluate_baseline(path, &findings)?),
            None => None,
        };
        if let Some(path) = check.write_baseline.as_deref()
            && cargo_process_succeeded
        {
            baseline::write_baseline(path, &findings, &run_metadata)?;
        }
        let base_outcome = apply_baseline_gate(base_outcome, baseline_evaluation.as_ref());
        let finalized_run = finalize_run(
            check,
            &findings,
            &meta_map,
            base_outcome,
            &run_metadata,
            baseline_evaluation.as_ref(),
            function_address_artifact,
        )?;

        if let Some(structured_output) = finalized_run.run_artifacts.structured_output.as_ref() {
            stdout_lock
                .write_all(structured_output.contents.as_bytes())
                .map_err(|error| format!("write stdout: {error}"))?;
        }

        stderr_lock
            .write_all(finalized_run.run_artifacts.summary.as_bytes())
            .map_err(|error| format!("write summary: {error}"))?;

        match finalized_run.error {
            Some(error) => Err(error),
            None => Ok(finalized_run.outcome.exit_code),
        }
    })();

    if !skip_lockfile_guard && lockfile_changed(&lockfile_path, &lockfile_before)? {
        restore_lockfile(&lockfile_path, &lockfile_before)?;
        let lockfile_error =
            format!("cargo octet check modified {}; restored the original lockfile", lockfile_path.display());
        return Err(match result {
            Err(error) => format!("{error}; {lockfile_error}"),
            Ok(_) => lockfile_error,
        });
    }

    result
}

fn build_run_artifacts(
    check: &CheckArgs,
    findings: &[Finding],
    meta_map: &std::collections::HashMap<&str, output::LintMeta>,
    outcome: &output::RunOutcome,
    metadata: &RunMetadata,
    baseline_evaluation: Option<&BaselineEvaluation>,
    function_address_artifact: Option<StructuredArtifact>,
) -> Result<RunArtifacts, String> {
    let summary = output::render_summary(findings, outcome, metadata, baseline_evaluation)?;
    let status_json = output::render_status_json(outcome, check.output_format.as_str(), metadata, baseline_evaluation)?;
    let structured_output =
        render_structured_output(check, findings, meta_map, outcome, metadata, baseline_evaluation)?;
    let additional_artifacts = function_address_artifact.into_iter().collect::<Vec<_>>();
    let provenance_jsonl = render_run_provenance_jsonl(
        &check.command_line,
        &summary,
        &status_json,
        structured_output.as_ref(),
        &additional_artifacts,
        metadata,
    )?;

    Ok(RunArtifacts {
        summary,
        status_json,
        provenance_jsonl,
        structured_output,
        additional_artifacts,
    })
}

fn finalize_run(
    check: &CheckArgs,
    findings: &[Finding],
    meta_map: &std::collections::HashMap<&str, output::LintMeta>,
    base_outcome: output::RunOutcome,
    metadata: &RunMetadata,
    baseline_evaluation: Option<&BaselineEvaluation>,
    function_address_artifact: Option<StructuredArtifact>,
) -> Result<FinalizedRun, String> {
    let terminated_error = terminated_run_error(&base_outcome);

    let Some(artifact_dir) = check.artifact_dir.as_deref() else {
        return Ok(FinalizedRun {
            outcome: base_outcome.clone(),
            run_artifacts: build_run_artifacts(
                check,
                findings,
                meta_map,
                &base_outcome,
                metadata,
                baseline_evaluation,
                function_address_artifact,
            )?,
            error: terminated_error,
        });
    };

    let artifact_root = match prepare_artifact_dir(artifact_dir) {
        Ok(root) => root,
        Err(error) => {
            let final_outcome = base_outcome.force_integration_failure();
            return Ok(FinalizedRun {
                outcome: final_outcome.clone(),
                run_artifacts: build_run_artifacts(
                    check,
                    findings,
                    meta_map,
                    &final_outcome,
                    metadata,
                    baseline_evaluation,
                    function_address_artifact,
                )?,
                error: Some(combine_errors(error, terminated_error)),
            });
        }
    };

    let initial_artifacts = build_run_artifacts(
        check,
        findings,
        meta_map,
        &base_outcome,
        metadata,
        baseline_evaluation,
        function_address_artifact.clone(),
    )?;
    match write_run_artifacts(&artifact_root, &check.command_line, &initial_artifacts) {
        Ok(()) => Ok(FinalizedRun {
            outcome: base_outcome,
            run_artifacts: initial_artifacts,
            error: terminated_error,
        }),
        Err(error) => {
            let final_outcome = base_outcome.force_integration_failure();
            let final_artifacts = build_run_artifacts(
                check,
                findings,
                meta_map,
                &final_outcome,
                metadata,
                baseline_evaluation,
                function_address_artifact,
            )?;
            invalidate_failed_bundle_best_effort(&artifact_root, &final_artifacts);
            rewrite_status_sidecars_best_effort(&artifact_root, &check.command_line, &final_artifacts);
            Ok(FinalizedRun {
                outcome: final_outcome,
                run_artifacts: final_artifacts,
                error: Some(combine_errors(error, terminated_error)),
            })
        }
    }
}

fn terminated_run_error(outcome: &output::RunOutcome) -> Option<String> {
    if outcome.cargo_process_exit.is_terminated() {
        Some(TERMINATED_WITHOUT_EXIT_CODE_ERROR.to_owned())
    } else {
        None
    }
}

fn combine_errors(primary: String, secondary: Option<String>) -> String {
    match secondary {
        Some(secondary) => format!("{primary}; {secondary}"),
        None => primary,
    }
}

fn render_structured_output(
    check: &CheckArgs,
    findings: &[Finding],
    meta_map: &std::collections::HashMap<&str, output::LintMeta>,
    outcome: &output::RunOutcome,
    metadata: &RunMetadata,
    baseline_evaluation: Option<&BaselineEvaluation>,
) -> Result<Option<StructuredArtifact>, String> {
    match check.output_format {
        OutputFormat::Human => Ok(None),
        OutputFormat::Json => Ok(Some(StructuredArtifact {
            file_name: JSON_RESULTS_ARTIFACT_NAME,
            contents: output::render_json_lines(findings)?,
        })),
        OutputFormat::Sarif => Ok(Some(StructuredArtifact {
            file_name: SARIF_RESULTS_ARTIFACT_NAME,
            contents: output::render_sarif(
                findings,
                meta_map,
                outcome,
                &check.command_line,
                metadata,
                baseline_evaluation,
            )?,
        })),
    }
}

fn render_run_provenance_jsonl(
    command_line: &str,
    summary: &str,
    status_json: &str,
    structured_output: Option<&StructuredArtifact>,
    additional_artifacts: &[StructuredArtifact],
    metadata: &RunMetadata,
) -> Result<String, String> {
    let command_contents = format!("{command_line}\n");
    let checked_scope = format!("profile:{}", metadata.profile_name);
    let mut records = vec![
        artifact_provenance_record(
            COMMAND_ARTIFACT_NAME,
            &command_contents,
            crate::evidence::EvidenceTier::ManualReviewRequired,
            RUNNER_PROVENANCE_SOURCE_ID,
            &checked_scope,
        ),
        artifact_provenance_record(
            STATUS_ARTIFACT_NAME,
            status_json,
            crate::evidence::EvidenceTier::ManualReviewRequired,
            RUNNER_PROVENANCE_SOURCE_ID,
            &checked_scope,
        ),
        artifact_provenance_record(
            SUMMARY_ARTIFACT_NAME,
            summary,
            crate::evidence::EvidenceTier::ManualReviewRequired,
            RUNNER_PROVENANCE_SOURCE_ID,
            &checked_scope,
        ),
    ];

    if let Some(structured_output) = structured_output {
        records.push(artifact_provenance_record(
            structured_output.file_name,
            &structured_output.contents,
            crate::evidence::EvidenceTier::StaticLint,
            RUNNER_STATIC_FINDINGS_SOURCE_ID,
            &checked_scope,
        ));
    }

    for artifact in additional_artifacts {
        records.push(artifact_provenance_record(
            artifact.file_name,
            &artifact.contents,
            crate::evidence::EvidenceTier::StaticLint,
            FUNCTION_ADDRESS_PROVENANCE_SOURCE_ID,
            &checked_scope,
        ));
    }

    let mut rendered = String::new();
    for record in records {
        let line = serde_json::to_string(&record).map_err(|error| format!("write provenance JSON: {error}"))?;
        rendered.push_str(&line);
        rendered.push('\n');
    }
    Ok(rendered)
}

fn artifact_provenance_record(
    file_name: &str,
    contents: &str,
    evidence_tier: crate::evidence::EvidenceTier,
    source_id: &str,
    checked_scope: &str,
) -> crate::evidence::ProvenanceRecord {
    crate::evidence::ProvenanceRecord {
        schema_version: crate::evidence::PROVENANCE_SCHEMA_VERSION.to_owned(),
        source_id: source_id.to_owned(),
        checker_id: RUNNER_CHECKER_ID.to_owned(),
        evidence_tier,
        artifact_locator: file_name.to_owned(),
        blake3: crate::evidence::artifact_blake3_hex(contents.as_bytes()),
        checked_scope: checked_scope.to_owned(),
        verifier_identity: None,
        proof_artifact_identity: None,
        human_action: (evidence_tier == crate::evidence::EvidenceTier::ManualReviewRequired)
            .then(|| RUNNER_MANUAL_REVIEW_ACTION.to_owned()),
        parent_artifacts: Vec::new(),
    }
}

fn write_run_artifacts(
    artifact_root: &ArtifactRoot,
    command_line: &str,
    run_artifacts: &RunArtifacts,
) -> Result<(), String> {
    let expected_files = expected_artifact_files(run_artifacts);
    remove_extra_artifact_paths(artifact_root, &expected_files)?;
    write_status_sidecars(artifact_root, command_line, run_artifacts)?;

    if let Some(structured_output) = run_artifacts.structured_output.as_ref() {
        let structured_path = RootRelativePath::new(structured_output.file_name)?;
        artifact_root.write(&structured_path, structured_output.contents.as_bytes())?;
    }

    for artifact in &run_artifacts.additional_artifacts {
        let path = RootRelativePath::new(artifact.file_name)?;
        artifact_root.write(&path, artifact.contents.as_bytes())?;
    }

    Ok(())
}

fn prepare_artifact_dir(artifact_dir: &Path) -> Result<ArtifactRoot, String> {
    ArtifactRoot::open_or_create(artifact_dir)
}

fn write_status_sidecars(
    artifact_root: &ArtifactRoot,
    command_line: &str,
    run_artifacts: &RunArtifacts,
) -> Result<(), String> {
    artifact_root.write(&RootRelativePath::new(COMMAND_ARTIFACT_NAME)?, format!("{command_line}\n").as_bytes())?;
    artifact_root.write(&RootRelativePath::new(STATUS_ARTIFACT_NAME)?, run_artifacts.status_json.as_bytes())?;
    artifact_root.write(&RootRelativePath::new(SUMMARY_ARTIFACT_NAME)?, run_artifacts.summary.as_bytes())?;
    artifact_root
        .write(&RootRelativePath::new(PROVENANCE_ARTIFACT_NAME)?, run_artifacts.provenance_jsonl.as_bytes())?;
    Ok(())
}

fn rewrite_status_sidecars_best_effort(artifact_root: &ArtifactRoot, command_line: &str, run_artifacts: &RunArtifacts) {
    let _ = write_status_sidecars(artifact_root, command_line, run_artifacts);
}

fn invalidate_failed_bundle_best_effort(artifact_root: &ArtifactRoot, run_artifacts: &RunArtifacts) {
    let Some(structured_output) = run_artifacts.structured_output.as_ref() else {
        return;
    };
    let Ok(relative) = RootRelativePath::new(structured_output.file_name) else {
        return;
    };

    let _ = artifact_root.remove_file_or_dir_all(&relative);
}

fn expected_artifact_files(run_artifacts: &RunArtifacts) -> Vec<&'static str> {
    let mut files = vec![
        COMMAND_ARTIFACT_NAME,
        PROVENANCE_ARTIFACT_NAME,
        STATUS_ARTIFACT_NAME,
        SUMMARY_ARTIFACT_NAME,
    ];
    if let Some(structured_artifact) = run_artifacts.structured_output.as_ref() {
        files.push(structured_artifact.file_name);
    }
    files.extend(run_artifacts.additional_artifacts.iter().map(|artifact| artifact.file_name));
    files
}

fn remove_extra_artifact_paths(artifact_root: &ArtifactRoot, expected_files: &[&'static str]) -> Result<(), String> {
    for file_name in artifact_root.read_entry_names()? {
        if expected_files.iter().any(|expected| *expected == file_name) {
            continue;
        }

        let relative = RootRelativePath::new(file_name.as_str())?;
        artifact_root.remove_file_or_dir_all(&relative)?;
    }

    Ok(())
}

fn classify_cargo_process_exit(status: std::process::ExitStatus) -> output::CargoProcessExit {
    match status.code() {
        Some(code) => output::CargoProcessExit::exit_code(code),
        None => output::CargoProcessExit::terminated(),
    }
}

fn update_lint_cache(workspace_root: &Path, findings: &mut [Finding]) -> Result<(), String> {
    let cache_root = CacheRoot::open_or_create(&workspace_root.join(LINT_CACHE_DIR))?;
    let cache_locator = RootRelativePath::new(LINT_CACHE_FILE)?;
    let mut cached: std::collections::BTreeSet<String> = match cache_root.read_to_string_optional(&cache_locator)? {
        Some(contents) => {
            serde_json::from_str(&contents).map_err(|error| format!("parse cache path `{LINT_CACHE_FILE}`: {error}"))?
        }
        None => std::collections::BTreeSet::new(),
    };

    let mut changed = false;
    for finding in findings {
        if finding.identity_status != "computed" {
            finding.cache_status.clear();
            finding.cache_status.push_str("bypassed");
            continue;
        }
        if cached.contains(&finding.semantic_id) {
            finding.cache_status.clear();
            finding.cache_status.push_str("hit");
        } else {
            cached.insert(finding.semantic_id.clone());
            finding.cache_status.clear();
            finding.cache_status.push_str("miss");
            changed = true;
        }
    }

    if changed {
        let mut rendered =
            serde_json::to_string_pretty(&cached).map_err(|error| format!("serialize lint cache: {error}"))?;
        rendered.push('\n');
        cache_root.write(&cache_locator, rendered.as_bytes())?;
    }

    Ok(())
}

fn canonicalize_findings(findings: &mut [Finding]) {
    findings.sort_by(|left, right| {
        left.file
            .cmp(&right.file)
            .then(left.crate_name.cmp(&right.crate_name))
            .then(left.line.cmp(&right.line))
            .then(left.column.cmp(&right.column))
            .then(left.lint.cmp(&right.lint))
            .then(left.stable_id.cmp(&right.stable_id))
    });

    for (offset, finding) in findings.iter_mut().enumerate() {
        finding.index = format!("F{}", offset + 1);
    }
}

fn normalize_lint_name(code: &str) -> String {
    code.strip_prefix("tigerstyle::").unwrap_or(code).to_owned()
}

fn json_u64_to_u32(value: Option<u64>) -> u32 {
    value.and_then(|line| u32::try_from(line).ok()).unwrap_or_default()
}

fn build_finding(
    index: usize,
    cargo_msg: &serde_json::Value,
    meta: &output::LintMeta,
    lint_name: &str,
    config_hash: &str,
    cache_enabled: bool,
) -> Finding {
    let message = &cargo_msg["message"];

    let primary_span = message["spans"]
        .as_array()
        .and_then(|spans| spans.iter().find(|s| s["is_primary"].as_bool() == Some(true)))
        .or_else(|| message["spans"].as_array().and_then(|s| s.first()));

    let file = primary_span.and_then(|s| s["file_name"].as_str()).unwrap_or("unknown").to_owned();
    let line = json_u64_to_u32(primary_span.and_then(|s| s["line_start"].as_u64()));
    let column = json_u64_to_u32(primary_span.and_then(|s| s["column_start"].as_u64()));

    let msg_text = message["message"].as_str().unwrap_or("").to_owned();
    let severity = message["level"].as_str().unwrap_or("warning").to_owned();
    let crate_name = cargo_msg["target"]["name"].as_str().unwrap_or("unknown").to_owned();

    let (suggestion, applicability) = extract_suggestion(message);
    let normalized_snippet_hash = primary_span
        .map(snippet_from_span)
        .filter(|snippet| !snippet.trim().is_empty())
        .map_or_else(|| "unsupported".to_owned(), |snippet| determinism::normalized_snippet_hash(&snippet));
    let identity_status = if normalized_snippet_hash == "unsupported" {
        "unsupported"
    } else {
        "computed"
    };
    let semantic_id = determinism::semantic_finding_id(
        lint_name,
        &crate_name,
        &msg_text,
        suggestion.as_deref(),
        &normalized_snippet_hash,
        Some(config_hash),
    );

    Finding {
        version: output::finding_version(),
        index: format!("F{index}"),
        stable_id: semantic_id.clone(),
        semantic_id,
        identity_status: identity_status.to_owned(),
        normalized_snippet_hash,
        config_hash: config_hash.to_owned(),
        cache_status: if cache_enabled { "pending" } else { "disabled" }.to_owned(),
        lint: lint_name.to_owned(),
        group: meta.group.to_owned(),
        severity,
        file,
        line,
        column,
        message: msg_text,
        suggestion,
        applicability,
        principle: meta.principle.to_owned(),
        effect_family: meta.effect_family.to_owned(),
        evidence_tier: crate::evidence::EvidenceTier::StaticLint,
        crate_name,
    }
}

fn snippet_from_span(span: &serde_json::Value) -> String {
    span["text"]
        .as_array()
        .map(|lines| lines.iter().filter_map(|line| line["text"].as_str()).collect::<Vec<_>>().join("\n"))
        .unwrap_or_default()
}

fn extract_suggestion(message: &serde_json::Value) -> (Option<String>, Option<String>) {
    let Some(children) = message["children"].as_array() else {
        return (None, None);
    };

    for child in children {
        if let Some(spans) = child["spans"].as_array() {
            for span in spans {
                if let Some(replacement) = span["suggested_replacement"].as_str() {
                    let applicability = span["suggestion_applicability"].as_str().map(ToOwned::to_owned);
                    return (Some(replacement.to_owned()), applicability);
                }
            }
        }
    }

    (None, None)
}

fn format_compiling_banner(json: &serde_json::Value) -> Option<String> {
    let fresh = json["fresh"].as_bool().unwrap_or(true);
    if fresh {
        return None;
    }

    let target_name = json["target"]["name"].as_str()?;

    let version = json["package_id"].as_str().and_then(|pid| pid.rsplit_once('#')).map(|(_, fragment)| {
        if let Some((_, ver)) = fragment.rsplit_once('@') {
            ver
        } else {
            fragment
        }
    });

    if let Some(ver) = version {
        Some(format!("   Compiling {target_name} v{ver}"))
    } else {
        Some(format!("   Compiling {target_name}"))
    }
}

// --- Everything below is unchanged from the original engine ---

struct TempDir {
    path: PathBuf,
}

impl TempDir {
    fn new(label: &str) -> Result<Self, String> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| format!("system clock error: {error}"))?
            .as_nanos();
        let path = env::temp_dir().join(format!("octet-{label}-{}-{nanos}", std::process::id()));
        fs::create_dir_all(&path).map_err(|error| format!("create {}: {error}", path.display()))?;
        Ok(Self { path })
    }

    fn path(&self) -> &Path {
        &self.path
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct ManagedPath {
    path: String,
    _tempdir: Option<TempDir>,
}

impl ManagedPath {
    fn explicit(path: String) -> Self {
        Self { path, _tempdir: None }
    }

    fn generated(path: String, tempdir: TempDir) -> Self {
        Self {
            path,
            _tempdir: Some(tempdir),
        }
    }

    fn path(&self) -> &str {
        &self.path
    }
}

fn resolve_driver(toolchain: &str) -> Result<ManagedPath, String> {
    match env::var("TIGERSTYLE_DRIVER_PATH") {
        Ok(value) if !value.is_empty() => Ok(ManagedPath::explicit(value)),
        Ok(_) => Err("environment variable `TIGERSTYLE_DRIVER_PATH` must not be empty".to_owned()),
        Err(env::VarError::NotPresent) => match env::var("TIGERSTYLE_DYLINT_REV") {
            Ok(rev) if !rev.is_empty() => bootstrap_driver(&rev, toolchain),
            Ok(_) => Err("environment variable `TIGERSTYLE_DYLINT_REV` must not be empty".to_owned()),
            Err(env::VarError::NotPresent) => Err(
                "missing required environment variable `TIGERSTYLE_DRIVER_PATH` or bootstrap revision `TIGERSTYLE_DYLINT_REV`"
                    .to_owned(),
            ),
            Err(env::VarError::NotUnicode(_)) => {
                Err("non-utf8 `TIGERSTYLE_DYLINT_REV` is not supported".to_owned())
            }
        },
        Err(env::VarError::NotUnicode(_)) => Err("non-utf8 `TIGERSTYLE_DRIVER_PATH` is not supported".to_owned()),
    }
}

fn bootstrap_driver(rev: &str, toolchain: &str) -> Result<ManagedPath, String> {
    let tempdir = TempDir::new("dylint-driver")?;
    let build_root = tempdir.path().join("bootstrap");
    let manifest_path = build_root.join("Cargo.toml");
    let source_path = build_root.join("src/main.rs");
    let target_dir = tempdir.path().join("target");

    write_bytes(
        &manifest_path,
        format!(
            "[package]\nname = \"tigerstyle-dylint-driver\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\ndylint_driver = {{ git = \"https://github.com/trailofbits/dylint\", rev = \"{rev}\" }}\n"
        )
        .as_bytes(),
    )?;
    write_bytes(
        &source_path,
        br#"#![feature(rustc_private)]

fn main() {
    let args: Vec<_> = std::env::args_os().collect();
    dylint_driver::dylint_driver(&args).unwrap_or_else(|error| {
        eprintln!("dylint-driver: {error:#}");
        std::process::exit(1);
    });
}
"#,
    )?;

    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .arg("--release")
        .current_dir(&build_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .env("LIBGIT2_SYS_USE_PKG_CONFIG", "1")
        .status()
        .map_err(|error| format!("run cargo build for dylint driver bootstrap: {error}"))?;
    if !status.success() {
        return Err("dylint driver bootstrap failed".to_owned());
    }

    let driver_path = target_dir.join("release/tigerstyle-dylint-driver");
    if !driver_path.is_file() {
        return Err(format!("bootstrapped dylint driver missing at {}", driver_path.display()));
    }

    Ok(ManagedPath::generated(driver_path.to_string_lossy().into_owned(), tempdir))
}

fn resolve_lint_lib(toolchain: &str) -> Result<ManagedPath, String> {
    match env::var("TIGERSTYLE_LINT_LIB") {
        Ok(value) if !value.is_empty() => Ok(ManagedPath::explicit(value)),
        Ok(_) => Err("environment variable `TIGERSTYLE_LINT_LIB` must not be empty".to_owned()),
        Err(env::VarError::NotPresent) => build_lint_lib(toolchain),
        Err(env::VarError::NotUnicode(_)) => Err("non-utf8 `TIGERSTYLE_LINT_LIB` is not supported".to_owned()),
    }
}

fn build_lint_lib(toolchain: &str) -> Result<ManagedPath, String> {
    let tempdir = TempDir::new("lint-lib")?;
    let target_dir = tempdir.path().join("target");
    let source_root = materialize_embedded_source()?;
    let manifest_path = source_root.join("Cargo.toml");
    let built_lib_name = format!("{}octet.{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_EXTENSION);
    let dylint_lib_name =
        format!("{}octet@{toolchain}.{}", std::env::consts::DLL_PREFIX, std::env::consts::DLL_EXTENSION);

    let status = Command::new("cargo")
        .arg("build")
        .arg("--manifest-path")
        .arg(&manifest_path)
        .args(["--release", "--locked", "-p", "octet", "--lib"])
        .current_dir(&source_root)
        .env("CARGO_TARGET_DIR", &target_dir)
        .env("RUSTUP_TOOLCHAIN", toolchain)
        .status()
        .map_err(|error| format!("run cargo build for Octet lint library: {error}"))?;
    if !status.success() {
        return Err("Octet lint library build failed".to_owned());
    }

    let built_lib_path = target_dir.join("release").join(&built_lib_name);
    if !built_lib_path.is_file() {
        return Err(format!("built Octet lint library missing at {}", built_lib_path.display()));
    }

    let dylint_lib_path = tempdir.path().join(&dylint_lib_name);
    copy_file(&built_lib_path, &dylint_lib_path)?;
    Ok(ManagedPath::generated(dylint_lib_path.to_string_lossy().into_owned(), tempdir))
}

fn resolve_toolchain() -> Result<String, String> {
    match env::var("TIGERSTYLE_TOOLCHAIN") {
        Ok(value) if !value.is_empty() => Ok(value),
        Ok(_) => Err("environment variable `TIGERSTYLE_TOOLCHAIN` must not be empty".to_owned()),
        Err(env::VarError::NotPresent) => derive_embedded_toolchain(),
        Err(env::VarError::NotUnicode(_)) => Err("non-utf8 `TIGERSTYLE_TOOLCHAIN` is not supported".to_owned()),
    }
}

fn derive_embedded_toolchain() -> Result<String, String> {
    let source_root = materialize_embedded_source()?;
    let rust_toolchain_path = source_root.join("rust-toolchain.toml");
    let rust_toolchain = fs::read_to_string(&rust_toolchain_path)
        .map_err(|error| format!("read {}: {error}", rust_toolchain_path.display()))?;
    let document = rust_toolchain
        .parse::<toml::Table>()
        .map_err(|error| format!("parse {}: {error}", rust_toolchain_path.display()))?;
    let channel = document
        .get("toolchain")
        .and_then(toml::Value::as_table)
        .and_then(|table| table.get("channel"))
        .and_then(toml::Value::as_str)
        .ok_or_else(|| format!("missing `toolchain.channel` in {}", rust_toolchain_path.display()))?;
    let host = rustc_host_triple()?;

    if channel.ends_with(&host) {
        Ok(channel.to_owned())
    } else {
        Ok(format!("{channel}-{host}"))
    }
}

fn materialize_embedded_source() -> Result<PathBuf, String> {
    let sources_root = sources_cache_root()?;
    let final_dir = sources_root.join(embedded_source::EMBEDDED_SOURCE_HASH);
    if cache_marker_matches(&final_dir)? {
        return Ok(final_dir);
    }

    let staging_dir = sources_root.join(format!(
        ".{}-{}-{}",
        embedded_source::EMBEDDED_SOURCE_HASH,
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| format!("system clock error: {error}"))?
            .as_nanos()
    ));
    if staging_dir.exists() {
        fs::remove_dir_all(&staging_dir).map_err(|error| format!("remove {}: {error}", staging_dir.display()))?;
    }

    for (relative_path, contents) in embedded_source::EMBEDDED_SOURCE_FILES {
        write_bytes(&staging_dir.join(relative_path), contents)?;
    }
    write_bytes(&staging_dir.join(SOURCE_HASH_MARKER), embedded_source::EMBEDDED_SOURCE_HASH.as_bytes())?;

    if final_dir.exists() && !cache_marker_matches(&final_dir)? {
        fs::remove_dir_all(&final_dir).map_err(|error| format!("remove {}: {error}", final_dir.display()))?;
    }

    match fs::rename(&staging_dir, &final_dir) {
        Ok(()) => Ok(final_dir),
        Err(error) => {
            let _ = fs::remove_dir_all(&staging_dir);
            if final_dir.exists() && cache_marker_matches(&final_dir)? {
                Ok(final_dir)
            } else {
                Err(format!("rename {} -> {}: {error}", staging_dir.display(), final_dir.display()))
            }
        }
    }
}

fn sources_cache_root() -> Result<PathBuf, String> {
    let mut candidates = Vec::new();
    if let Some(path) = env::var_os("XDG_CACHE_HOME") {
        candidates.push(PathBuf::from(path));
    }
    if let Some(path) = env::var_os("HOME") {
        candidates.push(PathBuf::from(path).join(".cache"));
    }
    candidates.push(env::temp_dir().join("octet-cache"));

    let mut failures = Vec::new();
    for candidate in candidates {
        let sources_root = candidate.join("octet").join("sources");
        match fs::create_dir_all(&sources_root) {
            Ok(()) => return Ok(sources_root),
            Err(error) => failures.push(format!("create {}: {error}", sources_root.display())),
        }
    }

    Err(failures.join("; "))
}

fn cache_marker_matches(path: &Path) -> Result<bool, String> {
    let marker_path = path.join(SOURCE_HASH_MARKER);
    let marker = match fs::read_to_string(&marker_path) {
        Ok(marker) => marker,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("read {}: {error}", marker_path.display())),
    };
    Ok(marker.trim_end() == embedded_source::EMBEDDED_SOURCE_HASH)
}

fn rustc_host_triple() -> Result<String, String> {
    let rustc = env::var("RUSTC").unwrap_or_else(|_| "rustc".to_owned());
    let output = Command::new(rustc).arg("-vV").output().map_err(|error| format!("run rustc -vV: {error}"))?;
    if !output.status.success() {
        return Err(format!("rustc -vV failed: {}", String::from_utf8_lossy(&output.stderr)));
    }

    let stdout = String::from_utf8(output.stdout).map_err(|_| "rustc -vV stdout was not utf8".to_owned())?;
    stdout
        .lines()
        .find_map(|line| line.strip_prefix("host: "))
        .map(ToOwned::to_owned)
        .ok_or_else(|| "rustc -vV output missing `host:` line".to_owned())
}

enum LockfileSnapshot {
    Missing,
    Present(Vec<u8>),
}

fn snapshot_lockfile(path: &Path) -> Result<LockfileSnapshot, String> {
    match fs::read(path) {
        Ok(bytes) => Ok(LockfileSnapshot::Present(bytes)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(LockfileSnapshot::Missing),
        Err(error) => Err(format!("read {}: {error}", path.display())),
    }
}

fn lockfile_changed(path: &Path, before: &LockfileSnapshot) -> Result<bool, String> {
    match before {
        LockfileSnapshot::Missing => Ok(path.exists()),
        LockfileSnapshot::Present(original) => match fs::read(path) {
            Ok(current) => Ok(&current != original),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
            Err(error) => Err(format!("read {}: {error}", path.display())),
        },
    }
}

fn restore_lockfile(path: &Path, before: &LockfileSnapshot) -> Result<(), String> {
    match before {
        LockfileSnapshot::Missing => fs::remove_file(path)
            .or_else(|error| {
                if error.kind() == std::io::ErrorKind::NotFound {
                    Ok(())
                } else {
                    Err(error)
                }
            })
            .map_err(|error| format!("remove {}: {error}", path.display())),
        LockfileSnapshot::Present(original) => write_bytes(path, original),
    }
}

fn write_bytes(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::write(path, bytes).map_err(|error| format!("write {}: {error}", path.display()))
}

fn copy_file(from: &Path, to: &Path) -> Result<(), String> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
    }
    fs::copy(from, to)
        .map(|_| ())
        .map_err(|error| format!("copy {} -> {}: {error}", from.display(), to.display()))
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeSet;

    use super::embedded_source;

    const REQUIRED_EMBEDDED_PATHS: &[&str] = &[
        "Cargo.toml",
        "crates/octet-lint-metadata/Cargo.toml",
        "crates/octet-lint-metadata/src/strict_profile.rs",
        "standards/effect-closure/generated/effect-closure-catalog.json",
        "standards/strict-profiles/generated/strict-rust-profiles.json",
    ];

    #[test]
    fn embedded_lint_source_contains_profile_and_effect_closure_runtime_inputs() {
        let paths = embedded_source::EMBEDDED_SOURCE_FILES
            .iter()
            .map(|(path, _contents)| *path)
            .collect::<BTreeSet<_>>();

        for required in REQUIRED_EMBEDDED_PATHS {
            assert!(paths.contains(required), "embedded lint source is missing `{required}`");
        }
    }

    #[test]
    fn embedded_lint_source_paths_are_unique_and_workspace_relative() {
        let paths = embedded_source::EMBEDDED_SOURCE_FILES.iter().map(|(path, _contents)| *path).collect::<Vec<_>>();
        let unique = paths.iter().copied().collect::<BTreeSet<_>>();

        assert_eq!(paths.len(), unique.len(), "embedded source paths must be unique");
        assert!(paths.iter().all(|path| !path.starts_with('/') && !path.contains("../")));
    }
}

// r[impl release_evidence.current_head_staleness_guard.command]
// r[impl release_evidence.current_head_staleness_guard.parser]
// r[impl release_evidence.current_head_staleness_guard.git_checks]
// r[impl release_evidence.current_head_snapshot.check_index]
// r[verify release_evidence.current_head_snapshot.validation]
// r[verify release_evidence.current_head_staleness_guard.operator_docs]
// r[verify release_evidence.current_head_staleness_guard.archive_ready]
use std::{error::Error, path::PathBuf, process::Command};

use serde_json::{json, Value};

use crate::support::{
    path_string, print_json, read_option, read_string_option, with_portable_content_hash_receipt,
    CapabilityRoot, CapabilityRootKind,
};

use super::common::release_evidence_issue;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CheckIndexMode {
    AuditedPayload,
    CurrentHead,
}

impl CheckIndexMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::AuditedPayload => "audited-payload",
            Self::CurrentHead => "current-head",
        }
    }
}

pub(super) fn run_release_evidence_check_index(args: Vec<String>) -> Result<(), Box<dyn Error>> {
    let options = release_evidence_check_index_options(&args)?;
    let receipt = release_evidence_check_index_receipt(&options);
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

pub(super) struct ReleaseEvidenceCheckIndexOptions {
    pub(super) root: PathBuf,
    pub(super) index: PathBuf,
    pub(super) mode: Option<CheckIndexMode>,
}

fn release_evidence_check_index_options(
    args: &[String],
) -> Result<ReleaseEvidenceCheckIndexOptions, Box<dyn Error>> {
    let root = read_option(args.to_vec(), "--root", ".");
    let Some(index) = read_string_option(args, "--index").map(PathBuf::from) else {
        return Err("release-evidence check-index requires --index <file>".into());
    };
    let mode = match read_string_option(args, "--mode").as_deref() {
        Some("audited-payload") => Some(CheckIndexMode::AuditedPayload),
        Some("current-head") => Some(CheckIndexMode::CurrentHead),
        Some(other) => {
            return Err(format!("unsupported release-evidence check-index mode: {other}").into())
        }
        None => None,
    };
    Ok(ReleaseEvidenceCheckIndexOptions { root, index, mode })
}

pub(crate) fn release_evidence_check_index_receipt_for_path(
    root: PathBuf,
    index: PathBuf,
) -> Value {
    release_evidence_check_index_receipt(&ReleaseEvidenceCheckIndexOptions {
        root,
        index,
        mode: None,
    })
}

const RELEASE_EVIDENCE_INDEX_MAX_BYTES: u64 = 4 * 1024 * 1024;

pub(super) fn release_evidence_check_index_receipt(
    options: &ReleaseEvidenceCheckIndexOptions,
) -> Value {
    let root = options.root.clone();
    let index = options.index.clone();
    let mut issues = Vec::new();
    let markdown = CapabilityRoot::open_file_parent(CapabilityRootKind::Release, &index, false)
        .and_then(|(index_root, relative)| {
            index_root.read_bounded(&relative, RELEASE_EVIDENCE_INDEX_MAX_BYTES)
        })
        .ok()
        .and_then(|bytes| String::from_utf8(bytes).ok());
    let mode = options
        .mode
        .unwrap_or_else(|| infer_check_index_mode(&index, markdown.as_deref()));

    let parsed = match markdown {
        Some(markdown) => parse_release_evidence_index_payload(&markdown, &mut issues),
        None => {
            issues.push(release_evidence_issue(
                "release_evidence.check_index.unreadable",
                "index",
                "readable release evidence Markdown index",
                "unreadable",
            ));
            None
        }
    };

    let live_head = git_output(&root, &["rev-parse", "--short", "HEAD"]);
    if live_head.is_none() {
        issues.push(release_evidence_issue(
            "release_evidence.check_index.git_head_unavailable",
            "git.head",
            "readable git HEAD",
            "unavailable",
        ));
    }

    let payload_commit = parsed
        .as_ref()
        .map(|payload| payload.payload_commit.clone());
    let payload_exists = payload_commit
        .as_deref()
        .map(|commit| {
            git_command_success(
                &root,
                &["rev-parse", "--verify", &format!("{commit}^{{commit}}")],
            )
        })
        .unwrap_or(false);
    if payload_commit.is_some() && !payload_exists {
        issues.push(release_evidence_issue(
            "release_evidence.check_index.payload_commit_missing",
            "payload.commit",
            "commit present in local repository",
            "missing",
        ));
    }

    let payload_ancestor_of_head = payload_commit
        .as_deref()
        .filter(|_| payload_exists)
        .map(|commit| git_command_success(&root, &["merge-base", "--is-ancestor", commit, "HEAD"]))
        .unwrap_or(false);
    if payload_commit.is_some() && payload_exists && !payload_ancestor_of_head {
        issues.push(release_evidence_issue(
            "release_evidence.check_index.payload_not_ancestor",
            "payload.commit",
            "payload commit ancestor of HEAD",
            "not_ancestor",
        ));
    }

    let current_head_match = payload_commit
        .as_deref()
        .zip(live_head.as_deref())
        .map(|(payload, head)| {
            payload == head || head.starts_with(payload) || payload.starts_with(head)
        })
        .unwrap_or(false);
    if mode == CheckIndexMode::CurrentHead && payload_commit.is_some() && !current_head_match {
        issues.push(release_evidence_issue(
            "release_evidence.check_index.stale_current_head",
            "payload.commit",
            "payload commit equals live HEAD for current-head index",
            "stale",
        ));
    }

    let valid = issues.is_empty();
    with_portable_content_hash_receipt(
        &root,
        json!({
            "schema": "cairn.release-evidence.check-index.v1",
            "command": "release-evidence check-index",
            "root": path_string(&root),
            "index": path_string(&index),
            "mode": mode.as_str(),
            "valid": valid,
            "verdict": if valid { "pass" } else { "fail" },
            "payload_commit": payload_commit.unwrap_or_default(),
            "payload": parsed.map(|payload| payload.to_json()).unwrap_or_else(|| json!({})),
            "live_head": live_head.unwrap_or_default(),
            "payload_exists": payload_exists,
            "payload_ancestor_of_head": payload_ancestor_of_head,
            "current_head_match": current_head_match,
            "issues": issues,
        }),
    )
}

fn infer_check_index_mode(index: &std::path::Path, markdown: Option<&str>) -> CheckIndexMode {
    let path_claims_current = index
        .to_string_lossy()
        .to_ascii_lowercase()
        .contains("current-head");
    let title_claims_current = markdown
        .map(|text| {
            text.lines()
                .take(5)
                .any(|line| line.to_ascii_lowercase().contains("current-head"))
        })
        .unwrap_or(false);
    if path_claims_current || title_claims_current {
        CheckIndexMode::CurrentHead
    } else {
        CheckIndexMode::AuditedPayload
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub(super) struct ReleaseEvidenceIndexPayload {
    pub(super) payload_commit: String,
    pub(super) branch: String,
    pub(super) clean: bool,
    pub(super) upstream: String,
    pub(super) ahead_behind: String,
}

impl ReleaseEvidenceIndexPayload {
    fn to_json(&self) -> Value {
        json!({
            "payload_commit": self.payload_commit,
            "branch": self.branch,
            "clean": self.clean,
            "upstream": self.upstream,
            "ahead_behind": self.ahead_behind,
        })
    }
}

pub(super) fn parse_release_evidence_index_payload(
    markdown: &str,
    issues: &mut Vec<Value>,
) -> Option<ReleaseEvidenceIndexPayload> {
    let payload_commit =
        parse_backticked_field(markdown, "Payload commit", issues, "payload.commit");
    let branch = parse_backticked_field(markdown, "Branch", issues, "payload.branch");
    let clean_text = parse_backticked_field(markdown, "Worktree clean", issues, "payload.clean");
    let upstream = parse_backticked_field(markdown, "Upstream", issues, "payload.upstream");
    let ahead_behind =
        parse_backticked_field(markdown, "Ahead/behind", issues, "payload.ahead_behind");
    let clean = match clean_text.as_deref() {
        Some("true") => Some(true),
        Some("false") => Some(false),
        Some(_) => {
            issues.push(release_evidence_issue(
                "release_evidence.check_index.payload_clean_malformed",
                "payload.clean",
                "true or false",
                "malformed",
            ));
            None
        }
        None => None,
    };
    Some(ReleaseEvidenceIndexPayload {
        payload_commit: payload_commit?,
        branch: branch?,
        clean: clean?,
        upstream: upstream?,
        ahead_behind: ahead_behind?,
    })
}

fn parse_backticked_field(
    markdown: &str,
    label: &str,
    issues: &mut Vec<Value>,
    field_path: &str,
) -> Option<String> {
    let prefix = format!("- {label}: `");
    let Some(line) = markdown.lines().find(|line| line.starts_with(&prefix)) else {
        issues.push(release_evidence_issue(
            "release_evidence.check_index.payload_field_missing",
            field_path,
            "generated payload field",
            "missing",
        ));
        return None;
    };
    let rest = line.strip_prefix(&prefix)?;
    let Some((value, suffix)) = rest.split_once('`') else {
        issues.push(release_evidence_issue(
            "release_evidence.check_index.payload_field_malformed",
            field_path,
            "backticked field value",
            "malformed",
        ));
        return None;
    };
    if !suffix.is_empty() || value.is_empty() {
        issues.push(release_evidence_issue(
            "release_evidence.check_index.payload_field_malformed",
            field_path,
            "single non-empty backticked value",
            "malformed",
        ));
        return None;
    }
    Some(value.to_owned())
}

fn git_output(root: &std::path::Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn git_command_success(root: &std::path::Path, args: &[&str]) -> bool {
    Command::new("git")
        .args(args)
        .current_dir(root)
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

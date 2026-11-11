use std::fs;
use std::path::Path;
use std::path::PathBuf;

use crate::cli::EvidenceIndexArgs;
use crate::cli::EvidenceIndexMode;
use crate::valence_core_shim::evidence_codebase_index::EvidenceArtifactInput;
use crate::valence_core_shim::evidence_codebase_index::EvidenceCodebaseIndex;
use crate::valence_core_shim::evidence_codebase_index::EvidenceIndexBuildMode;
use crate::valence_core_shim::evidence_codebase_index::EvidenceIndexIssueSeverity;
use crate::valence_core_shim::evidence_codebase_index::build_evidence_codebase_index;
use crate::valence_core_shim::evidence_codebase_index::query_evidence_index;
use crate::valence_core_shim::evidence_codebase_index::render_evidence_index_html;
use crate::valence_core_shim::evidence_index::EvidenceIndexInput;
use crate::valence_core_shim::evidence_index::check_page_text_with_expected;
use crate::valence_core_shim::evidence_index::render_valence_evidence_index;

const DEFAULT_PAGE: &str = "docs/current-head-valence-evidence.md";
const SUPPORTED_DIRECTORY_EXTENSIONS: &[&str] = &["json", "jsonl", "md"];
const SKIPPED_DIRECTORY_NAMES: &[&str] = &[".git", ".jj", "target"];
const EXIT_SUCCESS: i32 = 0;
const EXIT_FAILURE: i32 = 1;

pub fn run(args: EvidenceIndexArgs) -> Result<i32, String> {
    match args.mode {
        EvidenceIndexMode::Render => run_valence_render(args),
        EvidenceIndexMode::Check => run_valence_check(args),
        EvidenceIndexMode::CodebaseBuild => run_codebase_build(&args),
        EvidenceIndexMode::CodebaseQuery => run_codebase_query(&args),
        EvidenceIndexMode::CodebaseHtml => run_codebase_html(&args),
    }
}

fn run_valence_render(args: EvidenceIndexArgs) -> Result<i32, String> {
    let input = EvidenceIndexInput {
        capture_time: args.capture_time.ok_or_else(|| "missing --capture-time for evidence index render".to_owned())?,
        payload_commit: args
            .payload_commit
            .ok_or_else(|| "missing --payload-commit for evidence index render".to_owned())?,
    };
    print!("{}", render_valence_evidence_index(&input));
    Ok(EXIT_SUCCESS)
}

fn run_valence_check(args: EvidenceIndexArgs) -> Result<i32, String> {
    let page = args.page.unwrap_or_else(|| PathBuf::from(DEFAULT_PAGE));
    let text = fs::read_to_string(&page).map_err(|err| format!("failed to read {}: {err}", page.display()))?;
    let report = check_page_text_with_expected(&text, args.payload_commit, args.capture_time);
    if report.valid {
        println!("valence_evidence_index.valid: PASS");
        println!("valence_evidence_index.issue_count=0");
        Ok(EXIT_SUCCESS)
    } else {
        println!("valence_evidence_index.valid: FAIL");
        for code in &report.issue_codes {
            println!("valence_evidence_index.issue={code}");
        }
        Ok(EXIT_FAILURE)
    }
}

fn run_codebase_build(args: &EvidenceIndexArgs) -> Result<i32, String> {
    let own_output = args.output.as_deref().map(resolve_output_path).transpose()?.flatten();
    let inputs = collect_artifact_inputs(&args.artifacts, &args.artifact_dirs, own_output.as_deref())?;
    let mode = if args.advisory {
        EvidenceIndexBuildMode::Advisory
    } else {
        EvidenceIndexBuildMode::Strict
    };
    let index = build_evidence_codebase_index(&inputs, mode);
    let rendered = render_json(&index)?;
    write_or_print(args.output.as_deref(), &rendered)?;
    let has_error = index.issues.iter().any(|issue| issue.severity == EvidenceIndexIssueSeverity::Error);
    Ok(i32::from(has_error))
}

fn run_codebase_query(args: &EvidenceIndexArgs) -> Result<i32, String> {
    let index_path = args.index.as_deref().ok_or_else(|| "missing --index for evidence-index query".to_owned())?;
    let identity = args.identity.as_deref().ok_or_else(|| "missing --identity for evidence-index query".to_owned())?;
    let index = read_index(index_path)?;
    let report = query_evidence_index(&index, identity);
    println!("{}", render_json(&report)?);
    Ok(i32::from(report.match_count == 0))
}

fn run_codebase_html(args: &EvidenceIndexArgs) -> Result<i32, String> {
    let index_path = args.index.as_deref().ok_or_else(|| "missing --index for evidence-index html".to_owned())?;
    let index = read_index(index_path)?;
    let html = render_evidence_index_html(&index);
    write_or_print(args.output.as_deref(), &html)?;
    Ok(EXIT_SUCCESS)
}

/// Resolves the build's own output path so directory scans can skip a previously written index.
fn resolve_output_path(output: &Path) -> Result<Option<PathBuf>, String> {
    if output.exists() {
        return fs::canonicalize(output)
            .map(Some)
            .map_err(|error| format!("canonicalize {}: {error}", output.display()));
    }
    let Some(file_name) = output.file_name() else {
        return Ok(None);
    };
    let parent = output.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or(Path::new("."));
    if !parent.is_dir() {
        return Ok(None);
    }
    fs::canonicalize(parent)
        .map(|parent| Some(parent.join(file_name)))
        .map_err(|error| format!("canonicalize {}: {error}", parent.display()))
}

fn collect_artifact_inputs(
    artifacts: &[PathBuf],
    artifact_dirs: &[PathBuf],
    own_output: Option<&Path>,
) -> Result<Vec<EvidenceArtifactInput>, String> {
    let mut paths = artifacts.to_vec();
    for directory in artifact_dirs {
        collect_supported_files(directory, own_output, &mut paths)?;
    }
    paths.sort();
    paths.dedup();

    paths
        .iter()
        .map(|path| {
            let bytes = fs::read(path).map_err(|error| format!("read {}: {error}", path.display()))?;
            Ok(EvidenceArtifactInput {
                locator: locator_for(path),
                bytes,
            })
        })
        .collect()
}

fn collect_supported_files(path: &Path, own_output: Option<&Path>, paths: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if supported_directory_file(path) && !is_own_output(path, own_output) {
            paths.push(path.to_path_buf());
        }
        return Ok(());
    }
    if !path.is_dir() {
        return Err(format!("artifact directory {} is neither file nor directory", path.display()));
    }
    for entry in fs::read_dir(path).map_err(|error| format!("read dir {}: {error}", path.display()))? {
        let entry = entry.map_err(|error| format!("read dir entry {}: {error}", path.display()))?;
        let child = entry.path();
        if child
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| SKIPPED_DIRECTORY_NAMES.contains(&name))
        {
            continue;
        }
        collect_supported_files(&child, own_output, paths)?;
    }
    Ok(())
}

fn is_own_output(path: &Path, own_output: Option<&Path>) -> bool {
    own_output.is_some_and(|own_output| {
        path.file_name() == own_output.file_name() && fs::canonicalize(path).is_ok_and(|path| path == own_output)
    })
}

fn supported_directory_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| SUPPORTED_DIRECTORY_EXTENSIONS.contains(&extension))
}

fn locator_for(path: &Path) -> String {
    std::env::current_dir()
        .ok()
        .and_then(|current_dir| path.strip_prefix(current_dir).ok().map(Path::to_path_buf))
        .unwrap_or_else(|| path.to_path_buf())
        .display()
        .to_string()
}

fn read_index(path: &Path) -> Result<EvidenceCodebaseIndex, String> {
    let text = fs::read_to_string(path).map_err(|error| format!("read {}: {error}", path.display()))?;
    serde_json::from_str(&text).map_err(|error| format!("parse {}: {error}", path.display()))
}

fn render_json<T: serde::Serialize + ?Sized>(value: &T) -> Result<String, String> {
    let mut rendered =
        serde_json::to_string_pretty(value).map_err(|error| format!("serialize evidence index JSON: {error}"))?;
    rendered.push('\n');
    Ok(rendered)
}

fn write_or_print(path: Option<&Path>, text: &str) -> Result<(), String> {
    if let Some(path) = path {
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|error| format!("create {}: {error}", parent.display()))?;
        }
        fs::write(path, text).map_err(|error| format!("write {}: {error}", path.display()))?;
    } else {
        print!("{text}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::valence_core_shim::evidence_index::check_page_text;

    #[test]
    fn evidence_index_cli_uses_valence_core_renderer() {
        let input = EvidenceIndexInput {
            capture_time: "2026-05-23T12:00:00Z".to_owned(),
            payload_commit: "0123456789abcdef0123456789abcdef01234567".to_owned(),
        };
        let rendered = render_valence_evidence_index(&input);
        let report = check_page_text(&rendered);
        assert!(report.valid);
        assert!(report.issue_codes.is_empty());
    }

    #[test]
    fn codebase_index_directory_collector_filters_supported_artifacts() {
        let root = unique_temp_dir("collector");
        fs::create_dir_all(root.join("target")).expect("create target dir");
        fs::write(root.join("evidence.json"), b"{}").expect("write json");
        fs::write(root.join("note.txt"), b"ignored").expect("write txt");
        fs::write(root.join("target/ignored.json"), b"{}").expect("write ignored target json");

        let mut paths = Vec::new();
        collect_supported_files(&root, None, &mut paths).expect("collect files");
        paths.sort();

        assert_eq!(paths, vec![root.join("evidence.json")]);
        fs::remove_dir_all(root).expect("remove temp dir");
    }

    #[test]
    fn codebase_index_directory_collector_skips_own_output() {
        let root = unique_temp_dir("own-output");
        fs::create_dir_all(root.join("nested")).expect("create nested dir");
        fs::write(root.join("evidence.json"), b"{}").expect("write evidence json");
        fs::write(root.join("index.json"), b"{}").expect("write earlier index output");
        fs::write(root.join("nested/index.json"), b"{}").expect("write same-named evidence json");
        let own_output = resolve_output_path(&root.join("nested/../index.json"))
            .expect("resolve output path")
            .expect("output parent exists");

        let mut paths = Vec::new();
        collect_supported_files(&root, Some(&own_output), &mut paths).expect("collect files");
        paths.sort();

        assert_eq!(paths, vec![root.join("evidence.json"), root.join("nested/index.json")]);
        fs::remove_dir_all(root).expect("remove temp dir");
    }

    #[test]
    fn codebase_index_output_path_resolves_before_first_write() {
        let root = unique_temp_dir("first-write");
        fs::create_dir_all(&root).expect("create temp dir");

        let resolved = resolve_output_path(&root.join("index.json")).expect("resolve output path");

        assert_eq!(resolved, Some(fs::canonicalize(&root).expect("canonicalize root").join("index.json")));
        assert_eq!(resolve_output_path(&root.join("missing/index.json")).expect("resolve output path"), None);
        fs::remove_dir_all(root).expect("remove temp dir");
    }

    fn unique_temp_dir(name: &str) -> PathBuf {
        let mut path = std::env::temp_dir();
        path.push(format!("octet-evidence-index-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        path
    }
}

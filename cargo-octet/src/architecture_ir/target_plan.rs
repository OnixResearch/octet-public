//! Cargo owns target selection. This adapter obtains a bounded, non-executing build graph.

use std::path::Path;

use octet_standards::project_architecture_policy::ArchitectureTarget;
use serde::Deserialize;

const UNIT_GRAPH_VERSION: u64 = 1;
const PLAN_TIMEOUT_MS: u64 = 120_000;
const PLAN_POLL_INTERVAL_MS: u64 = 10;
const PLAN_TEARDOWN_TIMEOUT_MS: u64 = 5_000;
const ACCEPTED_EXIT_CODE: i32 = 0;
const PLAN_STDIN_BYTE_LIMIT: usize = 1;

#[derive(Debug, Deserialize)]
pub(super) struct CargoUnitGraph {
    pub version: u64,
    pub units: Vec<CargoPlannedUnit>,
    pub roots: Vec<usize>,
    #[serde(skip)]
    selection_identity: String,
}

#[derive(Debug, Deserialize)]
pub(super) struct CargoPlannedUnit {
    pub pkg_id: String,
    pub target: CargoPlannedTarget,
    pub platform: Option<String>,
    pub mode: String,
    pub features: Vec<String>,
    pub profile: serde_json::Value,
}

#[derive(Debug, Deserialize)]
pub(super) struct CargoPlannedTarget {
    pub name: String,
    pub kind: Vec<String>,
    pub crate_types: Vec<String>,
    pub src_path: String,
}

impl CargoUnitGraph {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let mut graph: Self =
            serde_json::from_slice(bytes).map_err(|error| format!("parse Cargo unit graph: {error}"))?;
        if graph.version != UNIT_GRAPH_VERSION || graph.roots.is_empty() || graph.units.is_empty() {
            return Err("Cargo unit graph has an unsupported version or empty selection".to_owned());
        }
        if graph.roots.iter().any(|root| *root >= graph.units.len()) {
            return Err("Cargo unit graph contains an invalid root index".to_owned());
        }
        graph.selection_identity = super::resolved_plan::identity(bytes)?;
        Ok(graph)
    }

    pub fn require_equivalent(&self, declared: &Self) -> Result<(), String> {
        if self.selection_identity != declared.selection_identity {
            return Err("Cargo --all-features plan does not match the explicit architecture policy row".to_owned());
        }
        Ok(())
    }
}

/// Plans the invocation once and binds it to exactly one candidate policy row.
///
/// Explicit feature selections have already been matched against each row. For `--all-features`,
/// every candidate row is replanned with only `--all-features` replaced by its explicit features,
/// and exactly one row must produce an equivalent complete Cargo plan.
// r[impl octet.architecture.features.equivalence]
pub(super) fn select_policy_row<'a>(
    workspace_root: &Path,
    toolchain: &str,
    scope_args: &[String],
    cargo_args: &[String],
    rows: &[&'a ArchitectureTarget],
    max_bytes: usize,
) -> Result<(&'a ArchitectureTarget, CargoUnitGraph), String> {
    if rows.is_empty() {
        return Err("Cargo invocation matches no architecture policy target".to_owned());
    }
    let actual = load(workspace_root, toolchain, scope_args, cargo_args, max_bytes)?;
    if !cargo_args.iter().any(|argument| argument == "--all-features") {
        let [row] = rows else {
            return Err("Cargo invocation matches more than one architecture policy target".to_owned());
        };
        return Ok((*row, actual));
    }
    let mut equivalent = Vec::with_capacity(rows.len());
    let mut mismatch = None;
    for row in rows {
        let mut declared_args = cargo_args
            .iter()
            .filter(|argument| argument.as_str() != "--all-features")
            .cloned()
            .collect::<Vec<_>>();
        if !row.features.is_empty() {
            declared_args.push("--features".to_owned());
            declared_args.push(row.features.join(","));
        }
        let declared = load(workspace_root, toolchain, scope_args, &declared_args, max_bytes)?;
        match actual.require_equivalent(&declared) {
            Ok(()) => equivalent.push(*row),
            Err(error) => mismatch = Some(error),
        }
    }
    match (equivalent.as_slice(), mismatch) {
        ([row], _) => Ok((*row, actual)),
        ([], Some(error)) => Err(error),
        _ => Err("Cargo --all-features plan matches more than one explicit architecture policy row".to_owned()),
    }
}

pub(super) fn load(
    workspace_root: &Path,
    toolchain: &str,
    scope_args: &[String],
    cargo_args: &[String],
    max_bytes: usize,
) -> Result<CargoUnitGraph, String> {
    let mut args = vec![
        "build".into(),
        "--unit-graph".into(),
        "-Zunstable-options".into(),
        "--locked".into(),
    ];
    args.extend(scope_args.iter().map(std::ffi::OsString::from));
    args.extend(cargo_args.iter().map(std::ffi::OsString::from));
    let mut environment = std::env::vars_os().collect::<std::collections::BTreeMap<_, _>>();
    environment.insert("RUSTUP_TOOLCHAIN".into(), toolchain.into());
    let output = bounded_exec::run(bounded_exec::RunRequest {
        command: bounded_exec::CommandSpec {
            program: cargo_program()?,
            args,
            current_dir: workspace_root.to_path_buf(),
            environment_mode: bounded_exec::EnvironmentMode::Clear,
            environment: environment.into_iter().collect(),
            input: bounded_exec::Input::Null,
        },
        limits: bounded_exec::ExecutionLimits {
            timeout_ms: PLAN_TIMEOUT_MS,
            stdin_max_bytes: PLAN_STDIN_BYTE_LIMIT,
            stdout_max_bytes: max_bytes,
            stderr_max_bytes: max_bytes,
            poll_interval_ms: PLAN_POLL_INTERVAL_MS,
            teardown_timeout_ms: PLAN_TEARDOWN_TIMEOUT_MS,
        },
        termination_scope: bounded_exec::TerminationScope::ProcessGroup,
        outcome_policy: bounded_exec::OutcomePolicy::new(vec![ACCEPTED_EXIT_CODE], true, true)
            .map_err(|error| format!("construct Cargo plan outcome policy: {error:?}"))?,
    })
    .map_err(|error| format!("run bounded Cargo plan: {error}"))?;
    if output.disposition != bounded_exec::Disposition::Succeeded {
        return Err(format!("Cargo unit planning did not complete: {:?}", output.disposition));
    }
    CargoUnitGraph::parse(&output.stdout.bytes)
}

fn cargo_program() -> Result<std::path::PathBuf, String> {
    let path = std::env::var_os("PATH").ok_or_else(|| "Cargo planner has no executable search path".to_owned())?;
    for directory in std::env::split_paths(&path) {
        let candidate = directory.join("cargo");
        if candidate.is_absolute() && candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err("Cargo planner has no absolute Cargo executable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::CargoUnitGraph;

    const FIXTURE_TOOLCHAIN: &str = "nightly-2026-03-21";
    const FIXTURE_PLAN_BYTE_LIMIT: usize = 2 * 1024 * 1024;

    #[test]
    fn cargo_resolver_distinguishes_library_binary_and_test_compilations() {
        let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("workspace");
        let graph = super::load(
            workspace,
            FIXTURE_TOOLCHAIN,
            &["-p".to_owned(), "octet-architecture-ir".to_owned()],
            &["--all-targets".to_owned()],
            FIXTURE_PLAN_BYTE_LIMIT,
        )
        .expect("bounded Cargo graph");
        let selected = graph
            .units
            .iter()
            .filter(|unit| unit.pkg_id.contains("/octet-architecture-ir#"))
            .collect::<Vec<_>>();
        for kind in ["lib", "bin"] {
            for mode in ["build", "test"] {
                assert!(
                    selected.iter().any(|unit| unit.target.kind.iter().any(|value| value == kind) && unit.mode == mode)
                );
            }
        }
    }

    #[test]
    fn empty_or_unknown_graph_is_rejected() {
        for bytes in [
            br#"{"version":1,"units":[],"roots":[]}"#.as_slice(),
            br#"{"version":0,"units":[],"roots":[]}"#.as_slice(),
            b"{}".as_slice(),
            b"not-json".as_slice(),
        ] {
            assert!(CargoUnitGraph::parse(bytes).is_err());
        }
    }

    #[test]
    fn complete_graph_preserves_test_mode_and_rejects_missing_root() {
        let graph = br#"{"version":1,"units":[{"pkg_id":"fixture","target":{"name":"fixture","kind":["lib"],"crate_types":["lib"],"src_path":"src/lib.rs"},"platform":null,"mode":"test","features":[],"profile":{},"dependencies":[]}],"roots":[0]}"#;
        let parsed = CargoUnitGraph::parse(graph).expect("complete graph");
        assert_eq!(parsed.units[0].mode, "test");
        let mut altered: serde_json::Value = serde_json::from_slice(graph).expect("JSON");
        altered["roots"] = serde_json::json!([parsed.units.len()]);
        assert!(CargoUnitGraph::parse(&serde_json::to_vec(&altered).expect("serialize")).is_err());
    }
}

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use octet_architecture_ir::CompilerArchitectureCoverage;
use octet_architecture_ir::CompilerArchitectureFact;
use octet_architecture_ir::CompilerArchitectureIr;
use octet_architecture_ir::CoverageStatus;
use octet_standards::project_architecture_policy::ArchitectureMode;
use octet_standards::project_architecture_policy::architecture_waiver_policy_blake3;

use super::super::PreparedCollector;
use super::super::SelectedProjectArchitecturePolicy;
use super::FIXTURE_FILES;
use super::TOOLCHAIN;
use super::policy_with_mode;
use super::required_artifact;

// The proc-macro appends tokens read from a non-Rust file. Editing that file recompiles the macro
// and its dependents without changing any source envelope, so every unit keeps its identity while
// the dependent library's compiler facts change.
const GENERATED_MACRO: &str = r#"extern crate proc_macro;

#[proc_macro]
pub fn identity(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let mut output = input;
    output.extend(include_str!("generated.txt").parse::<proc_macro::TokenStream>().expect("generated tokens"));
    output
}
"#;

struct FreshnessFixture<'a> {
    root: &'a Path,
    library: &'a Path,
    driver: &'a Path,
    scope: Vec<String>,
    args: Vec<String>,
}

struct CollectedRun {
    expected_unit_ids: BTreeSet<String>,
    workspace_artifacts: Vec<(String, bool)>,
    ir: CompilerArchitectureIr,
}

impl FreshnessFixture<'_> {
    fn cargo_check(&self, context: Option<&Path>) -> Vec<(String, bool)> {
        let mut command = Command::new("cargo");
        command
            .current_dir(self.root)
            .args(["check", "--locked", "--message-format=json", "--workspace"])
            .args(&self.args)
            .env("RUSTUP_TOOLCHAIN", TOOLCHAIN)
            .env("CARGO_TARGET_DIR", self.root.join("target"))
            .env("RUSTC_WRAPPER", "")
            .env("RUSTC_WORKSPACE_WRAPPER", self.driver)
            .env("DYLINT_NO_DEPS", "1")
            .env("DYLINT_LIBS", serde_json::to_string(&[self.library]).expect("library selection"))
            .env_remove(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV);
        if let Some(context) = context {
            command.env(octet_architecture_ir::COMPILER_ARCHITECTURE_COLLECTOR_CONTEXT_ENV, context);
        }
        let output = command.output().expect("compile fixture through real collector");
        assert!(output.status.success(), "compiler failed: {}", String::from_utf8_lossy(&output.stderr));
        String::from_utf8(output.stdout)
            .expect("Cargo JSON is UTF-8")
            .lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|message| message["reason"] == "compiler-artifact")
            .filter(|message| message["package_id"].as_str().is_some_and(|id| id.starts_with("path+file://")))
            .map(|message| {
                let label = format!(
                    "{}:{}:test={}",
                    message["target"]["name"].as_str().expect("artifact target name"),
                    message["target"]["kind"],
                    message["profile"]["test"]
                );
                (label, message["fresh"].as_bool().expect("artifact freshness"))
            })
            .collect()
    }

    fn prepare(&self, policy: &SelectedProjectArchitecturePolicy) -> PreparedCollector {
        super::super::prepare_collector(self.root, policy, TOOLCHAIN, self.library, &self.scope, &self.args)
            .expect("plan actual Cargo compilation units")
    }

    fn collect(&self, step: &str, policy: &SelectedProjectArchitecturePolicy) -> CollectedRun {
        let prepared = self.prepare(policy);
        let workspace_artifacts = self.cargo_check(Some(&prepared.context_path));
        let artifacts = super::super::normalize_collected_shards(&prepared).expect("normalize actual shards");
        let coverage: CompilerArchitectureCoverage = serde_json::from_str(&artifacts.coverage).expect("coverage");
        assert!(
            artifacts.incomplete_error.is_none(),
            "{step}: {:?}; workspace artifacts: {workspace_artifacts:?}",
            artifacts.incomplete_error
        );
        assert_eq!(coverage.status, CoverageStatus::Complete, "{step}");
        assert!(coverage.duplicate_unit_ids.is_empty(), "{step}: {:?}", coverage.duplicate_unit_ids);
        assert_eq!(coverage.expected_unit_ids, coverage.observed_unit_ids, "{step}");
        let shard_count = fs::read_dir(&prepared.shard_directory).expect("shards").count();
        assert_eq!(shard_count, prepared.expected_units.len(), "{step}: one retained shard per unit");
        eprintln!("{step}: complete; workspace artifacts {workspace_artifacts:?}");
        CollectedRun {
            expected_unit_ids: prepared.expected_units.iter().map(|expected| expected.unit.unit_id.clone()).collect(),
            workspace_artifacts,
            ir: serde_json::from_str(&artifacts.architecture_ir).expect("IR"),
        }
    }
}

fn has_item(ir: &CompilerArchitectureIr, name: &str) -> bool {
    ir.facts.iter().any(
        |fact| matches!(fact, CompilerArchitectureFact::Item { qualified_path, .. } if qualified_path.ends_with(name)),
    )
}

#[test]
#[ignore = "requires current OCTET_UI_LIBRARY and pinned OCTET_UI_DRIVER"]
fn real_cargo_cached_shards_follow_cargo_freshness() {
    let library = required_artifact("OCTET_UI_LIBRARY");
    let driver = required_artifact("OCTET_UI_DRIVER");
    let owner = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("owner root");
    let source = owner.join("fixtures/architecture-targets");
    let temporary = tempfile::TempDir::new().expect("fixture workspace");
    let root = temporary.path();
    for relative in FIXTURE_FILES {
        let target = root.join(relative);
        fs::create_dir_all(target.parent().expect("fixture parent")).expect("create fixture parent");
        fs::copy(source.join(relative), target).expect("copy controlled fixture source");
    }
    fs::write(root.join("macro/src/lib.rs"), GENERATED_MACRO).expect("generated-token proc-macro");
    fs::write(root.join("macro/src/generated.txt"), "").expect("empty generated tokens");
    let fixture = FreshnessFixture {
        root,
        library: &library,
        driver: &driver,
        scope: vec!["--workspace".to_owned()],
        args: vec!["--all-targets".to_owned(), "--features".to_owned(), "extra".to_owned()],
    };
    let (_, host) = super::super::toolchain_identity(TOOLCHAIN).expect("pinned host");
    let policy = policy_with_mode(root, &host, false, ArchitectureMode::Inventory);

    let lint_only = fixture.cargo_check(None);
    assert!(lint_only.iter().all(|(_, fresh)| !fresh), "first lint-only build compiles the workspace");
    let first = fixture.collect("architecture run after lint-only run", &policy);

    fs::remove_dir_all(root.join(super::super::ARCHITECTURE_SHARD_DIRECTORY)).expect("delete shard directory");
    let rebuilt = fixture.collect("architecture run after shard directory deletion", &policy);
    assert_eq!(rebuilt.expected_unit_ids, first.expected_unit_ids);

    let unchanged = fixture.collect("unchanged architecture rerun", &policy);
    assert_eq!(unchanged.expected_unit_ids, first.expected_unit_ids);
    assert!(!unchanged.workspace_artifacts.is_empty());
    assert!(
        unchanged.workspace_artifacts.iter().all(|(_, fresh)| *fresh),
        "unchanged rerun recompiled workspace units: {:?}",
        unchanged.workspace_artifacts
    );

    let mut revised = policy_with_mode(root, &host, false, ArchitectureMode::Inventory);
    revised.policy.capability_classifications[0].rationale.push_str(" (revised)");
    revised.waiver_policy_blake3 = architecture_waiver_policy_blake3(&revised.policy);
    assert_ne!(revised.waiver_policy_blake3, policy.waiver_policy_blake3);
    let policy_changed = fixture.collect("architecture run after non-waiver policy change", &revised);
    assert!(policy_changed.expected_unit_ids.is_disjoint(&first.expected_unit_ids));

    fs::write(root.join("macro/src/lib.rs"), format!("{GENERATED_MACRO}\npub(crate) const EDITED: bool = true;\n"))
        .expect("edit proc-macro source");
    let macro_source_edit = fixture.collect("architecture run after proc-macro source edit", &revised);
    assert!(macro_source_edit.expected_unit_ids.is_disjoint(&policy_changed.expected_unit_ids));
    assert!(!has_item(&macro_source_edit.ir, "macro_generated_value"));

    fs::write(root.join("macro/src/generated.txt"), "pub fn macro_generated_value() -> u32 { 7 }\n")
        .expect("edit proc-macro token input");
    let macro_input_edit = fixture.collect("architecture run after proc-macro token input edit", &revised);
    assert_eq!(
        macro_input_edit.expected_unit_ids, macro_source_edit.expected_unit_ids,
        "the token input lies outside every unit identity"
    );
    assert!(
        has_item(&macro_input_edit.ir, "macro_generated_value"),
        "dependent facts follow the recompiled unit"
    );
}

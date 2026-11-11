//! Real Cargo resolution controls. These tests never execute a build or build script.

use std::path::Path;

use octet_standards::project_architecture_policy::ArchitectureTarget;

use super::target_plan;

const TOOLCHAIN: &str = "nightly-2026-03-21";
const PLAN_BYTE_LIMIT: usize = 2 * 1024 * 1024;
const COMPLETE_FEATURES: &[&str] = &["feature-app/extra", "feature-helper/helper-extra"];
const FIXTURE_FILES: &[&str] = &[
    "Cargo.toml",
    "Cargo.lock",
    "app/Cargo.toml",
    "app/build.rs",
    "app/src/lib.rs",
    "app/src/main.rs",
    "helper/Cargo.toml",
    "helper/src/lib.rs",
    "optional/Cargo.toml",
    "optional/src/lib.rs",
];

fn fixture() -> tempfile::TempDir {
    let owner = Path::new(env!("CARGO_MANIFEST_DIR")).parent().expect("owner root");
    let source = owner.join("fixtures/architecture-features");
    let fixture = tempfile::TempDir::new().expect("fixture directory");
    for relative in FIXTURE_FILES {
        let target = fixture.path().join(relative);
        std::fs::create_dir_all(target.parent().expect("fixture parent")).expect("fixture parent directory");
        std::fs::copy(source.join(relative), target).expect("copy controlled Cargo fixture");
    }
    fixture
}

fn row(features: &[&str], default_features: bool) -> ArchitectureTarget {
    ArchitectureTarget {
        triple: "fixture-host".to_owned(),
        features: features.iter().map(|feature| (*feature).to_owned()).collect(),
        default_features,
    }
}

fn select<'a>(
    root: &Path,
    rows: &[&'a ArchitectureTarget],
    no_defaults: bool,
) -> Result<(&'a ArchitectureTarget, target_plan::CargoUnitGraph), String> {
    let mut args = vec!["--all-targets".to_owned(), "--all-features".to_owned()];
    if no_defaults {
        args.push("--no-default-features".to_owned());
    }
    target_plan::select_policy_row(root, TOOLCHAIN, &["--workspace".to_owned()], &args, rows, PLAN_BYTE_LIMIT)
}

fn plan(root: &Path, features: &[&str], no_defaults: bool) -> Result<target_plan::CargoUnitGraph, String> {
    let row = row(features, !no_defaults);
    select(root, &[&row], no_defaults).map(|(_, graph)| graph)
}

// r[verify octet.architecture.features.equivalence]
#[test]
fn all_features_binds_workspace_optional_dependencies_and_gated_targets() {
    let root = fixture();
    let graph = plan(root.path(), COMPLETE_FEATURES, false).expect("complete declared row");
    assert!(graph.units.iter().any(|unit| unit.target.kind == ["bin"] && unit.mode == "test"));
    let optional = graph.units.iter().filter(|unit| unit.target.name == "feature_optional").collect::<Vec<_>>();
    assert!(!optional.is_empty(), "optional dependency must remain selected");
    assert!(
        optional.iter().all(|unit| unit.features == ["leaf"]),
        "unselected dependency features must not be enabled"
    );
    for row in [&[][..], &COMPLETE_FEATURES[..1], &COMPLETE_FEATURES[1..]] {
        assert!(plan(root.path(), row, false).expect_err("under-declared row").contains("does not match"));
    }
    assert!(
        graph.units.iter().any(|unit| unit.mode == "run-custom-build"),
        "the failing build script must be planned but never executed"
    );
}

// r[verify octet.architecture.features.rejection]
#[test]
fn no_default_features_still_requires_the_actual_complete_feature_set() {
    let root = fixture();
    assert!(
        plan(root.path(), COMPLETE_FEATURES, true)
            .expect_err("default feature difference")
            .contains("does not match")
    );
    let mut explicit_default = COMPLETE_FEATURES.to_vec();
    explicit_default.push("feature-app/default");
    plan(root.path(), &explicit_default, true).expect("explicitly reviewed default feature");
}

// r[verify octet.architecture.features.equivalence]
#[test]
fn all_features_selects_the_one_equivalent_row_of_a_feature_matrix() {
    let root = fixture();
    let complete = row(COMPLETE_FEATURES, true);
    let partial = row(&COMPLETE_FEATURES[..1], true);
    let empty = row(&[], true);
    let (selected, _) = select(root.path(), &[&partial, &complete, &empty], false).expect("one equivalent row");
    assert_eq!(selected, &complete);
    assert!(
        select(root.path(), &[&partial, &empty], false)
            .expect_err("no equivalent row")
            .contains("does not match")
    );

    let mut with_default = COMPLETE_FEATURES.to_vec();
    with_default.push("feature-app/default");
    let redundant = row(&with_default, true);
    assert!(
        select(root.path(), &[&complete, &redundant], false)
            .expect_err("two equivalent rows")
            .contains("more than one")
    );
}

use std::path::Path;
use std::path::PathBuf;

use crate::cli::SupplementFamilyArg;
use crate::cli::SupplementScopeArg;
use crate::determinism;

pub(crate) const RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH: &str = "cargo-octet/tests/fixtures/hir-spike/checked.rs";
pub(crate) const RUSTC_HIR_SPIKE_FUNCTION: &str = "checked";
pub(crate) const RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH: &str = "cargo-octet/tests/fixtures/hir-spike/clamp.rs";
pub(crate) const RUSTC_HIR_SPIKE_BRANCH_FUNCTION: &str = "clamp_to_limit";
pub(crate) const RUSTC_THIR_SPIKE_FIXTURE_RELATIVE_PATH: &str = "cargo-octet/tests/fixtures/thir-spike/checked.rs";
pub(crate) const RUSTC_THIR_SPIKE_FUNCTION: &str = "checked";
pub(crate) const RUSTC_THIR_SPIKE_CALL_FIXTURE_RELATIVE_PATH: &str =
    "cargo-octet/tests/fixtures/thir-spike/call_return.rs";
pub(crate) const RUSTC_THIR_SPIKE_CALL_FUNCTION: &str = "call_then_return";
pub(crate) const RUSTC_THIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH: &str =
    "cargo-octet/tests/fixtures/thir-spike/branch_control_flow.rs";
pub(crate) const RUSTC_THIR_SPIKE_BRANCH_FUNCTION: &str = "choose_branch";
pub(crate) const RUSTC_MIR_SPIKE_FIXTURE_RELATIVE_PATH: &str = "cargo-octet/tests/fixtures/mir-spike/checked.rs";
pub(crate) const RUSTC_MIR_SPIKE_FUNCTION: &str = "checked";
pub(crate) const RUSTC_MIR_SPIKE_CALL_FIXTURE_RELATIVE_PATH: &str =
    "cargo-octet/tests/fixtures/mir-spike/call_return.rs";
pub(crate) const RUSTC_MIR_SPIKE_CALL_FUNCTION: &str = "call_then_return";
pub(crate) const RUSTC_MIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH: &str =
    "cargo-octet/tests/fixtures/mir-spike/branch_control_flow.rs";
pub(crate) const RUSTC_MIR_SPIKE_BRANCH_FUNCTION: &str = "choose_branch";

const CURRENT_DIR_ARG: &str = ".";
const CHECKED_SHAPE_ALIAS: &str = "checked";
const CHECKED_ARITHMETIC_SHAPE: &str = "checked-arithmetic";
const BRANCH_SHAPE_ALIAS: &str = "branch";
const BRANCH_CONTROL_FLOW_SHAPE: &str = "branch-control-flow";
const CLAMP_SHAPE_ALIAS: &str = "clamp";
const CALL_RETURN_SHAPE: &str = "call-return";
const HIR_EXPECTED_SHAPES: &str = "`checked-arithmetic` or `branch-control-flow`";
const THIR_MIR_EXPECTED_SHAPES: &str = "`checked-arithmetic`, `call-return`, or `branch-control-flow`";
const CHECKED_FIXTURE_INDEX: usize = 0;
const BRANCH_FIXTURE_INDEX: usize = 1;
const CALL_RETURN_FIXTURE_INDEX: usize = 1;
const THIR_MIR_BRANCH_FIXTURE_INDEX: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct SupplementFixtureSpec {
    pub(crate) relative_path: &'static str,
    pub(crate) function: &'static str,
    pub(crate) shape: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SupplementFixtureSelection {
    pub(crate) path: PathBuf,
    pub(crate) function: String,
}

pub(crate) const RUSTC_HIR_SPIKE_FIXTURES: &[SupplementFixtureSpec] = &[
    SupplementFixtureSpec {
        relative_path: RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH,
        function: RUSTC_HIR_SPIKE_FUNCTION,
        shape: CHECKED_ARITHMETIC_SHAPE,
    },
    SupplementFixtureSpec {
        relative_path: RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH,
        function: RUSTC_HIR_SPIKE_BRANCH_FUNCTION,
        shape: BRANCH_CONTROL_FLOW_SHAPE,
    },
];

pub(crate) const RUSTC_THIR_SPIKE_FIXTURES: &[SupplementFixtureSpec] = &[
    SupplementFixtureSpec {
        relative_path: RUSTC_THIR_SPIKE_FIXTURE_RELATIVE_PATH,
        function: RUSTC_THIR_SPIKE_FUNCTION,
        shape: CHECKED_ARITHMETIC_SHAPE,
    },
    SupplementFixtureSpec {
        relative_path: RUSTC_THIR_SPIKE_CALL_FIXTURE_RELATIVE_PATH,
        function: RUSTC_THIR_SPIKE_CALL_FUNCTION,
        shape: CALL_RETURN_SHAPE,
    },
    SupplementFixtureSpec {
        relative_path: RUSTC_THIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH,
        function: RUSTC_THIR_SPIKE_BRANCH_FUNCTION,
        shape: BRANCH_CONTROL_FLOW_SHAPE,
    },
];

pub(crate) const RUSTC_MIR_SPIKE_FIXTURES: &[SupplementFixtureSpec] = &[
    SupplementFixtureSpec {
        relative_path: RUSTC_MIR_SPIKE_FIXTURE_RELATIVE_PATH,
        function: RUSTC_MIR_SPIKE_FUNCTION,
        shape: CHECKED_ARITHMETIC_SHAPE,
    },
    SupplementFixtureSpec {
        relative_path: RUSTC_MIR_SPIKE_CALL_FIXTURE_RELATIVE_PATH,
        function: RUSTC_MIR_SPIKE_CALL_FUNCTION,
        shape: CALL_RETURN_SHAPE,
    },
    SupplementFixtureSpec {
        relative_path: RUSTC_MIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH,
        function: RUSTC_MIR_SPIKE_BRANCH_FUNCTION,
        shape: BRANCH_CONTROL_FLOW_SHAPE,
    },
];

pub(crate) fn validate_fixture_inventory_scope(
    family: SupplementFamilyArg,
    scope: SupplementScopeArg,
) -> Result<(), String> {
    match (family, scope) {
        (SupplementFamilyArg::Hir, SupplementScopeArg::RustcHirFixtureSpike)
        | (SupplementFamilyArg::Thir, SupplementScopeArg::RustcThirFixtureSpike)
        | (SupplementFamilyArg::Mir, SupplementScopeArg::RustcMirFixtureSpike) => Ok(()),
        (SupplementFamilyArg::Hir, _) => {
            Err("object supplement fixtures for `--family hir` requires `--spike rustc-hir-fixture`".to_owned())
        }
        (SupplementFamilyArg::Thir, _) => {
            Err("object supplement fixtures for `--family thir` requires `--spike rustc-thir-fixture`".to_owned())
        }
        (SupplementFamilyArg::Mir, _) => {
            Err("object supplement fixtures for `--family mir` requires `--spike rustc-mir-fixture`".to_owned())
        }
    }
}

pub(crate) fn fixture_inventory_specs(
    family: SupplementFamilyArg,
    scope: SupplementScopeArg,
) -> Result<&'static [SupplementFixtureSpec], String> {
    validate_fixture_inventory_scope(family, scope)?;
    Ok(match (family, scope) {
        (SupplementFamilyArg::Hir, SupplementScopeArg::RustcHirFixtureSpike) => RUSTC_HIR_SPIKE_FIXTURES,
        (SupplementFamilyArg::Thir, SupplementScopeArg::RustcThirFixtureSpike) => RUSTC_THIR_SPIKE_FIXTURES,
        (SupplementFamilyArg::Mir, SupplementScopeArg::RustcMirFixtureSpike) => RUSTC_MIR_SPIKE_FIXTURES,
        _ => unreachable!("scope validated above"),
    })
}

pub(crate) fn parse_supplement_fixture_shape(
    scope: SupplementScopeArg,
    value: &str,
) -> Result<SupplementFixtureSelection, String> {
    let fixture = match scope {
        SupplementScopeArg::RustcHirFixtureSpike => match value {
            CHECKED_SHAPE_ALIAS | CHECKED_ARITHMETIC_SHAPE => RUSTC_HIR_SPIKE_FIXTURES[CHECKED_FIXTURE_INDEX],
            BRANCH_SHAPE_ALIAS | BRANCH_CONTROL_FLOW_SHAPE | CLAMP_SHAPE_ALIAS => {
                RUSTC_HIR_SPIKE_FIXTURES[BRANCH_FIXTURE_INDEX]
            }
            other => {
                return Err(format!(
                    "unsupported HIR supplement fixture shape `{other}`; expected {HIR_EXPECTED_SHAPES}"
                ));
            }
        },
        SupplementScopeArg::RustcThirFixtureSpike => match value {
            CHECKED_SHAPE_ALIAS | CHECKED_ARITHMETIC_SHAPE => RUSTC_THIR_SPIKE_FIXTURES[CHECKED_FIXTURE_INDEX],
            CALL_RETURN_SHAPE => RUSTC_THIR_SPIKE_FIXTURES[CALL_RETURN_FIXTURE_INDEX],
            BRANCH_SHAPE_ALIAS | BRANCH_CONTROL_FLOW_SHAPE | CLAMP_SHAPE_ALIAS => {
                RUSTC_THIR_SPIKE_FIXTURES[THIR_MIR_BRANCH_FIXTURE_INDEX]
            }
            other => {
                return Err(format!(
                    "unsupported THIR supplement fixture shape `{other}`; expected {THIR_MIR_EXPECTED_SHAPES}"
                ));
            }
        },
        SupplementScopeArg::RustcMirFixtureSpike => match value {
            CHECKED_ARITHMETIC_SHAPE => RUSTC_MIR_SPIKE_FIXTURES[CHECKED_FIXTURE_INDEX],
            CALL_RETURN_SHAPE => RUSTC_MIR_SPIKE_FIXTURES[CALL_RETURN_FIXTURE_INDEX],
            BRANCH_SHAPE_ALIAS | BRANCH_CONTROL_FLOW_SHAPE | CLAMP_SHAPE_ALIAS => {
                RUSTC_MIR_SPIKE_FIXTURES[THIR_MIR_BRANCH_FIXTURE_INDEX]
            }
            other => {
                return Err(format!(
                    "unsupported MIR supplement fixture shape `{other}`; expected {THIR_MIR_EXPECTED_SHAPES}"
                ));
            }
        },
        SupplementScopeArg::Pilot => {
            return Err(format!("unsupported supplement fixture shape `{value}`; expected {HIR_EXPECTED_SHAPES}"));
        }
    };
    Ok(SupplementFixtureSelection {
        path: PathBuf::from(fixture.relative_path),
        function: fixture.function.to_owned(),
    })
}

pub(crate) fn selected_fixture_path(scope: SupplementScopeArg, paths: &[PathBuf]) -> Result<PathBuf, String> {
    match scope {
        SupplementScopeArg::RustcHirFixtureSpike => hir_fixture_path(paths),
        SupplementScopeArg::RustcThirFixtureSpike => explicit_fixture_path(
            paths,
            "rustc THIR fixture spike requires explicit fixture selector `--fixture-shape checked-arithmetic` or `--fixture-shape call-return`",
            "rustc THIR fixture spike accepts exactly one fixture path",
        ),
        SupplementScopeArg::RustcMirFixtureSpike => explicit_fixture_path(
            paths,
            "rustc MIR fixture spike requires explicit fixture selector `--fixture-shape checked-arithmetic`, `--fixture-shape call-return`, or `--fixture-shape branch-control-flow`",
            "rustc MIR fixture spike accepts exactly one fixture path",
        ),
        SupplementScopeArg::Pilot => Err("pilot supplement scope does not use selected rustc fixture paths".to_owned()),
    }
}

pub(crate) fn fixture_for(
    scope: SupplementScopeArg,
    fixture_path: &Path,
    function: &str,
) -> Result<SupplementFixtureSpec, String> {
    let normalized_path = determinism::normalize_path(fixture_path);
    let fixtures = fixtures_for_scope(scope)?;
    fixtures
        .iter()
        .copied()
        .find(|fixture| fixture.relative_path == normalized_path && fixture.function == function)
        .ok_or_else(|| bounded_fixture_error(scope, fixtures, &normalized_path, function))
}

fn hir_fixture_path(paths: &[PathBuf]) -> Result<PathBuf, String> {
    if paths_request_default(paths) {
        return Ok(PathBuf::from(RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH));
    }
    explicit_fixture_path(paths, "", "rustc HIR fixture spike accepts exactly one fixture path")
}

fn explicit_fixture_path(
    paths: &[PathBuf],
    missing_selector_error: &str,
    too_many_paths_error: &str,
) -> Result<PathBuf, String> {
    if paths_request_default(paths) {
        return Err(missing_selector_error.to_owned());
    }
    if paths.len() != 1 {
        return Err(too_many_paths_error.to_owned());
    }
    Ok(paths[0].clone())
}

fn paths_request_default(paths: &[PathBuf]) -> bool {
    paths.is_empty() || paths == [PathBuf::from(CURRENT_DIR_ARG)]
}

fn fixtures_for_scope(scope: SupplementScopeArg) -> Result<&'static [SupplementFixtureSpec], String> {
    match scope {
        SupplementScopeArg::RustcHirFixtureSpike => Ok(RUSTC_HIR_SPIKE_FIXTURES),
        SupplementScopeArg::RustcThirFixtureSpike => Ok(RUSTC_THIR_SPIKE_FIXTURES),
        SupplementScopeArg::RustcMirFixtureSpike => Ok(RUSTC_MIR_SPIKE_FIXTURES),
        SupplementScopeArg::Pilot => Err("pilot supplement scope has no fixture-bound rustc fixture set".to_owned()),
    }
}

fn bounded_fixture_error(
    scope: SupplementScopeArg,
    fixtures: &[SupplementFixtureSpec],
    normalized_path: &str,
    function: &str,
) -> String {
    let allowed = fixtures
        .iter()
        .map(|fixture| format!("{}::{}", fixture.relative_path, fixture.function))
        .collect::<Vec<_>>()
        .join(", ");
    match scope {
        SupplementScopeArg::RustcHirFixtureSpike => format!(
            "rustc HIR spike is fixture-bounded to checked-in fixture/function pairs [{allowed}], got {normalized_path}::{function}"
        ),
        SupplementScopeArg::RustcThirFixtureSpike => format!(
            "rustc THIR fixture contract is bounded to checked-in fixture/function pairs [{allowed}], got {normalized_path}::{function}"
        ),
        SupplementScopeArg::RustcMirFixtureSpike => format!(
            "rustc MIR fixture contract is bounded to checked-in fixture/function pairs [{allowed}], got {normalized_path}::{function}"
        ),
        SupplementScopeArg::Pilot => "pilot supplement scope has no fixture-bound rustc fixture set".to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fixture_shape_core_accepts_selected_positive_shapes() {
        let hir = parse_supplement_fixture_shape(SupplementScopeArg::RustcHirFixtureSpike, BRANCH_CONTROL_FLOW_SHAPE)
            .expect("HIR branch fixture shape");
        assert_eq!(hir.path, PathBuf::from(RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH));
        assert_eq!(hir.function, RUSTC_HIR_SPIKE_BRANCH_FUNCTION);

        let thir = parse_supplement_fixture_shape(SupplementScopeArg::RustcThirFixtureSpike, CALL_RETURN_SHAPE)
            .expect("THIR call-return fixture shape");
        assert_eq!(thir.path, PathBuf::from(RUSTC_THIR_SPIKE_CALL_FIXTURE_RELATIVE_PATH));
        assert_eq!(thir.function, RUSTC_THIR_SPIKE_CALL_FUNCTION);

        let mir = parse_supplement_fixture_shape(SupplementScopeArg::RustcMirFixtureSpike, CHECKED_ARITHMETIC_SHAPE)
            .expect("MIR checked fixture shape");
        assert_eq!(mir.path, PathBuf::from(RUSTC_MIR_SPIKE_FIXTURE_RELATIVE_PATH));
        assert_eq!(mir.function, RUSTC_MIR_SPIKE_FUNCTION);
    }

    #[test]
    fn fixture_shape_core_rejects_unsupported_shape_without_path_io() {
        let error = parse_supplement_fixture_shape(SupplementScopeArg::RustcThirFixtureSpike, "not-selected")
            .expect_err("unsupported fixture shape should fail closed");

        assert!(error.contains("unsupported THIR supplement fixture shape `not-selected`"));
        assert!(error.contains(THIR_MIR_EXPECTED_SHAPES));
    }

    #[test]
    fn fixture_path_core_accepts_hir_default_and_explicit_single_path() {
        let default_path =
            selected_fixture_path(SupplementScopeArg::RustcHirFixtureSpike, &[]).expect("HIR default fixture path");
        assert_eq!(default_path, PathBuf::from(RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH));

        let explicit_path = PathBuf::from(RUSTC_MIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH);
        let selected =
            selected_fixture_path(SupplementScopeArg::RustcMirFixtureSpike, std::slice::from_ref(&explicit_path))
                .expect("MIR explicit fixture path");
        assert_eq!(selected, explicit_path);
    }

    #[test]
    fn fixture_path_core_rejects_missing_or_ambiguous_fixture_paths() {
        let missing = selected_fixture_path(SupplementScopeArg::RustcMirFixtureSpike, &[])
            .expect_err("MIR fixture path requires explicit selector");
        assert!(missing.contains("requires explicit fixture selector"));

        let ambiguous = selected_fixture_path(SupplementScopeArg::RustcHirFixtureSpike, &[
            PathBuf::from(RUSTC_HIR_SPIKE_FIXTURE_RELATIVE_PATH),
            PathBuf::from(RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH),
        ])
        .expect_err("multiple fixture paths should fail closed");
        assert!(ambiguous.contains("accepts exactly one fixture path"));
    }

    #[test]
    fn fixture_inventory_core_rejects_unsupported_family_scope_pair() {
        let error =
            validate_fixture_inventory_scope(SupplementFamilyArg::Hir, SupplementScopeArg::RustcMirFixtureSpike)
                .expect_err("wrong fixture family/scope should fail closed");

        assert!(error.contains("requires `--spike rustc-hir-fixture`"));
    }

    #[test]
    fn fixture_lookup_core_accepts_registered_pair_and_rejects_mismatch() {
        let fixture = fixture_for(
            SupplementScopeArg::RustcHirFixtureSpike,
            Path::new(RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH),
            RUSTC_HIR_SPIKE_BRANCH_FUNCTION,
        )
        .expect("registered fixture/function pair");
        assert_eq!(fixture.shape, BRANCH_CONTROL_FLOW_SHAPE);

        let error = fixture_for(
            SupplementScopeArg::RustcHirFixtureSpike,
            Path::new(RUSTC_HIR_SPIKE_BRANCH_FIXTURE_RELATIVE_PATH),
            RUSTC_HIR_SPIKE_FUNCTION,
        )
        .expect_err("mismatched fixture/function pair should fail closed");
        assert!(error.contains(RUSTC_HIR_SPIKE_BRANCH_FUNCTION));
        assert!(error.contains(RUSTC_HIR_SPIKE_FUNCTION));
    }
}

use super::CompilerUnitObservation;
use super::UnitSelectionError;
use super::select_compilation_unit;
use super::validate_collector_units;
use crate::CollectorCompilationUnit;

fn selected(test_configuration: bool) -> CollectorCompilationUnit {
    let mut unit = crate::tests::unit();
    unit.source_path = "crates/fixture/src/lib.rs".to_owned();
    if test_configuration {
        unit.target_kind = "lib+test".to_owned();
        unit.crate_types = vec!["bin".to_owned()];
    }
    unit.unit_id = crate::compilation_unit_identity(&unit);
    CollectorCompilationUnit {
        rustc_crate_name: "fixture".to_owned(),
        manifest_dir: "crates/fixture".to_owned(),
        source_path: unit.source_path.clone(),
        test_configuration,
        source_roles: vec![crate::SourceRoleScope {
            source_scope: "crates/fixture/src".to_owned(),
            role: unit.role,
        }],
        unit,
    }
}

fn observation(selected: &CollectorCompilationUnit) -> CompilerUnitObservation<'_> {
    CompilerUnitObservation {
        crate_name: &selected.rustc_crate_name,
        manifest_dir: &selected.manifest_dir,
        source_path: &selected.source_path,
        test_configuration: selected.test_configuration,
        target_triple: &selected.unit.target_triple,
        features: &selected.unit.features,
    }
}

#[test]
fn same_crate_normal_and_test_sessions_select_distinct_units() {
    let normal = selected(false);
    let test = selected(true);
    let units = vec![normal.clone(), test.clone()];
    assert_eq!(validate_collector_units(&units), Ok(()));
    assert_eq!(select_compilation_unit(&units, &observation(&normal)), Ok(Some(&units[0])));
    assert_eq!(select_compilation_unit(&units, &observation(&test)), Ok(Some(&units[1])));
}

#[test]
fn same_name_binary_and_library_match_source_roots() {
    let library = selected(false);
    let mut binary = library.clone();
    binary.source_path = "crates/fixture/src/main.rs".to_owned();
    binary.unit.source_path = binary.source_path.clone();
    binary.unit.target_kind = "bin".to_owned();
    binary.unit.unit_id = crate::compilation_unit_identity(&binary.unit);
    let units = vec![library, binary.clone()];
    assert_eq!(validate_collector_units(&units), Ok(()));
    assert_eq!(select_compilation_unit(&units, &observation(&binary)), Ok(Some(&units[1])));
}

#[test]
fn duplicate_observation_keys_are_rejected_even_with_different_unit_ids() {
    let normal = selected(false);
    let mut conflicting = normal.clone();
    conflicting.unit.unit_id = "another-unit".to_owned();
    let units = vec![normal.clone(), conflicting];
    assert_eq!(validate_collector_units(&units), Err(UnitSelectionError::AmbiguousUnit));
    assert_eq!(select_compilation_unit(&units, &observation(&normal)), Err(UnitSelectionError::AmbiguousUnit));
}

#[test]
fn empty_context_and_empty_source_are_rejected() {
    assert_eq!(validate_collector_units(&[]), Err(UnitSelectionError::InvalidContext));
    let mut invalid = selected(false);
    invalid.source_path.clear();
    assert_eq!(validate_collector_units(&[invalid]), Err(UnitSelectionError::InvalidContext));
}

#[test]
fn every_observed_binding_is_required() {
    let normal = selected(false);
    let units = vec![normal.clone()];
    let mut wrong = observation(&normal);
    wrong.crate_name = "another";
    assert_eq!(select_compilation_unit(&units, &wrong), Err(UnitSelectionError::UnexpectedUnit));
    wrong = observation(&normal);
    wrong.source_path = "crates/fixture/src/main.rs";
    assert_eq!(select_compilation_unit(&units, &wrong), Err(UnitSelectionError::UnexpectedUnit));
    wrong = observation(&normal);
    wrong.test_configuration = true;
    assert_eq!(select_compilation_unit(&units, &wrong), Err(UnitSelectionError::UnexpectedUnit));
    wrong = observation(&normal);
    wrong.target_triple = "aarch64-unknown-linux-gnu";
    assert_eq!(select_compilation_unit(&units, &wrong), Err(UnitSelectionError::UnexpectedUnit));
    wrong = observation(&normal);
    wrong.features = &[];
    assert_eq!(select_compilation_unit(&units, &wrong), Err(UnitSelectionError::UnexpectedUnit));
}

#[test]
fn external_package_does_not_match_a_selected_crate_name() {
    let normal = selected(false);
    let mut external = observation(&normal);
    external.manifest_dir = "vendor/fixture";
    assert_eq!(select_compilation_unit(std::slice::from_ref(&normal), &external), Ok(None));
}

// r[verify octet.project_architecture_gate.test_ownership]
#[test]
fn resealed_root_drift_and_test_promotion_are_invalid_context() {
    let normal = selected(false);
    let mut drift = normal.clone();
    drift.unit.source_path = "tests/unobserved.rs".to_owned();
    drift.unit.unit_id = crate::compilation_unit_identity(&drift.unit);
    assert_eq!(select_compilation_unit(&[drift], &observation(&normal)), Err(UnitSelectionError::InvalidContext));
    for kind in ["lib", "lib+test", "bin+test"] {
        let mut forged = selected(kind.ends_with("+test"));
        forged.unit.target_kind = kind.to_owned();
        forged.unit.role = crate::CompilerArchitectureRole::Test;
        forged.source_roles[0].role = crate::CompilerArchitectureRole::Test;
        forged.unit.unit_id = crate::compilation_unit_identity(&forged.unit);
        assert_eq!(validate_collector_units(&[forged]), Err(UnitSelectionError::InvalidContext));
    }
    let mut test = selected(true);
    test.unit.target_kind = "test+test".to_owned();
    test.unit.role = crate::CompilerArchitectureRole::Test;
    test.source_roles[0].role = crate::CompilerArchitectureRole::Test;
    test.unit.unit_id = crate::compilation_unit_identity(&test.unit);
    assert_eq!(validate_collector_units(&[test.clone()]), Ok(()));
    test.test_configuration = false;
    assert_eq!(validate_collector_units(&[test]), Err(UnitSelectionError::InvalidContext));
}

#[test]
fn feature_duplicates_are_invalid_context() {
    let mut invalid = selected(false);
    invalid.unit.features.push(invalid.unit.features[0].clone());
    assert_eq!(validate_collector_units(&[invalid]), Err(UnitSelectionError::InvalidContext));
}

#[test]
fn missing_configuration_fields_do_not_default_to_a_normal_library() {
    let normal = selected(false);
    let mut json = serde_json::to_value(&normal).expect("serialize context unit");
    json.as_object_mut().expect("object").remove("test_configuration");
    assert!(serde_json::from_value::<CollectorCompilationUnit>(json).is_err());
}

//! Executed corpus tests for the bounded four-version merge relation.

use std::collections::BTreeMap;

use octet_merge_rail::checker::admission_identity;
use octet_merge_rail::checker::check_request;
use octet_merge_rail::corpus::corpus;
use octet_merge_rail::corpus::run_experiment;
use octet_merge_rail::model::CounterexampleClass;
use octet_merge_rail::model::Outcome;
use octet_merge_rail::model::PijulProvenance;
use octet_merge_rail::model::Value;
use octet_merge_rail::receipt::NON_CLAIMS;
use octet_merge_rail::receipt::validate_receipt;

#[test]
fn corpus_matches_every_independent_expectation() {
    let report = run_experiment();
    assert_eq!(report.metrics.cases, 6);
    assert_eq!(report.metrics.matched_expected, 6);
    assert_eq!(report.metrics.precision.numerator, 6);
    assert_eq!(report.metrics.precision.denominator, 6);
    assert_eq!(report.metrics.coverage.numerator, 6);
    assert_eq!(report.metrics.coverage.denominator, 6);
    let counts = BTreeMap::from([
        ("verified".to_owned(), 3usize),
        ("counterexample".to_owned(), 3usize),
        ("unknown".to_owned(), 0usize),
        ("unsupported".to_owned(), 0usize),
        ("timeout".to_owned(), 0usize),
        ("tool-error".to_owned(), 0usize),
    ]);
    assert_eq!(report.metrics.outcome_counts, counts);
    assert!(report.metrics.total_work_units > 0);
    for case in &report.cases {
        assert!(case.matched, "case {} mismatched: {} != {}", case.name, case.actual, case.expected);
        assert!(!case.receipt_identity.is_empty());
        assert!(!case.input_identity.is_empty());
    }
}

#[test]
fn corpus_is_deterministic() {
    let first = serde_json::to_string_pretty(&run_experiment()).expect("serialize first report");
    let second = serde_json::to_string_pretty(&run_experiment()).expect("serialize second report");
    assert_eq!(first, second);
}

#[test]
fn every_corpus_receipt_validates_against_its_admission() {
    for case in corpus() {
        let receipt = check_request(&case.request).expect("corpus case must admit");
        let admission = admission_identity(&case.request).expect("corpus case must admit");
        validate_receipt(&receipt, &admission).expect("receipt must validate");
        assert_eq!(receipt.input_identity, admission);
        assert_eq!(receipt.non_claims.len(), NON_CLAIMS.len());
        assert_eq!(receipt.roles.len(), 4);
        assert_eq!(receipt.bindings.len(), 4);
    }
}

#[test]
fn verified_case_records_observable_coverage_and_work() {
    let case = corpus()
        .into_iter()
        .find(|case| case.name == "independent-changes")
        .expect("independent-changes case");
    let receipt = check_request(&case.request).expect("case must admit");
    match receipt.outcome {
        Outcome::Verified { observables, inputs } => {
            assert_eq!(observables, 2);
            assert_eq!(inputs, 4);
        }
        other => panic!("expected verified, got {other:?}"),
    }
    assert_eq!(receipt.work.evaluations, 16);
    assert_eq!(receipt.work.relation_checks, 4);
    assert!(receipt.dependencies.is_empty());
    assert!(receipt.solver.is_none());
}

#[test]
fn text_clean_interference_is_detected_with_exact_values() {
    let case = corpus()
        .into_iter()
        .find(|case| case.name == "text-clean-semantic-interference")
        .expect("text-clean case");
    let receipt = check_request(&case.request).expect("case must admit");
    let Outcome::Counterexample {
        counterexamples,
        truncated,
    } = receipt.outcome
    else {
        panic!("expected a counterexample");
    };
    assert!(!truncated);
    let first = counterexamples.first().expect("one counterexample");
    assert_eq!(first.observable, "f");
    assert_eq!(first.class, CounterexampleClass::Interference);
    assert_eq!(first.base, Value::Int(1));
    assert_eq!(first.a, Value::Int(1));
    assert_eq!(first.b, Value::Int(1));
    assert_eq!(first.merge, Value::Int(2));
}

#[test]
fn incompatible_change_binds_both_conflicting_values() {
    let case = corpus().into_iter().find(|case| case.name == "incompatible-changes").expect("incompatible case");
    let receipt = check_request(&case.request).expect("case must admit");
    let Outcome::Counterexample { counterexamples, .. } = receipt.outcome else {
        panic!("expected a counterexample");
    };
    let first = counterexamples.first().expect("one counterexample");
    assert_eq!(first.class, CounterexampleClass::Incompatible);
    assert_eq!(first.base, Value::Int(0));
    assert_eq!(first.a, Value::Int(1));
    assert_eq!(first.b, Value::Int(2));
    assert_eq!(first.merge, Value::Int(1));
}

#[test]
fn lost_change_class_is_bound_to_the_changed_role() {
    let case = corpus().into_iter().find(|case| case.name == "lost-changes").expect("lost-changes case");
    let receipt = check_request(&case.request).expect("case must admit");
    let Outcome::Counterexample { counterexamples, .. } = receipt.outcome else {
        panic!("expected a counterexample");
    };
    let first = counterexamples.first().expect("one counterexample");
    assert_eq!(first.observable, "f");
    assert_eq!(first.class, CounterexampleClass::LostAChange);
}

#[test]
fn pijul_provenance_is_recorded_but_not_evaluated() {
    let mut case = corpus().into_iter().find(|case| case.name == "identical").expect("identical case");
    let provenance = PijulProvenance {
        change_identities: vec!["pijul-change-identity".to_owned()],
        note: "recorded-only native provenance".to_owned(),
    };
    case.request.provenance = Some(provenance.clone());
    let receipt = check_request(&case.request).expect("case must admit");
    assert_eq!(receipt.provenance, Some(provenance));
    assert!(matches!(receipt.outcome, Outcome::Verified { .. }));
    validate_receipt(&receipt, &receipt.input_identity).expect("receipt must validate");
}

#[test]
fn corpus_expectations_are_not_derived_from_actual_outcomes() {
    // The expectation table is authored independently; this test pins the
    // reviewed labels so a silent expectation edit is visible in review.
    let labels = corpus().iter().map(|case| (case.name, case.expected.label())).collect::<Vec<_>>();
    assert_eq!(labels, vec![
        ("identical", "verified".to_owned()),
        ("independent-changes", "verified".to_owned()),
        ("equivalent-edits", "verified".to_owned()),
        ("incompatible-changes", "counterexample:incompatible@f".to_owned()),
        ("lost-changes", "counterexample:lost-a-change@f".to_owned()),
        ("text-clean-semantic-interference", "counterexample:interference@f".to_owned()),
    ]);
}

//! Negative controls for the bounded four-version merge relation.

mod common;

use octet_merge_rail::checker::admission_identity;
use octet_merge_rail::checker::check_request;
use octet_merge_rail::checker::check_request_with_backend;
use octet_merge_rail::evaluate::Backend;
use octet_merge_rail::evaluate::BackendError;
use octet_merge_rail::evaluate::BackendIdentity;
use octet_merge_rail::evaluate::EvalContext;
use octet_merge_rail::evaluate::EvalOutcome;
use octet_merge_rail::evaluate::ResolvedProgram;
use octet_merge_rail::model::Budget;
use octet_merge_rail::model::Expr;
use octet_merge_rail::model::Function;
use octet_merge_rail::model::Outcome;
use octet_merge_rail::model::Role;
use octet_merge_rail::model::TimeoutPhase;
use octet_merge_rail::model::UnknownReason;
use octet_merge_rail::model::UnsupportedConstruct;
use octet_merge_rail::model::UnsupportedReason;
use octet_merge_rail::model::Value;
use octet_merge_rail::receipt::NON_CLAIMS;
use octet_merge_rail::receipt::validate_receipt;

#[test]
fn missing_observable_is_unsupported_not_verified() {
    let mut request = common::base_request();
    for variant in &mut request.variants {
        if variant.role == Role::A {
            variant.program.remove("f");
        }
    }
    common::reseal(&mut request);
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, .. } => assert_eq!(*reason, UnsupportedReason::MissingObservable),
        other => panic!("expected unsupported, got {other:?}"),
    }
    assert!(receipt.non_claims.contains(&NON_CLAIMS[0].to_owned()));
    validate_receipt(&receipt, &receipt.input_identity).expect("unsupported receipt is well formed");
}

#[test]
fn vacuous_precondition_is_unknown_not_verified() {
    let mut request = common::base_request();
    request.observables[0].domain.precondition = vec![
        octet_merge_rail::model::Predicate::new(0, octet_merge_rail::model::CompareOp::GreaterOrEqual, 1),
        octet_merge_rail::model::Predicate::new(0, octet_merge_rail::model::CompareOp::LessOrEqual, 0),
    ];
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unknown { reason, .. } => assert_eq!(*reason, UnknownReason::VacuousPrecondition),
        other => panic!("expected unknown, got {other:?}"),
    }
    validate_receipt(&receipt, &receipt.input_identity).expect("unknown receipt is well formed");
}

#[test]
fn changed_callee_is_unsupported_without_an_observable_mapping() {
    let mut request = common::callee_request();
    common::set_body(&mut request, Role::A, "h", &Expr::plus(Expr::Arg(0), Expr::Int(2)));
    common::reseal(&mut request);
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, .. } => assert_eq!(*reason, UnsupportedReason::ChangedDependency),
        other => panic!("expected unsupported, got {other:?}"),
    }
    assert!(receipt.dependencies.is_empty());
}

#[test]
fn wrong_source_binding_is_rejected() {
    let mut request = common::base_request();
    for variant in &mut request.variants {
        variant.source_text = format!("{}: f(x) = x + 1", variant.role.as_str());
    }
    common::reseal(&mut request);
    let replacement = request
        .bindings
        .iter()
        .find(|binding| binding.role == Role::B)
        .expect("role b binding")
        .source_identity
        .clone();
    request
        .bindings
        .iter_mut()
        .find(|binding| binding.role == Role::A)
        .expect("role a binding")
        .source_identity = replacement;
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, .. } => assert_eq!(*reason, UnsupportedReason::BindingMismatch),
        other => panic!("expected unsupported, got {other:?}"),
    }
}

#[test]
fn wrong_subject_binding_is_rejected() {
    let mut request = common::base_request();
    common::set_body(&mut request, Role::A, "f", &Expr::plus(Expr::Arg(0), Expr::Int(2)));
    common::reseal(&mut request);
    let replacement = request
        .bindings
        .iter()
        .find(|binding| binding.role == Role::Merge)
        .expect("merge binding")
        .subject_identity
        .clone();
    request
        .bindings
        .iter_mut()
        .find(|binding| binding.role == Role::A)
        .expect("role a binding")
        .subject_identity = replacement;
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, .. } => assert_eq!(*reason, UnsupportedReason::BindingMismatch),
        other => panic!("expected unsupported, got {other:?}"),
    }
}

#[test]
fn swapped_roles_are_detected() {
    let mut request = common::base_request();
    for variant in &mut request.variants {
        let (source, body) = match variant.role {
            Role::A => ("a: f(x) = x + 2", Expr::plus(Expr::Arg(0), Expr::Int(2))),
            Role::B => ("b: f(x) = x + 3", Expr::plus(Expr::Arg(0), Expr::Int(3))),
            _ => continue,
        };
        variant.source_text = source.to_owned();
        variant.program.insert("f".to_owned(), Function::new(1, body));
    }
    common::reseal(&mut request);
    let a_index = request.variants.iter().position(|variant| variant.role == Role::A).expect("role a");
    let b_index = request.variants.iter().position(|variant| variant.role == Role::B).expect("role b");
    let a_text = std::mem::take(&mut request.variants[a_index].source_text);
    let b_text = std::mem::replace(&mut request.variants[b_index].source_text, a_text);
    request.variants[a_index].source_text = b_text;
    let a_program = std::mem::take(&mut request.variants[a_index].program);
    let b_program = std::mem::replace(&mut request.variants[b_index].program, a_program);
    request.variants[a_index].program = b_program;
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, .. } => assert_eq!(*reason, UnsupportedReason::RoleMismatch),
        other => panic!("expected unsupported, got {other:?}"),
    }
}

#[test]
fn unsupported_construct_is_reported_without_evaluation() {
    let mut request = common::base_request();
    common::set_body(&mut request, Role::A, "f", &Expr::Unsupported {
        construct: UnsupportedConstruct::MacroExpansion,
    });
    common::reseal(&mut request);
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, detail } => {
            assert_eq!(*reason, UnsupportedReason::UnsupportedConstruct);
            assert!(detail.contains("macro-expansion"));
        }
        other => panic!("expected unsupported, got {other:?}"),
    }
}

#[test]
fn interrupted_verification_reports_timeout_not_a_conflict() {
    let mut request = common::base_request();
    request.budget = Budget::new(1, 4_096);
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Timeout {
            phase,
            completed,
            limit,
        } => {
            assert_eq!(*phase, TimeoutPhase::Evaluation);
            assert_eq!(*completed, 1);
            assert_eq!(*limit, 1);
        }
        other => panic!("expected timeout, got {other:?}"),
    }
    assert_eq!(receipt.work.evaluations, 1);
    assert_eq!(receipt.work.relation_checks, 0);
    validate_receipt(&receipt, &receipt.input_identity).expect("timeout receipt is well formed");
}

#[test]
fn backend_failure_is_a_tool_error_not_a_semantic_conflict() {
    struct FaultBackend;

    impl Backend for FaultBackend {
        fn identity(&self) -> BackendIdentity {
            BackendIdentity {
                id: "fault-injection-backend".to_owned(),
                version: "1".to_owned(),
            }
        }

        fn evaluate(
            &self,
            _program: &ResolvedProgram,
            _item: &str,
            _args: &[Value],
            _context: &EvalContext,
        ) -> Result<EvalOutcome, BackendError> {
            Err(BackendError {
                message: "injected backend failure".to_owned(),
            })
        }
    }

    let request = common::base_request();
    let receipt = check_request_with_backend(&request, &FaultBackend).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::ToolError { backend, message } => {
            assert_eq!(backend, "fault-injection-backend");
            assert_eq!(message, "injected backend failure");
        }
        other => panic!("expected tool-error, got {other:?}"),
    }
    validate_receipt(&receipt, &receipt.input_identity).expect("tool-error receipt is well formed");
}

#[test]
fn stale_receipt_is_rejected_against_a_changed_request() {
    let request = common::base_request();
    let receipt = check_request(&request).expect("request structure is valid");
    let own_admission = admission_identity(&request).expect("valid admission");
    validate_receipt(&receipt, &own_admission).expect("fresh receipt validates");

    let mut changed = request;
    changed.observables[0].domain.values = vec![0, 1];
    let changed_admission = admission_identity(&changed).expect("valid changed admission");
    assert_ne!(own_admission, changed_admission);
    let issues = validate_receipt(&receipt, &changed_admission).expect_err("stale receipt must fail");
    assert!(issues.iter().any(|issue| issue.code == "stale_input"));
}

#[test]
fn tampered_receipt_identity_is_rejected() {
    let request = common::base_request();
    let admission = admission_identity(&request).expect("valid admission");
    let mut receipt = check_request(&request).expect("request structure is valid");
    receipt.work.evaluations = receipt.work.evaluations.saturating_add(1);
    let issues = validate_receipt(&receipt, &admission).expect_err("tampered receipt must fail");
    assert!(issues.iter().any(|issue| issue.code == "receipt_identity_mismatch"));
}

#[test]
fn missing_call_mapping_is_unsupported() {
    let mut request = common::base_request();
    for variant in &mut request.variants {
        variant.program.insert("g".to_owned(), Function::new(1, Expr::plus(Expr::Arg(0), Expr::Int(1))));
        variant.program.get_mut("f").expect("f").body = Expr::call("g", vec![Expr::Arg(0)]);
    }
    common::reseal(&mut request);
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, .. } => assert_eq!(*reason, UnsupportedReason::MissingItemMapping),
        other => panic!("expected unsupported, got {other:?}"),
    }
}

#[test]
fn domain_overflow_is_unsupported_not_verified() {
    let mut request = common::base_request();
    request.observables[0].domain.values = vec![-1, 0, 1];
    request.observables[0].domain.max_cases = 1;
    let receipt = check_request(&request).expect("request structure is valid");
    match &receipt.outcome {
        Outcome::Unsupported { reason, .. } => assert_eq!(*reason, UnsupportedReason::DomainExceedsBound),
        other => panic!("expected unsupported, got {other:?}"),
    }
}

#[test]
fn duplicate_role_is_an_admission_error_without_a_verdict() {
    let mut request = common::base_request();
    let duplicate = request.variants[0].clone();
    request.variants.push(duplicate);
    let error = check_request(&request).expect_err("duplicate role must not admit");
    assert_eq!(error.code, "duplicate-role");
}

#[test]
fn missing_binding_is_an_admission_error_without_a_verdict() {
    let mut request = common::base_request();
    request.bindings.retain(|binding| binding.role != Role::Merge);
    let error = check_request(&request).expect_err("missing binding must not admit");
    assert_eq!(error.code, "missing-binding");
}

#[test]
fn missing_role_is_an_admission_error_without_a_verdict() {
    let mut request = common::base_request();
    request.variants.retain(|variant| variant.role != Role::Merge);
    let error = check_request(&request).expect_err("missing role must not admit");
    assert_eq!(error.code, "missing-role");
}

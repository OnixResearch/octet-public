//! Frontend correspondence and admission tests for the bounded Rust subset.

use std::collections::BTreeMap;

use octet_merge_rail::checker::SOURCE_DOMAIN;
use octet_merge_rail::checker::SUBJECT_DOMAIN;
use octet_merge_rail::checker::check_request;
use octet_merge_rail::corpus::assumptions;
use octet_merge_rail::corpus::budget;
use octet_merge_rail::frontend::FrontendError;
use octet_merge_rail::frontend::FunctionMapping;
use octet_merge_rail::frontend::parse_source;
use octet_merge_rail::identity::canonical_identity;
use octet_merge_rail::identity::framed_blake3;
use octet_merge_rail::identity::is_identity;
use octet_merge_rail::model::Expr;
use octet_merge_rail::model::InputDomain;
use octet_merge_rail::model::ItemMapping;
use octet_merge_rail::model::MergeRequest;
use octet_merge_rail::model::Observable;
use octet_merge_rail::model::Outcome;
use octet_merge_rail::model::Role;
use octet_merge_rail::model::RoleMap;
use octet_merge_rail::model::UnsupportedConstruct;
use octet_merge_rail::model::UnsupportedReason;
use octet_merge_rail::model::Variant;

const ADD_SOURCE: &str = "fn add(a: i64, b: i64) -> i64 { a + b }\n";
const CALL_SOURCE: &str = "fn choose(flag: i64, a: i64, b: i64) -> i64 { if flag { a } else { b } }\nfn call(x: i64) -> i64 { choose(x, 1, 2) }\n";

#[test]
fn parses_scalar_arithmetic_and_binds_correspondence() {
    let output = parse_source(ADD_SOURCE).expect("admitted source");
    assert_eq!(output.functions, vec![FunctionMapping {
        name: "add".to_owned(),
        params: 2
    }]);
    let function = output.program.get("add").expect("add function");
    assert_eq!(function.params, 2);
    assert_eq!(function.body, Expr::plus(Expr::Arg(0), Expr::Arg(1)));
    assert!(is_identity(&output.source_identity));
    assert!(is_identity(&output.subject_identity));
    assert_eq!(output.source_identity, framed_blake3(SOURCE_DOMAIN, &[ADD_SOURCE.as_bytes()]));
    assert_eq!(
        output.subject_identity,
        canonical_identity(SUBJECT_DOMAIN, &output.program).expect("subject identity")
    );
    let report = output.correspondence();
    assert!(report.obligation.contains("syntactic mapping"));
    assert_eq!(report.functions, output.functions);
    assert!(report.non_claims.iter().any(|claim| claim.contains("does not prove Rust semantics")));
}

#[test]
fn parses_conditionals_negation_and_direct_calls() {
    let output = parse_source(CALL_SOURCE).expect("admitted source");
    let choose = output.program.get("choose").expect("choose function");
    assert_eq!(choose.body, Expr::condition(Expr::Arg(0), Expr::Arg(1), Expr::Arg(2)));
    let call = output.program.get("call").expect("call function");
    assert_eq!(call.body, Expr::call("choose", vec![Expr::Arg(0), Expr::Int(1), Expr::Int(2)]));
    let negation = parse_source("fn negate(x: i64) -> i64 { -x }\n").expect("negation");
    assert_eq!(negation.program.get("negate").expect("negate").body, Expr::negate(Expr::Arg(0)));
}

#[test]
fn rejects_unsupported_constructs_fail_closed() {
    let cases = [
        ("fn f(x: i64) -> i64 { let y = x; y }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        (
            "fn f(x: i64) -> i64 { unsafe { x } }\n",
            "unsupported-construct",
            Some(UnsupportedConstruct::Unsafe),
        ),
        ("fn f(x: i64) -> i64 { loop { x } }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> i64 { x.abs() }\n", "unsupported-construct", Some(UnsupportedConstruct::Dispatch)),
        ("fn f(x: u64) -> i64 { 1 }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> i64 { x; x }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> i64 { if x { 1 } }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> u64 { 1 }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> i64 { x as i64 }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> i64 { 1.0 }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("struct S;\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> i64 { }\n", "unsupported-construct", Some(UnsupportedConstruct::Other)),
        ("fn f(x: i64) -> i64 { x + }\n", "parse", None),
    ];
    for (source, code, construct) in cases {
        let error: FrontendError = parse_source(source).expect_err("source must be rejected");
        assert_eq!(error.code, code, "source: {source}");
        assert_eq!(error.construct, construct, "source: {source}");
        assert!(!error.detail.is_empty());
    }
}

#[test]
fn frontend_output_admits_identical_roles_and_verifies() {
    let output = parse_source(ADD_SOURCE).expect("admitted source");
    let variants = Role::ALL.iter().map(|role| output.clone().into_variant(*role)).collect::<Vec<Variant>>();
    let bindings = Role::ALL.iter().map(|role| output.binding(*role)).collect::<Vec<_>>();
    let items = vec![ItemMapping {
        logical: "add".to_owned(),
        names: RoleMap {
            base: "add".to_owned(),
            a: "add".to_owned(),
            b: "add".to_owned(),
            merge: "add".to_owned(),
        },
    }];
    let observables = vec![Observable {
        logical: "add".to_owned(),
        arity: 2,
        domain: InputDomain::new(vec![-1, 0, 1], 9, Vec::new()),
    }];
    let request = MergeRequest {
        assumptions: assumptions(),
        variants,
        bindings,
        items,
        observables,
        budget: budget(),
        provenance: None,
    };
    let receipt = check_request(&request).expect("admission");
    assert!(matches!(receipt.outcome, Outcome::Verified { .. }));
    assert_eq!(receipt.roles.len(), 4);
    assert_eq!(receipt.bindings.len(), 4);
}

#[test]
fn frontend_binding_drift_is_unsupported() {
    let output = parse_source(ADD_SOURCE).expect("admitted source");
    let variants = Role::ALL.iter().map(|role| output.clone().into_variant(*role)).collect::<Vec<Variant>>();
    let mut bindings = Role::ALL.iter().map(|role| output.binding(*role)).collect::<Vec<_>>();
    bindings[0].subject_identity = "b3:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff".to_owned();
    let request = MergeRequest {
        assumptions: assumptions(),
        variants,
        bindings,
        items: vec![ItemMapping {
            logical: "add".to_owned(),
            names: RoleMap {
                base: "add".to_owned(),
                a: "add".to_owned(),
                b: "add".to_owned(),
                merge: "add".to_owned(),
            },
        }],
        observables: vec![Observable {
            logical: "add".to_owned(),
            arity: 2,
            domain: InputDomain::new(vec![-1, 0, 1], 9, Vec::new()),
        }],
        budget: budget(),
        provenance: None,
    };
    let receipt = check_request(&request).expect("admission");
    assert!(matches!(receipt.outcome, Outcome::Unsupported {
        reason: UnsupportedReason::BindingMismatch,
        ..
    }));
}

#[test]
fn frontend_correspondence_is_deterministic() {
    let first = parse_source(ADD_SOURCE).expect("first parse");
    let second = parse_source(ADD_SOURCE).expect("second parse");
    assert_eq!(first, second);
    let source = "fn pick(x: i64) -> i64 { if x { 1 } else { 2 } }\n";
    let first = parse_source(source).expect("first conditional");
    let second = parse_source(source).expect("second conditional");
    assert_eq!(first.source_identity, second.source_identity);
    assert_eq!(first.subject_identity, second.subject_identity);
    assert_eq!(first.program, second.program);
    let program: BTreeMap<_, _> = first.program.clone();
    assert_eq!(program.len(), 1);
}

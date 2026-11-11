//! Lean external backend tests for the bounded merge rail.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::sync::atomic::AtomicU64;
use std::sync::atomic::Ordering;

use octet_merge_rail::checker::check_request_with_backend;
use octet_merge_rail::corpus::assumptions;
use octet_merge_rail::corpus::budget;
use octet_merge_rail::evaluate::Backend;
use octet_merge_rail::evaluate::EvalContext;
use octet_merge_rail::evaluate::EvalOutcome;
use octet_merge_rail::external_lean::LEAN_BACKEND_ID;
use octet_merge_rail::external_lean::LEAN_ENVIRONMENT_VARIABLE;
use octet_merge_rail::external_lean::LeanBackend;
use octet_merge_rail::frontend::parse_source;
use octet_merge_rail::model::AbortKind;
use octet_merge_rail::model::ArithmeticModel;
use octet_merge_rail::model::Expr;
use octet_merge_rail::model::Function;
use octet_merge_rail::model::InputDomain;
use octet_merge_rail::model::ItemMapping;
use octet_merge_rail::model::MergeRequest;
use octet_merge_rail::model::Observable;
use octet_merge_rail::model::Outcome;
use octet_merge_rail::model::Role;
use octet_merge_rail::model::RoleMap;
use octet_merge_rail::model::UnsupportedReason;
use octet_merge_rail::model::Value;
use octet_merge_rail::model::Variant;

static TEMP_COUNTER: AtomicU64 = AtomicU64::new(0);

const ADD_SOURCE: &str = "fn add(a: i64, b: i64) -> i64 { a + b }\n";

#[test]
fn lean_backend_verifies_frontend_request_when_available() {
    if std::env::var_os(LEAN_ENVIRONMENT_VARIABLE).is_none() {
        eprintln!("skipping Lean verification: {LEAN_ENVIRONMENT_VARIABLE} is unset");
        return;
    }
    let backend = LeanBackend::from_environment(unique_dir("real")).expect("Lean backend");
    assert_eq!(backend.identity().id, LEAN_BACKEND_ID);
    let output = parse_source(ADD_SOURCE).expect("frontend output");
    let request = identical_request(&output, 1);
    let receipt = check_request_with_backend(&request, &backend).expect("admission");
    assert!(matches!(receipt.outcome, Outcome::Verified { .. }), "{:?}", receipt.outcome);
    assert_eq!(receipt.backend.id, LEAN_BACKEND_ID);
}

#[test]
fn lean_backend_verifies_checked_overflow_when_available() {
    if std::env::var_os(LEAN_ENVIRONMENT_VARIABLE).is_none() {
        eprintln!("skipping Lean verification: {LEAN_ENVIRONMENT_VARIABLE} is unset");
        return;
    }
    let source = "fn inc(x: i64) -> i64 { x + 1 }\n";
    let output = parse_source(source).expect("frontend output");
    let backend = LeanBackend::from_environment(unique_dir("overflow")).expect("Lean backend");
    let context = EvalContext {
        arithmetic: ArithmeticModel::Checked,
        max_steps: 64,
    };
    let value = backend
        .evaluate(&output.program, "inc", &[Value::Int(i64::MAX)], &context)
        .expect("Lean evaluation");
    assert_eq!(value, EvalOutcome::Value(Value::Aborted(AbortKind::Overflow)));
}

#[test]
fn lean_backend_reports_tool_error_on_rejection() {
    let directory = unique_dir("reject");
    let fake = fake_lean(&directory, "exit 1\n");
    let backend = LeanBackend::new(fake, &directory);
    let output = parse_source(ADD_SOURCE).expect("frontend output");
    let context = EvalContext {
        arithmetic: ArithmeticModel::Checked,
        max_steps: 64,
    };
    let result = backend.evaluate(&output.program, "add", &[Value::Int(1), Value::Int(2)], &context);
    assert!(result.is_err(), "rejected proof must be a backend failure");
}

#[test]
fn lean_backend_accepts_successful_process() {
    let directory = unique_dir("accept");
    let fake = fake_lean(&directory, "exit 0\n");
    let backend = LeanBackend::new(fake, &directory);
    let output = parse_source(ADD_SOURCE).expect("frontend output");
    let context = EvalContext {
        arithmetic: ArithmeticModel::Checked,
        max_steps: 64,
    };
    let result = backend
        .evaluate(&output.program, "add", &[Value::Int(1), Value::Int(2)], &context)
        .expect("successful process");
    assert_eq!(result, EvalOutcome::Value(Value::Int(3)));
}

#[test]
fn lean_backend_times_out_as_tool_error() {
    let directory = unique_dir("timeout");
    let fake = fake_lean(&directory, "while :; do :; done\n");
    let backend = LeanBackend::new(fake, &directory).with_timeout_ms(20);
    let output = parse_source(ADD_SOURCE).expect("frontend output");
    let context = EvalContext {
        arithmetic: ArithmeticModel::Checked,
        max_steps: 64,
    };
    let started = std::time::Instant::now();
    let result = backend.evaluate(&output.program, "add", &[Value::Int(1), Value::Int(2)], &context);
    assert!(result.is_err(), "timeout must be a backend failure");
    assert!(started.elapsed() < std::time::Duration::from_secs(3));
}

#[test]
fn lean_backend_rejects_wrapping_arithmetic() {
    let backend = LeanBackend::new("unused", unique_dir("wrapping"));
    let output = parse_source(ADD_SOURCE).expect("frontend output");
    let context = EvalContext {
        arithmetic: ArithmeticModel::Wrapping,
        max_steps: 64,
    };
    let result = backend
        .evaluate(&output.program, "add", &[Value::Int(1), Value::Int(2)], &context)
        .expect("backend response");
    assert!(matches!(result, EvalOutcome::Unsupported {
        reason: UnsupportedReason::UnsupportedConstruct,
        ..
    }));
}

#[test]
fn lean_backend_rejects_cycles_before_invoking_lean() {
    let backend = LeanBackend::new("unused", unique_dir("cycle"));
    let mut program = BTreeMap::new();
    program.insert("f".to_owned(), Function::new(0, Expr::call("g", Vec::new())));
    program.insert("g".to_owned(), Function::new(0, Expr::call("f", Vec::new())));
    let context = EvalContext {
        arithmetic: ArithmeticModel::Checked,
        max_steps: 64,
    };
    let result = backend.evaluate(&program, "f", &[], &context).expect("backend response");
    assert!(matches!(result, EvalOutcome::Unsupported {
        reason: UnsupportedReason::Recursion,
        ..
    }));
}

fn identical_request(output: &octet_merge_rail::frontend::FrontendOutput, max_cases: usize) -> MergeRequest {
    let variants = Role::ALL.iter().map(|role| output.clone().into_variant(*role)).collect::<Vec<Variant>>();
    let bindings = Role::ALL.iter().map(|role| output.binding(*role)).collect::<Vec<_>>();
    MergeRequest {
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
            domain: InputDomain::new(vec![0], max_cases, Vec::new()),
        }],
        budget: budget(),
        provenance: None,
    }
}

fn unique_dir(label: &str) -> PathBuf {
    let id = TEMP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let directory =
        std::env::temp_dir().join(format!("octet-merge-rail-lean-test-{label}-{}-{id}", std::process::id()));
    std::fs::create_dir_all(&directory).expect("test work directory");
    directory
}

fn fake_lean(directory: &std::path::Path, body: &str) -> PathBuf {
    let shell = std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
        .map(|path| path.join("sh"))
        .find(|path| path.is_file())
        .or_else(|| {
            std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())
                .map(|path| path.join("bash"))
                .find(|path| path.is_file())
        })
        .expect("a shell executable");
    let path = directory.join("fake-lean");
    std::fs::write(&path, format!("#!{}\n{body}", shell.display())).expect("fake Lean script");
    let mut permissions = std::fs::metadata(&path).expect("fake Lean metadata").permissions();
    std::os::unix::fs::PermissionsExt::set_mode(&mut permissions, 0o755);
    std::fs::set_permissions(&path, permissions).expect("fake Lean permissions");
    path
}

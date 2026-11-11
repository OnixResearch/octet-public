//! Shared builders for merge-rail negative-control tests.

#![allow(dead_code)]

use octet_merge_rail::corpus::sealed_bindings;
use octet_merge_rail::model::ArithmeticModel;
use octet_merge_rail::model::Assumptions;
use octet_merge_rail::model::Budget;
use octet_merge_rail::model::CompareOp;
use octet_merge_rail::model::DependencyPolicy;
use octet_merge_rail::model::Expr;
use octet_merge_rail::model::ExternalEffects;
use octet_merge_rail::model::Function;
use octet_merge_rail::model::InputDomain;
use octet_merge_rail::model::ItemMapping;
use octet_merge_rail::model::MemoryModel;
use octet_merge_rail::model::MergeRequest;
use octet_merge_rail::model::Observable;
use octet_merge_rail::model::ObservableCompleteness;
use octet_merge_rail::model::PreconditionSemantics;
use octet_merge_rail::model::Predicate;
use octet_merge_rail::model::Program;
use octet_merge_rail::model::Role;
use octet_merge_rail::model::RoleMap;
use octet_merge_rail::model::Variant;

/// All roles with the same `f(x) = x + 1` source and program.
#[must_use]
pub fn base_request() -> MergeRequest {
    let function = Function::new(1, Expr::plus(Expr::Arg(0), Expr::Int(1)));
    let variants = Role::ALL
        .iter()
        .map(|role| Variant {
            role: *role,
            source_text: "f(x) = x + 1".to_owned(),
            program: program(&[("f", function.clone())]),
        })
        .collect::<Vec<_>>();
    let mut request = MergeRequest {
        assumptions: assumptions(),
        variants,
        bindings: Vec::new(),
        items: vec![mapping("f")],
        observables: vec![observable("f")],
        budget: Budget::new(4_096, 4_096),
        provenance: None,
    };
    request.bindings = sealed_bindings(&request.variants);
    request
}

/// All roles with a calling `f(x) = h(x)` and a callee `h(x) = x + 1`.
#[must_use]
pub fn callee_request() -> MergeRequest {
    let f = Function::new(1, Expr::call("h", vec![Expr::Arg(0)]));
    let h = Function::new(1, Expr::plus(Expr::Arg(0), Expr::Int(1)));
    let variants = Role::ALL
        .iter()
        .map(|role| Variant {
            role: *role,
            source_text: "f(x) = h(x); h(x) = x + 1".to_owned(),
            program: program(&[("f", f.clone()), ("h", h.clone())]),
        })
        .collect::<Vec<_>>();
    let mut request = MergeRequest {
        assumptions: assumptions(),
        variants,
        bindings: Vec::new(),
        items: vec![mapping("f"), mapping("h")],
        observables: vec![observable("f")],
        budget: Budget::new(4_096, 4_096),
        provenance: None,
    };
    request.bindings = sealed_bindings(&request.variants);
    request
}

/// Reviewed assumptions.
#[must_use]
pub fn assumptions() -> Assumptions {
    Assumptions {
        arithmetic: ArithmeticModel::Checked,
        memory: MemoryModel::PureScalar,
        dependencies: DependencyPolicy::IdenticalOrObservable,
        preconditions: PreconditionSemantics::IntegerConjunction,
        external_effects: ExternalEffects::Forbidden,
        observables: ObservableCompleteness::DeclaredClosureOnly,
    }
}

/// Build a role-local program from named functions.
#[must_use]
pub fn program(functions: &[(&str, Function)]) -> Program {
    functions.iter().map(|(name, function)| ((*name).to_owned(), function.clone())).collect()
}

/// Build a same-name item mapping for one logical item.
#[must_use]
pub fn mapping(logical: &str) -> ItemMapping {
    let names = RoleMap {
        base: logical.to_owned(),
        a: logical.to_owned(),
        b: logical.to_owned(),
        merge: logical.to_owned(),
    };
    ItemMapping {
        logical: logical.to_owned(),
        names,
    }
}

/// Build one unary observable over `[-1, 0, 1]` with `x >= 0`.
#[must_use]
pub fn observable(logical: &str) -> Observable {
    Observable {
        logical: logical.to_owned(),
        arity: 1,
        domain: InputDomain::new(vec![-1, 0, 1], 8, vec![Predicate::new(0, CompareOp::GreaterOrEqual, 0)]),
    }
}

/// Replace one item body in one role.
pub fn set_body(request: &mut MergeRequest, role: Role, item: &str, body: &Expr) {
    for variant in &mut request.variants {
        if variant.role == role
            && let Some(function) = variant.program.get_mut(item)
        {
            function.body = body.clone();
        }
    }
}

/// Reseal all role bindings after deliberate variant edits.
pub fn reseal(request: &mut MergeRequest) {
    request.bindings = sealed_bindings(&request.variants);
}

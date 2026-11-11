use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_hir::def::DefKind;
use rustc_hir::def::Res;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_lint_pass;
use rustc_session::declare_tool_lint;
use rustc_span::Span;
use rustc_span::Symbol;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::source;
use crate::test_context;

declare_tool_lint! {
    /// Detects `Option`/`Result` `unwrap` and `expect` in non-test code, through
    /// method calls, UFCS calls such as `Option::expect(value, "..")`, and
    /// function paths such as `map(Result::unwrap)`. These panic on failure,
    /// crashing the process instead of propagating the error.
    pub tigerstyle::NO_UNWRAP,
    Warn,
    "`.unwrap()`/`.expect()` in production code"
}

declare_lint_pass!(NoUnwrap => [NO_UNWRAP]);

fn is_option_or_result(cx: &LateContext<'_>, ty: ty::Ty<'_>) -> bool {
    if let ty::Adt(adt, _) = ty.kind() {
        let did = adt.did();
        cx.tcx.is_diagnostic_item(sym::Option, did) || cx.tcx.is_diagnostic_item(sym::Result, did)
    } else {
        false
    }
}

fn is_known_constructor_call(receiver: &hir::Expr<'_>) -> bool {
    let ExprKind::Call(callee, [_]) = &receiver.kind else {
        return false;
    };

    let ExprKind::Path(hir::QPath::Resolved(_, path)) = &callee.kind else {
        return false;
    };

    matches!(path.segments.last().map(|segment| segment.ident.name.as_str()), Some("Some" | "Ok"))
}

fn match_def_path(cx: &LateContext<'_>, did: rustc_hir::def_id::DefId, path: &[&str]) -> bool {
    cx.get_def_path(did).iter().copied().eq(path.iter().map(|segment| Symbol::intern(segment)))
}

fn parse_literal_string_and_target<'tcx>(
    cx: &LateContext<'tcx>,
    receiver: &'tcx hir::Expr<'tcx>,
) -> Option<(String, ty::Ty<'tcx>)> {
    let ExprKind::MethodCall(segment, target, args, _) = &receiver.kind else {
        return None;
    };
    if segment.ident.name.as_str() != "parse" || !args.is_empty() {
        return None;
    }

    let ExprKind::Lit(lit) = target.kind else {
        return None;
    };
    let rustc_ast::LitKind::Str(symbol, _) = lit.node else {
        return None;
    };

    let ty::Adt(adt, args) = cx.typeck_results().expr_ty(receiver).kind() else {
        return None;
    };
    if !cx.tcx.is_diagnostic_item(sym::Result, adt.did()) {
        return None;
    }

    Some((symbol.as_str().to_string(), args[0].expect_ty()))
}

fn literal_parse_succeeds_for_target(cx: &LateContext<'_>, literal: &str, target_ty: ty::Ty<'_>) -> bool {
    match target_ty.kind() {
        ty::Bool => literal.parse::<bool>().is_ok(),
        ty::Char => literal.parse::<char>().is_ok(),
        ty::Int(int_ty) => match int_ty {
            ty::IntTy::Isize => literal.parse::<isize>().is_ok(),
            ty::IntTy::I8 => literal.parse::<i8>().is_ok(),
            ty::IntTy::I16 => literal.parse::<i16>().is_ok(),
            ty::IntTy::I32 => literal.parse::<i32>().is_ok(),
            ty::IntTy::I64 => literal.parse::<i64>().is_ok(),
            ty::IntTy::I128 => literal.parse::<i128>().is_ok(),
        },
        ty::Uint(uint_ty) => match uint_ty {
            ty::UintTy::Usize => literal.parse::<usize>().is_ok(),
            ty::UintTy::U8 => literal.parse::<u8>().is_ok(),
            ty::UintTy::U16 => literal.parse::<u16>().is_ok(),
            ty::UintTy::U32 => literal.parse::<u32>().is_ok(),
            ty::UintTy::U64 => literal.parse::<u64>().is_ok(),
            ty::UintTy::U128 => literal.parse::<u128>().is_ok(),
        },
        ty::Float(float_ty) => match float_ty {
            ty::FloatTy::F32 => literal.parse::<f32>().is_ok(),
            ty::FloatTy::F64 => literal.parse::<f64>().is_ok(),
            ty::FloatTy::F16 | ty::FloatTy::F128 => false,
        },
        ty::Adt(adt, _) => {
            match_def_path(cx, adt.did(), &["alloc", "string", "String"])
                || match_def_path(cx, adt.did(), &["std", "string", "String"])
                || match_def_path(cx, adt.did(), &["core", "net", "ip_addr", "IpAddr"])
                    && literal.parse::<std::net::IpAddr>().is_ok()
                || match_def_path(cx, adt.did(), &["core", "net", "ip_addr", "Ipv4Addr"])
                    && literal.parse::<std::net::Ipv4Addr>().is_ok()
                || match_def_path(cx, adt.did(), &["core", "net", "ip_addr", "Ipv6Addr"])
                    && literal.parse::<std::net::Ipv6Addr>().is_ok()
                || match_def_path(cx, adt.did(), &["core", "net", "socket_addr", "SocketAddr"])
                    && literal.parse::<std::net::SocketAddr>().is_ok()
                || match_def_path(cx, adt.did(), &["core", "net", "socket_addr", "SocketAddrV4"])
                    && literal.parse::<std::net::SocketAddrV4>().is_ok()
                || match_def_path(cx, adt.did(), &["core", "net", "socket_addr", "SocketAddrV6"])
                    && literal.parse::<std::net::SocketAddrV6>().is_ok()
        }
        _ => false,
    }
}

fn is_provably_infallible_literal_parse<'tcx>(cx: &LateContext<'tcx>, receiver: &'tcx hir::Expr<'tcx>) -> bool {
    let Some((literal, target_ty)) = parse_literal_string_and_target(cx, receiver) else {
        return false;
    };

    literal_parse_succeeds_for_target(cx, &literal, target_ty)
}

fn span_contains(outer: Span, inner: Span) -> bool {
    outer.lo() <= inner.lo() && inner.hi() <= outer.hi()
}

fn same_receiver(lhs: &hir::Expr<'_>, rhs: &hir::Expr<'_>) -> bool {
    match (&lhs.kind, &rhs.kind) {
        (ExprKind::Path(hir::QPath::Resolved(_, lhs_path)), ExprKind::Path(hir::QPath::Resolved(_, rhs_path))) => {
            lhs_path.res == rhs_path.res
        }
        (ExprKind::Field(lhs_base, lhs_ident), ExprKind::Field(rhs_base, rhs_ident)) => {
            lhs_ident.name == rhs_ident.name && same_receiver(lhs_base, rhs_base)
        }
        _ => false,
    }
}

fn guard_method_name(cx: &LateContext<'_>, receiver: &hir::Expr<'_>) -> Option<&'static str> {
    let receiver_ty = cx.typeck_results().expr_ty_adjusted(receiver);
    if let ty::Adt(adt, _) = receiver_ty.kind() {
        let did = adt.did();
        if cx.tcx.is_diagnostic_item(sym::Option, did) {
            return Some("is_some");
        }
        if cx.tcx.is_diagnostic_item(sym::Result, did) {
            return Some("is_ok");
        }
    }

    None
}

fn condition_requires_guard(cond: &hir::Expr<'_>, receiver: &hir::Expr<'_>, guard_name: &str) -> bool {
    match &cond.kind {
        ExprKind::MethodCall(segment, guard_receiver, args, _) => {
            args.is_empty() && segment.ident.name.as_str() == guard_name && same_receiver(receiver, guard_receiver)
        }
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, hir::BinOpKind::And) => {
            condition_requires_guard(lhs, receiver, guard_name) || condition_requires_guard(rhs, receiver, guard_name)
        }
        ExprKind::Binary(op, lhs, rhs) if matches!(op.node, hir::BinOpKind::Or) => {
            condition_requires_guard(lhs, receiver, guard_name) && condition_requires_guard(rhs, receiver, guard_name)
        }
        ExprKind::Block(block, _) => {
            block.expr.is_some_and(|tail| condition_requires_guard(tail, receiver, guard_name))
        }
        _ => false,
    }
}

fn has_same_branch_guard(cx: &LateContext<'_>, expr: &hir::Expr<'_>, receiver: &hir::Expr<'_>) -> bool {
    let Some(guard_name) = guard_method_name(cx, receiver) else {
        return false;
    };

    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        let hir::Node::Expr(parent) = node else {
            continue;
        };

        let ExprKind::If(cond, then_expr, _) = &parent.kind else {
            continue;
        };

        if span_contains(then_expr.span, expr.span) && condition_requires_guard(cond, receiver, guard_name) {
            return true;
        }
    }

    false
}

fn should_skip_unwrap<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
    receiver: Option<&'tcx hir::Expr<'tcx>>,
    name: &str,
) -> bool {
    if test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
        return true;
    }

    if name != "unwrap" {
        return false;
    }

    // A bare `Option::unwrap` path has no receiver to prove `Some`/`Ok`.
    receiver.is_some_and(|receiver| {
        is_known_constructor_call(receiver)
            || is_provably_infallible_literal_parse(cx, receiver)
            || has_same_branch_guard(cx, expr, receiver)
    })
}

/// Name of the `Option`/`Result` `unwrap` or `expect` method that a path such as
/// `Result::unwrap`, `Option::<u32>::expect`, or `<Option<u32>>::unwrap` resolves to.
/// Paths whose method segment the user did not spell are generated code.
fn path_unwrap_name(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Option<&'static str> {
    let ExprKind::Path(qpath) = &expr.kind else {
        return None;
    };
    let Res::Def(DefKind::AssocFn, did) = cx.typeck_results().qpath_res(qpath, expr.hir_id) else {
        return None;
    };
    let name = match cx.tcx.item_name(did).as_str() {
        "unwrap" => "unwrap",
        "expect" => "expect",
        _ => return None,
    };
    let impl_did = cx.tcx.inherent_impl_of_assoc(did)?;
    if !is_option_or_result(cx, cx.tcx.type_of(impl_did).instantiate_identity()) {
        return None;
    }
    let method_ident = match qpath {
        hir::QPath::Resolved(_, path) => path.segments.last()?.ident,
        hir::QPath::TypeRelative(_, segment) => segment.ident,
    };
    source::ident_is_spelled_at_its_span(cx.sess(), method_ident).then_some(name)
}

fn is_call_callee(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> bool {
    matches!(
        cx.tcx.parent_hir_node(expr.hir_id),
        hir::Node::Expr(hir::Expr { kind: ExprKind::Call(callee, _), .. }) if callee.hir_id == expr.hir_id
    )
}

/// The receiver (if any) and method name of an `unwrap`/`expect` use: method-call
/// syntax, a UFCS call such as `Option::expect(value, "..")`, or a bare function
/// path such as `map(Result::unwrap)`.
fn unwrap_use<'tcx>(
    cx: &LateContext<'tcx>,
    expr: &'tcx hir::Expr<'tcx>,
) -> Option<(Option<&'tcx hir::Expr<'tcx>>, &'static str)> {
    match &expr.kind {
        ExprKind::MethodCall(segment, receiver, _args, _span) => {
            let name = match segment.ident.name.as_str() {
                "unwrap" => "unwrap",
                "expect" => "expect",
                _ => return None,
            };
            // `#[tokio::main]` re-spans its generated `.expect` at a user token.
            if !source::ident_is_spelled_at_its_span(cx.sess(), segment.ident) {
                return None;
            }
            // Only fire when the receiver is Option or Result. Use the adjusted type
            // so auto-derefed receivers such as `&Option<T>` are covered too.
            is_option_or_result(cx, cx.typeck_results().expr_ty_adjusted(receiver)).then_some((Some(*receiver), name))
        }
        ExprKind::Call(callee, [receiver, ..]) => path_unwrap_name(cx, callee).map(|name| (Some(receiver), name)),
        // The callee of a UFCS call is reported with the call above.
        ExprKind::Path(_) if !is_call_callee(cx, expr) => path_unwrap_name(cx, expr).map(|name| (None, name)),
        _ => None,
    }
}

impl<'tcx> LateLintPass<'tcx> for NoUnwrap {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        let Some((receiver, name)) = unwrap_use(cx, expr) else {
            return;
        };

        if should_skip_unwrap(cx, expr, receiver, name) {
            return;
        }

        let message = format!("`.{name}()` in production code will panic on failure");
        if let Some(suggestion) = receiver.and_then(|receiver| unwrap_suggestion(cx, expr, receiver)) {
            emit_tiger_style_lint_with_suggestion(cx, NO_UNWRAP, expr.span, message, "no_unwrap", suggestion);
        } else {
            emit_tiger_style_lint(cx, NO_UNWRAP, expr.span, message, "no_unwrap");
        }
    }
}

fn unwrap_suggestion(cx: &LateContext<'_>, expr: &hir::Expr<'_>, receiver: &hir::Expr<'_>) -> Option<LintSuggestion> {
    let sm = cx.sess().source_map();
    let receiver_snippet = sm.span_to_snippet(receiver.span).ok()?;
    let replacement = format!("{receiver_snippet}?");

    Some(LintSuggestion::Single {
        span: expr.span,
        message: "propagate the error with `?` instead of panicking".to_owned(),
        replacement,
        applicability: Applicability::MaybeIncorrect,
    })
}

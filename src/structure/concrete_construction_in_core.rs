use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_middle::ty;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;
use crate::test_context;

const LINT_NAME: &str = "concrete_construction_in_core";
const NEW_CALLEE_SUFFIX: &str = "::new";
const DEFAULT_CALLEE_SUFFIX: &str = "::default";
const STANDARD_VALUE_CRATES: &[&str] = &["alloc", "core", "std"];

declare_tool_lint! {
    /// Detects concrete infrastructure construction inside declared core scopes.
    /// Core code should accept injected ports/capabilities instead of constructing
    /// external concrete dependencies directly.
    pub tigerstyle::CONCRETE_CONSTRUCTION_IN_CORE,
    Warn,
    "concrete dependency constructed in core scope"
}

pub struct ConcreteConstructionInCore {
    core_source_scopes: Vec<String>,
    core_value_provider_types: Vec<String>,
}

impl ConcreteConstructionInCore {
    pub fn new(core_source_scopes: Vec<String>, core_value_provider_types: Vec<String>) -> Self {
        Self {
            core_source_scopes,
            core_value_provider_types,
        }
    }
}

impl_lint_pass!(ConcreteConstructionInCore => [CONCRETE_CONSTRUCTION_IN_CORE]);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstructedTypeOrigin {
    LocalDomain,
    StandardValue,
    ReviewedValue,
    External,
    Unsupported,
}

fn path_is_in_core_scope(cx: &LateContext<'_>, span: rustc_span::Span, scopes: &[String]) -> bool {
    crate::source::span_in_scopes(cx.sess(), span, scopes)
}

fn direct_constructor_name(cx: &LateContext<'_>, callee: &hir::Expr<'_>) -> Option<String> {
    let snippet = cx.sess().source_map().span_to_snippet(callee.span).ok()?;
    let type_name = snippet
        .strip_suffix(NEW_CALLEE_SUFFIX)
        .or_else(|| snippet.strip_suffix(DEFAULT_CALLEE_SUFFIX))?
        .trim();
    if type_name.is_empty() {
        return None;
    }
    Some(type_name.to_owned())
}

/// The type a constructor call builds: the `Self` type of the called
/// associated function, so fallible constructors returning `Result<T, _>` or
/// `Option<T>` classify `T` rather than the standard wrapper. Other callees
/// fall back to the call's result type.
fn constructed_type<'tcx>(cx: &LateContext<'tcx>, expr: &hir::Expr<'_>, callee: &hir::Expr<'_>) -> ty::Ty<'tcx> {
    if let hir::ExprKind::Path(qpath) = &callee.kind
        && let hir::def::Res::Def(hir::def::DefKind::AssocFn, did) = cx.typeck_results().qpath_res(qpath, callee.hir_id)
    {
        if let Some(impl_did) = cx.tcx.associated_item(did).impl_container(cx.tcx) {
            return cx.tcx.type_of(impl_did).instantiate_identity();
        }
        // Trait associated functions take `Self` as their first generic argument.
        if let Some(self_ty) = cx.typeck_results().node_args(callee.hir_id).types().next() {
            return self_ty;
        }
    }
    cx.typeck_results().expr_ty(expr)
}

fn constructed_type_origin(
    cx: &LateContext<'_>,
    constructed: ty::Ty<'_>,
    value_provider_types: &[String],
) -> ConstructedTypeOrigin {
    let ty::Adt(adt, _) = constructed.kind() else {
        return ConstructedTypeOrigin::Unsupported;
    };
    if adt.did().is_local() {
        return ConstructedTypeOrigin::LocalDomain;
    }
    let path = cx.get_def_path(adt.did());
    let Some(crate_name) = path.first().map(rustc_span::Symbol::as_str) else {
        return ConstructedTypeOrigin::Unsupported;
    };
    if STANDARD_VALUE_CRATES.contains(&crate_name) {
        ConstructedTypeOrigin::StandardValue
    } else if value_provider_types
        .iter()
        .any(|provider| provider.split("::").eq(path.iter().map(rustc_span::Symbol::as_str)))
    {
        ConstructedTypeOrigin::ReviewedValue
    } else {
        ConstructedTypeOrigin::External
    }
}

impl<'tcx> LateLintPass<'tcx> for ConcreteConstructionInCore {
    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        // Cheapest gates first: most crates declare no core scopes, and most
        // expressions are not direct `Type::new()` / `Type::default()` calls.
        if self.core_source_scopes.is_empty() {
            return;
        }
        let hir::ExprKind::Call(callee, _) = expr.kind else {
            return;
        };
        if expr.span.from_expansion() {
            return;
        }
        let Some(type_name) = direct_constructor_name(cx, callee) else {
            return;
        };
        if constructed_type_origin(cx, constructed_type(cx, expr, callee), &self.core_value_provider_types)
            != ConstructedTypeOrigin::External
            || !path_is_in_core_scope(cx, expr.span, &self.core_source_scopes)
            || test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span)
        {
            return;
        }

        emit_tiger_style_lint(
            cx,
            CONCRETE_CONSTRUCTION_IN_CORE,
            expr.span,
            format!(
                "core scope constructs concrete type `{type_name}`; inject the dependency through a port or capability"
            ),
            LINT_NAME,
        );
    }
}

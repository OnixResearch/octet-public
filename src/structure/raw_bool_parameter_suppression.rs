use rustc_lint::LateLintPass;

rustc_session::declare_tool_lint! {
    /// Rejects broad or unjustified source suppression of
    /// [`RAW_BOOL_PARAMETER`](crate::structure::raw_bool_parameter::RAW_BOOL_PARAMETER).
    ///
    /// An unavoidable public compatibility signature may use one
    /// `#[expect(tigerstyle::raw_bool_parameter, reason = "external compatibility
    /// contract: <specific contract>; owner: <named owner>")]` on the exact
    /// callable. `#[allow]`, crate/module/enclosing-item expectations, and
    /// broad `warnings` or structure-group suppressions are rejected.
    ///
    /// The guard governs exceptions; it is not a second parameter finding.
    pub tigerstyle::RAW_BOOL_PARAMETER_SUPPRESSION,
    Warn,
    "raw boolean API suppressions must be local, owned compatibility expectations"
}

/// How strongly the guard reports an invalid suppression.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Enforcement {
    /// Report through the lint level. A caller may allow the guard itself.
    Advisory,
    /// Report every invalid suppression as a hard error. Selected by an
    /// explicit `deny` for the guard, including the full deny-all catalog.
    StrictAll,
}

/// Rejects invalid `raw_bool_parameter` suppressions.
pub struct RawBoolParameterSuppression {
    enforcement: Enforcement,
}

impl RawBoolParameterSuppression {
    pub fn new(enforcement: Enforcement) -> Self {
        Self { enforcement }
    }
}

rustc_session::impl_lint_pass!(RawBoolParameterSuppression => [RAW_BOOL_PARAMETER_SUPPRESSION]);

/// Whether `reason` names one exact external compatibility contract and owner.
fn justified_reason(reason: &str) -> bool {
    let Some(rest) = reason.strip_prefix("external compatibility contract: ") else {
        return false;
    };
    let Some((contract, owner)) = rest.split_once("; owner: ") else {
        return false;
    };
    contract.trim().len() >= 12
        && !contract.contains("TODO")
        && !owner.is_empty()
        && owner.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

/// Whether one meta item path names `raw_bool_parameter`.
fn is_guarded_lint_named(segments: &[rustc_ast::ast::PathSegment]) -> bool {
    matches!(segments, [lint] if lint.ident.name.as_str() == "raw_bool_parameter")
        || matches!(
            segments,
            [tool, lint] if tool.ident.name.as_str() == octet_lint_metadata::CANONICAL_NAMESPACE
                && lint.ident.name.as_str() == "raw_bool_parameter"
        )
}

/// Whether one meta item path names a broad suppression that can hide
/// `raw_bool_parameter` without naming it.
fn is_broad_group_named(segments: &[rustc_ast::ast::PathSegment]) -> bool {
    matches!(segments, [lint] if lint.ident.name.as_str() == "warnings")
        || matches!(
            segments,
            [tool, group] if tool.ident.name.as_str() == octet_lint_metadata::CANONICAL_NAMESPACE
                && group.ident.name.as_str() == "structure"
        )
}

fn lint_attrs(
    cx: &rustc_lint::LateContext<'_>,
    hir_id: rustc_hir::HirId,
    local_callable: bool,
    enforcement: Enforcement,
) {
    for attr in cx.tcx.hir_attrs(hir_id) {
        let is_allow = attr.has_name(rustc_span::sym::allow);
        let is_expect = attr.has_name(rustc_span::sym::expect);
        if !is_allow && !is_expect {
            continue;
        }
        if attr.span().from_expansion() {
            continue;
        }
        let Some(items) = attr.meta_item_list() else {
            continue;
        };
        let mut is_guarded_lint_named_item = false;
        let mut is_broad_group_named_item = false;
        for item in &items {
            let Some(meta) = item.meta_item() else {
                continue;
            };
            is_guarded_lint_named_item |= is_guarded_lint_named(&meta.path.segments);
            is_broad_group_named_item |= is_broad_group_named(&meta.path.segments);
        }
        if !is_guarded_lint_named_item && !is_broad_group_named_item {
            continue;
        }
        let reason = items
            .iter()
            .filter_map(|item| item.meta_item())
            .find(|meta| meta.has_name(rustc_span::sym::reason))
            .and_then(rustc_ast::MetaItem::value_str);
        let message = if is_broad_group_named_item {
            "broad warning or structure-group suppression can hide `raw_bool_parameter`; use a local owned compatibility expectation"
        } else if is_allow {
            "`raw_bool_parameter` does not permit `#[allow]`; use a local, owned `#[expect]` for an unavoidable compatibility API"
        } else if !local_callable {
            "`raw_bool_parameter` expectations must attach to one exact callable, not a crate, module, or enclosing item"
        } else if !reason.is_some_and(|reason| justified_reason(reason.as_str())) {
            "`raw_bool_parameter` expectation requires `reason = \"external compatibility contract: <specific contract>; owner: <named owner>\"`"
        } else {
            continue;
        };
        let is_hard_error = enforcement == Enforcement::StrictAll;
        if is_hard_error {
            // A source-level `allow` can suppress even a denied lint; an
            // unconditional compiler error cannot be bypassed.
            cx.tcx.dcx().span_err(attr.span(), format!("tigerstyle::raw_bool_parameter_suppression: {message}"));
        } else {
            crate::diagnostics::emit_tiger_style_lint(
                cx,
                RAW_BOOL_PARAMETER_SUPPRESSION,
                attr.span(),
                message,
                "raw_bool_parameter_suppression",
            );
        }
    }
}

impl<'tcx> LateLintPass<'tcx> for RawBoolParameterSuppression {
    fn check_crate(&mut self, cx: &rustc_lint::LateContext<'tcx>) {
        lint_attrs(cx, rustc_hir::CRATE_HIR_ID, false, self.enforcement);
    }

    fn check_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx rustc_hir::Item<'tcx>) {
        lint_attrs(cx, item.hir_id(), matches!(item.kind, rustc_hir::ItemKind::Fn { .. }), self.enforcement);
    }

    fn check_impl_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx rustc_hir::ImplItem<'tcx>) {
        lint_attrs(cx, item.hir_id(), matches!(item.kind, rustc_hir::ImplItemKind::Fn(..)), self.enforcement);
    }

    fn check_trait_item(&mut self, cx: &rustc_lint::LateContext<'tcx>, item: &'tcx rustc_hir::TraitItem<'tcx>) {
        lint_attrs(cx, item.hir_id(), matches!(item.kind, rustc_hir::TraitItemKind::Fn(..)), self.enforcement);
    }

    fn check_expr(&mut self, cx: &rustc_lint::LateContext<'tcx>, expr: &'tcx rustc_hir::Expr<'tcx>) {
        lint_attrs(cx, expr.hir_id, false, self.enforcement);
    }

    fn check_stmt(&mut self, cx: &rustc_lint::LateContext<'tcx>, stmt: &'tcx rustc_hir::Stmt<'tcx>) {
        lint_attrs(cx, stmt.hir_id, false, self.enforcement);
    }

    fn check_arm(&mut self, cx: &rustc_lint::LateContext<'tcx>, arm: &'tcx rustc_hir::Arm<'tcx>) {
        lint_attrs(cx, arm.hir_id, false, self.enforcement);
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn requires_owned_specific_contract() {
        assert!(super::justified_reason("external compatibility contract: frozen wire protocol v1; owner: api-team"));
        assert!(!super::justified_reason("external compatibility contract: TODO; owner: someone"));
        assert!(!super::justified_reason("external compatibility contract: legacy callback ABI"));
        assert!(!super::justified_reason("temporary"));
    }
}

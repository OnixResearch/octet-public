use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_lint::LateLintPass;
use rustc_session::declare_tool_lint;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects numeric variable bindings whose names contain time or size
    /// words (timeout, delay, size, length, etc.) without a unit suffix
    /// (_ms, _secs, _bytes, etc.). Bare names create ambiguity about the
    /// unit of measure.
    pub tigerstyle::NUMERIC_UNITS,
    Allow,
    "numeric variable name missing unit suffix"
}

pub struct NumericUnits;

rustc_session::impl_lint_pass!(NumericUnits => [NUMERIC_UNITS]);

/// Words that imply a measurable quantity, matched as whole identifier words.
const QUANTITY_WORDS: &[&str] = &[
    "timeout", "delay", "duration", "interval", "latency", "size", "length", "capacity", "limit", "offset",
];

/// Unit suffixes that resolve ambiguity. `magic_number` shares this list.
pub(crate) const UNIT_SUFFIXES: &[&str] = &[
    "_ns", "_nanos", "_us", "_micros", "_ms", "_millis", "_sec", "_secs", "_seconds", "_mins", "_minutes", "_hours",
    "_days", "_bits", "_bytes", "_kb", "_mb", "_gb", "_kib", "_mib", "_gib",
];

/// Names that are common loop/index variables, not quantities.
const SKIP_NAMES: &[&str] = &["i", "j", "k", "n", "count", "index", "idx", "len"];

fn has_quantity_word(name: &str) -> bool {
    crate::naming::identifier_words::identifier_has_word(name, QUANTITY_WORDS)
}

fn has_unit_suffix(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    UNIT_SUFFIXES.iter().any(|s| lower.ends_with(s))
}

fn should_skip(name: &str) -> bool {
    SKIP_NAMES.contains(&name)
}

fn placeholder_unit_name(name: &str) -> String {
    format!("{name}_<unit>")
}

fn should_report_quantity(name: &str, raw_numeric: bool) -> bool {
    raw_numeric && !should_skip(name) && has_quantity_word(name) && !has_unit_suffix(name)
}

fn resolved_type_is_raw_primitive(resolved: rustc_middle::ty::Ty<'_>) -> bool {
    matches!(resolved.kind(), rustc_middle::ty::Int(_) | rustc_middle::ty::Uint(_) | rustc_middle::ty::Float(_))
}

fn report_late_quantity(cx: &rustc_lint::LateContext<'_>, name: &str, span: rustc_span::Span) {
    let placeholder = placeholder_unit_name(name);
    emit_tiger_style_lint_with_suggestion(
        cx,
        NUMERIC_UNITS,
        span,
        format!("`{name}` looks like a raw numeric quantity but has no unit suffix"),
        "numeric_units",
        LintSuggestion::Single {
            span,
            message: "rename with an explicit unit suffix like `_ms`, `_secs`, or `_bytes`".to_owned(),
            replacement: placeholder,
            applicability: Applicability::HasPlaceholders,
        },
    );
}

impl<'tcx> LateLintPass<'tcx> for NumericUnits {
    fn check_local(&mut self, cx: &rustc_lint::LateContext<'tcx>, local: &'tcx hir::LetStmt<'tcx>) {
        // Async fn and async closure parameters are lowered into `let`
        // statements; `check_fn` owns parameters.
        if !matches!(local.source, hir::LocalSource::Normal) {
            return;
        }
        let hir::PatKind::Binding(_, _, ident, _) = local.pat.kind else {
            return;
        };
        let name = ident.name.as_str();
        let resolved = cx.typeck_results().pat_ty(local.pat);
        if should_report_quantity(name, resolved_type_is_raw_primitive(resolved)) {
            report_late_quantity(cx, name, ident.span);
        }
    }

    fn check_field_def(&mut self, cx: &rustc_lint::LateContext<'tcx>, field: &'tcx hir::FieldDef<'tcx>) {
        if field.span.from_expansion() {
            return;
        }
        let name = field.ident.name.as_str();
        let resolved = cx.tcx.type_of(field.def_id).instantiate_identity();
        if should_report_quantity(name, resolved_type_is_raw_primitive(resolved)) {
            report_late_quantity(cx, name, field.ident.span);
        }
    }

    fn check_fn(
        &mut self,
        cx: &rustc_lint::LateContext<'tcx>,
        kind: rustc_hir::intravisit::FnKind<'tcx>,
        _decl: &'tcx hir::FnDecl<'tcx>,
        body: &'tcx hir::Body<'tcx>,
        span: rustc_span::Span,
        def_id: hir::def_id::LocalDefId,
    ) {
        if matches!(kind, rustc_hir::intravisit::FnKind::Closure) || span.from_expansion() {
            return;
        }
        let signature = cx.tcx.fn_sig(def_id).instantiate_identity().skip_binder();
        for (parameter, resolved) in body.params.iter().zip(signature.inputs()) {
            let hir::PatKind::Binding(_, _, ident, _) = parameter.pat.kind else {
                continue;
            };
            let name = ident.name.as_str();
            if should_report_quantity(name, resolved_type_is_raw_primitive(*resolved)) {
                report_late_quantity(cx, name, ident.span);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::should_report_quantity;

    #[test]
    fn raw_quantities_need_units() {
        assert!(should_report_quantity("timeout", true));
        assert!(!should_report_quantity("timeout_ms", true));
    }

    #[test]
    fn typed_quantities_and_non_quantities_are_accepted() {
        assert!(!should_report_quantity("timeout", false));
        assert!(!should_report_quantity("attempts", true));
    }

    #[test]
    fn quantity_words_match_whole_words_only() {
        assert!(should_report_quantity("connect_timeout", true));
        assert!(!should_report_quantity("proportion", true));
        assert!(!should_report_quantity("report_lines", true));
        assert!(!should_report_quantity("port", true));
    }

    #[test]
    fn spelled_out_and_binary_units_are_suffixes() {
        for name in [
            "timeout_seconds",
            "delay_minutes",
            "interval_mins",
            "latency_hours",
            "retention_limit_days",
            "frame_size_bits",
            "cache_size_kib",
            "buffer_capacity_mib",
            "disk_limit_gib",
        ] {
            assert!(!should_report_quantity(name, true), "{name}");
        }
        assert!(should_report_quantity("timeout_kibble", true));
    }
}

use rustc_errors::Applicability;
use rustc_hir as hir;
use rustc_hir::ExprKind;
use rustc_lint::LateContext;
use rustc_lint::LateLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::Span;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;
use crate::test_context;

declare_tool_lint! {
    /// Detects `todo!()` in non-test code. Production code should replace
    /// temporary stubs with implemented behavior or an explicit error path.
    pub tigerstyle::NO_TODO,
    Warn,
    "unfinished production stub"
}

declare_tool_lint! {
    /// Detects misuse of `#[allow(tigerstyle::no_todo)]` /
    /// `#[allow(no_todo)]`. That escape hatch is reserved for temporary
    /// `todo!()` stubs with an adjacent `//` justification comment.
    pub tigerstyle::UNJUSTIFIED_NO_TODO_ALLOW,
    Warn,
    "invalid `#[allow(tigerstyle::no_todo)]` escape hatch"
}

pub struct NoTodo {
    emit_todo_lint: bool,
    emit_unjustified_allow: bool,
    seen_unjustified_allow_spans: Vec<Span>,
    candidate_allow_spans: Vec<Span>,
    live_allow_spans: Vec<Span>,
}

impl NoTodo {
    pub fn new(emit_no_todo: bool, emit_unjustified_allow: bool) -> Self {
        Self {
            emit_todo_lint: emit_no_todo,
            emit_unjustified_allow,
            seen_unjustified_allow_spans: Vec::new(),
            candidate_allow_spans: Vec::new(),
            live_allow_spans: Vec::new(),
        }
    }
}

impl_lint_pass!(NoTodo => [NO_TODO, UNJUSTIFIED_NO_TODO_ALLOW]);

const PANIC_FAMILY_MACROS: &[&str] = &["panic!", "todo!", "unimplemented!", "unreachable!"];
const NO_TODO_BARE_NAME: &str = "no_todo";
const NO_TODO_TOOL_NAME: &str = "tigerstyle::no_todo";

fn panic_family_macro_name_from_callsite(snippet: &str) -> Option<&'static str> {
    let trimmed = snippet.trim();
    PANIC_FAMILY_MACROS.iter().copied().find(|name| trimmed.starts_with(name))
}

fn remember_span(spans: &mut Vec<Span>, span: Span) {
    if !spans.contains(&span) {
        spans.push(span);
    }
}

fn lint_name_tokens(snippet: &str) -> impl Iterator<Item = &str> {
    snippet
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_' || ch == ':'))
        .filter(|token| !token.is_empty())
}

fn attr_matches_supported_no_todo(snippet: &str) -> bool {
    lint_name_tokens(snippet).any(|token| token == NO_TODO_BARE_NAME || token == NO_TODO_TOOL_NAME)
}

fn is_line_comment(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.starts_with("//") && !trimmed.starts_with("///") && !trimmed.starts_with("//!")
}

#[derive(Clone, Copy)]
struct NoTodoAllow {
    attr_span: Span,
    justified: bool,
}

fn no_todo_allow_attr_span(cx: &LateContext<'_>, hir_id: hir::HirId) -> Option<Span> {
    let source_map = cx.sess().source_map();
    cx.tcx.hir_attrs(hir_id).iter().find_map(|attr| {
        if !attr.has_name(sym::allow) {
            return None;
        }

        // r[impl octet.runner_self_check.warning_budget.cleanup]
        let has_supported_lint = source_map.span_to_snippet(attr.span()).map_or_else(
            |_| attr_matches_supported_no_todo(&format!("{attr:?}")),
            |snippet| attr_matches_supported_no_todo(&snippet),
        );
        has_supported_lint.then_some(attr.span())
    })
}

fn no_todo_allow_for_attr_span(cx: &LateContext<'_>, attr_span: Span) -> NoTodoAllow {
    let source_map = cx.sess().source_map();
    let Ok(lines) = source_map.span_to_lines(attr_span) else {
        return NoTodoAllow {
            attr_span,
            justified: false,
        };
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return NoTodoAllow {
            attr_span,
            justified: false,
        };
    };

    if lines.file.get_line(first_line_idx).is_some_and(|line| line.trim().contains("//")) {
        return NoTodoAllow {
            attr_span,
            justified: true,
        };
    }

    let justified = first_line_idx > 0
        && lines.file.get_line(first_line_idx - 1).is_some_and(|prev_line| is_line_comment(&prev_line));

    NoTodoAllow { attr_span, justified }
}

fn enclosing_no_todo_allows(cx: &LateContext<'_>, expr: &hir::Expr<'_>) -> Vec<NoTodoAllow> {
    let mut allows = Vec::new();

    for (_, node) in cx.tcx.hir_parent_iter(expr.hir_id) {
        match node {
            hir::Node::Expr(parent_expr) => {
                if matches!(parent_expr.kind, ExprKind::Block(..))
                    && let Some(attr_span) = no_todo_allow_attr_span(cx, parent_expr.hir_id)
                {
                    allows.push(no_todo_allow_for_attr_span(cx, attr_span));
                }
            }
            hir::Node::Item(item) => {
                if let Some(attr_span) = no_todo_allow_attr_span(cx, item.hir_id()) {
                    allows.push(no_todo_allow_for_attr_span(cx, attr_span));
                }
            }
            hir::Node::ImplItem(impl_item) => {
                if let Some(attr_span) = no_todo_allow_attr_span(cx, impl_item.hir_id()) {
                    allows.push(no_todo_allow_for_attr_span(cx, attr_span));
                }
            }
            hir::Node::TraitItem(trait_item) => {
                if let Some(attr_span) = no_todo_allow_attr_span(cx, trait_item.hir_id()) {
                    allows.push(no_todo_allow_for_attr_span(cx, attr_span));
                }
            }
            _ => {}
        }
    }

    allows
}

impl NoTodo {
    fn collect_candidate_allow(&mut self, cx: &LateContext<'_>, hir_id: hir::HirId, span: Span) {
        if !self.emit_unjustified_allow || test_context::late_hir_id_is_in_test_context(cx, hir_id, span) {
            return;
        }

        if let Some(attr_span) = no_todo_allow_attr_span(cx, hir_id) {
            remember_span(&mut self.candidate_allow_spans, attr_span);
        }
    }

    fn mark_allow_live(&mut self, attr_span: Span) {
        remember_span(&mut self.live_allow_spans, attr_span);
    }

    fn emit_unjustified_allow_message_once(&mut self, cx: &LateContext<'_>, attr_span: Span, message: String) {
        if self.seen_unjustified_allow_spans.contains(&attr_span) {
            return;
        }
        self.seen_unjustified_allow_spans.push(attr_span);

        emit_tiger_style_lint(cx, UNJUSTIFIED_NO_TODO_ALLOW, attr_span, message, "unjustified_no_todo_allow");
    }

    fn emit_unjustified_allow_once(&mut self, cx: &LateContext<'_>, attr_span: Span, name: &str, justified: bool) {
        let message = if justified {
            format!(
                "`#[allow(tigerstyle::no_todo)]` / `#[allow(no_todo)]` is reserved for temporary `todo!()` stubs, not `{name}`"
            )
        } else {
            "`#[allow(tigerstyle::no_todo)]` / `#[allow(no_todo)]` needs an adjacent `//` justification comment and may only wrap temporary `todo!()` stubs".to_owned()
        };

        self.emit_unjustified_allow_message_once(cx, attr_span, message);
    }

    fn emit_orphan_allow_once(&mut self, cx: &LateContext<'_>, attr_span: Span) {
        self.emit_unjustified_allow_message_once(
            cx,
            attr_span,
            "`#[allow(tigerstyle::no_todo)]` / `#[allow(no_todo)]` no longer covers any `todo!()` stub; remove it"
                .to_owned(),
        );
    }
}

impl<'tcx> LateLintPass<'tcx> for NoTodo {
    fn check_item(&mut self, cx: &LateContext<'tcx>, item: &'tcx hir::Item<'tcx>) {
        let hir::ItemKind::Fn { .. } = &item.kind else {
            return;
        };
        self.collect_candidate_allow(cx, item.hir_id(), item.span);
    }

    fn check_impl_item(&mut self, cx: &LateContext<'tcx>, impl_item: &'tcx hir::ImplItem<'tcx>) {
        let hir::ImplItemKind::Fn(_, _) = &impl_item.kind else {
            return;
        };
        self.collect_candidate_allow(cx, impl_item.hir_id(), impl_item.span);
    }

    fn check_trait_item(&mut self, cx: &LateContext<'tcx>, trait_item: &'tcx hir::TraitItem<'tcx>) {
        let hir::TraitItemKind::Fn(_, hir::TraitFn::Provided(_)) = &trait_item.kind else {
            return;
        };
        self.collect_candidate_allow(cx, trait_item.hir_id(), trait_item.span);
    }

    fn check_expr(&mut self, cx: &LateContext<'tcx>, expr: &'tcx hir::Expr<'tcx>) {
        if matches!(expr.kind, ExprKind::Block(..)) {
            self.collect_candidate_allow(cx, expr.hir_id, expr.span);
        }

        if !expr.span.from_expansion() || !matches!(expr.kind, ExprKind::Block(..)) {
            return;
        }

        if test_context::late_hir_id_is_in_test_context(cx, expr.hir_id, expr.span) {
            return;
        }

        let callsite = expr.span.source_callsite();
        let source_map = cx.sess().source_map();
        let Ok(snippet) = source_map.span_to_snippet(callsite) else {
            return;
        };

        let Some(name) = panic_family_macro_name_from_callsite(&snippet) else {
            return;
        };

        let allows = enclosing_no_todo_allows(cx, expr);
        for allow in &allows {
            self.mark_allow_live(allow.attr_span);
        }

        if let Some(allow) = allows.first().copied() {
            if name == "todo!" && allow.justified {
                return;
            }

            if self.emit_unjustified_allow {
                self.emit_unjustified_allow_once(cx, allow.attr_span, name, allow.justified);
            }
            return;
        }

        if name != "todo!" || !self.emit_todo_lint {
            return;
        }

        emit_tiger_style_lint_with_suggestion(
            cx,
            NO_TODO,
            callsite,
            "`todo!()` marks unfinished production code",
            "no_todo",
            LintSuggestion::Single {
                span: callsite,
                message: "replace `todo!()` with an explicit error path".to_owned(),
                replacement: "return Err(/* TODO: implement error */)".to_owned(),
                applicability: Applicability::HasPlaceholders,
            },
        );
    }

    fn check_crate_post(&mut self, cx: &LateContext<'tcx>) {
        if !self.emit_unjustified_allow {
            return;
        }

        let orphan_allow_spans = self
            .candidate_allow_spans
            .iter()
            .copied()
            .filter(|attr_span| !self.live_allow_spans.contains(attr_span))
            .collect::<Vec<_>>();

        for attr_span in orphan_allow_spans {
            self.emit_orphan_allow_once(cx, attr_span);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::attr_matches_supported_no_todo;
    use super::panic_family_macro_name_from_callsite;

    #[test]
    fn finds_panic_family_macro_name() {
        assert_eq!(panic_family_macro_name_from_callsite("panic!(\"oops\")"), Some("panic!"));
        assert_eq!(panic_family_macro_name_from_callsite("todo!(\"later\")"), Some("todo!"));
        assert_eq!(panic_family_macro_name_from_callsite("unimplemented!(\"x\")"), Some("unimplemented!"));
        assert_eq!(panic_family_macro_name_from_callsite("unreachable!(\"x\")"), Some("unreachable!"));
        assert_eq!(panic_family_macro_name_from_callsite("assert!(x > 0)"), None);
    }

    #[test]
    fn matches_supported_no_todo_spellings() {
        assert!(attr_matches_supported_no_todo("#[allow(no_todo)]"));
        assert!(attr_matches_supported_no_todo("#[allow(tigerstyle::no_todo)]"));
        assert!(!attr_matches_supported_no_todo("#[allow(unjustified_no_todo_allow)]"));
    }
}

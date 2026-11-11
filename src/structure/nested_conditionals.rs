use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::visit::Visitor;
use rustc_ast::visit::{self};
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects `if`/`match` nesting deeper than a configurable limit
    /// (default: 3) inside a function or method body. Deep nesting buries
    /// the important logic and makes control flow hard to follow. An
    /// `else if` continues its chain at the same depth. Conditionals that
    /// external macros expand to, such as those inside `assert_eq!`, do not
    /// count; conditionals built by local `macro_rules!` do.
    ///
    /// Configure in `dylint.toml`:
    /// ```toml
    /// [octet]
    /// nesting_max_depth = 4
    /// ```
    pub tigerstyle::NESTED_CONDITIONALS,
    Warn,
    "conditional nesting exceeds depth limit"
}

pub struct NestedConditionals {
    max_depth: usize,
}

impl NestedConditionals {
    pub fn new(max_depth: usize) -> Self {
        Self { max_depth }
    }
}

impl_lint_pass!(NestedConditionals => [NESTED_CONDITIONALS]);

struct NestingVisitor<'a, 'b> {
    cx: &'a EarlyContext<'b>,
    depth: usize,
    max_depth: usize,
    reported: bool,
}

impl NestingVisitor<'_, '_> {
    /// Enters one branching level; returns `false` (after reporting) when the
    /// new depth exceeds the limit. Conditionals from external macros never
    /// enter a level, so `reported` is set only for a diagnostic at a span
    /// rustc keeps: local source or a local `macro_rules!` body.
    fn enter_branch(&mut self, expr: &Expr) -> bool {
        self.depth += 1;
        if self.depth <= self.max_depth {
            return true;
        }

        let depth = self.depth;
        let max = self.max_depth;
        emit_tiger_style_lint(
            self.cx,
            NESTED_CONDITIONALS,
            expr.span,
            format!("conditional nesting is {depth} levels deep (limit: {max})"),
            "nested_conditionals",
        );
        self.reported = true;
        self.depth -= 1;
        false
    }
}

impl<'ast> Visitor<'ast> for NestingVisitor<'_, '_> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if self.reported {
            return;
        }

        // Conditionals that external macros expand to, such as the `match`
        // and `if` inside `assert_eq!`, are not nesting the author wrote, and
        // rustc drops diagnostics at their spans. Their operands are still
        // walked at the current depth. Local `macro_rules!` bodies count.
        if expr.span.in_external_macro(self.cx.sess().source_map()) {
            visit::walk_expr(self, expr);
            return;
        }

        match &expr.kind {
            ExprKind::If(cond, then_block, else_expr) => {
                if !self.enter_branch(expr) {
                    return;
                }
                self.visit_expr(cond);
                self.visit_block(then_block);
                self.depth -= 1;

                let Some(else_expr) = else_expr else {
                    return;
                };
                if matches!(else_expr.kind, ExprKind::If(..)) {
                    // `else if` continues the same chain at the current depth.
                    self.visit_expr(else_expr);
                } else {
                    self.depth += 1;
                    self.visit_expr(else_expr);
                    self.depth -= 1;
                }
            }
            ExprKind::Match(..) => {
                if !self.enter_branch(expr) {
                    return;
                }
                visit::walk_expr(self, expr);
                self.depth -= 1;
            }
            _ => visit::walk_expr(self, expr),
        }
    }
}

fn check_fn_body(cx: &EarlyContext<'_>, max_depth: usize, func: &rustc_ast::Fn) {
    let Some(body) = &func.body else {
        return;
    };

    let mut visitor = NestingVisitor {
        cx,
        depth: 0,
        max_depth,
        reported: false,
    };
    visit::walk_block(&mut visitor, body);
}

impl EarlyLintPass for NestedConditionals {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if let ItemKind::Fn(func) = &item.kind {
            check_fn_body(cx, self.max_depth, func);
        }
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            check_fn_body(cx, self.max_depth, func);
        }
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        if let rustc_ast::AssocItemKind::Fn(func) = &item.kind {
            check_fn_body(cx, self.max_depth, func);
        }
    }
}

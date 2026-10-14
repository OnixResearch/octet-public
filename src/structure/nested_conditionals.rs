use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_ast::visit::Visitor;
use rustc_ast::visit::{self};
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects `if`/`match` nesting deeper than a configurable limit
    /// (default: 3) inside a function body. Deep nesting buries the
    /// important logic and makes control flow hard to follow.
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

impl<'ast> Visitor<'ast> for NestingVisitor<'_, '_> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        if self.reported {
            return;
        }

        let is_branching = matches!(expr.kind, ExprKind::If(..) | ExprKind::Match(..));

        if is_branching {
            self.depth += 1;

            if self.depth > self.max_depth {
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
                return;
            }

            visit::walk_expr(self, expr);
            self.depth -= 1;
        } else {
            visit::walk_expr(self, expr);
        }
    }
}

impl EarlyLintPass for NestedConditionals {
    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        let ItemKind::Fn(func) = &item.kind else {
            return;
        };

        let Some(body) = &func.body else {
            return;
        };

        let mut visitor = NestingVisitor {
            cx,
            depth: 0,
            max_depth: self.max_depth,
            reported: false,
        };
        visit::walk_block(&mut visitor, body);
    }
}

use rustc_ast::Expr;
use rustc_ast::ExprKind;
use rustc_ast::Item;
use rustc_ast::ItemKind;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_span::Span;

use crate::diagnostics::emit_tiger_style_lint;

declare_tool_lint! {
    /// Detects async functions, IO operations, and system time calls in
    /// files under a `verified/` directory. Verified modules must be pure —
    /// no async, no IO, no system time — so they remain testable and
    /// formally verifiable. Test-only code (`cfg(test)` items and `#[test]`
    /// functions, including test macros such as `#[tokio::test]`) is exempt.
    pub tigerstyle::VERIFIED_PURITY,
    Deny,
    "impure operation in verified module"
}

/// How a verified module imports the `std::time` module itself.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TimeImport {
    /// `use std::time;` or `use std::time as clock;` binds this name.
    Module(rustc_span::Symbol),
    /// `use std::time::*;`
    Glob,
}

#[derive(Default)]
pub struct VerifiedPurity {
    /// Number of enclosing test-only items (`#[cfg(test)]` items and harness
    /// test functions).
    test_depth: usize,
    /// Functions registered as harness tests under `--test`.
    harness_test_fns: std::collections::HashSet<rustc_ast::NodeId>,
    /// Module-level `std::time` module imports, keyed by the importing
    /// module's item id (`CRATE_NODE_ID` for the crate root).
    time_imports: Vec<(rustc_ast::NodeId, TimeImport)>,
    /// Block-local `std::time` module imports: the enclosing module's id and
    /// the span of the block the import is visible in.
    block_time_imports: Vec<(rustc_ast::NodeId, Span, TimeImport)>,
    /// Ids of the modules enclosing the current node, innermost last.
    module_stack: Vec<rustc_ast::NodeId>,
}

rustc_session::impl_lint_pass!(VerifiedPurity => [VERIFIED_PURITY]);

fn is_in_verified_module(cx: &EarlyContext<'_>, span: Span) -> bool {
    let source_map = cx.sess().source_map();
    let filename = source_map.span_to_filename(span);
    let display = filename.prefer_local_unconditionally();
    let path = format!("{display}");
    path.contains("/verified/") || path.contains("\\verified\\")
}

/// Impure path prefixes that should not appear in verified modules.
const IMPURE_PATHS: &[&str] = &[
    "std::io::",
    "std::fs::",
    "std::net::",
    "std::process::",
    "std::time::SystemTime",
    "std::time::Instant",
    "tokio::",
    "async_std::",
];

/// Wall-clock types that a `std::time::*` glob import brings into scope.
const GLOB_CLOCK_TYPES: &[&str] = &["Instant", "SystemTime"];

fn impure_message(path_str: &str) -> Option<&'static str> {
    if !IMPURE_PATHS.iter().any(|impure| path_str.contains(impure)) {
        return None;
    }

    Some(if path_str.contains("time") {
        "system time access in verified module"
    } else if path_str.contains("io") || path_str.contains("fs") {
        "I/O operation in verified module"
    } else if path_str.contains("net") {
        "network operation in verified module"
    } else if path_str.contains("process") {
        "subprocess operation in verified module"
    } else {
        "impure operation in verified module"
    })
}

/// Calls `visit` with the full path and the leaf of every import in `tree`.
fn for_each_import<'a>(
    tree: &'a rustc_ast::UseTree,
    mut visit: impl FnMut(Vec<rustc_span::Symbol>, &'a rustc_ast::UseTree),
) {
    let mut pending = vec![(Vec::new(), tree)];
    while let Some((mut path, tree)) = pending.pop() {
        path.extend(tree.prefix.segments.iter().map(|segment| segment.ident.name));
        match &tree.kind {
            rustc_ast::UseTreeKind::Nested { items, .. } => {
                pending.extend(items.iter().rev().map(|(nested, _)| (path.clone(), nested)));
            }
            rustc_ast::UseTreeKind::Simple(_) | rustc_ast::UseTreeKind::Glob => visit(path, tree),
        }
    }
}

fn joined_path(path: &[rustc_span::Symbol]) -> String {
    path.iter().map(rustc_span::Symbol::as_str).collect::<Vec<_>>().join("::")
}

/// Importing an impure path is itself the violation: later uses such as
/// `Instant::now()` no longer spell the impure owner path.
fn check_use_tree(cx: &EarlyContext<'_>, tree: &rustc_ast::UseTree) {
    for_each_import(tree, |path, leaf| {
        // A trailing separator lets module imports such as `use std::fs;` match `std::fs::`.
        let path_str = format!("{}::", joined_path(&path));
        if let Some(msg) = impure_message(&path_str) {
            emit_tiger_style_lint(cx, VERIFIED_PURITY, leaf.span, msg, "verified_purity");
        }
    });
}

fn is_std_time(path: &[rustc_span::Symbol]) -> bool {
    matches!(path, [std, time] if std.as_str() == "std" && time.as_str() == "time")
}

/// How one import binds the `std::time` module itself, if it does.
fn time_import(mut path: Vec<rustc_span::Symbol>, leaf: &rustc_ast::UseTree) -> Option<TimeImport> {
    match &leaf.kind {
        rustc_ast::UseTreeKind::Glob => is_std_time(&path).then_some(TimeImport::Glob),
        rustc_ast::UseTreeKind::Simple(rename) => {
            if path.last() == Some(&rustc_span::kw::SelfLower) {
                path.pop();
            }
            is_std_time(&path).then(|| TimeImport::Module(rename.map_or(path[1], |ident| ident.name)))
        }
        rustc_ast::UseTreeKind::Nested { .. } => None,
    }
}

/// Test-only items: `cfg(test)` gates and, under `--test`, harness test functions.
fn item_is_test(
    cx: &EarlyContext<'_>,
    harness_test_fns: &std::collections::HashSet<rustc_ast::NodeId>,
    item: &Item,
) -> bool {
    harness_test_fns.contains(&item.id) || crate::test_context::ast_attrs_indicate_test(cx.sess(), &item.attrs)
}

/// Records how verified files import the `std::time` module itself, outside
/// test-only code. Such imports are not violations, since the module also
/// holds the pure `Duration`, but they let later paths like
/// `time::Instant::now()` reach the wall clock without spelling
/// `std::time::Instant`.
struct TimeImportCollector<'a, 'b> {
    cx: &'a EarlyContext<'b>,
    harness_test_fns: &'a std::collections::HashSet<rustc_ast::NodeId>,
    /// Id of the innermost enclosing module.
    module: rustc_ast::NodeId,
    /// Span of the innermost enclosing block within that module.
    block: Option<Span>,
    module_imports: Vec<(rustc_ast::NodeId, TimeImport)>,
    block_imports: Vec<(rustc_ast::NodeId, Span, TimeImport)>,
}

impl<'ast> rustc_ast::visit::Visitor<'ast> for TimeImportCollector<'_, '_> {
    fn visit_item(&mut self, item: &'ast Item) {
        if item_is_test(self.cx, self.harness_test_fns, item) {
            return;
        }
        match &item.kind {
            // A file-backed module's span is its `mod` declaration in the parent
            // file, so modules are walked wherever they are declared.
            ItemKind::Mod(..) => {
                let enclosing = (self.module, self.block.take());
                self.module = item.id;
                rustc_ast::visit::walk_item(self, item);
                (self.module, self.block) = enclosing;
            }
            _ if !is_in_verified_module(self.cx, item.span) => {}
            ItemKind::Use(tree) => for_each_import(tree, |path, leaf| {
                let Some(import) = time_import(path, leaf) else {
                    return;
                };
                match self.block {
                    Some(block) => self.block_imports.push((self.module, block, import)),
                    None => self.module_imports.push((self.module, import)),
                }
            }),
            _ => rustc_ast::visit::walk_item(self, item),
        }
    }

    fn visit_assoc_item(&mut self, item: &'ast rustc_ast::AssocItem, ctxt: rustc_ast::visit::AssocCtxt) {
        if !crate::test_context::ast_attrs_indicate_test(self.cx.sess(), &item.attrs) {
            rustc_ast::visit::walk_assoc_item(self, item, ctxt);
        }
    }

    fn visit_block(&mut self, block: &'ast rustc_ast::Block) {
        let enclosing = self.block.replace(block.span);
        rustc_ast::visit::walk_block(self, block);
        self.block = enclosing;
    }
}

impl VerifiedPurity {
    fn record_time_module_imports(&mut self, cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) {
        let mut collector = TimeImportCollector {
            cx,
            harness_test_fns: &self.harness_test_fns,
            module: rustc_ast::CRATE_NODE_ID,
            block: None,
            module_imports: Vec::new(),
            block_imports: Vec::new(),
        };
        rustc_ast::visit::walk_crate(&mut collector, krate);
        let (module_imports, block_imports) = (collector.module_imports, collector.block_imports);
        self.time_imports = module_imports;
        self.block_time_imports = block_imports;
    }

    /// Whether `import` is visible at `span`: imported by the current module,
    /// or by an enclosing block of that module.
    fn imports_in_scope(&self, span: Span, import: TimeImport) -> bool {
        let module = self.module_stack.last().copied().unwrap_or(rustc_ast::CRATE_NODE_ID);
        self.time_imports.contains(&(module, import))
            || self.block_time_imports.iter().any(|&(block_module, block, block_import)| {
                block_module == module && block_import == import && block.contains(span)
            })
    }

    /// The path an expression spells, with the `std::time` imports in scope
    /// expanded.
    fn expanded_expr_path(&self, span: Span, path: &rustc_ast::Path) -> String {
        let spelled = path.segments.iter().map(|segment| segment.ident.name).collect::<Vec<_>>();
        match spelled.as_slice() {
            [first, rest @ ..] if self.imports_in_scope(span, TimeImport::Module(*first)) => {
                format!("std::time::{}", joined_path(rest))
            }
            [first, ..]
                if GLOB_CLOCK_TYPES.contains(&first.as_str()) && self.imports_in_scope(span, TimeImport::Glob) =>
            {
                format!("std::time::{}", joined_path(&spelled))
            }
            _ => joined_path(&spelled),
        }
    }

    fn enter_item_scope(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if item_is_test(cx, &self.harness_test_fns, item) {
            self.test_depth += 1;
        }
    }

    fn exit_item_scope(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if item_is_test(cx, &self.harness_test_fns, item) {
            self.test_depth -= 1;
        }
    }

    fn enter_test_scope(&mut self, cx: &EarlyContext<'_>, attrs: &[rustc_ast::Attribute]) {
        if crate::test_context::ast_attrs_indicate_test(cx.sess(), attrs) {
            self.test_depth += 1;
        }
    }

    fn exit_test_scope(&mut self, cx: &EarlyContext<'_>, attrs: &[rustc_ast::Attribute]) {
        if crate::test_context::ast_attrs_indicate_test(cx.sess(), attrs) {
            self.test_depth -= 1;
        }
    }

    fn is_checked(&self, cx: &EarlyContext<'_>, span: Span) -> bool {
        self.test_depth == 0 && is_in_verified_module(cx, span)
    }
}

impl EarlyLintPass for VerifiedPurity {
    fn check_crate(&mut self, cx: &EarlyContext<'_>, krate: &rustc_ast::Crate) {
        self.harness_test_fns = crate::test_context::ast_harness_test_fns(cx.sess(), krate);
        self.record_time_module_imports(cx, krate);
    }

    fn check_item(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        if matches!(item.kind, ItemKind::Mod(..)) {
            self.module_stack.push(item.id);
        }
        self.enter_item_scope(cx, item);
        if !self.is_checked(cx, item.span) {
            return;
        }

        if let ItemKind::Use(tree) = &item.kind {
            check_use_tree(cx, tree);
        }
    }

    fn check_item_post(&mut self, cx: &EarlyContext<'_>, item: &Item) {
        self.exit_item_scope(cx, item);
        if matches!(item.kind, ItemKind::Mod(..)) {
            self.module_stack.pop();
        }
    }

    fn check_impl_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        self.enter_test_scope(cx, &item.attrs);
    }

    fn check_impl_item_post(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        self.exit_test_scope(cx, &item.attrs);
    }

    fn check_trait_item(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        self.enter_test_scope(cx, &item.attrs);
    }

    fn check_trait_item_post(&mut self, cx: &EarlyContext<'_>, item: &rustc_ast::AssocItem) {
        self.exit_test_scope(cx, &item.attrs);
    }

    fn check_fn(
        &mut self,
        cx: &EarlyContext<'_>,
        kind: rustc_ast::visit::FnKind<'_>,
        span: Span,
        _: rustc_ast::NodeId,
    ) {
        // Free functions, inherent and trait methods, and trait declarations
        // all arrive here, so every `async fn` form is covered.
        if let rustc_ast::visit::FnKind::Fn(_, _, func) = kind
            && func.sig.header.coroutine_kind.is_some()
            && self.is_checked(cx, span)
        {
            emit_tiger_style_lint(cx, VERIFIED_PURITY, span, "async function in verified module", "verified_purity");
        }
    }

    fn check_expr(&mut self, cx: &EarlyContext<'_>, expr: &Expr) {
        if !self.is_checked(cx, expr.span) {
            return;
        }

        match &expr.kind {
            ExprKind::Gen(_, _, rustc_ast::GenBlockKind::Async | rustc_ast::GenBlockKind::AsyncGen, _) => {
                emit_tiger_style_lint(
                    cx,
                    VERIFIED_PURITY,
                    expr.span,
                    "async block in verified module",
                    "verified_purity",
                );
            }
            ExprKind::Closure(closure) if closure.coroutine_kind.is_some() => {
                emit_tiger_style_lint(
                    cx,
                    VERIFIED_PURITY,
                    expr.span,
                    "async closure in verified module",
                    "verified_purity",
                );
            }
            // Check for impure path references in expressions
            ExprKind::Path(_, path) => {
                if let Some(msg) = impure_message(&self.expanded_expr_path(expr.span, path)) {
                    emit_tiger_style_lint(cx, VERIFIED_PURITY, expr.span, msg, "verified_purity");
                }
            }
            _ => {}
        }
    }
}

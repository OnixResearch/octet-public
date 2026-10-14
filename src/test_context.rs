use rustc_hir as hir;
use rustc_lint::LateContext;
use rustc_lint::LintContext;
use rustc_session::Session;
use rustc_span::Span;
use rustc_span::sym;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct TruthDomain {
    can_be_true: bool,
    can_be_false: bool,
}

impl TruthDomain {
    const KNOWN_TRUE: Self = Self {
        can_be_true: true,
        can_be_false: false,
    };
    const KNOWN_FALSE: Self = Self {
        can_be_true: false,
        can_be_false: true,
    };
    const UNKNOWN: Self = Self {
        can_be_true: true,
        can_be_false: true,
    };

    fn all(parts: &[Self]) -> Self {
        Self {
            can_be_true: parts.iter().all(|part| part.can_be_true),
            can_be_false: parts.iter().any(|part| part.can_be_false),
        }
    }

    fn any(parts: &[Self]) -> Self {
        Self {
            can_be_true: parts.iter().any(|part| part.can_be_true),
            can_be_false: parts.iter().all(|part| part.can_be_false),
        }
    }

    fn not(self) -> Self {
        Self {
            can_be_true: self.can_be_false,
            can_be_false: self.can_be_true,
        }
    }
}

pub(crate) fn hir_attrs_indicate_test(sess: &Session, attrs: &[hir::Attribute]) -> bool {
    attrs.iter().any(|attr| hir_attr_indicates_test(sess, attr))
}

pub(crate) fn late_hir_id_is_in_test_context(cx: &LateContext<'_>, hir_id: hir::HirId, _span: Span) -> bool {
    for (_, node) in cx.tcx.hir_parent_iter(hir_id) {
        if let hir::Node::Item(item) = node
            && (hir_attrs_indicate_test(cx.sess(), cx.tcx.hir_attrs(item.hir_id()))
                || item_span_indicates_test(cx.sess(), item.span))
        {
            return true;
        }
    }

    false
}

pub(crate) fn item_span_indicates_test(sess: &Session, span: Span) -> bool {
    let source_map = sess.source_map();
    let Ok(lines) = source_map.span_to_lines(span) else {
        return false;
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return false;
    };

    for idx in (0..first_line_idx).rev() {
        let Some(line) = lines.file.get_line(idx) else {
            continue;
        };
        let trimmed = line.trim();

        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }

        if trimmed.starts_with("#[") || trimmed.starts_with("#![") {
            if cfg_attr_source_requires_test(trimmed) {
                return true;
            }
            continue;
        }

        break;
    }

    false
}

fn hir_attr_indicates_test(sess: &Session, attr: &hir::Attribute) -> bool {
    if attr.has_name(sym::test) {
        return true;
    }

    if !attr.has_name(sym::cfg) {
        return false;
    }

    let Ok(source) = sess.source_map().span_to_snippet(attr.span()) else {
        return false;
    };

    cfg_attr_source_requires_test(&source)
}

fn cfg_attr_source_requires_test(source: &str) -> bool {
    let trimmed = source.trim();
    let inner = trimmed
        .strip_prefix("#[")
        .and_then(|rest| rest.strip_suffix(']'))
        .or_else(|| trimmed.strip_prefix("#![").and_then(|rest| rest.strip_suffix(']')))
        .unwrap_or(trimmed)
        .trim();

    let Some(cfg_expr) = inner.strip_prefix("cfg(").and_then(|rest| rest.strip_suffix(')')) else {
        return false;
    };

    !cfg_expr_truth(cfg_expr.trim(), false).can_be_true
}

fn cfg_expr_truth(expr: &str, test_enabled: bool) -> TruthDomain {
    let expr = expr.trim();

    if expr == "test" {
        return if test_enabled {
            TruthDomain::KNOWN_TRUE
        } else {
            TruthDomain::KNOWN_FALSE
        };
    }

    if let Some(args) = parse_cfg_call(expr, "all") {
        let parts = split_cfg_args(args).into_iter().map(|part| cfg_expr_truth(part, test_enabled)).collect::<Vec<_>>();
        return TruthDomain::all(&parts);
    }

    if let Some(args) = parse_cfg_call(expr, "any") {
        let parts = split_cfg_args(args).into_iter().map(|part| cfg_expr_truth(part, test_enabled)).collect::<Vec<_>>();
        return TruthDomain::any(&parts);
    }

    if let Some(args) = parse_cfg_call(expr, "not") {
        let parts = split_cfg_args(args);
        if let [part] = parts.as_slice() {
            return cfg_expr_truth(part, test_enabled).not();
        }
        return TruthDomain::UNKNOWN;
    }

    TruthDomain::UNKNOWN
}

fn parse_cfg_call<'a>(expr: &'a str, name: &str) -> Option<&'a str> {
    let rest = expr.strip_prefix(name)?.trim_start();
    let inner = rest.strip_prefix('(')?.strip_suffix(')')?;
    Some(inner)
}

fn split_cfg_args(source: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (idx, ch) in source.char_indices() {
        if in_string {
            if escaped {
                escaped = false;
                continue;
            }
            match ch {
                '\\' => escaped = true,
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }

        match ch {
            '"' => in_string = true,
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            ',' if depth == 0 => {
                parts.push(source[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }

    let tail = source[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }

    parts
}

#[cfg(test)]
mod tests {
    use super::cfg_attr_source_requires_test;
    use super::cfg_expr_truth;

    #[test]
    fn cfg_attr_detection_matches_test_only_forms() {
        assert!(cfg_attr_source_requires_test("#[cfg(test)]"));
        assert!(cfg_attr_source_requires_test("#![cfg(test)]"));
        assert!(cfg_attr_source_requires_test("#[cfg(all(test, feature = \"bench\"))]"));
        assert!(cfg_attr_source_requires_test("cfg(not(not(test)))"));
        assert!(!cfg_attr_source_requires_test("#[cfg(any(test, feature = \"bench\"))]"));
        assert!(!cfg_attr_source_requires_test("#[cfg(not(test))]"));
        assert!(!cfg_attr_source_requires_test("#![cfg(not(test))]"));
        assert!(!cfg_attr_source_requires_test("#[cfg(feature = \"testing\")]"));
        assert!(!cfg_attr_source_requires_test("#[allow(dead_code)]"));
    }

    #[test]
    fn cfg_not_test_can_still_be_true_without_test() {
        assert!(cfg_expr_truth("not(test)", false).can_be_true);
        assert!(!cfg_expr_truth("all(test, feature = \"bench\")", false).can_be_true);
    }
}

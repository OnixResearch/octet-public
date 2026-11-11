mod macro_body;

use std::collections::HashSet;

use macro_body::BodyAttribute;
use macro_body::Reason;
use rustc_ast::AttrId;
use rustc_ast::Attribute;
use rustc_ast::MacroDef;
use rustc_ast::MetaItemInner;
use rustc_errors::Applicability;
use rustc_lint::EarlyContext;
use rustc_lint::EarlyLintPass;
use rustc_lint::LintContext;
use rustc_session::declare_tool_lint;
use rustc_session::impl_lint_pass;
use rustc_span::BytePos;
use rustc_span::Span;
use rustc_span::sym;

use crate::diagnostics::LintSuggestion;
use crate::diagnostics::emit_tiger_style_lint;
use crate::diagnostics::emit_tiger_style_lint_with_suggestion;

declare_tool_lint! {
    /// Detects `#[allow(...)]` and `#[expect(...)]` attributes that suppress
    /// Clippy or Octet lints without a justification. Governed lint paths are
    /// `clippy::*`, `tigerstyle::*` (including Octet groups such as
    /// `tigerstyle::safety`), and Octet's bare compatibility aliases such as
    /// `no_unwrap`. rustc's own lints stay out of scope, and `no_todo` allows
    /// belong to `unjustified_no_todo_allow`. Only attributes written in the
    /// crate's source (directly, through `cfg_attr`, or in a local
    /// `macro_rules!`) are governed, not output of other crates' macros. A
    /// macro body's literal attributes are also checked where the macro is
    /// defined, so an exported macro is governed in its own crate. Under a
    /// selected strict profile, a `#[warn(...)]` that reaches one of the
    /// profile's required lints lowers it from `deny` and is governed too.
    ///
    /// A non-doc `//` comment on the attribute's lines or on the line directly
    /// above it, or a non-empty `reason = "..."` argument inside the attribute,
    /// counts as the justification. A comment or reason that starts with the
    /// `TODO(octet)` placeholder does not. Unjustified suppressions hide real
    /// bugs — verified-logic documents every `#[allow(...)]` with a rationale.
    pub tigerstyle::UNJUSTIFIED_ALLOW,
    Allow,
    "`#[allow]` attribute without justification comment or reason"
}

/// Early pass state for [`UNJUSTIFIED_ALLOW`].
#[derive(Clone, Default)]
pub struct UnjustifiedAllow {
    /// Paths that reach a selected strict profile's floor lints, other than
    /// the governance lints [`SuppressionGovernanceGuard`] protects. A source
    /// `warn` through one of them lowers a floor lint from `deny`, so it is
    /// governed like an `allow`. Empty without a profile, where `warn` is the
    /// normal way to enable a lint.
    floor_paths: LintPathSet,
    /// Paths that reach the governance lints the profile guard protects. The
    /// guard rejects an attribute through one of them outright, so this lint
    /// skips it rather than also asking for a justification that could never
    /// make it legal. Empty without a profile.
    guarded_paths: LintPathSet,
    dedupe: ReportDedupe,
}

impl_lint_pass!(UnjustifiedAllow => [UNJUSTIFIED_ALLOW]);

impl UnjustifiedAllow {
    /// Pass for a selected strict profile's floor lints; an empty floor gives
    /// the unprofiled pass.
    pub(crate) fn for_profile_floor<'a>(floor_lints: impl IntoIterator<Item = &'a str>) -> Self {
        let (governance, floor): (Vec<&str>, Vec<&str>) =
            floor_lints.into_iter().partition(|lint_name| SUPPRESSION_GOVERNANCE_LINTS.contains(lint_name));
        Self {
            floor_paths: LintPathSet::for_lints(floor),
            guarded_paths: LintPathSet::for_lints(governance),
            dedupe: ReportDedupe::default(),
        }
    }
}

/// Placeholder this lint's own suggestion inserts; it never justifies.
const PLACEHOLDER_PREFIX: &str = "TODO(octet)";
/// Lint tools whose suppressions this lint governs.
const GOVERNED_TOOLS: &[&str] = &["clippy", octet_lint_metadata::CANONICAL_NAMESPACE];
/// Octet lint whose allows `unjustified_no_todo_allow` audits instead.
const DEDICATED_AUDIT_LINT: &str = "no_todo";

/// Whether a lint path inside `allow(...)` / `expect(...)` names a Clippy
/// lint, an Octet lint or group, or an Octet bare compatibility alias.
fn is_governed_lint_path<'a>(mut segments: impl Iterator<Item = &'a str>) -> bool {
    match (segments.next(), segments.next()) {
        (Some(tool), Some(lint)) => {
            GOVERNED_TOOLS.contains(&tool)
                && !(tool == octet_lint_metadata::CANONICAL_NAMESPACE && lint == DEDICATED_AUDIT_LINT)
        }
        (Some(name), None) => {
            name != DEDICATED_AUDIT_LINT && crate::LINT_INVENTORY.iter().any(|entry| entry.name == name)
        }
        _ => false,
    }
}

/// Whether comment or reason text explains the suppression: non-empty and not
/// the unedited placeholder.
fn is_justification_text(text: &str) -> bool {
    let text = text.trim();
    !text.is_empty() && !text.starts_with(PLACEHOLDER_PREFIX)
}

/// Whether `comment`, which starts at `//`, is a justifying non-doc line
/// comment. `///` and `//!` doc comments document items, not suppressions.
fn is_justifying_line_comment(comment: &str) -> bool {
    comment
        .trim_start()
        .strip_prefix("//")
        .is_some_and(|body| !body.starts_with('/') && !body.starts_with('!') && is_justification_text(body))
}

/// Whether a source line holds a justifying `//` comment outside string
/// literals, wherever it starts on the line.
fn line_has_justifying_comment(line: &str) -> bool {
    let mut chars = line.char_indices().peekable();
    let mut is_in_string = false;
    while let Some((offset, character)) = chars.next() {
        if is_in_string {
            match character {
                '\\' => {
                    chars.next();
                }
                '"' => is_in_string = false,
                _ => {}
            }
        } else if character == '"' {
            is_in_string = true;
        } else if character == '/' && chars.peek().is_some_and(|&(_, next)| next == '/') {
            return is_justifying_line_comment(&line[offset..]);
        }
    }
    false
}

fn suppression_name(attr: &Attribute) -> Option<&'static str> {
    if attr.has_name(sym::allow) {
        Some("allow")
    } else if attr.has_name(sym::expect) {
        Some("expect")
    } else {
        None
    }
}

fn suppresses_governed_lint(items: &[MetaItemInner]) -> bool {
    items
        .iter()
        .filter_map(MetaItemInner::meta_item)
        .any(|meta| is_governed_lint_path(meta.path.segments.iter().map(|segment| segment.ident.as_str())))
}

fn has_reason_argument(items: &[MetaItemInner]) -> bool {
    items.iter().any(|item| {
        item.meta_item()
            .filter(|meta| meta.has_name(sym::reason))
            .and_then(rustc_ast::MetaItem::value_str)
            .is_some_and(|reason| is_justification_text(reason.as_str()))
    })
}

/// Where a user-written lint-level attribute sits in source.
#[derive(Clone, Copy)]
enum AttributeSite {
    /// The attribute's own span covers its name: written directly, through
    /// `cfg_attr`, or literally in a local `macro_rules!` body.
    Whole(Span),
    /// Only the attribute name is the user's text: the attribute was
    /// forwarded through a local macro's `meta` or `ident` fragment, so its
    /// own span points at the macro body. Reported at the name.
    Forwarded(Span),
}

impl AttributeSite {
    fn span(self) -> Span {
        match self {
            Self::Whole(span) | Self::Forwarded(span) => span,
        }
    }
}

/// Site of a lint-level attribute the user wrote, or `None` when a macro from
/// another crate generated it. The decision follows the attribute's name
/// token: user text spelling `name`, outside another crate's macro. The name
/// segment's ident span is used because a name forwarded through an `ident`
/// fragment keeps the user's span there, while the path span points at the
/// macro body. Proc macros such as `#[tokio::main]` re-span generated
/// attributes onto user tokens, whose text spells something else.
fn user_attribute_site(cx: &EarlyContext<'_>, attr: &Attribute, name: &str) -> Option<AttributeSite> {
    let source_map = cx.sess().source_map();
    let path_span = attr.get_normal_item().path.segments.last()?.ident.span;
    let is_user_name = !path_span.in_external_macro(source_map)
        && source_map.span_to_snippet(path_span).is_ok_and(|snippet| snippet == name);
    if !is_user_name {
        return None;
    }
    Some(if attr.span.contains(path_span) {
        AttributeSite::Whole(attr.span)
    } else {
        AttributeSite::Forwarded(path_span)
    })
}

fn justification_stub(cx: &EarlyContext<'_>, span: Span) -> String {
    let source_map = cx.sess().source_map();
    let indent_width = source_map.lookup_char_pos(span.lo()).col.0;
    let indent = " ".repeat(indent_width);
    format!("// {PLACEHOLDER_PREFIX}: explain why this allow is safe here\n{indent}")
}

/// Whether a justifying comment sits on the attribute's own source lines or on
/// the line directly above them.
fn has_comment_nearby(cx: &EarlyContext<'_>, span: Span) -> bool {
    let Ok(lines) = cx.sess().source_map().span_to_lines(span) else {
        return true; // assume justified if we can't read source
    };
    let Some(first_line_idx) = lines.lines.first().map(|line| line.line_index) else {
        return true;
    };

    let has_attribute_line_comment = lines
        .lines
        .iter()
        .filter_map(|line| lines.file.get_line(line.line_index))
        .any(|line| line_has_justifying_comment(&line));
    let has_previous_line_comment = first_line_idx
        .checked_sub(1)
        .and_then(|line_idx| lines.file.get_line(line_idx))
        .is_some_and(|line| is_justifying_line_comment(&line));

    has_attribute_line_comment || has_previous_line_comment
}

/// Lint paths of the attribute's lint list, as segment iterators.
fn lint_paths(items: &[MetaItemInner]) -> impl Iterator<Item = impl Iterator<Item = &str> + Clone> {
    items
        .iter()
        .filter_map(MetaItemInner::meta_item)
        .map(|meta| meta.path.segments.iter().map(|segment| segment.ident.as_str()))
}

/// Lints of `set` that a `deny` or `forbid` among `later` attributes raises
/// back. rustc applies a node's lint attributes in order, so a later raise
/// undoes an earlier `allow`, `expect`, or `warn` of the same lint.
fn raised_lints(set: &LintPathSet, later: &[Attribute]) -> Vec<&'static str> {
    let mut raised = Vec::new();
    for attr in later.iter().filter(|attr| attr.has_name(sym::deny) || attr.has_name(sym::forbid)) {
        let Some(items) = attr.meta_item_list() else {
            continue;
        };
        for lint_name in set.reached_lints(lint_paths(&items)) {
            if !raised.contains(&lint_name) {
                raised.push(lint_name);
            }
        }
    }
    raised
}

/// Lints of `set` that the attribute lowers and no `later` attribute on the
/// same node raises back.
fn lowered_lints(set: &LintPathSet, items: &[MetaItemInner], later: &[Attribute]) -> Vec<&'static str> {
    let mut lowered = set.reached_lints(lint_paths(items));
    if !lowered.is_empty() {
        let raised = raised_lints(set, later);
        lowered.retain(|lint_name| !raised.contains(lint_name));
    }
    lowered
}

/// Name of a lint-level attribute this lint governs: `allow` or `expect`
/// naming a governed lint, or, under a strict profile, `warn` lowering a floor
/// lint that no `later` attribute raises back.
fn governed_level_name(
    attr: &Attribute,
    items: &[MetaItemInner],
    later: &[Attribute],
    floor_paths: &LintPathSet,
) -> Option<&'static str> {
    match suppression_name(attr) {
        Some(name) => suppresses_governed_lint(items).then_some(name),
        None => (attr.has_name(sym::warn) && !lowered_lints(floor_paths, items, later).is_empty()).then_some("warn"),
    }
}

impl UnjustifiedAllow {
    fn check_attrs(&mut self, cx: &EarlyContext<'_>, attrs: &[Attribute]) {
        // Skip items in #[cfg(test)] blocks
        if attrs.iter().any(|a| a.has_name(sym::test)) {
            return;
        }

        for (index, attr) in attrs.iter().enumerate() {
            let later = &attrs[index + 1..];
            if lowering_level_name(attr).is_none() {
                continue;
            }
            let Some(items) = attr.meta_item_list() else {
                continue;
            };
            let Some(name) = governed_level_name(attr, &items, later, &self.floor_paths) else {
                continue;
            };
            // The profile guard reports this attribute as an error to remove.
            if !lowered_lints(&self.guarded_paths, &items, later).is_empty() {
                continue;
            }
            let Some(site) = user_attribute_site(cx, attr, name) else {
                continue;
            };
            if has_reason_argument(&items)
                || has_comment_nearby(cx, site.span())
                || !self.dedupe.is_first_report(site.span())
            {
                continue;
            }
            emit_unjustified(cx, site, name);
        }
    }

    /// Checks the literal lint-level attributes in a macro body where the macro
    /// is defined, with the same rules as [`Self::check_attrs`].
    fn check_macro_body(&mut self, cx: &EarlyContext<'_>, mac: &MacroDef) {
        for run in macro_body::attribute_runs(&mac.body.tokens, mac.macro_rules) {
            for (index, attribute) in run.iter().enumerate() {
                let later = &run[index + 1..];
                let Some(name) = body_lowering_level_name(attribute) else {
                    continue;
                };
                let is_governed = if name == "warn" {
                    match body_lowered_lints(&self.floor_paths, attribute, later) {
                        Some(lowered) => !lowered.is_empty(),
                        None => continue,
                    }
                } else {
                    attribute.lint_paths().any(is_governed_lint_path)
                };
                // The profile guard reports an attribute it reaches as an
                // error; one it might reach is left to the expansion site.
                let is_guarded =
                    body_lowered_lints(&self.guarded_paths, attribute, later).is_none_or(|lowered| !lowered.is_empty());
                if !is_governed || is_guarded {
                    continue;
                }
                let has_reason = match attribute.reason {
                    Reason::Absent => false,
                    Reason::Literal(reason) => is_justification_text(reason.as_str()),
                    Reason::Fragment => true,
                };
                if has_reason
                    || has_comment_nearby(cx, attribute.span)
                    || !self.dedupe.is_first_definition_report(attribute.span)
                {
                    continue;
                }
                emit_unjustified(cx, AttributeSite::Whole(attribute.span), name);
            }
        }
    }
}

fn emit_unjustified(cx: &EarlyContext<'_>, site: AttributeSite, name: &str) {
    let message = format!("`#[{name}(...)]` without justification comment");
    match site {
        AttributeSite::Whole(span) => emit_tiger_style_lint_with_suggestion(
            cx,
            UNJUSTIFIED_ALLOW,
            span,
            message,
            "unjustified_allow",
            LintSuggestion::Single {
                span: span.shrink_to_lo(),
                message: "add a local justification comment".to_owned(),
                replacement: justification_stub(cx, span),
                applicability: Applicability::HasPlaceholders,
            },
        ),
        AttributeSite::Forwarded(span) => {
            emit_tiger_style_lint(cx, UNJUSTIFIED_ALLOW, span, message, "unjustified_allow");
        }
    }
}

/// Name of a macro-body attribute that can set a lint below `deny`.
fn body_lowering_level_name(attribute: &BodyAttribute) -> Option<&'static str> {
    [(sym::allow, "allow"), (sym::expect, "expect"), (sym::warn, "warn")]
        .into_iter()
        .find_map(|(symbol, name)| (attribute.name == symbol).then_some(name))
}

/// Lints of `set` that a macro-body attribute lowers and no later `deny` or
/// `forbid` in its run of attributes raises back, or `None` when it lowers
/// some but a `$(...)` repetition later in the run may raise them, which only
/// an expansion decides.
fn body_lowered_lints(
    set: &LintPathSet,
    attribute: &BodyAttribute,
    later: &[BodyAttribute],
) -> Option<Vec<&'static str>> {
    let mut lowered = set.reached_lints(attribute.lint_paths());
    if lowered.is_empty() {
        return Some(lowered);
    }
    if attribute.is_raise_undecided {
        return None;
    }
    let raised = later
        .iter()
        .filter(|raise| raise.name == sym::deny || raise.name == sym::forbid)
        .flat_map(|raise| set.reached_lints(raise.lint_paths()))
        .collect::<Vec<_>>();
    lowered.retain(|lint_name| !raised.contains(lint_name));
    Some(lowered)
}

/// Report deduplication shared by both passes.
#[derive(Clone, Default)]
struct ReportDedupe {
    /// rustc reports a statement's attributes once for the statement and again
    /// for the `let`, item, or expression it wraps; this remembers the first
    /// attribute of the last checked list so each list is checked once.
    last_checked: Option<AttrId>,
    /// Definition-site positions of attributes already reported from local
    /// macro expansions. Each expansion brings fresh attribute ids and syntax
    /// contexts, so an attribute in a macro body would otherwise be reported
    /// once per invocation.
    macro_sites: HashSet<(BytePos, BytePos)>,
}

impl ReportDedupe {
    /// Whether `attrs` differs from the list checked last.
    fn is_new_attribute_list(&mut self, attrs: &[Attribute]) -> bool {
        let Some(first) = attrs.first() else {
            return false;
        };
        if self.last_checked == Some(first.id) {
            return false;
        }
        self.last_checked = Some(first.id);
        true
    }

    /// Whether a report at `span` has not been made yet: always for source
    /// spans, once per definition site for spans from a macro expansion.
    fn is_first_report(&mut self, span: Span) -> bool {
        !span.from_expansion() || self.macro_sites.insert((span.lo(), span.hi()))
    }

    /// Whether a report at `span`, an attribute in a macro body checked where
    /// the macro is defined, has not been made yet. It shares the positions of
    /// expansion-site reports, so a local macro that is also expanded in the
    /// crate is reported once, by whichever check runs first.
    fn is_first_definition_report(&mut self, span: Span) -> bool {
        self.macro_sites.insert((span.lo(), span.hi()))
    }
}

impl EarlyLintPass for UnjustifiedAllow {
    // rustc calls this for the attributes of every lint-level node: the crate
    // root, items, associated and foreign items, statements, fields,
    // variants, expressions, arms, parameters, and generic parameters.
    fn check_attributes(&mut self, cx: &EarlyContext<'_>, attrs: &[Attribute]) {
        if self.dedupe.is_new_attribute_list(attrs) {
            self.check_attrs(cx, attrs);
        }
    }

    fn check_mac_def(&mut self, cx: &EarlyContext<'_>, mac: &MacroDef) {
        if !is_external_macro_output(cx, mac) {
            self.check_macro_body(cx, mac);
        }
    }
}

/// Whether a macro definition was produced by expanding another crate's
/// macro. Its body is that crate's code, checked where that crate defines it.
fn is_external_macro_output(cx: &EarlyContext<'_>, mac: &MacroDef) -> bool {
    mac.body.dspan.entire().in_external_macro(cx.sess().source_map())
}

/// Profile-floor lints that audit source suppressions.
const SUPPRESSION_GOVERNANCE_LINTS: &[&str] = &["unjustified_allow", "unjustified_no_todo_allow"];

/// A lint-attribute path that reaches one chosen Octet lint.
#[derive(Clone)]
struct LintPath {
    /// Bare alias, canonical lint name, or Octet group, as written in source.
    path: String,
    /// Lint the path reaches.
    lint_name: &'static str,
}

/// Lint-attribute paths that reach a chosen set of Octet lints: each lint's
/// bare alias, canonical name, and Octet group. A group therefore counts only
/// through a chosen member.
#[derive(Clone, Default)]
struct LintPathSet {
    paths: Vec<LintPath>,
}

impl LintPathSet {
    /// Paths for the inventory lints among `lint_names`; other names are
    /// ignored.
    fn for_lints<'a>(lint_names: impl IntoIterator<Item = &'a str>) -> Self {
        let paths = lint_names
            .into_iter()
            .filter_map(|lint_name| {
                crate::LINT_INVENTORY.iter().map(|entry| entry.name).find(|name| *name == lint_name)
            })
            .flat_map(|lint_name| {
                let group = crate::diagnostics::diagnostic_meta(lint_name).domain_group;
                [
                    lint_name.to_owned(),
                    crate::diagnostics::canonical_lint_name(lint_name),
                    group.to_owned(),
                ]
                .into_iter()
                .map(move |path| LintPath { path, lint_name })
            })
            .collect();
        Self { paths }
    }

    fn is_empty(&self) -> bool {
        self.paths.is_empty()
    }

    /// Chosen lints that the lint paths reach, in first-seen order.
    fn reached_lints<'a, P>(&self, paths: impl Iterator<Item = P>) -> Vec<&'static str>
    where P: Iterator<Item = &'a str> + Clone {
        let mut reached = Vec::new();
        for segments in paths {
            for path in &self.paths {
                if segments.clone().eq(path.path.split("::")) && !reached.contains(&path.lint_name) {
                    reached.push(path.lint_name);
                }
            }
        }
        reached
    }
}

/// Early pass a selected strict profile adds. rustc applies a node's lint
/// attributes before any lint inspects that node, so a source `allow`,
/// `expect`, or `warn` that reaches a suppression-governance lint would
/// silence its own audit. The guard reports such an attribute as a session
/// error, which no lint level can suppress, and leaves every other lint level
/// alone.
#[derive(Clone)]
pub struct SuppressionGovernanceGuard {
    profile_id: String,
    protected: LintPathSet,
    dedupe: ReportDedupe,
}

impl_lint_pass!(SuppressionGovernanceGuard => []);

impl SuppressionGovernanceGuard {
    /// Guard for the governance lints among a selected profile's floor, or
    /// `None` when the floor holds none of them.
    pub(crate) fn for_profile<'a>(profile_id: &str, floor_lints: impl IntoIterator<Item = &'a str>) -> Option<Self> {
        let protected = LintPathSet::for_lints(
            floor_lints.into_iter().filter(|lint_name| SUPPRESSION_GOVERNANCE_LINTS.contains(lint_name)),
        );
        (!protected.is_empty()).then(|| Self {
            profile_id: profile_id.to_owned(),
            protected,
            dedupe: ReportDedupe::default(),
        })
    }
}

/// Name of a source attribute that can set a lint below the profile's `deny`.
/// rustc source attributes have no `force_warn`; only the command line does.
fn lowering_level_name(attr: &Attribute) -> Option<&'static str> {
    suppression_name(attr).or_else(|| attr.has_name(sym::warn).then_some("warn"))
}

impl EarlyLintPass for SuppressionGovernanceGuard {
    fn check_attributes(&mut self, cx: &EarlyContext<'_>, attrs: &[Attribute]) {
        if !self.dedupe.is_new_attribute_list(attrs) {
            return;
        }
        for (index, attr) in attrs.iter().enumerate() {
            let Some(name) = lowering_level_name(attr) else {
                continue;
            };
            let Some(items) = attr.meta_item_list() else {
                continue;
            };
            let reached = lowered_lints(&self.protected, &items, &attrs[index + 1..]);
            if reached.is_empty() {
                continue;
            }
            let Some(site) = user_attribute_site(cx, attr, name) else {
                continue;
            };
            if self.dedupe.is_first_report(site.span()) {
                self.report(cx, site.span(), name, &reached);
            }
        }
    }

    fn check_mac_def(&mut self, cx: &EarlyContext<'_>, mac: &MacroDef) {
        if is_external_macro_output(cx, mac) {
            return;
        }
        for run in macro_body::attribute_runs(&mac.body.tokens, mac.macro_rules) {
            for (index, attribute) in run.iter().enumerate() {
                let Some(name) = body_lowering_level_name(attribute) else {
                    continue;
                };
                let reached = body_lowered_lints(&self.protected, attribute, &run[index + 1..]).unwrap_or_default();
                if !reached.is_empty() && self.dedupe.is_first_definition_report(attribute.span) {
                    self.report(cx, attribute.span, name, &reached);
                }
            }
        }
    }
}

impl SuppressionGovernanceGuard {
    fn report(&self, cx: &EarlyContext<'_>, span: Span, name: &str, reached: &[&str]) {
        let lints = reached.iter().map(|lint_name| format!("`{lint_name}`")).collect::<Vec<_>>().join(", ");
        cx.sess().dcx().span_err(
            span,
            format!(
                "strict profile `{}` requires {lints} at `deny`, but this `#[{name}(...)]` would silence it; source attributes cannot lower a profile's suppression-governance lints, so remove the attribute",
                self.profile_id
            ),
        );
    }
}

#[cfg(test)]
mod tests {
    use super::LintPathSet;
    use super::SuppressionGovernanceGuard;
    use super::UnjustifiedAllow;
    use super::is_governed_lint_path;
    use super::is_justifying_line_comment;
    use super::line_has_justifying_comment;

    fn governed(path: &str) -> bool {
        is_governed_lint_path(path.split("::"))
    }

    #[test]
    fn governs_clippy_octet_lints_groups_and_bare_aliases() {
        assert!(governed("clippy::needless_return"));
        assert!(governed("clippy::all"));
        assert!(governed("tigerstyle::no_unwrap"));
        assert!(governed("tigerstyle::zero_alloc_attr_contract"));
        assert!(governed("tigerstyle::safety"));
        assert!(governed("no_unwrap"));
        assert!(governed("zero_alloc_attr_contract"));
    }

    #[test]
    fn leaves_rustc_lints_and_no_todo_allows_alone() {
        assert!(!governed("dead_code"));
        assert!(!governed("rustdoc::broken_intra_doc_links"));
        assert!(!governed("reason"));
        assert!(!governed("no_todo"));
        assert!(!governed("tigerstyle::no_todo"));
    }

    #[test]
    fn previous_line_justification_excludes_doc_comments_and_placeholders() {
        assert!(is_justifying_line_comment("    // Verified casts: Verus checks every narrowing."));
        assert!(!is_justifying_line_comment("/// Returns a constant."));
        assert!(!is_justifying_line_comment("//! Crate docs."));
        assert!(!is_justifying_line_comment("//"));
        assert!(!is_justifying_line_comment("// TODO(octet): explain why this allow is safe here"));
        assert!(!is_justifying_line_comment("let value = 1; // trailing"));
    }

    #[test]
    fn attribute_line_justification_reads_the_whole_line() {
        assert!(line_has_justifying_comment("#[allow(clippy::needless_return)] // Mirrors the state table."));
        assert!(line_has_justifying_comment("    clippy::needless_return, // Mirrors the state table."));
        assert!(!line_has_justifying_comment("#[allow(clippy::needless_return)]"));
        assert!(!line_has_justifying_comment(
            "#[allow(clippy::needless_return)] // TODO(octet): explain why this allow is safe here"
        ));
        assert!(!line_has_justifying_comment("#[allow(clippy::x, reason = \"see https://example.com\")]"));
        assert!(!line_has_justifying_comment("#[allow(clippy::x)] /// Doc for the item."));
    }

    fn reached_paths(paths_set: &LintPathSet, paths: &[&str]) -> Vec<&'static str> {
        paths_set.reached_lints(paths.iter().map(|path| path.split("::")))
    }

    fn reached(guard: &SuppressionGovernanceGuard, paths: &[&str]) -> Vec<&'static str> {
        reached_paths(&guard.protected, paths)
    }

    #[test]
    fn profile_guard_resolves_governance_names_aliases_and_groups() {
        let guard = SuppressionGovernanceGuard::for_profile("robust", [
            "no_unwrap",
            "unjustified_allow",
            "unjustified_no_todo_allow",
        ])
        .expect("robust floor holds governance lints");

        assert_eq!(reached(&guard, &["unjustified_allow"]), ["unjustified_allow"]);
        assert_eq!(reached(&guard, &["tigerstyle::unjustified_allow"]), ["unjustified_allow"]);
        assert_eq!(reached(&guard, &["tigerstyle::structure"]), ["unjustified_allow"]);
        assert_eq!(reached(&guard, &["tigerstyle::safety"]), ["unjustified_no_todo_allow"]);
        assert_eq!(
            reached(&guard, &[
                "unjustified_no_todo_allow",
                "tigerstyle::structure",
                "unjustified_allow"
            ]),
            ["unjustified_no_todo_allow", "unjustified_allow"]
        );
        for path in [
            "no_unwrap",
            "tigerstyle::no_unwrap",
            "clippy::all",
            "structure",
            "nonstandard_style",
        ] {
            assert!(reached(&guard, &[path]).is_empty(), "`{path}` reaches no governance lint");
        }
    }

    #[test]
    fn profile_guard_exists_only_for_floors_with_governance_lints() {
        assert!(SuppressionGovernanceGuard::for_profile("custom", ["no_unwrap", "no_panic"]).is_none());
        assert!(SuppressionGovernanceGuard::for_profile("custom", std::iter::empty()).is_none());
    }

    #[test]
    fn profile_floor_governs_warn_paths_that_reach_non_governance_floor_lints() {
        let floor = UnjustifiedAllow::for_profile_floor(["no_unwrap", "platform_dependent_cast", "unjustified_allow"]);
        let reached = |paths: &[&str]| reached_paths(&floor.floor_paths, paths);

        assert_eq!(reached(&["no_unwrap"]), ["no_unwrap"]);
        assert_eq!(reached(&["tigerstyle::no_unwrap"]), ["no_unwrap"]);
        assert_eq!(reached(&["tigerstyle::safety"]), ["no_unwrap"]);
        assert_eq!(reached(&["tigerstyle::structure"]), ["platform_dependent_cast"]);
        // Non-floor lints and groups without a floor member are not lowered;
        // the governance lints belong to the profile guard.
        for path in [
            "function_length",
            "tigerstyle::function_length",
            "tigerstyle::naming",
            "unjustified_allow",
            "clippy::unwrap_used",
        ] {
            assert!(reached(&[path]).is_empty(), "`{path}` reaches no governed floor lint");
        }

        // The governance lints go to the guarded set, which this lint skips.
        assert_eq!(reached_paths(&floor.guarded_paths, &["tigerstyle::structure"]), ["unjustified_allow"]);
        assert!(reached_paths(&floor.guarded_paths, &["tigerstyle::safety"]).is_empty());

        // Without a profile no `warn` is governed.
        assert!(UnjustifiedAllow::default().floor_paths.is_empty());
        assert!(UnjustifiedAllow::for_profile_floor(std::iter::empty()).floor_paths.is_empty());
    }
}

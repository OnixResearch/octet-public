//! Lint-level attributes written literally in macro bodies.
//!
//! Where a macro is defined, an attribute in its transcriber is only tokens,
//! so the attribute checks see it there through this scan. That covers
//! `#[macro_export]` macros whose expansions land in other crates, where the
//! expansion-site checks treat the attribute as another crate's macro output.

use rustc_ast::token::Delimiter;
use rustc_ast::token::LitKind;
use rustc_ast::token::Token;
use rustc_ast::token::TokenKind;
use rustc_ast::tokenstream::TokenStream;
use rustc_ast::tokenstream::TokenTree;
use rustc_span::Span;
use rustc_span::Symbol;
use rustc_span::sym;

/// The `reason = ...` argument of a body attribute.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum Reason {
    Absent,
    /// A string literal; its text without the quotes.
    Literal(Symbol),
    /// A macro fragment, which only expansion decides.
    Fragment,
}

/// A lint-level attribute written literally in a macro body: `#[name(...)]` or
/// `#![name(...)]` whose name is a literal `allow`, `expect`, `warn`, `deny`,
/// or `forbid`.
#[derive(Debug)]
pub(super) struct BodyAttribute {
    pub(super) name: Symbol,
    /// From `#` to `]`, the span an expansion of the attribute also carries.
    pub(super) span: Span,
    /// Literal lint paths. A path built from a fragment is left out, since only
    /// expansion decides what it names.
    pub(super) paths: Vec<Vec<Symbol>>,
    pub(super) reason: Reason,
    /// A `$(...)` repetition follows this attribute in its run. It may expand
    /// into a later `deny` or `forbid` that raises what this attribute lowers,
    /// so only an expansion can tell, and the definition-site check defers.
    pub(super) is_raise_undecided: bool,
}

impl BodyAttribute {
    pub(super) fn lint_paths(&self) -> impl Iterator<Item = impl Iterator<Item = &str> + Clone> {
        self.paths.iter().map(|path| path.iter().map(Symbol::as_str))
    }
}

/// Runs of consecutive lint-level attributes in a macro's transcribers, in
/// source order. A `macro_rules!` matcher is skipped: it matches tokens rather
/// than producing them.
pub(super) fn attribute_runs(body: &TokenStream, is_macro_rules: bool) -> Vec<Vec<BodyAttribute>> {
    let mut runs = Vec::new();
    if !is_macro_rules {
        collect_runs(body, &mut runs);
        return runs;
    }
    let trees = body.iter().collect::<Vec<_>>();
    for (index, tree) in trees.iter().enumerate() {
        let TokenTree::Delimited(_, _, _, stream) = tree else {
            continue;
        };
        let is_matcher = trees.get(index + 1).is_some_and(|next| is_token(next, &TokenKind::FatArrow));
        if !is_matcher {
            collect_runs(stream, &mut runs);
        }
    }
    runs
}

fn is_token(tree: &TokenTree, kind: &TokenKind) -> bool {
    matches!(tree, TokenTree::Token(Token { kind: found, .. }, _) if found == kind)
}

fn collect_runs(stream: &TokenStream, runs: &mut Vec<Vec<BodyAttribute>>) {
    let trees = stream.iter().collect::<Vec<_>>();
    let mut run = Vec::new();
    let mut index = 0;
    while let Some(rest) = trees.get(index..).filter(|rest| !rest.is_empty()) {
        if let Some((attributes, consumed)) = attribute_at(rest) {
            // Any attribute, such as `#[inline]` or `#[$meta]`, keeps the run.
            run.extend(attributes);
            index += consumed;
            continue;
        }
        // A `///` doc comment is an attribute of the same item.
        if matches!(
            rest.first(),
            Some(TokenTree::Token(
                Token {
                    kind: TokenKind::DocComment(..),
                    ..
                },
                _
            ))
        ) {
            index += 1;
            continue;
        }
        if let Some((inner, consumed)) = repetition_at(rest) {
            for attribute in &mut run {
                attribute.is_raise_undecided = true;
            }
            collect_runs(inner, runs);
            index += consumed;
            continue;
        }
        if !run.is_empty() {
            runs.push(std::mem::take(&mut run));
        }
        if let Some(TokenTree::Delimited(_, _, _, inner)) = rest.first() {
            collect_runs(inner, runs);
        }
        index += 1;
    }
    if !run.is_empty() {
        runs.push(run);
    }
}

/// A `$( ... ) separator? operator` repetition starting at `trees`: its
/// contents and the number of trees it spans.
fn repetition_at<'a>(trees: &[&'a TokenTree]) -> Option<(&'a TokenStream, usize)> {
    let [
        dollar,
        TokenTree::Delimited(_, _, Delimiter::Parenthesis, inner),
        after @ ..,
    ] = trees
    else {
        return None;
    };
    if !is_token(dollar, &TokenKind::Dollar) {
        return None;
    }
    let is_operator = |tree: &&TokenTree| {
        [TokenKind::Star, TokenKind::Plus, TokenKind::Question].iter().any(|kind| is_token(tree, kind))
    };
    let operator_offset = after.iter().take(2).position(is_operator).map_or(0, |offset| offset + 1);
    Some((inner, 2 + operator_offset))
}

/// The lint-level attributes of an attribute starting at `trees`, with the
/// number of trees it spans, or `None` when `trees` does not start with one.
/// The list is empty unless the attribute is a literal lint-level attribute or
/// a `cfg_attr` that expands into some.
fn attribute_at(trees: &[&TokenTree]) -> Option<(Vec<BodyAttribute>, usize)> {
    let (TokenTree::Token(pound, _), rest) = trees.split_first()? else {
        return None;
    };
    if pound.kind != TokenKind::Pound {
        return None;
    }
    let (rest, bang) = match rest.split_first() {
        Some((bang, rest)) if is_token(bang, &TokenKind::Bang) => (rest, 1),
        _ => (rest, 0),
    };
    let TokenTree::Delimited(dspan, _, Delimiter::Bracket, inner) = rest.first()? else {
        return None;
    };
    let mut attributes = Vec::new();
    level_attributes(&inner.iter().collect::<Vec<_>>(), Some(pound.span.to(dspan.close)), &mut attributes);
    Some((attributes, 2 + bang))
}

fn is_level_name(name: Symbol) -> bool {
    [sym::allow, sym::expect, sym::warn, sym::deny, sym::forbid].contains(&name)
}

/// Collects the lint-level attributes that the attribute content `trees`
/// yields: itself when it is `name(...)` with a level name, or, for
/// `cfg_attr(predicate, a(...), b(...))`, each attribute after the predicate.
/// `written` is the span of the whole written attribute; an attribute that a
/// `cfg_attr` expands into is spanned from its name to its closing
/// parenthesis instead, as rustc spans the attribute it expands into.
fn level_attributes(trees: &[&TokenTree], written: Option<Span>, attributes: &mut Vec<BodyAttribute>) {
    let [
        TokenTree::Token(
            Token {
                kind: TokenKind::Ident(name, _),
                span: name_span,
            },
            _,
        ),
        TokenTree::Delimited(dspan, _, Delimiter::Parenthesis, arguments),
    ] = trees
    else {
        return;
    };
    let arguments = arguments.iter().collect::<Vec<_>>();
    if *name == sym::cfg_attr {
        for expanded in arguments.split(|tree| is_token(tree, &TokenKind::Comma)).skip(1) {
            level_attributes(expanded, None, attributes);
        }
        return;
    }
    if !is_level_name(*name) {
        return;
    }
    let mut attribute = BodyAttribute {
        name: *name,
        span: written.unwrap_or_else(|| name_span.to(dspan.close)),
        paths: Vec::new(),
        reason: Reason::Absent,
        is_raise_undecided: false,
    };
    for argument in arguments.split(|tree| is_token(tree, &TokenKind::Comma)) {
        if let Some(reason) = reason_argument(argument) {
            attribute.reason = reason;
        } else if let Some(path) = literal_path(argument) {
            attribute.paths.push(path);
        }
    }
    attributes.push(attribute);
}

fn reason_argument(argument: &[&TokenTree]) -> Option<Reason> {
    let [
        TokenTree::Token(
            Token {
                kind: TokenKind::Ident(name, _),
                ..
            },
            _,
        ),
        eq,
        value @ ..,
    ] = argument
    else {
        return None;
    };
    if *name != sym::reason || !is_token(eq, &TokenKind::Eq) {
        return None;
    }
    Some(match value {
        [
            TokenTree::Token(
                Token {
                    kind: TokenKind::Literal(lit),
                    ..
                },
                _,
            ),
        ] if matches!(lit.kind, LitKind::Str | LitKind::StrRaw(_)) => Reason::Literal(lit.symbol),
        _ => Reason::Fragment,
    })
}

/// Segments of a literal `a::b` lint path, or `None` for anything else, such
/// as a path containing a `$fragment`.
fn literal_path(argument: &[&TokenTree]) -> Option<Vec<Symbol>> {
    let mut segments = Vec::new();
    for (index, tree) in argument.iter().enumerate() {
        match tree {
            TokenTree::Token(
                Token {
                    kind: TokenKind::Ident(segment, _),
                    ..
                },
                _,
            ) if index % 2 == 0 => {
                segments.push(*segment);
            }
            tree if index % 2 == 1 && is_token(tree, &TokenKind::PathSep) => {}
            _ => return None,
        }
    }
    (!segments.is_empty() && argument.len() % 2 == 1).then_some(segments)
}

#[cfg(test)]
mod tests {
    use rustc_ast::AttrStyle;
    use rustc_ast::token::CommentKind;
    use rustc_ast::token::Delimiter;
    use rustc_ast::token::IdentIsRaw;
    use rustc_ast::token::Lit;
    use rustc_ast::token::LitKind;
    use rustc_ast::token::TokenKind;
    use rustc_ast::tokenstream::DelimSpacing;
    use rustc_ast::tokenstream::DelimSpan;
    use rustc_ast::tokenstream::Spacing;
    use rustc_ast::tokenstream::TokenStream;
    use rustc_ast::tokenstream::TokenTree;
    use rustc_span::DUMMY_SP;
    use rustc_span::Symbol;

    use super::Reason;
    use super::attribute_runs;

    fn token(kind: TokenKind) -> TokenTree {
        TokenTree::token_alone(kind, DUMMY_SP)
    }

    fn ident(name: &str) -> TokenTree {
        token(TokenKind::Ident(Symbol::intern(name), IdentIsRaw::No))
    }

    fn group(delimiter: Delimiter, trees: Vec<TokenTree>) -> TokenTree {
        TokenTree::Delimited(
            DelimSpan::dummy(),
            DelimSpacing::new(Spacing::Alone, Spacing::Alone),
            delimiter,
            TokenStream::new(trees),
        )
    }

    /// `#[name(arguments)]`.
    fn attribute(name: &str, arguments: Vec<TokenTree>) -> Vec<TokenTree> {
        vec![
            token(TokenKind::Pound),
            group(Delimiter::Bracket, vec![ident(name), group(Delimiter::Parenthesis, arguments)]),
        ]
    }

    fn path(segments: &[&str]) -> Vec<TokenTree> {
        let mut trees = Vec::new();
        for (index, segment) in segments.iter().enumerate() {
            if index > 0 {
                trees.push(token(TokenKind::PathSep));
            }
            trees.push(ident(segment));
        }
        trees
    }

    /// `(matcher) => { transcriber };`
    fn macro_rules_arm(matcher: Vec<TokenTree>, transcriber: Vec<TokenTree>) -> Vec<TokenTree> {
        vec![
            group(Delimiter::Parenthesis, matcher),
            token(TokenKind::FatArrow),
            group(Delimiter::Brace, transcriber),
            token(TokenKind::Semi),
        ]
    }

    fn names(runs: &[Vec<super::BodyAttribute>]) -> Vec<Vec<String>> {
        runs.iter().map(|run| run.iter().map(|attribute| attribute.name.to_string()).collect()).collect()
    }

    #[test]
    fn scans_transcribers_into_runs_of_literal_level_attributes() {
        rustc_span::create_default_session_globals_then(|| {
            let mut arguments = path(&["tigerstyle", "safety"]);
            arguments.push(token(TokenKind::Comma));
            arguments.extend(path(&["no_unwrap"]));
            arguments.extend([
                token(TokenKind::Comma),
                ident("reason"),
                token(TokenKind::Eq),
                token(TokenKind::Literal(Lit::new(LitKind::Str, Symbol::intern("audited"), None))),
            ]);
            let mut transcriber = attribute("warn", arguments);
            // A non-level attribute keeps the run; a fragment name is no level.
            transcriber.extend([
                token(TokenKind::Pound),
                group(Delimiter::Bracket, vec![token(TokenKind::Dollar), ident("meta")]),
            ]);
            transcriber.extend(attribute("deny", path(&["tigerstyle", "safety"])));
            transcriber.extend([ident("mod"), ident("generated"), group(Delimiter::Brace, Vec::new())]);
            transcriber.extend(attribute("allow", path(&["clippy", "all"])));
            // The matcher's literal attribute is not produced by the macro.
            let body = TokenStream::new(macro_rules_arm(attribute("allow", path(&["no_panic"])), transcriber));

            let runs = attribute_runs(&body, true);

            assert_eq!(names(&runs), [vec!["warn", "deny"], vec!["allow"]]);
            let warn = &runs[0][0];
            let paths = warn.lint_paths().map(Iterator::collect::<Vec<_>>).collect::<Vec<_>>();
            assert_eq!(paths, [vec!["tigerstyle", "safety"], vec!["no_unwrap"]]);
            assert_eq!(warn.reason, Reason::Literal(Symbol::intern("audited")));
        });
    }

    #[test]
    fn expands_cfg_attr_into_each_level_attribute_after_the_predicate() {
        rustc_span::create_default_session_globals_then(|| {
            let mut arguments = vec![ident("all"), group(Delimiter::Parenthesis, Vec::new())];
            arguments.push(token(TokenKind::Comma));
            arguments.extend([
                ident("allow"),
                group(Delimiter::Parenthesis, path(&["tigerstyle", "structure"])),
            ]);
            arguments.push(token(TokenKind::Comma));
            arguments.extend([ident("inline")]);
            arguments.push(token(TokenKind::Comma));
            arguments.extend([ident("warn"), group(Delimiter::Parenthesis, path(&["no_unwrap"]))]);
            let body = TokenStream::new(attribute("cfg_attr", arguments));

            let runs = attribute_runs(&body, false);

            assert_eq!(names(&runs), [vec!["allow", "warn"]]);
        });
    }

    #[test]
    fn doc_comments_keep_runs_and_repetitions_leave_raises_undecided() {
        rustc_span::create_default_session_globals_then(|| {
            let doc = token(TokenKind::DocComment(CommentKind::Line, AttrStyle::Outer, Symbol::intern(" docs")));
            let mut transcriber = attribute("allow", path(&["tigerstyle", "structure"]));
            transcriber.push(doc);
            transcriber.extend(attribute("deny", path(&["unjustified_allow"])));
            transcriber.extend([ident("fn"), ident("documented")]);
            transcriber.extend(attribute("warn", path(&["tigerstyle", "safety"])));
            transcriber.extend([
                token(TokenKind::Dollar),
                group(Delimiter::Parenthesis, attribute("allow", path(&["clippy", "all"]))),
                token(TokenKind::Star),
            ]);
            transcriber.extend(attribute("deny", path(&["tigerstyle", "no_unwrap"])));
            transcriber.extend([ident("mod"), ident("repeated")]);
            let body = TokenStream::new(transcriber);

            let runs = attribute_runs(&body, false);

            // The repetition's own attributes form a run of their own.
            assert_eq!(names(&runs), [vec!["allow", "deny"], vec!["allow"], vec!["warn", "deny"]]);
            let undecided = runs
                .iter()
                .map(|run| run.iter().map(|attribute| attribute.is_raise_undecided).collect::<Vec<_>>())
                .collect::<Vec<_>>();
            assert_eq!(undecided, [vec![false, false], vec![false], vec![true, false]]);
        });
    }

    #[test]
    fn leaves_fragment_paths_and_reasons_to_expansion() {
        rustc_span::create_default_session_globals_then(|| {
            let mut arguments = vec![token(TokenKind::Dollar), ident("lint")];
            arguments.extend([token(TokenKind::Comma), ident("reason"), token(TokenKind::Eq)]);
            arguments.extend([token(TokenKind::Dollar), ident("why")]);
            let body = TokenStream::new(attribute("allow", arguments));

            let runs = attribute_runs(&body, false);

            assert_eq!(names(&runs), [vec!["allow"]]);
            assert!(runs[0][0].paths.is_empty());
            assert_eq!(runs[0][0].reason, Reason::Fragment);
        });
    }
}

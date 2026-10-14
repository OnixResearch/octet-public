pub(crate) use octet_lint_metadata::ExampleMode;
pub(crate) use octet_lint_metadata::GROUP_NAMES;
#[cfg(test)]
pub(crate) use octet_lint_metadata::GROUP_SAFETY;
use rustc_errors::Applicability;
use rustc_errors::DiagDecorator;
use rustc_lint::Lint;
use rustc_lint::LintContext;
use rustc_span::Span;
use serde::Serialize;
pub(crate) type LintDiagnosticMeta = octet_lint_metadata::LintMetadata;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct LintGroupSnapshot {
    pub group_name: &'static str,
    pub lint_names: Vec<&'static str>,
}

#[cfg_attr(not(test), allow(dead_code))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub(crate) struct CompatibilityAliasSnapshot {
    pub alias_name: &'static str,
    pub canonical_lint_name: String,
}

fn help_text(meta: &LintDiagnosticMeta) -> String {
    match meta.example_mode {
        ExampleMode::Required => format!(
            "Tiger Style ({}): {} Example: {}",
            meta.principle,
            meta.guidance,
            meta.example_text.expect("missing required example text"),
        ),
        ExampleMode::None => format!("Tiger Style ({}): {}", meta.principle, meta.guidance),
    }
}

pub(crate) const DIAGNOSTIC_METADATA: &[LintDiagnosticMeta] = octet_lint_metadata::LINT_METADATA;

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn all_metadata() -> &'static [LintDiagnosticMeta] {
    DIAGNOSTIC_METADATA
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn canonical_lint_name(lint_name: &str) -> String {
    octet_lint_metadata::canonical_lint_name(lint_name)
}

pub(crate) fn lint_names_for_group(group_name: &str) -> Vec<&'static str> {
    DIAGNOSTIC_METADATA
        .iter()
        .filter(|meta| meta.domain_group == group_name)
        .map(|meta| meta.lint_name)
        .collect()
}

pub(crate) fn group_snapshots() -> Vec<LintGroupSnapshot> {
    GROUP_NAMES
        .iter()
        .map(|group_name| LintGroupSnapshot {
            group_name,
            lint_names: lint_names_for_group(group_name),
        })
        .collect()
}

#[cfg_attr(not(test), allow(dead_code))]
pub(crate) fn compatibility_alias_snapshots() -> Vec<CompatibilityAliasSnapshot> {
    DIAGNOSTIC_METADATA
        .iter()
        .map(|meta| CompatibilityAliasSnapshot {
            alias_name: meta.lint_name,
            canonical_lint_name: canonical_lint_name(meta.lint_name),
        })
        .collect()
}

pub(crate) fn diagnostic_meta(lint_name: &str) -> &'static LintDiagnosticMeta {
    DIAGNOSTIC_METADATA
        .iter()
        .find(|meta| meta.lint_name == lint_name)
        .unwrap_or_else(|| panic!("missing diagnostic metadata for lint `{lint_name}`"))
}

pub(crate) enum LintSuggestion {
    Single {
        span: Span,
        message: String,
        replacement: String,
        applicability: Applicability,
    },
    Multipart {
        message: String,
        parts: Vec<(Span, String)>,
        applicability: Applicability,
    },
}

fn applicability_label(applicability: Applicability) -> &'static str {
    match applicability {
        Applicability::MachineApplicable => "MachineApplicable",
        Applicability::MaybeIncorrect => "MaybeIncorrect",
        Applicability::HasPlaceholders => "HasPlaceholders",
        Applicability::Unspecified => "Unspecified",
    }
}

pub(crate) fn emit_tiger_style_lint(
    cx: &impl LintContext,
    lint: &'static Lint,
    span: Span,
    message: impl Into<String>,
    lint_name: &'static str,
) {
    let meta = diagnostic_meta(lint_name);
    let message = message.into();

    cx.opt_span_lint(
        lint,
        Some(span),
        DiagDecorator(|diag| {
            diag.primary_message(message.clone());
            diag.help(help_text(meta));
            diag.note(format!("see Tiger Style guide: {}", meta.guide_section));
        }),
    );
}

pub(crate) fn emit_tiger_style_lint_with_suggestion(
    cx: &impl LintContext,
    lint: &'static Lint,
    span: Span,
    message: impl Into<String>,
    lint_name: &'static str,
    suggestion: LintSuggestion,
) {
    let meta = diagnostic_meta(lint_name);
    let message = message.into();
    let applicability = match &suggestion {
        LintSuggestion::Single { applicability, .. } | LintSuggestion::Multipart { applicability, .. } => {
            *applicability
        }
    };

    cx.opt_span_lint(
        lint,
        Some(span),
        DiagDecorator(|diag| {
            diag.primary_message(message.clone());
            match suggestion {
                LintSuggestion::Single {
                    span,
                    message,
                    replacement,
                    applicability,
                } => {
                    diag.span_suggestion_verbose(span, message, replacement, applicability);
                }
                LintSuggestion::Multipart {
                    message,
                    parts,
                    applicability,
                } => {
                    diag.multipart_suggestion(message, parts, applicability);
                }
            }
            diag.help(help_text(meta));
            diag.note(format!("structured suggestion applicability: {}", applicability_label(applicability)));
            diag.note(format!("see Tiger Style guide: {}", meta.guide_section));
        }),
    );
}

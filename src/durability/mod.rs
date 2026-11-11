//! Bounded durability-claim source checks.
//!
//! The checks analyze consumer-declared boundary functions only. They report
//! selected source shapes, never filesystem, device, or hardware durability.

pub(crate) mod discard;
pub(crate) mod flush;
pub(crate) mod retry;
mod scan;

use octet_lint_metadata::durability::DurabilityBoundary;
use octet_lint_metadata::durability::DurabilityCatalog;
use octet_lint_metadata::durability::DurabilityProfile;
use rustc_lint::Lint;
use rustc_lint::LintContext;
use rustc_session::Session;
use rustc_span::Span;

use crate::config::Config;
use crate::diagnostics::emit_tiger_style_lint_with_notes;
use crate::source::normalize_path;
use crate::source::source_file_path;

/// Vocabulary plus the validated consumer profile shared by the durability
/// lints. Absent when no `durability_profile_path` is configured.
#[derive(Clone)]
pub(crate) struct Context {
    catalog: DurabilityCatalog,
    profile: DurabilityProfile,
}

impl Context {
    pub(crate) fn from_config(cfg: &Config) -> Option<Self> {
        let profile = cfg.durability_profile()?.clone();
        Some(Self {
            catalog: cfg.durability_catalog().clone(),
            profile,
        })
    }

    pub(crate) fn catalog(&self) -> &DurabilityCatalog {
        &self.catalog
    }

    /// Resolves the declared boundary that owns `def_path` at `span`.
    pub(crate) fn boundary_at(&self, sess: &Session, span: Span, def_path: &str) -> Option<&DurabilityBoundary> {
        let relative = relative_source_path(sess, span)?;
        self.profile.resolve(&relative, def_path)
    }

    /// Emits one durability finding with bounded repair guidance and the
    /// profile-independent claim non-claims.
    pub(crate) fn emit(
        &self,
        cx: &impl LintContext,
        lint: &'static Lint,
        span: Span,
        message: String,
        lint_name: &'static str,
    ) {
        let mut notes = Vec::new();
        if let Some(repair) = self.catalog.repair_for(lint_name) {
            notes.push(format!("bounded repair guidance: {repair}"));
        }
        notes.push(format!(
            "bounded repair options: {}; no single repair proves durability",
            self.catalog.repair_options_text()
        ));
        notes.push(self.catalog.non_claims.join(" "));
        emit_tiger_style_lint_with_notes(cx, lint, span, message, lint_name, &notes);
    }
}

/// Workspace-relative source path for `span`, using `/` separators.
pub(crate) fn relative_source_path(sess: &Session, span: Span) -> Option<String> {
    let file = sess.source_map().lookup_source_file(span.lo());
    let path = normalize_path(&source_file_path(&file));
    let root = normalize_path(sess.opts.working_dir.local_path()?);
    path.strip_prefix(&root).ok().map(|relative| relative.to_string_lossy().replace('\\', "/"))
}

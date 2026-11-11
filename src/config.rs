//! Per-project configuration loaded from `dylint.toml` in the workspace root.
//!
//! Projects place a `dylint.toml` next to their `Cargo.toml`:
//!
//! ```toml
//! [octet]
//! excessive_file_length_max = 300
//! function_length_max = 100
//! module_file_count_max = 7
//! nesting_max_depth = 4
//! assertion_density_min_lines = 30
//! assertion_density_min_assertions = 3
//! max_parameters = 7
//! trait_method_max = 7
//! impl_blocks_max = 4
//! module_pub_items_max = 12
//! core_source_scopes = ["crates/domain-core/src/"]
//! core_value_provider_types = ["reviewed_values::Value"]
//! sealed_enum_marker = "octet::sealed_enum"
//! state_machine_core_marker = "octet::state_machine_core"
//! state_machine_enum_marker = "octet::state_machine_enum"
//! state_machine_core_scopes = ["crates/domain-core/src/fsm/"]
//! state_machine_generated_adapter_scopes = ["crates/domain-core/src/generated/"]
//! acronym_allowlist = ["IO", "ID"]
//! indexable_lint_exceptions = ["rustc_private_aliases", "lint_identity_names"]
//! disabled_lints = ["no_unwrap", "no_panic"]
//!
//! [octet.lint_levels]
//! no_unwrap = "deny"
//! ```

use std::collections::BTreeMap;
use std::collections::BTreeSet;
use std::collections::HashMap;

use serde::Deserialize;

pub(crate) const INDEXABLE_EXCEPTION_RUSTC_PRIVATE_ALIASES: &str = "rustc_private_aliases";
pub(crate) const INDEXABLE_EXCEPTION_LINT_IDENTITY_NAMES: &str = "lint_identity_names";
pub(crate) const INDEXABLE_EXCEPTION_GENERATED_SEARCHABLE_NAMES: &str = "generated_searchable_names";
const DEFAULT_INDEXABLE_EXCEPTIONS: &[&str] = &[
    INDEXABLE_EXCEPTION_RUSTC_PRIVATE_ALIASES,
    INDEXABLE_EXCEPTION_LINT_IDENTITY_NAMES,
    INDEXABLE_EXCEPTION_GENERATED_SEARCHABLE_NAMES,
];
const NOMINAL_DOMAIN_LINTS: &[&str] = &["primitive_domain_alias", "raw_domain_value", "newtype_invariant_bypass"];

/// Boolean API lints whose workspace-wide suppression is forbidden.
const BOOLEAN_API_LINTS: &[&str] = &["raw_bool_parameter", "raw_bool_parameter_suppression"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct IndexableLintExceptions {
    pub rustc_private_aliases: bool,
    pub lint_identity_names: bool,
    pub generated_searchable_names: bool,
}

/// All supported per-lint severity overrides.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LintLevel {
    Allow,
    Warn,
    Deny,
}

impl LintLevel {
    #[must_use]
    pub fn rustc_level(self) -> rustc_lint::Level {
        match self {
            Self::Allow => rustc_lint::Level::Allow,
            Self::Warn => rustc_lint::Level::Warn,
            Self::Deny => rustc_lint::Level::Deny,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ConfigDiagnosticKind {
    UnknownConfigKey,
    UnknownDisabledLint,
    UnknownIndexableExceptionClass,
    UnknownLintLevel,
    ConflictingDisabledLintLevel,
    InvalidLintLevel,
    NominalInventoryLevelCapped,
    EmptySourceScope,
    UnmatchedSourceScope,
}

impl ConfigDiagnosticKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownConfigKey => "unknown_config_key",
            Self::UnknownDisabledLint => "unknown_disabled_lint",
            Self::UnknownIndexableExceptionClass => "unknown_indexable_exception_class",
            Self::UnknownLintLevel => "unknown_lint_level",
            Self::ConflictingDisabledLintLevel => "conflicting_disabled_lint_level",
            Self::InvalidLintLevel => "invalid_lint_level",
            Self::NominalInventoryLevelCapped => "nominal_inventory_level_capped",
            Self::EmptySourceScope => "empty_source_scope",
            Self::UnmatchedSourceScope => "unmatched_source_scope",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ConfigDiagnostic {
    pub kind: ConfigDiagnosticKind,
    pub source: &'static str,
    pub field: &'static str,
    pub item: String,
    pub message: String,
}

impl ConfigDiagnostic {
    fn new(kind: ConfigDiagnosticKind, field: &'static str, item: impl Into<String>, message: String) -> Self {
        Self {
            kind,
            source: "dylint.toml",
            field,
            item: item.into(),
            message,
        }
    }
}

fn parse_lint_level(raw: &str) -> Option<LintLevel> {
    match raw {
        "allow" => Some(LintLevel::Allow),
        "warn" => Some(LintLevel::Warn),
        "deny" => Some(LintLevel::Deny),
        _ => None,
    }
}

fn parse_indexable_exception_class(class: &str) -> Option<&'static str> {
    DEFAULT_INDEXABLE_EXCEPTIONS.iter().copied().find(|known| *known == class)
}

/// All configurable thresholds and allowlists. Missing fields use defaults.
#[derive(Debug, Deserialize, PartialEq)]
#[serde(default)]
pub struct Config {
    /// Maximum non-blank, non-comment lines in a function body.
    pub function_length_max: usize,

    /// Maximum physical lines in a production source file.
    pub excessive_file_length_max: usize,

    /// Maximum Rust source files in one module directory.
    pub module_file_count_max: usize,

    /// Maximum if/match nesting depth before warning.
    pub nesting_max_depth: usize,

    /// Minimum code lines before assertion density is checked.
    pub assertion_density_min_lines: usize,

    /// Minimum assertion count required in functions exceeding the line threshold.
    pub assertion_density_min_assertions: usize,

    /// Maximum counted parameters before warning.
    pub max_parameters: usize,

    /// Maximum methods permitted in one trait before `fat_trait` warns.
    pub trait_method_max: usize,

    /// Maximum inherent impl blocks permitted per type before `type_accumulation` warns.
    pub impl_blocks_max: usize,

    /// Maximum public items permitted in one module before `type_accumulation` warns.
    pub module_pub_items_max: usize,

    /// Source path scopes where concrete construction in core is checked.
    pub core_source_scopes: Vec<String>,

    /// Exact compiler definition paths of project-reviewed pure value ADTs.
    pub core_value_provider_types: Vec<String>,

    /// Attribute marker that suppresses fragile same-crate exhaustive enum match diagnostics.
    pub sealed_enum_marker: String,

    /// Attribute marker that opts a transition function into state-machine exhaustiveness checks.
    pub state_machine_core_marker: String,

    /// Attribute marker that documents state/event enums as deliberately closed FSM domains.
    pub state_machine_enum_marker: String,

    /// Source path scopes where transition functions enter the FSM profile without an attribute
    /// marker.
    pub state_machine_core_scopes: Vec<String>,

    /// Source path scopes excluded as generated FSM adapters outside the pure transition core.
    pub state_machine_generated_adapter_scopes: Vec<String>,

    /// Workspace-relative path to a generated durability-claims profile.
    ///
    /// Consumers author the profile in Nickel, export deterministic JSON, and
    /// point this key at the generated file. Source checks analyze declared
    /// boundary functions only.
    pub durability_profile_path: Option<String>,

    /// Acronyms that should not trigger the `acronym_style` lint.
    /// Entries are case-sensitive and matched against the uppercase run
    /// (e.g., `"IO"` allows `IOError`).
    pub acronym_allowlist: Vec<String>,

    /// Lint names that should not register for this workspace.
    pub disabled_lints: Vec<String>,

    /// Scoped exception classes for high-volume naming/indexable lints.
    pub indexable_lint_exceptions: Vec<String>,

    /// Optional staged nominal-domain mode. Omission leaves the lints dormant
    /// unless source attributes or per-lint levels enable them.
    pub nominal_domain_mode: Option<octet_lint_metadata::nominal_domain::EnforcementMode>,

    /// Raw per-lint severity overrides loaded from `dylint.toml`.
    pub lint_levels: HashMap<String, String>,

    /// Unrecognized keys in the config section, reported as warnings.
    #[serde(flatten)]
    // serde's flatten catch-all must be a map; `IgnoredAny` keeps only the key
    // names without buffering the unknown values.
    #[allow(clippy::zero_sized_map_values)]
    unknown_keys: BTreeMap<String, serde::de::IgnoredAny>,

    /// Validated per-lint severity overrides used during registration.
    #[serde(skip)]
    validated_lint_levels: HashMap<String, LintLevel>,

    /// Lints whose validated level is a selected strict-profile floor.
    #[serde(skip)]
    profile_floor_lints: BTreeSet<String>,

    /// Identifier of the selected strict profile, if any.
    #[serde(skip)]
    strict_profile_id: Option<String>,

    /// Validated durability profile loaded from `durability_profile_path`.
    #[serde(skip)]
    durability_profile: Option<octet_lint_metadata::durability::DurabilityProfile>,

    /// Embedded durability vocabulary loaded once during registration.
    #[serde(skip)]
    durability_catalog: octet_lint_metadata::durability::DurabilityCatalog,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            function_length_max: 70,
            excessive_file_length_max: 300,
            module_file_count_max: 7,
            nesting_max_depth: 3,
            assertion_density_min_lines: 20,
            assertion_density_min_assertions: 2,
            max_parameters: 5,
            trait_method_max: 7,
            impl_blocks_max: 4,
            module_pub_items_max: 12,
            core_source_scopes: Vec::new(),
            core_value_provider_types: Vec::new(),
            sealed_enum_marker: "octet::sealed_enum".to_owned(),
            state_machine_core_marker: "octet::state_machine_core".to_owned(),
            state_machine_enum_marker: "octet::state_machine_enum".to_owned(),
            state_machine_core_scopes: Vec::new(),
            state_machine_generated_adapter_scopes: Vec::new(),
            durability_profile_path: None,
            acronym_allowlist: Vec::new(),
            disabled_lints: Vec::new(),
            indexable_lint_exceptions: DEFAULT_INDEXABLE_EXCEPTIONS.iter().map(|class| (*class).to_owned()).collect(),
            nominal_domain_mode: None,
            lint_levels: HashMap::new(),
            unknown_keys: BTreeMap::new(),
            validated_lint_levels: HashMap::new(),
            profile_floor_lints: BTreeSet::new(),
            strict_profile_id: None,
            durability_profile: None,
            durability_catalog: octet_lint_metadata::durability::DurabilityCatalog::default(),
        }
    }
}

pub(crate) fn canonical_lint_names() -> Vec<&'static str> {
    crate::LINT_INVENTORY.iter().map(|lint| lint.name).collect()
}

impl Config {
    #[must_use]
    pub fn lint_is_disabled(&self, lint_name: &str) -> bool {
        self.disabled_lints.iter().any(|disabled| disabled == lint_name)
    }

    #[must_use]
    pub fn lint_level(&self, lint_name: &str) -> Option<LintLevel> {
        self.validated_lint_levels.get(lint_name).copied()
    }

    /// Merges config-owned lint levels into rustc's command-line lint options.
    ///
    /// rustc applies `lint_opts` in order and the last entry for a lint wins.
    /// Consumer `lint_levels` and the nominal-domain mode go before the
    /// existing options, so explicit command-line `-A`/`-W`/`-D` flags win over
    /// `dylint.toml`. Selected strict-profile denies go after them, so the
    /// profile floor stays authoritative over `-A`/`-W`. A `--force-warn` or
    /// `--cap-lints` below `deny` would still beat a later deny in rustc, so
    /// registration rejects those for floor lints instead.
    pub(crate) fn merge_into_lint_opts(&self, lint_opts: &mut Vec<(String, rustc_lint::Level)>) {
        let mut consumer_levels = Vec::new();
        for lint in crate::LINT_INVENTORY {
            let Some(level) = self.lint_level(lint.name) else {
                continue;
            };
            let option = (lint.name.to_owned(), level.rustc_level());
            if self.profile_floor_lints.contains(lint.name) {
                lint_opts.push(option);
            } else {
                consumer_levels.push(option);
            }
        }
        lint_opts.splice(0..0, consumer_levels);
    }

    /// Validated durability profile, when `durability_profile_path` is set.
    #[must_use]
    pub fn durability_profile(&self) -> Option<&octet_lint_metadata::durability::DurabilityProfile> {
        self.durability_profile.as_ref()
    }

    /// Embedded durability vocabulary used by the durability lints.
    #[must_use]
    pub fn durability_catalog(&self) -> &octet_lint_metadata::durability::DurabilityCatalog {
        &self.durability_catalog
    }

    /// Identifier of the strict profile applied to this config, if any.
    pub(crate) fn strict_profile_id(&self) -> Option<&str> {
        self.strict_profile_id.as_deref()
    }

    /// Lints the selected strict profile requires at `deny`.
    pub(crate) fn profile_floor_lints(&self) -> impl Iterator<Item = &str> {
        self.profile_floor_lints.iter().map(String::as_str)
    }

    #[must_use]
    pub fn nominal_domain_mode(&self) -> octet_lint_metadata::nominal_domain::EnforcementMode {
        self.nominal_domain_mode.unwrap_or(octet_lint_metadata::nominal_domain::EnforcementMode::Warn)
    }

    fn apply_nominal_domain_mode(&mut self, diagnostics: &mut Vec<ConfigDiagnostic>) {
        let Some(mode) = self.nominal_domain_mode else {
            return;
        };
        for lint_name in NOMINAL_DOMAIN_LINTS {
            if self.lint_is_disabled(lint_name) {
                continue;
            }
            let selected = match mode {
                octet_lint_metadata::nominal_domain::EnforcementMode::Inventory
                | octet_lint_metadata::nominal_domain::EnforcementMode::Warn => LintLevel::Warn,
                octet_lint_metadata::nominal_domain::EnforcementMode::Deny => LintLevel::Deny,
            };
            let level = self.validated_lint_levels.entry((*lint_name).to_owned()).or_insert(selected);
            // Inventory findings include suffix-only candidates, so inventory
            // mode never lets a consumer level turn them into hard errors.
            let capped =
                mode == octet_lint_metadata::nominal_domain::EnforcementMode::Inventory && *level == LintLevel::Deny;
            if capped {
                diagnostics.push(ConfigDiagnostic::new(
                    ConfigDiagnosticKind::NominalInventoryLevelCapped,
                    "octet.lint_levels",
                    *lint_name,
                    format!(
                        "octet `nominal_domain_mode = \"inventory\"` caps `{lint_name}` at `warn`; ignoring `deny` from `dylint.toml` lint_levels"
                    ),
                ));
            }
            if capped || selected == LintLevel::Deny || *level == LintLevel::Allow {
                *level = selected;
            }
        }
    }

    pub(crate) fn apply_strict_profile(
        &mut self,
        profile: &octet_lint_metadata::strict_profile::StrictProfile,
    ) -> Result<(), String> {
        if profile.requires_core_scopes && self.core_source_scopes.is_empty() {
            return Err(format!(
                "strict profile `{}` requires non-empty `core_source_scopes` in `dylint.toml`",
                profile.profile_id
            ));
        }

        let required_lints = profile.required_lints.iter().map(String::as_str).collect::<BTreeSet<_>>();
        for lint_name in BOOLEAN_API_LINTS {
            if required_lints.contains(lint_name)
                && (self.disabled_lints.iter().any(|disabled| disabled == lint_name)
                    || self.lint_levels.get(*lint_name).is_some_and(|level| level != "deny"))
            {
                return Err(format!(
                    "strict profile `{}` rejects workspace-wide suppression of `{lint_name}`; use a local owned compatibility expectation",
                    profile.profile_id
                ));
            }
        }
        self.disabled_lints.retain(|lint_name| !required_lints.contains(lint_name.as_str()));
        if profile_denies_nominal_domain(profile) {
            self.nominal_domain_mode = Some(octet_lint_metadata::nominal_domain::EnforcementMode::Deny);
        }
        for lint_name in &profile.required_lints {
            self.validated_lint_levels.insert(lint_name.clone(), LintLevel::Deny);
            self.profile_floor_lints.insert(lint_name.clone());
        }
        self.strict_profile_id = Some(profile.profile_id.clone());
        Ok(())
    }

    /// Rejects workspace-wide suppression of the Boolean API lints.
    ///
    /// A consumer may scope one compatibility expectation to one callable, but
    /// a crate-wide `disabled_lints` entry or an `allow` level would hide the
    /// strict migration.
    fn reject_broad_boolean_suppression(&self) -> Result<(), String> {
        for lint_name in BOOLEAN_API_LINTS {
            if self.lint_is_disabled(lint_name)
                || self.lint_levels.get(*lint_name).is_some_and(|level| level == "allow")
            {
                return Err(format!(
                    "workspace-wide suppression of `{lint_name}` is forbidden; use a local owned compatibility expectation"
                ));
            }
        }
        Ok(())
    }

    /// Promotes the Boolean API suppression guard when the compiler command
    /// line denies it, including the full deny-all catalog.
    fn enforce_compiler_boolean_guard(&mut self, lint_opts: &[(String, rustc_lint::Level)]) {
        let guard_denied = lint_opts.iter().any(|(name, level)| {
            let name = name.strip_prefix("tigerstyle::").unwrap_or(name);
            matches!(name, "raw_bool_parameter_suppression" | "raw-bool-parameter-suppression")
                && matches!(level, rustc_lint::Level::Deny | rustc_lint::Level::Forbid)
        });
        if guard_denied {
            self.validated_lint_levels.insert("raw_bool_parameter_suppression".to_owned(), LintLevel::Deny);
        }
    }

    /// Validates the parsed config and applies the selected strict profile.
    ///
    /// A profile that denies the nominal-domain lints raises their mode before
    /// validation, so the inventory-cap diagnostic never contradicts the
    /// profile. Diagnostics are returned even when the profile cannot apply.
    fn resolve(
        &mut self,
        profile: Option<&octet_lint_metadata::strict_profile::StrictProfile>,
    ) -> (Vec<ConfigDiagnostic>, Result<(), String>) {
        if profile.is_some_and(profile_denies_nominal_domain) {
            self.nominal_domain_mode = Some(octet_lint_metadata::nominal_domain::EnforcementMode::Deny);
        }
        let diagnostics = self.validate_diagnostics();
        let applied = profile.map_or(Ok(()), |profile| self.apply_strict_profile(profile));
        (diagnostics, applied)
    }

    /// Resolves every source scope on disk and keeps what rustc's file names
    /// can match: the normalized scope text, which rustc uses for files it
    /// reaches through that path, and, when it differs, the canonical
    /// workspace-relative path an in-root scope resolves to, so a symlinked
    /// or `..` scope covers its target. Scopes that resolve to nothing, to the
    /// workspace root itself, or outside it are reported.
    fn resolve_source_scopes(
        &mut self,
        resolve: impl Fn(&str) -> crate::source::ScopeResolution,
    ) -> Vec<ConfigDiagnostic> {
        [
            ("octet.core_source_scopes", &mut self.core_source_scopes),
            ("octet.state_machine_core_scopes", &mut self.state_machine_core_scopes),
            ("octet.state_machine_generated_adapter_scopes", &mut self.state_machine_generated_adapter_scopes),
        ]
        .into_iter()
        .flat_map(|(field, scopes)| resolve_scope_list(field, scopes, &resolve))
        .collect()
    }

    pub(crate) fn indexable_lint_exceptions(&self) -> IndexableLintExceptions {
        let enabled = |class: &str| self.indexable_lint_exceptions.iter().any(|configured| configured == class);
        IndexableLintExceptions {
            rustc_private_aliases: enabled(INDEXABLE_EXCEPTION_RUSTC_PRIVATE_ALIASES),
            lint_identity_names: enabled(INDEXABLE_EXCEPTION_LINT_IDENTITY_NAMES),
            generated_searchable_names: enabled(INDEXABLE_EXCEPTION_GENERATED_SEARCHABLE_NAMES),
        }
    }

    fn validate_diagnostics(&mut self) -> Vec<ConfigDiagnostic> {
        let known_lints = canonical_lint_names();
        let disabled_lints = self.disabled_lints.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let mut diagnostics = self
            .unknown_keys
            .keys()
            .map(|key| {
                ConfigDiagnostic::new(
                    ConfigDiagnosticKind::UnknownConfigKey,
                    "octet",
                    key,
                    format!("unknown octet config key in `dylint.toml`: `{key}`"),
                )
            })
            .collect::<Vec<_>>();
        for (field, scopes) in [
            ("octet.core_source_scopes", &mut self.core_source_scopes),
            ("octet.state_machine_core_scopes", &mut self.state_machine_core_scopes),
            ("octet.state_machine_generated_adapter_scopes", &mut self.state_machine_generated_adapter_scopes),
        ] {
            normalize_source_scopes(field, scopes, &mut diagnostics);
        }
        let mut unknown_disabled_lints = BTreeSet::new();

        for lint_name in &self.disabled_lints {
            if !known_lints.iter().any(|known| known == &lint_name.as_str())
                && unknown_disabled_lints.insert(lint_name.as_str())
            {
                diagnostics.push(ConfigDiagnostic::new(
                    ConfigDiagnosticKind::UnknownDisabledLint,
                    "octet.disabled_lints",
                    lint_name,
                    format!("unknown octet lint in `dylint.toml` disabled_lints: `{lint_name}`"),
                ));
            }
        }

        let mut unknown_exception_classes = BTreeSet::new();
        for class in &self.indexable_lint_exceptions {
            if parse_indexable_exception_class(class).is_none() && unknown_exception_classes.insert(class.as_str()) {
                diagnostics.push(ConfigDiagnostic::new(
                    ConfigDiagnosticKind::UnknownIndexableExceptionClass,
                    "octet.indexable_lint_exceptions",
                    class,
                    format!("unknown octet indexable_lint_exceptions class in `dylint.toml`: `{class}`"),
                ));
            }
        }

        self.validated_lint_levels.clear();
        let mut lint_levels = self.lint_levels.iter().collect::<Vec<_>>();
        lint_levels.sort_by_key(|(lint_name, _)| *lint_name);

        for (lint_name, raw_level) in lint_levels {
            if !known_lints.iter().any(|known| known == &lint_name.as_str()) {
                diagnostics.push(ConfigDiagnostic::new(
                    ConfigDiagnosticKind::UnknownLintLevel,
                    "octet.lint_levels",
                    lint_name,
                    format!("unknown octet lint in `dylint.toml` lint_levels: `{lint_name}`"),
                ));
                continue;
            }

            if disabled_lints.contains(lint_name.as_str()) {
                diagnostics.push(ConfigDiagnostic::new(
                    ConfigDiagnosticKind::ConflictingDisabledLintLevel,
                    "octet.lint_levels",
                    lint_name,
                    format!(
                        "conflicting octet config in `dylint.toml`: `{lint_name}` appears in both `disabled_lints` and `lint_levels`; ignoring level override"
                    ),
                ));
                continue;
            }

            let Some(level) = parse_lint_level(raw_level) else {
                diagnostics.push(ConfigDiagnostic::new(
                    ConfigDiagnosticKind::InvalidLintLevel,
                    "octet.lint_levels",
                    lint_name,
                    format!(
                        "invalid octet lint level in `dylint.toml` lint_levels for `{lint_name}`: `{raw_level}` (expected `allow`, `warn`, or `deny`)"
                    ),
                ));
                continue;
            };

            self.validated_lint_levels.insert(lint_name.clone(), level);
        }
        self.apply_nominal_domain_mode(&mut diagnostics);

        diagnostics
    }
}

/// `dylint.toml` config sections in precedence order: `[octet]`, then the
/// legacy `[slotcar]` and `[tigerstyle]` sections.
const DYLINT_TOML_SECTIONS: [&str; 3] = [env!("CARGO_PKG_NAME"), "slotcar", "tigerstyle"];

fn profile_denies_nominal_domain(profile: &octet_lint_metadata::strict_profile::StrictProfile) -> bool {
    profile.required_lints.iter().any(|lint_name| NOMINAL_DOMAIN_LINTS.contains(&lint_name.as_str()))
}

/// Normalizes configured source scopes the way `cargo octet` does and drops
/// entries that normalize to an empty path.
fn normalize_source_scopes(field: &'static str, scopes: &mut Vec<String>, diagnostics: &mut Vec<ConfigDiagnostic>) {
    let (kept, empty): (Vec<_>, Vec<_>) = scopes
        .drain(..)
        .map(|raw| (crate::source::normalize_scope(&raw), raw))
        .partition(|(scope, _raw)| !scope.is_empty());
    diagnostics.extend(empty.into_iter().map(|(_scope, raw)| {
        let message = format!("empty octet source scope in `dylint.toml` {field}: `{raw}`; ignoring it");
        ConfigDiagnostic::new(ConfigDiagnosticKind::EmptySourceScope, field, raw, message)
    }));
    *scopes = kept.into_iter().map(|(scope, _raw)| scope).collect();
}

/// Selects the highest-precedence config section that is present. Every
/// lower-precedence section is left unparsed and reported, because the
/// selected section replaces it as a whole.
fn prefer_octet_or_legacy_config<T>(
    sections: impl IntoIterator<Item = (&'static str, Option<T>)>,
) -> (Option<(&'static str, T)>, Vec<String>) {
    let mut present = sections.into_iter().filter_map(|(name, section)| section.map(|section| (name, section)));
    let selected = present.next();
    let notices = selected.as_ref().map_or_else(Vec::new, |(winner, _section)| {
        present
            .map(|(name, _section)| {
                format!(
                    "ignoring legacy `[{name}]` section in `dylint.toml` because `[{winner}]` takes precedence; move its settings into `[octet]`"
                )
            })
            .collect()
    });
    (selected, notices)
}

fn resolve_scope_list(
    field: &'static str,
    scopes: &mut Vec<String>,
    resolve: &impl Fn(&str) -> crate::source::ScopeResolution,
) -> Vec<ConfigDiagnostic> {
    let resolved = std::mem::take(scopes)
        .into_iter()
        .map(|scope| {
            let resolution = resolve(&scope);
            (scope, resolution)
        })
        .collect::<Vec<_>>();
    *scopes = resolved
        .iter()
        .flat_map(|(scope, resolution)| match resolution {
            crate::source::ScopeResolution::Inside(canonical) if canonical != scope => {
                vec![scope.clone(), canonical.clone()]
            }
            _ => vec![scope.clone()],
        })
        .collect();
    let list = field.trim_start_matches("octet.");
    resolved
        .into_iter()
        .filter_map(|(scope, resolution)| {
            let reason = match resolution {
                crate::source::ScopeResolution::Inside(_) => return None,
                crate::source::ScopeResolution::Missing => {
                    "names no file or directory under the workspace root; scopes are workspace-relative path prefixes, so a scope that used to match by substring must be rewritten as a path from the workspace root"
                }
                crate::source::ScopeResolution::Root => {
                    "resolves to the workspace root itself, so it matches nothing; name a source directory or file below the root"
                }
                crate::source::ScopeResolution::Outside => {
                    "resolves to a target outside the workspace root, so it matches only files rustc compiles through this in-root path, never the target's own location"
                }
            };
            let message = format!("octet `{list}` entry `{scope}` in `dylint.toml` {reason}");
            Some(ConfigDiagnostic::new(ConfigDiagnosticKind::UnmatchedSourceScope, field, scope, message))
        })
        .collect()
}

/// Load the project config from `dylint.toml`, falling back to defaults.
///
/// # Panics
///
/// Panics when Dylint cannot read `dylint.toml`, when the selected config
/// section is malformed TOML, when the embedded Boolean API policy is invalid,
/// when a workspace-wide Boolean API suppression is requested, or when the
/// selected strict profile cannot apply. A shadowed legacy section is reported
/// but never parsed; unknown config keys, invalid lint names, and invalid
/// levels are reported as warnings instead of panics after parsing succeeds.
pub fn load(
    sess: &rustc_session::Session,
    durability_catalog: &octet_lint_metadata::durability::DurabilityCatalog,
) -> Config {
    let sections = DYLINT_TOML_SECTIONS.map(|name| {
        let section = dylint_linting::config_toml(name)
            .unwrap_or_else(|error| panic!("Could not read config section `[{name}]`: {error}"));
        (name, section)
    });
    let (selected, notices) = prefer_octet_or_legacy_config(sections);
    for notice in notices {
        sess.dcx().warn(notice);
    }
    let mut cfg = selected.map_or_else(Config::default, |(name, section)| {
        section.try_into::<Config>().unwrap_or_else(|error| {
            panic!("Could not parse config section `[{name}]` as `{}`: {error}", std::any::type_name::<Config>())
        })
    });
    cfg.durability_catalog = durability_catalog.clone();
    let profile = selected_strict_profile().unwrap_or_else(|error| panic!("{error}"));
    octet_lint_metadata::strict_profile::boolean_api_policy().unwrap_or_else(|error| panic!("{error}"));
    cfg.reject_broad_boolean_suppression().unwrap_or_else(|error| panic!("{error}"));
    let (mut diagnostics, applied) = cfg.resolve(profile.as_ref());
    // Scopes resolve against the same root as `source::span_in_scopes`, with
    // symlinks and `..` followed as `cargo octet` does when it validates them.
    if let Some(root) = sess.opts.working_dir.local_path().and_then(|root| root.canonicalize().ok()) {
        diagnostics.extend(cfg.resolve_source_scopes(|scope| crate::source::resolve_scope(&root, scope)));
    }
    for diagnostic in diagnostics {
        sess.dcx().warn(diagnostic.message);
    }
    applied.unwrap_or_else(|error| panic!("{error}"));
    // An invalid profile fails closed: the error is reported and the durability
    // lints stay inert instead of silently accepting an unanalyzed boundary.
    if let Some(error) = cfg.resolve_durability_profile(sess, durability_catalog) {
        sess.dcx().err(error);
    }
    cfg.enforce_compiler_boolean_guard(&sess.opts.lint_opts);
    cfg
}

impl Config {
    /// Reads and validates the generated durability profile.
    ///
    /// Returns a deterministic error message when the configured path cannot be
    /// read or the profile violates the catalog contract. No profile configured
    /// means no durability boundary is declared.
    fn resolve_durability_profile(
        &mut self,
        sess: &rustc_session::Session,
        catalog: &octet_lint_metadata::durability::DurabilityCatalog,
    ) -> Option<String> {
        let configured = self.durability_profile_path.as_deref()?;
        let trimmed = configured.trim();
        if trimmed.is_empty() {
            return Some("octet `durability_profile_path` in `dylint.toml` must not be empty".to_owned());
        }
        let path = std::path::Path::new(trimmed);
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else {
            let root = sess
                .opts
                .working_dir
                .local_path()
                .map(std::path::Path::to_path_buf)
                .or_else(|| std::env::current_dir().ok())
                .unwrap_or_default();
            root.join(path)
        };
        let text = std::fs::read_to_string(&resolved).map_err(|error| {
            format!("octet `durability_profile_path` in `dylint.toml` cannot read `{}`: {error}", resolved.display())
        });
        let profile = match text.and_then(|text| {
            octet_lint_metadata::durability::parse_durability_profile(&text, catalog).map_err(|error| {
                format!("octet `durability_profile_path` in `dylint.toml` rejected `{}`: {error}", resolved.display())
            })
        }) {
            Ok(profile) => profile,
            Err(error) => return Some(error),
        };
        self.durability_profile = Some(profile);
        None
    }
}

fn selected_strict_profile() -> Result<Option<octet_lint_metadata::strict_profile::StrictProfile>, String> {
    let profile_id = match std::env::var(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV) {
        Ok(profile_id) => profile_id,
        Err(std::env::VarError::NotPresent) => return Ok(None),
        Err(error) => return Err(format!("read strict-profile selection: {error}")),
    };
    octet_lint_metadata::strict_profile::strict_profile(&profile_id).map(Some)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::Config;
    use super::ConfigDiagnostic;
    use super::ConfigDiagnosticKind;
    use super::DYLINT_TOML_SECTIONS;
    use super::LintLevel;
    use super::NOMINAL_DOMAIN_LINTS;
    use super::prefer_octet_or_legacy_config;

    const CONFIG_VALID_LEVEL_LAW: &str = "config valid lint-level overrides normalize deterministically";
    const CONFIG_DISABLED_PRECEDENCE_LAW: &str = "config disabled_lints wins over lint_levels";
    const CONFIG_NEGATIVE_LEVEL_LAW: &str = "config invalid generated lint levels fail closed";
    const CONFIG_LAW_REPLAY_COMMAND: &str = "nix develop -c bash -c 'cargo test --lib config_law -- --nocapture'";
    const GENERATED_LINT_NAMES: &[&str] = &["no_unwrap", "no_panic"];
    const GENERATED_LINT_LEVELS: &[(&str, LintLevel)] = &[
        ("allow", LintLevel::Allow),
        ("warn", LintLevel::Warn),
        ("deny", LintLevel::Deny),
    ];
    const GENERATED_INVALID_LINT_LEVELS: &[&str] = &["fatal", "Deny", " warn "];
    const CONFIG_VALID_LEVEL_CASE_BUDGET: usize = 6;
    const CONFIG_DISABLED_PRECEDENCE_CASE_BUDGET: usize = 6;
    const CONFIG_NEGATIVE_LEVEL_CASE_BUDGET: usize = 6;

    /// Selects and parses a config section the way `load` does, returning the
    /// shadowed-section notices alongside it.
    fn try_parse_config(doc: &str) -> Result<(Config, Vec<String>), toml::de::Error> {
        let mut document = toml::from_str::<toml::Table>(doc).expect("parse test dylint.toml");
        let (selected, notices) =
            prefer_octet_or_legacy_config(DYLINT_TOML_SECTIONS.map(|name| (name, document.remove(name))));
        let cfg = selected.map_or_else(|| Ok(Config::default()), |(_name, section)| section.try_into::<Config>())?;
        Ok((cfg, notices))
    }

    fn parse_config(doc: &str) -> Config {
        try_parse_config(doc).expect("parse selected config section").0
    }

    fn parse_and_validate(doc: &str) -> (Config, Vec<String>) {
        let mut cfg = parse_config(doc);
        let warnings = cfg.validate_diagnostics().into_iter().map(|diagnostic| diagnostic.message).collect();
        (cfg, warnings)
    }

    fn parse_and_validate_diagnostics(doc: &str) -> (Config, Vec<ConfigDiagnostic>) {
        let mut cfg = parse_config(doc);
        let diagnostics = cfg.validate_diagnostics();
        (cfg, diagnostics)
    }

    fn generated_lint_level_config(lint_name: &str, raw_level: &str) -> String {
        format!("[octet.lint_levels]\n{lint_name} = \"{raw_level}\"\n")
    }

    fn generated_disabled_and_lint_level_config(lint_name: &str, raw_level: &str) -> String {
        format!("[octet]\ndisabled_lints = [\"{lint_name}\"]\n\n[octet.lint_levels]\n{lint_name} = \"{raw_level}\"\n")
    }

    #[test]
    fn nominal_domain_modes_apply_a_deterministic_severity_floor() {
        assert_eq!(Config::default().nominal_domain_mode(), octet_lint_metadata::nominal_domain::EnforcementMode::Warn);
        for (raw_mode, mode, expected_level) in [
            ("inventory", octet_lint_metadata::nominal_domain::EnforcementMode::Inventory, LintLevel::Warn),
            ("warn", octet_lint_metadata::nominal_domain::EnforcementMode::Warn, LintLevel::Warn),
            ("deny", octet_lint_metadata::nominal_domain::EnforcementMode::Deny, LintLevel::Deny),
        ] {
            let (cfg, warnings) = parse_and_validate(&format!("[octet]\nnominal_domain_mode = \"{raw_mode}\"\n"));

            assert!(warnings.is_empty(), "unexpected warnings for {raw_mode}: {warnings:?}");
            assert_eq!(cfg.nominal_domain_mode, Some(mode));
            for lint_name in NOMINAL_DOMAIN_LINTS {
                assert_eq!(cfg.lint_level(lint_name), Some(expected_level));
            }
        }
    }

    #[test]
    fn nominal_domain_deny_cannot_be_relaxed_by_a_per_lint_allow() {
        let (cfg, warnings) = parse_and_validate(
            "[octet]\nnominal_domain_mode = \"deny\"\n\n[octet.lint_levels]\nprimitive_domain_alias = \"allow\"\n",
        );

        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(cfg.lint_level("primitive_domain_alias"), Some(LintLevel::Deny));
    }

    #[test]
    fn malformed_nominal_domain_mode_fails_toml_deserialization() {
        let parsed = try_parse_config("[octet]\nnominal_domain_mode = \"enforce-everything\"\n");

        assert!(parsed.is_err());
    }

    #[test]
    fn inventory_mode_caps_consumer_nominal_denies_at_warn() {
        let (cfg, warnings) = parse_and_validate(
            "[octet]\nnominal_domain_mode = \"inventory\"\n\n[octet.lint_levels]\nprimitive_domain_alias = \"deny\"\nraw_domain_value = \"allow\"\n",
        );

        assert_eq!(warnings, vec![
            "octet `nominal_domain_mode = \"inventory\"` caps `primitive_domain_alias` at `warn`; ignoring `deny` from `dylint.toml` lint_levels"
        ]);
        for lint_name in NOMINAL_DOMAIN_LINTS {
            assert_eq!(cfg.lint_level(lint_name), Some(LintLevel::Warn), "{lint_name}");
        }
    }

    #[test]
    fn inventory_cap_reports_typed_diagnostic_and_warn_mode_keeps_consumer_deny() {
        let (_cfg, diagnostics) = parse_and_validate_diagnostics(
            "[octet]\nnominal_domain_mode = \"inventory\"\n\n[octet.lint_levels]\nnewtype_invariant_bypass = \"deny\"\n",
        );
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].kind, ConfigDiagnosticKind::NominalInventoryLevelCapped);
        assert_eq!(diagnostics[0].kind.as_str(), "nominal_inventory_level_capped");
        assert_eq!(diagnostics[0].field, "octet.lint_levels");
        assert_eq!(diagnostics[0].item, "newtype_invariant_bypass");

        let (cfg, warnings) = parse_and_validate(
            "[octet]\nnominal_domain_mode = \"warn\"\n\n[octet.lint_levels]\nprimitive_domain_alias = \"deny\"\n",
        );
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(cfg.lint_level("primitive_domain_alias"), Some(LintLevel::Deny));
    }

    #[test]
    fn unknown_config_keys_warn_without_failing_validation() {
        let (cfg, warnings) = parse_and_validate(
            "[octet]\nmax_parameters = 6\ndisabled_lint = [\"no_unwrap\"]\n\n[octet.lint-levels]\nno_unwrap = \"deny\"\n",
        );

        assert_eq!(warnings, vec![
            "unknown octet config key in `dylint.toml`: `disabled_lint`",
            "unknown octet config key in `dylint.toml`: `lint-levels`",
        ]);
        assert_eq!(cfg.max_parameters, 6);
        assert!(!cfg.lint_is_disabled("no_unwrap"));
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn unknown_config_keys_report_typed_diagnostics() {
        let (_cfg, diagnostics) =
            parse_and_validate_diagnostics("[octet]\nnot_a_key = 1\n\n[octet.lint_levels]\nno_unwrap = \"deny\"\n");

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].kind, ConfigDiagnosticKind::UnknownConfigKey);
        assert_eq!(diagnostics[0].kind.as_str(), "unknown_config_key");
        assert_eq!(diagnostics[0].source, "dylint.toml");
        assert_eq!(diagnostics[0].field, "octet");
        assert_eq!(diagnostics[0].item, "not_a_key");
    }

    #[test]
    fn config_law_valid_lint_levels_are_bounded_and_replayable() {
        assert!(CONFIG_LAW_REPLAY_COMMAND.contains("config_law"));

        let mut case_count = 0;
        for (raw_level, expected_level) in GENERATED_LINT_LEVELS {
            for lint_name in GENERATED_LINT_NAMES {
                let (cfg, diagnostics) =
                    parse_and_validate_diagnostics(&generated_lint_level_config(lint_name, raw_level));

                assert!(
                    diagnostics.is_empty(),
                    "{CONFIG_VALID_LEVEL_LAW} emitted diagnostics for {lint_name}={raw_level}: {diagnostics:?}"
                );
                assert_eq!(
                    cfg.lint_level(lint_name),
                    Some(*expected_level),
                    "{CONFIG_VALID_LEVEL_LAW} failed for {lint_name}={raw_level}"
                );
                case_count += 1;
            }
        }

        assert_eq!(case_count, CONFIG_VALID_LEVEL_CASE_BUDGET, "{CONFIG_VALID_LEVEL_LAW}");
    }

    #[test]
    fn config_law_disabled_lints_win_over_generated_level_overrides() {
        let mut case_count = 0;
        for (raw_level, _expected_level) in GENERATED_LINT_LEVELS {
            for lint_name in GENERATED_LINT_NAMES {
                let (cfg, diagnostics) =
                    parse_and_validate_diagnostics(&generated_disabled_and_lint_level_config(lint_name, raw_level));

                assert!(
                    cfg.lint_is_disabled(lint_name),
                    "{CONFIG_DISABLED_PRECEDENCE_LAW} did not disable {lint_name}"
                );
                assert_eq!(cfg.lint_level(lint_name), None, "{CONFIG_DISABLED_PRECEDENCE_LAW} kept {lint_name} level");
                assert_eq!(diagnostics.len(), 1, "{CONFIG_DISABLED_PRECEDENCE_LAW} diagnostic count for {lint_name}");
                assert_eq!(diagnostics[0].kind, ConfigDiagnosticKind::ConflictingDisabledLintLevel);
                assert_eq!(diagnostics[0].field, "octet.lint_levels");
                assert_eq!(diagnostics[0].item, *lint_name);
                case_count += 1;
            }
        }

        assert_eq!(case_count, CONFIG_DISABLED_PRECEDENCE_CASE_BUDGET, "{CONFIG_DISABLED_PRECEDENCE_LAW}");
    }

    #[test]
    fn config_law_invalid_generated_levels_fail_closed_with_stable_diagnostics() {
        let mut case_count = 0;
        for raw_level in GENERATED_INVALID_LINT_LEVELS {
            for lint_name in GENERATED_LINT_NAMES {
                let (cfg, diagnostics) =
                    parse_and_validate_diagnostics(&generated_lint_level_config(lint_name, raw_level));

                assert_eq!(
                    cfg.lint_level(lint_name),
                    None,
                    "{CONFIG_NEGATIVE_LEVEL_LAW} accepted {lint_name}={raw_level}"
                );
                assert_eq!(diagnostics.len(), 1, "{CONFIG_NEGATIVE_LEVEL_LAW} diagnostic count for {lint_name}");
                assert_eq!(diagnostics[0].kind, ConfigDiagnosticKind::InvalidLintLevel);
                assert_eq!(diagnostics[0].source, "dylint.toml");
                assert_eq!(diagnostics[0].field, "octet.lint_levels");
                assert_eq!(diagnostics[0].item, *lint_name);
                assert!(
                    diagnostics[0].message.contains(raw_level),
                    "{CONFIG_NEGATIVE_LEVEL_LAW} diagnostic should replay failing level {raw_level}"
                );
                case_count += 1;
            }
        }

        assert_eq!(case_count, CONFIG_NEGATIVE_LEVEL_CASE_BUDGET, "{CONFIG_NEGATIVE_LEVEL_LAW}");
    }

    #[test]
    fn legacy_slotcar_config_is_used_when_octet_section_is_absent() {
        let (selected, notices) =
            try_parse_config("[slotcar]\nacronym_allowlist = [\"IO\"]\n").expect("slotcar parses");

        assert_eq!(selected.acronym_allowlist, vec!["IO".to_owned()]);
        assert!(notices.is_empty(), "a used legacy section is not reported: {notices:?}");
    }

    #[test]
    fn octet_config_wins_over_legacy_slotcar_and_tigerstyle_config() {
        let (selected, notices) = try_parse_config(
            "[octet]\nacronym_allowlist = [\"RPC\"]\n\n[slotcar]\nacronym_allowlist = [\"IO\"]\n\n[tigerstyle]\nmax_parameters = 9\n",
        )
        .expect("octet parses");

        assert_eq!(selected.acronym_allowlist, vec!["RPC".to_owned()]);
        assert_eq!(selected.max_parameters, Config::default().max_parameters);
        assert_eq!(notices.len(), 2, "both shadowed legacy sections are reported: {notices:?}");
        assert!(notices[0].contains("`[slotcar]`") && notices[0].contains("`[octet]`"), "{notices:?}");
        assert!(notices[1].contains("`[tigerstyle]`") && notices[1].contains("`[octet]`"), "{notices:?}");
    }

    #[test]
    fn lint_levels_table_alone_shadows_legacy_settings_with_a_notice() {
        let (selected, notices) =
            try_parse_config("[octet.lint_levels]\nno_unwrap = \"deny\"\n\n[tigerstyle]\nmax_parameters = 9\n")
                .expect("octet parses");

        assert_eq!(selected.max_parameters, Config::default().max_parameters);
        assert_eq!(notices.len(), 1, "{notices:?}");
        assert!(notices[0].contains("`[tigerstyle]`"), "{notices:?}");
    }

    #[test]
    fn malformed_shadowed_legacy_sections_are_not_parsed() {
        let (selected, notices) = try_parse_config(
            "[octet]\nmax_parameters = 6\n\n[slotcar]\nmax_parameters = \"seven\"\n\n[tigerstyle]\ndisabled_lints = 3\n",
        )
        .expect("only the selected section is parsed");

        assert_eq!(selected.max_parameters, 6);
        assert_eq!(notices.len(), 2, "{notices:?}");
        assert!(
            try_parse_config("[slotcar]\nmax_parameters = \"seven\"\n").is_err(),
            "a malformed selected section still fails to parse"
        );
    }

    #[test]
    fn omitted_disabled_lints_and_lint_levels_default_to_empty() {
        let cfg = parse_config("[octet]\n");
        assert_eq!(cfg.max_parameters, 5);
        assert!(cfg.disabled_lints.is_empty());
        assert!(cfg.lint_levels.is_empty());
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn indexable_exception_classes_default_to_scoped_allowlist() {
        let cfg = parse_config("[octet]\n");
        let exceptions = cfg.indexable_lint_exceptions();

        assert!(exceptions.rustc_private_aliases);
        assert!(exceptions.lint_identity_names);
        assert!(exceptions.generated_searchable_names);
    }

    #[test]
    fn indexable_exception_classes_can_be_narrowed_and_validate_unknowns() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
indexable_lint_exceptions = ["rustc_private_aliases", "not_a_real_exception"]
"#,
        );
        let exceptions = cfg.indexable_lint_exceptions();

        assert!(exceptions.rustc_private_aliases);
        assert!(!exceptions.lint_identity_names);
        assert!(!exceptions.generated_searchable_names);
        assert_eq!(warnings, vec![
            "unknown octet indexable_lint_exceptions class in `dylint.toml`: `not_a_real_exception`"
        ]);
    }

    #[test]
    fn explicit_disabled_lints_and_lint_levels_deserialize_successfully() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
max_parameters = 7
disabled_lints = ["no_panic"]

[octet.lint_levels]
no_unwrap = "deny"
"#,
        );

        assert!(warnings.is_empty());
        assert_eq!(cfg.max_parameters, 7);
        assert_eq!(cfg.disabled_lints, vec!["no_panic".to_owned()]);
        assert_eq!(cfg.lint_levels.get("no_unwrap"), Some(&"deny".to_owned()));
        assert_eq!(cfg.lint_level("no_unwrap"), Some(LintLevel::Deny));
    }

    #[test]
    fn unknown_disabled_lints_warn_without_failing_validation() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
disabled_lints = ["not_a_real_lint"]
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` disabled_lints: `not_a_real_lint`"]);
        assert!(cfg.lint_levels.is_empty());
    }

    #[test]
    fn mixed_disabled_lints_keeps_valid_entry_while_warning_on_unknown_name() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
disabled_lints = ["not_a_real_lint", "no_unwrap"]
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` disabled_lints: `not_a_real_lint`"]);
        assert!(cfg.lint_is_disabled("no_unwrap"));
    }

    #[test]
    fn unknown_lint_levels_warn_without_failing_validation() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
not_a_real_lint = "deny"
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` lint_levels: `not_a_real_lint`"]);
        assert_eq!(cfg.lint_level("not_a_real_lint"), None);
    }

    #[test]
    fn mixed_lint_levels_ignores_unknown_name_but_keeps_valid_override() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
not_a_real_lint = "deny"
no_unwrap = "warn"
"#,
        );

        assert_eq!(warnings, vec!["unknown octet lint in `dylint.toml` lint_levels: `not_a_real_lint`"]);
        assert_eq!(cfg.lint_level("no_unwrap"), Some(LintLevel::Warn));
    }

    #[test]
    fn invalid_lint_level_warns_and_keeps_default_behavior() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
no_unwrap = "fatal"
"#,
        );

        assert_eq!(warnings, vec![
            "invalid octet lint level in `dylint.toml` lint_levels for `no_unwrap`: `fatal` (expected `allow`, `warn`, or `deny`)"
        ]);
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn mixed_lint_levels_ignore_invalid_value_but_keep_valid_override() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet.lint_levels]
no_unwrap = "fatal"
no_panic = "deny"
"#,
        );

        assert_eq!(warnings, vec![
            "invalid octet lint level in `dylint.toml` lint_levels for `no_unwrap`: `fatal` (expected `allow`, `warn`, or `deny`)"
        ]);
        assert_eq!(cfg.lint_level("no_panic"), Some(LintLevel::Deny));
    }

    #[test]
    fn multiple_independent_config_errors_accumulate_with_typed_context() {
        let (cfg, diagnostics) = parse_and_validate_diagnostics(
            r#"
[octet]
disabled_lints = ["not_a_real_lint", "no_unwrap"]
indexable_lint_exceptions = ["bad_exception"]

[octet.lint_levels]
no_panic = "fatal"
no_unwrap = "deny"
not_a_real_level_lint = "warn"
"#,
        );

        let diagnostic_shape = diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.kind, diagnostic.source, diagnostic.field, diagnostic.item.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(diagnostic_shape, vec![
            (ConfigDiagnosticKind::UnknownDisabledLint, "dylint.toml", "octet.disabled_lints", "not_a_real_lint",),
            (
                ConfigDiagnosticKind::UnknownIndexableExceptionClass,
                "dylint.toml",
                "octet.indexable_lint_exceptions",
                "bad_exception",
            ),
            (ConfigDiagnosticKind::InvalidLintLevel, "dylint.toml", "octet.lint_levels", "no_panic"),
            (ConfigDiagnosticKind::ConflictingDisabledLintLevel, "dylint.toml", "octet.lint_levels", "no_unwrap",),
            (ConfigDiagnosticKind::UnknownLintLevel, "dylint.toml", "octet.lint_levels", "not_a_real_level_lint",),
        ]);
        assert_eq!(diagnostics[2].kind.as_str(), "invalid_lint_level");
        assert!(cfg.lint_is_disabled("no_unwrap"));
        assert_eq!(cfg.lint_level("no_panic"), None);
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn overlapping_disabled_lints_and_lint_levels_warn_and_keep_lint_disabled() {
        let (cfg, warnings) = parse_and_validate(
            r#"
[octet]
disabled_lints = ["no_unwrap"]

[octet.lint_levels]
no_unwrap = "deny"
"#,
        );

        assert_eq!(warnings, vec![
            "conflicting octet config in `dylint.toml`: `no_unwrap` appears in both `disabled_lints` and `lint_levels`; ignoring level override"
        ]);
        assert!(cfg.lint_is_disabled("no_unwrap"));
        assert_eq!(cfg.lint_level("no_unwrap"), None);
    }

    #[test]
    fn raw_lint_levels_preserve_original_strings() {
        let cfg = parse_config(
            r#"
[octet.lint_levels]
no_unwrap = "deny"
"#,
        );

        let mut expected = HashMap::new();
        expected.insert("no_unwrap".to_owned(), "deny".to_owned());
        assert_eq!(cfg.lint_levels, expected);
    }

    const INDEXABLE_POLICY_LINTS: &[&str] = &[
        "drop_must_use",
        "fallible_int_fallback",
        "magic_number",
        "excessive_file_length",
        "module_file_count",
        "underscore_in_module_filename",
        "non_trait_imports",
        "renamed_imports",
        "path_segment_repetition",
    ];

    #[test]
    fn indexable_policy_lints_are_accepted_by_config_validation() {
        let disabled = INDEXABLE_POLICY_LINTS
            .iter()
            .map(|lint_name| format!("\"{lint_name}\""))
            .collect::<Vec<_>>()
            .join(", ");
        let levels = INDEXABLE_POLICY_LINTS
            .iter()
            .map(|lint_name| format!("{lint_name} = \"deny\""))
            .collect::<Vec<_>>()
            .join("\n");

        let (disabled_cfg, disabled_warnings) =
            parse_and_validate(&format!("[octet]\ndisabled_lints = [{disabled}]\n"));
        assert!(disabled_warnings.is_empty(), "indexable disabled_lints should validate without warnings");
        for lint_name in INDEXABLE_POLICY_LINTS {
            assert!(disabled_cfg.lint_is_disabled(lint_name), "{lint_name} should be accepted in disabled_lints");
        }

        let (levels_cfg, level_warnings) = parse_and_validate(&format!("[octet.lint_levels]\n{levels}\n"));
        assert!(level_warnings.is_empty(), "indexable lint_levels should validate without warnings");
        for lint_name in INDEXABLE_POLICY_LINTS {
            assert_eq!(
                levels_cfg.lint_level(lint_name),
                Some(LintLevel::Deny),
                "{lint_name} should accept lint_levels override"
            );
        }
    }

    #[test]
    fn strict_profile_strengthens_required_lints_without_changing_unrelated_levels() {
        let mut cfg = parse_config(
            r#"
[octet]
disabled_lints = ["no_unwrap", "acronym_style"]

[octet.lint_levels]
no_panic = "allow"
function_length = "warn"
"#,
        );
        let diagnostics = cfg.validate_diagnostics();
        assert!(diagnostics.is_empty(), "fixture config should validate: {diagnostics:?}");
        let profile = octet_lint_metadata::strict_profile::strict_profile("robust").expect("robust profile");

        cfg.apply_strict_profile(&profile).expect("robust profile should apply");

        assert!(!cfg.lint_is_disabled("no_unwrap"));
        assert!(cfg.lint_is_disabled("acronym_style"));
        assert_eq!(cfg.lint_level("no_unwrap"), Some(LintLevel::Deny));
        assert_eq!(cfg.lint_level("no_panic"), Some(LintLevel::Deny));
        assert_eq!(cfg.lint_level("function_length"), Some(LintLevel::Warn));
    }

    #[test]
    fn boolean_api_rejects_workspace_allow_and_strict_warn_override() {
        let mut disabled = parse_config("[octet]\ndisabled_lints = [\"raw_bool_parameter\"]\n");
        disabled.validate_diagnostics();
        assert!(disabled.reject_broad_boolean_suppression().is_err());

        let mut allowed = parse_config("[octet.lint_levels]\nraw_bool_parameter_suppression = \"allow\"\n");
        allowed.validate_diagnostics();
        assert!(allowed.reject_broad_boolean_suppression().is_err());

        let mut weaker = parse_config("[octet.lint_levels]\nraw_bool_parameter = \"warn\"\n");
        weaker.validate_diagnostics();
        assert!(weaker.reject_broad_boolean_suppression().is_ok());
        let robust = octet_lint_metadata::strict_profile::strict_profile("robust").expect("strict profile");
        assert!(weaker.apply_strict_profile(&robust).is_err());

        let mut cli_denied = parse_config("[octet]\n");
        cli_denied.validate_diagnostics();
        cli_denied.enforce_compiler_boolean_guard(&[(
            "tigerstyle::raw_bool_parameter_suppression".to_owned(),
            rustc_lint::Level::Deny,
        )]);
        assert_eq!(cli_denied.lint_level("raw_bool_parameter_suppression"), Some(LintLevel::Deny));
    }

    #[test]
    fn reviewed_profiles_select_exact_only_nominal_domain_denial() {
        let mut cfg = parse_config("[octet]\nnominal_domain_mode = \"inventory\"\n");
        let diagnostics = cfg.validate_diagnostics();
        assert!(diagnostics.is_empty(), "fixture config should validate: {diagnostics:?}");
        let profile = octet_lint_metadata::strict_profile::strict_profile("bounded-systems").expect("bounded profile");

        cfg.apply_strict_profile(&profile).expect("bounded profile should apply");

        assert_eq!(cfg.nominal_domain_mode, Some(octet_lint_metadata::nominal_domain::EnforcementMode::Deny));
        for lint_name in NOMINAL_DOMAIN_LINTS {
            assert_eq!(cfg.lint_level(lint_name), Some(LintLevel::Deny));
        }
    }

    fn command_line_level(lint_opts: &[(String, rustc_lint::Level)], lint_name: &str) -> Option<rustc_lint::Level> {
        lint_opts.iter().rev().find(|(name, _)| name == lint_name).map(|(_, level)| *level)
    }

    #[test]
    fn explicit_command_line_levels_win_over_consumer_lint_levels() {
        let (cfg, warnings) =
            parse_and_validate("[octet.lint_levels]\nnumeric_units = \"warn\"\nno_unwrap = \"deny\"\n");
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        let mut lint_opts = vec![
            ("unknown_lints".to_owned(), rustc_lint::Level::Deny),
            ("numeric_units".to_owned(), rustc_lint::Level::Deny),
        ];

        cfg.merge_into_lint_opts(&mut lint_opts);

        assert_eq!(command_line_level(&lint_opts, "numeric_units"), Some(rustc_lint::Level::Deny));
        assert_eq!(command_line_level(&lint_opts, "no_unwrap"), Some(rustc_lint::Level::Deny));
        assert_eq!(lint_opts[2..], [
            ("unknown_lints".to_owned(), rustc_lint::Level::Deny),
            ("numeric_units".to_owned(), rustc_lint::Level::Deny),
        ]);
    }

    #[test]
    fn strict_profile_floor_wins_over_command_line_weakening() {
        let mut cfg = parse_config("[octet.lint_levels]\nno_panic = \"allow\"\nfunction_length = \"warn\"\n");
        let diagnostics = cfg.validate_diagnostics();
        assert!(diagnostics.is_empty(), "fixture config should validate: {diagnostics:?}");
        let profile = octet_lint_metadata::strict_profile::strict_profile("robust").expect("robust profile");
        cfg.apply_strict_profile(&profile).expect("robust profile should apply");
        let mut lint_opts = vec![
            ("no_panic".to_owned(), rustc_lint::Level::Allow),
            ("function_length".to_owned(), rustc_lint::Level::Allow),
        ];

        cfg.merge_into_lint_opts(&mut lint_opts);

        assert_eq!(command_line_level(&lint_opts, "no_panic"), Some(rustc_lint::Level::Deny));
        assert_eq!(command_line_level(&lint_opts, "function_length"), Some(rustc_lint::Level::Allow));
    }

    #[test]
    fn deterministic_core_profile_rejects_missing_core_scopes() {
        let mut cfg = parse_config("[octet]\n");
        let diagnostics = cfg.validate_diagnostics();
        assert!(diagnostics.is_empty(), "fixture config should validate: {diagnostics:?}");
        let profile =
            octet_lint_metadata::strict_profile::strict_profile("deterministic-core").expect("deterministic profile");

        let error = cfg.apply_strict_profile(&profile).expect_err("missing core scopes should fail closed");

        assert!(error.contains("deterministic-core"));
        assert!(error.contains("core_source_scopes"));
    }

    #[test]
    fn deterministic_core_profile_rejects_scopes_that_normalize_to_empty() {
        let mut cfg = parse_config("[octet]\ncore_source_scopes = [\" ./ \"]\n");
        let profile =
            octet_lint_metadata::strict_profile::strict_profile("deterministic-core").expect("deterministic profile");

        let diagnostics = cfg.validate_diagnostics();

        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].kind, ConfigDiagnosticKind::EmptySourceScope);
        assert_eq!(diagnostics[0].field, "octet.core_source_scopes");
        let error = cfg.apply_strict_profile(&profile).expect_err("no usable core scope remains");
        assert!(error.contains("core_source_scopes"));
    }

    #[test]
    fn source_scopes_normalize_like_the_runner() {
        let (cfg, warnings) = parse_and_validate(
            "[octet]\ncore_source_scopes = [\" ./crates/domain/src/ \", \"crates/model/src\"]\nstate_machine_core_scopes = [\"./crates/domain/src/fsm/\"]\nstate_machine_generated_adapter_scopes = [\"crates/domain/src/generated/\"]\n",
        );

        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        assert_eq!(cfg.core_source_scopes, ["crates/domain/src", "crates/model/src"]);
        assert_eq!(cfg.state_machine_core_scopes, ["crates/domain/src/fsm"]);
        assert_eq!(cfg.state_machine_generated_adapter_scopes, ["crates/domain/src/generated"]);
    }

    #[test]
    fn source_scopes_resolve_to_matchable_paths_and_report_dead_ones() {
        use crate::source::ScopeResolution;

        let (mut cfg, warnings) = parse_and_validate(
            "[octet]\ncore_source_scopes = [\"./src/\", \"linked\", \"crates/x/../../src/lib.rs\", \"source.rs\"]\nstate_machine_core_scopes = [\"src/fsm\", \"..\"]\nstate_machine_generated_adapter_scopes = [\"crates/..\"]\n",
        );
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");
        let resolve = |scope: &str| match scope {
            "src" | "src/fsm" => ScopeResolution::Inside(scope.to_owned()),
            "linked" => ScopeResolution::Inside("src".to_owned()),
            "crates/x/../../src/lib.rs" => ScopeResolution::Inside("src/lib.rs".to_owned()),
            ".." => ScopeResolution::Outside,
            "crates/.." => ScopeResolution::Root,
            _ => ScopeResolution::Missing,
        };

        let diagnostics = cfg.resolve_source_scopes(resolve);

        assert_eq!(cfg.core_source_scopes, [
            "src",
            "linked",
            "src",
            "crates/x/../../src/lib.rs",
            "src/lib.rs",
            "source.rs"
        ]);
        assert_eq!(cfg.state_machine_core_scopes, ["src/fsm", ".."]);
        assert_eq!(cfg.state_machine_generated_adapter_scopes, ["crates/.."]);
        let unmatched = diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.kind, diagnostic.field, diagnostic.item.as_str()))
            .collect::<Vec<_>>();
        assert_eq!(unmatched, [
            (ConfigDiagnosticKind::UnmatchedSourceScope, "octet.core_source_scopes", "source.rs"),
            (ConfigDiagnosticKind::UnmatchedSourceScope, "octet.state_machine_core_scopes", ".."),
            (
                ConfigDiagnosticKind::UnmatchedSourceScope,
                "octet.state_machine_generated_adapter_scopes",
                "crates/.."
            ),
        ]);
        assert_eq!(diagnostics[0].kind.as_str(), "unmatched_source_scope");
        assert!(crate::source::path_in_scopes(std::path::Path::new("src/main.rs"), &cfg.core_source_scopes));
    }

    // rustc names a module reached through an in-root symlink by its in-root
    // path, even when the symlink leads outside the workspace, so such a scope
    // keeps matching through its own text.
    #[cfg(unix)]
    #[test]
    fn scope_through_an_outside_symlink_matches_its_in_root_path() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time after the epoch")
            .as_nanos();
        let scratch = std::env::temp_dir().join(format!("octet-outside-scope-{}-{unique}", std::process::id()));
        let root = scratch.join("workspace");
        std::fs::create_dir_all(root.join("src")).expect("source directory");
        std::fs::create_dir_all(scratch.join("shared/core")).expect("shared directory");
        std::os::unix::fs::symlink(scratch.join("shared/core"), root.join("src/core")).expect("outside symlink");
        let canonical_root = root.canonicalize().expect("canonical root");
        let (mut cfg, warnings) = parse_and_validate("[octet]\ncore_source_scopes = [\"src/core\"]\n");
        assert!(warnings.is_empty(), "unexpected warnings: {warnings:?}");

        let diagnostics = cfg.resolve_source_scopes(|scope| crate::source::resolve_scope(&canonical_root, scope));
        std::fs::remove_dir_all(&scratch).expect("remove scratch directory");

        assert_eq!(cfg.core_source_scopes, ["src/core"]);
        assert!(crate::source::path_in_scopes(std::path::Path::new("src/core/mod.rs"), &cfg.core_source_scopes));
        assert!(!crate::source::path_in_scopes(std::path::Path::new("src/lib.rs"), &cfg.core_source_scopes));
        assert_eq!(diagnostics.len(), 1, "the outside target is reported: {diagnostics:?}");
        assert_eq!(diagnostics[0].kind, ConfigDiagnosticKind::UnmatchedSourceScope);
        assert_eq!(diagnostics[0].item, "src/core");
    }

    #[test]
    fn profile_denying_nominal_lints_suppresses_the_inventory_cap_diagnostic() {
        let doc =
            "[octet]\nnominal_domain_mode = \"inventory\"\n\n[octet.lint_levels]\nprimitive_domain_alias = \"deny\"\n";
        let bounded = octet_lint_metadata::strict_profile::strict_profile("bounded-systems").expect("bounded profile");
        let mut cfg = parse_config(doc);

        let (diagnostics, applied) = cfg.resolve(Some(&bounded));

        applied.expect("bounded profile should apply");
        assert!(diagnostics.is_empty(), "the profile denies, so nothing is capped: {diagnostics:?}");
        assert_eq!(cfg.lint_level("primitive_domain_alias"), Some(LintLevel::Deny));

        // `robust` does not deny the nominal lints, so the inventory cap still holds.
        let robust = octet_lint_metadata::strict_profile::strict_profile("robust").expect("robust profile");
        let mut cfg = parse_config(doc);
        let (diagnostics, applied) = cfg.resolve(Some(&robust));
        applied.expect("robust profile should apply");
        assert_eq!(diagnostics.len(), 1, "{diagnostics:?}");
        assert_eq!(diagnostics[0].kind, ConfigDiagnosticKind::NominalInventoryLevelCapped);
        assert_eq!(cfg.lint_level("primitive_domain_alias"), Some(LintLevel::Warn));
    }

    #[test]
    fn parsed_octet_section_wins_over_legacy_sections() {
        let cfg = parse_config(
            r"
[octet]
max_parameters = 7

[slotcar]
max_parameters = 8

[tigerstyle]
max_parameters = 9
",
        );

        assert_eq!(cfg.max_parameters, 7);
    }

    #[test]
    fn parsed_slotcar_section_wins_over_tigerstyle_fallback() {
        let cfg = parse_config(
            r"
[slotcar]
max_parameters = 8

[tigerstyle]
max_parameters = 9
",
        );

        assert_eq!(cfg.max_parameters, 8);
    }
}

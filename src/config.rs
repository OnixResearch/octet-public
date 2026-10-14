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
    UnknownDisabledLint,
    UnknownIndexableExceptionClass,
    UnknownLintLevel,
    ConflictingDisabledLintLevel,
    InvalidLintLevel,
}

impl ConfigDiagnosticKind {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UnknownDisabledLint => "unknown_disabled_lint",
            Self::UnknownIndexableExceptionClass => "unknown_indexable_exception_class",
            Self::UnknownLintLevel => "unknown_lint_level",
            Self::ConflictingDisabledLintLevel => "conflicting_disabled_lint_level",
            Self::InvalidLintLevel => "invalid_lint_level",
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

    /// Validated per-lint severity overrides used during registration.
    #[serde(skip)]
    validated_lint_levels: HashMap<String, LintLevel>,
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
            sealed_enum_marker: "octet::sealed_enum".to_owned(),
            state_machine_core_marker: "octet::state_machine_core".to_owned(),
            state_machine_enum_marker: "octet::state_machine_enum".to_owned(),
            state_machine_core_scopes: Vec::new(),
            state_machine_generated_adapter_scopes: Vec::new(),
            acronym_allowlist: Vec::new(),
            disabled_lints: Vec::new(),
            indexable_lint_exceptions: DEFAULT_INDEXABLE_EXCEPTIONS.iter().map(|class| (*class).to_owned()).collect(),
            nominal_domain_mode: None,
            lint_levels: HashMap::new(),
            validated_lint_levels: HashMap::new(),
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

    #[must_use]
    pub fn nominal_domain_mode(&self) -> octet_lint_metadata::nominal_domain::EnforcementMode {
        self.nominal_domain_mode.unwrap_or(octet_lint_metadata::nominal_domain::EnforcementMode::Warn)
    }

    fn apply_nominal_domain_mode(&mut self) {
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
            if selected == LintLevel::Deny || *level == LintLevel::Allow {
                *level = selected;
            }
        }
    }

    fn apply_strict_profile(
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
        self.disabled_lints.retain(|lint_name| !required_lints.contains(lint_name.as_str()));
        if NOMINAL_DOMAIN_LINTS.iter().any(|lint_name| required_lints.contains(lint_name)) {
            self.nominal_domain_mode = Some(octet_lint_metadata::nominal_domain::EnforcementMode::Deny);
        }
        for lint_name in &profile.required_lints {
            self.validated_lint_levels.insert(lint_name.clone(), LintLevel::Deny);
        }
        Ok(())
    }

    pub(crate) fn indexable_lint_exceptions(&self) -> IndexableLintExceptions {
        let enabled = |class: &str| self.indexable_lint_exceptions.iter().any(|configured| configured == class);
        IndexableLintExceptions {
            rustc_private_aliases: enabled(INDEXABLE_EXCEPTION_RUSTC_PRIVATE_ALIASES),
            lint_identity_names: enabled(INDEXABLE_EXCEPTION_LINT_IDENTITY_NAMES),
            generated_searchable_names: enabled(INDEXABLE_EXCEPTION_GENERATED_SEARCHABLE_NAMES),
        }
    }

    fn validate(&mut self) -> Vec<String> {
        self.validate_diagnostics().into_iter().map(|diagnostic| diagnostic.message).collect()
    }

    fn validate_diagnostics(&mut self) -> Vec<ConfigDiagnostic> {
        let known_lints = canonical_lint_names();
        let disabled_lints = self.disabled_lints.iter().map(String::as_str).collect::<BTreeSet<_>>();
        let mut diagnostics = Vec::new();
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
        self.apply_nominal_domain_mode();

        diagnostics
    }
}

fn prefer_octet_or_legacy_config(octet: Option<Config>, slotcar: Option<Config>, tigerstyle: Option<Config>) -> Config {
    octet.or(slotcar).or(tigerstyle).unwrap_or_default()
}

/// Load the project config from `dylint.toml`, falling back to defaults.
///
/// # Panics
///
/// Panics when Dylint returns malformed TOML for the Octet, Slotcar, or
/// Tigerstyle config sections; invalid lint names and levels are reported as
/// warnings instead of panics after parsing succeeds.
pub fn load(sess: &rustc_session::Session) -> Config {
    let mut cfg = prefer_octet_or_legacy_config(
        dylint_linting::config::<Config>(env!("CARGO_PKG_NAME")).unwrap_or_else(|error| {
            panic!("Could not parse config as `{}`: {}", std::any::type_name::<Config>(), error)
        }),
        dylint_linting::config::<Config>("slotcar").unwrap_or_else(|error| {
            panic!("Could not parse config as `{}`: {}", std::any::type_name::<Config>(), error)
        }),
        dylint_linting::config::<Config>("tigerstyle").unwrap_or_else(|error| {
            panic!("Could not parse config as `{}`: {}", std::any::type_name::<Config>(), error)
        }),
    );
    for warning in cfg.validate() {
        sess.dcx().warn(warning);
    }
    apply_selected_strict_profile(&mut cfg).unwrap_or_else(|error| panic!("{error}"));
    cfg
}

fn apply_selected_strict_profile(cfg: &mut Config) -> Result<(), String> {
    let profile_id = match std::env::var(octet_lint_metadata::strict_profile::STRICT_PROFILE_ENV) {
        Ok(profile_id) => profile_id,
        Err(std::env::VarError::NotPresent) => return Ok(()),
        Err(error) => return Err(format!("read strict-profile selection: {error}")),
    };
    let profile = octet_lint_metadata::strict_profile::strict_profile(&profile_id)?;
    cfg.apply_strict_profile(&profile)
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use serde::Deserialize;

    use super::Config;
    use super::ConfigDiagnostic;
    use super::ConfigDiagnosticKind;
    use super::LintLevel;
    use super::NOMINAL_DOMAIN_LINTS;
    use super::prefer_octet_or_legacy_config;

    #[derive(Deserialize)]
    struct Wrapper {
        octet: Option<Config>,
        slotcar: Option<Config>,
        tigerstyle: Option<Config>,
    }

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

    fn parse_config(doc: &str) -> Config {
        let wrapper = toml::from_str::<Wrapper>(doc).expect("parse test dylint.toml");
        prefer_octet_or_legacy_config(wrapper.octet, wrapper.slotcar, wrapper.tigerstyle)
    }

    fn parse_and_validate(doc: &str) -> (Config, Vec<String>) {
        let mut cfg = parse_config(doc);
        let warnings = cfg.validate();
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
        let parsed = toml::from_str::<Wrapper>("[octet]\nnominal_domain_mode = \"enforce-everything\"\n");

        assert!(parsed.is_err());
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
        let legacy = Config {
            acronym_allowlist: vec!["IO".to_owned()],
            ..Default::default()
        };

        let selected = prefer_octet_or_legacy_config(None, Some(legacy), None);

        assert_eq!(selected.acronym_allowlist, vec!["IO".to_owned()]);
    }

    #[test]
    fn octet_config_wins_over_legacy_slotcar_and_tigerstyle_config() {
        let primary = Config {
            acronym_allowlist: vec!["RPC".to_owned()],
            ..Default::default()
        };
        let legacy = Config {
            acronym_allowlist: vec!["IO".to_owned()],
            ..Default::default()
        };

        let selected = prefer_octet_or_legacy_config(
            Some(primary),
            Some(legacy),
            Some(Config {
                max_parameters: 9,
                ..Default::default()
            }),
        );

        assert_eq!(selected.acronym_allowlist, vec!["RPC".to_owned()]);
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

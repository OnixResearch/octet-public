#![feature(rustc_private)]

extern crate rustc_abi;
extern crate rustc_ast;
extern crate rustc_errors;
extern crate rustc_hir;
extern crate rustc_lint;
extern crate rustc_middle;
extern crate rustc_session;
extern crate rustc_span;

mod concurrency;
pub mod config;
mod diagnostics;
mod naming;
mod performance;
mod purity;
mod safety;
mod structure;
mod test_context;

use config::Config;

dylint_linting::dylint_library!();

pub(crate) struct LintInventoryEntry {
    pub name: &'static str,
    #[cfg_attr(not(test), allow(dead_code))]
    pub lint: &'static rustc_lint::Lint,
}

macro_rules! lint_inventory {
    ($( $name:literal, $lint:expr => |$lint_store:ident, $cfg:ident| $body:block ),+ $(,)?) => {
        pub(crate) const LINT_INVENTORY: &[LintInventoryEntry] = &[
            $(
                LintInventoryEntry {
                    name: $name,
                    lint: $lint,
                },
            )+
        ];

        fn register_inventory(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
            $(
                if !cfg.lint_is_disabled($name) {
                    lint_store.register_lints(&[$lint]);
                    let $lint_store = &mut *lint_store;
                    let $cfg = cfg;
                    $body
                }
            )+
        }
    };
}

#[cfg(test)]
fn default_level_name(level: rustc_lint::Level) -> &'static str {
    match level {
        rustc_lint::Level::Allow => "allow",
        rustc_lint::Level::Warn => "warn",
        rustc_lint::Level::ForceWarn => "force_warn",
        rustc_lint::Level::Deny => "deny",
        rustc_lint::Level::Forbid => "forbid",
        rustc_lint::Level::Expect => "expect",
    }
}

#[allow(invalid_reference_casting)]
fn apply_lint_level_overrides(sess: &rustc_session::Session, cfg: &Config) {
    let opts = std::ptr::addr_of!(sess.opts).cast_mut();

    // SAFETY: rustc calls `register_lints` during lint-store initialization,
    // before lint levels are computed from `sess.opts.lint_opts`. The API only
    // provides `&Session`, so the only way to feed config-owned per-lint levels
    // into rustc's normal lint-level machinery is to append to `lint_opts`
    // through a raw pointer at this setup point.
    unsafe {
        for lint in LINT_INVENTORY {
            if let Some(level) = cfg.lint_level(lint.name) {
                (*opts).lint_opts.push((lint.name.to_owned(), level.rustc_level()));
            }
        }
    }
}

fn lint_id_for_name(lint_name: &str) -> rustc_lint::LintId {
    let entry = LINT_INVENTORY
        .iter()
        .find(|lint| lint.name == lint_name)
        .unwrap_or_else(|| panic!("missing lint inventory entry for `{lint_name}`"));
    rustc_lint::LintId::of(entry.lint)
}

fn register_no_todo_bundle(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
    let emit_no_todo = !cfg.lint_is_disabled("no_todo");
    let emit_unjustified_allow = !cfg.lint_is_disabled("unjustified_no_todo_allow");
    if !emit_no_todo && !emit_unjustified_allow {
        return;
    }

    lint_store
        .register_late_pass(move |_| Box::new(safety::no_todo::NoTodo::new(emit_no_todo, emit_unjustified_allow)));
}

fn register_domain_groups(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
    for group in diagnostics::group_snapshots() {
        let lint_ids = group
            .lint_names
            .into_iter()
            .filter(|lint_name| !cfg.lint_is_disabled(lint_name))
            .map(lint_id_for_name)
            .collect::<Vec<_>>();
        if lint_ids.is_empty() {
            continue;
        }

        lint_store.register_group(true, group.group_name, None, lint_ids);
    }
}

fn register_compatibility_aliases(lint_store: &mut rustc_lint::LintStore, cfg: &Config) {
    for lint in LINT_INVENTORY {
        if cfg.lint_is_disabled(lint.name) {
            continue;
        }

        lint_store.register_group(true, lint.name, None, vec![rustc_lint::LintId::of(lint.lint)]);
    }
}

lint_inventory! {
    // Early passes (post-expansion, syntax-only).
    // compound_assertion detects the expanded `if !(a && b) { panic(..) }` pattern.
    "compound_assertion", safety::compound_assertion::COMPOUND_ASSERTION => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(safety::compound_assertion::CompoundAssertion));
    },
    "function_length", structure::function_length::FUNCTION_LENGTH => |lint_store, cfg| {
        let fl_max = cfg.function_length_max;
        lint_store.register_early_pass(move || Box::new(structure::function_length::FunctionLength::new(fl_max)));
    },
    "acronym_style", naming::acronym_style::ACRONYM_STYLE => |lint_store, cfg| {
        let allowlist = cfg.acronym_allowlist.clone();
        lint_store.register_early_pass(move || Box::new(naming::acronym_style::AcronymStyle::new(allowlist.clone())));
    },
    "numeric_units", naming::numeric_units::NUMERIC_UNITS => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(naming::numeric_units::NumericUnits));
    },
    "magic_number", structure::magic_number::MAGIC_NUMBER => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::magic_number::MagicNumber));
    },
    "excessive_file_length", structure::excessive_file_length::EXCESSIVE_FILE_LENGTH => |lint_store, cfg| {
        let max_lines = cfg.excessive_file_length_max;
        lint_store.register_early_pass(move || Box::new(structure::excessive_file_length::ExcessiveFileLength::new(max_lines)));
    },
    "module_file_count", structure::module_file_count::MODULE_FILE_COUNT => |lint_store, cfg| {
        let max_files = cfg.module_file_count_max;
        lint_store.register_early_pass(move || Box::new(structure::module_file_count::ModuleFileCount::new(max_files)));
    },
    "underscore_in_module_filename", naming::underscore_in_module_filename::UNDERSCORE_IN_MODULE_FILENAME => |lint_store, cfg| {
        let exceptions = cfg.indexable_lint_exceptions();
        lint_store.register_early_pass(move || Box::new(naming::underscore_in_module_filename::UnderscoreInModuleFilename::new(exceptions)));
    },
    "non_trait_imports", naming::non_trait_imports::NON_TRAIT_IMPORTS => |lint_store, cfg| {
        let exceptions = cfg.indexable_lint_exceptions();
        lint_store.register_late_pass(move |_| Box::new(naming::non_trait_imports::NonTraitImports::new(exceptions)));
    },
    "renamed_imports", naming::renamed_imports::RENAMED_IMPORTS => |lint_store, cfg| {
        let exceptions = cfg.indexable_lint_exceptions();
        lint_store.register_late_pass(move |_| Box::new(naming::renamed_imports::RenamedImports::new(exceptions)));
    },
    "path_segment_repetition", naming::path_segment_repetition::PATH_SEGMENT_REPETITION => |lint_store, cfg| {
        let exceptions = cfg.indexable_lint_exceptions();
        lint_store.register_late_pass(move |_| Box::new(naming::path_segment_repetition::PathSegmentRepetition::new(exceptions)));
    },
    "unbounded_loop", safety::unbounded_loop::UNBOUNDED_LOOP => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unbounded_loop::UnboundedLoop));
    },
    "compound_condition", structure::compound_condition::COMPOUND_CONDITION => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(structure::compound_condition::CompoundCondition));
    },
    "verified_purity", purity::verified_purity::VERIFIED_PURITY => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(purity::verified_purity::VerifiedPurity));
    },
    "unjustified_allow", structure::unjustified_allow::UNJUSTIFIED_ALLOW => |lint_store, _cfg| {
        lint_store.register_early_pass(|| Box::new(structure::unjustified_allow::UnjustifiedAllow));
    },
    "nested_conditionals", structure::nested_conditionals::NESTED_CONDITIONALS => |lint_store, cfg| {
        let nc_max = cfg.nesting_max_depth;
        lint_store.register_early_pass(move || Box::new(structure::nested_conditionals::NestedConditionals::new(nc_max)));
    },
    "assertion_density", safety::assertion_density::ASSERTION_DENSITY => |lint_store, cfg| {
        let ad_lines = cfg.assertion_density_min_lines;
        let ad_asserts = cfg.assertion_density_min_assertions;
        lint_store.register_early_pass(move || Box::new(safety::assertion_density::AssertionDensity::new(ad_lines, ad_asserts)));
    },
    "multi_lock_ordering", concurrency::multi_lock_ordering::MULTI_LOCK_ORDERING => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(concurrency::multi_lock_ordering::MultiLockOrdering));
    },
    "ignored_join_handle", concurrency::ignored_join_handle::IGNORED_JOIN_HANDLE => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(concurrency::ignored_join_handle::IgnoredJoinHandle));
    },

    // Late passes (post-type-resolution, need type info).
    "explicit_defaults", safety::explicit_defaults::EXPLICIT_DEFAULTS => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::explicit_defaults::ExplicitDefaults));
    },
    "unbounded_channel", safety::unbounded_channel::UNBOUNDED_CHANNEL => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unbounded_channel::UnboundedChannel));
    },
    "unbounded_collection_growth", safety::unbounded_collection_growth::UNBOUNDED_COLLECTION_GROWTH => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unbounded_collection_growth::UnboundedCollectionGrowth));
    },
    "sentinel_fallback", safety::sentinel_fallback::SENTINEL_FALLBACK => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::sentinel_fallback::SentinelFallback));
    },
    "usize_in_public_api", structure::usize_in_public_api::USIZE_IN_PUBLIC_API => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::usize_in_public_api::UsizeInPublicApi));
    },
    "borrowed_argument_types", structure::borrowed_argument_types::BORROWED_ARGUMENT_TYPES => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::borrowed_argument_types::BorrowedArgumentTypes));
    },
    "deref_polymorphism", structure::deref_polymorphism::DEREF_POLYMORPHISM => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::deref_polymorphism::DerefPolymorphism));
    },
    "clone_then_borrow", performance::clone_then_borrow::CLONE_THEN_BORROW => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(performance::clone_then_borrow::CloneThenBorrow));
    },
    "allocation_in_loop", performance::allocation_in_loop::ALLOCATION_IN_LOOP => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(performance::allocation_in_loop::AllocationInLoop));
    },
    "too_many_parameters", structure::too_many_parameters::TOO_MANY_PARAMETERS => |lint_store, cfg| {
        let max_parameters = cfg.max_parameters;
        lint_store.register_late_pass(move |_| Box::new(structure::too_many_parameters::TooManyParameters::new(max_parameters)));
    },
    "fat_trait", structure::fat_trait::FAT_TRAIT => |lint_store, cfg| {
        let max_methods = cfg.trait_method_max;
        lint_store.register_late_pass(move |_| Box::new(structure::fat_trait::FatTrait::new(max_methods)));
    },
    "concrete_construction_in_core", structure::concrete_construction_in_core::CONCRETE_CONSTRUCTION_IN_CORE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(structure::concrete_construction_in_core::ConcreteConstructionInCore::new(core_source_scopes.clone())));
    },
    "type_accumulation", structure::type_accumulation::TYPE_ACCUMULATION => |lint_store, cfg| {
        let impl_blocks_max = cfg.impl_blocks_max;
        let module_pub_items_max = cfg.module_pub_items_max;
        lint_store.register_late_pass(move |_| Box::new(structure::type_accumulation::TypeAccumulation::new(impl_blocks_max, module_pub_items_max)));
    },
    "negated_predicate", naming::negated_predicate::NEGATED_PREDICATE => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::negated_predicate::NegatedPredicate));
    },
    "bool_naming", naming::bool_naming::BOOL_NAMING => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::bool_naming::BoolNaming));
    },
    "ignored_result", safety::ignored_result::IGNORED_RESULT => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::ignored_result::IgnoredResult));
    },
    "drop_must_use", safety::drop_must_use::DROP_MUST_USE => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::drop_must_use::DropMustUse));
    },
    "fallible_int_fallback", safety::fallible_int_fallback::FALLIBLE_INT_FALLBACK => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::fallible_int_fallback::FallibleIntFallback));
    },
    "unchecked_narrowing", safety::unchecked_narrowing::UNCHECKED_NARROWING => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unchecked_narrowing::UncheckedNarrowing));
    },
    "unchecked_division", safety::unchecked_division::UNCHECKED_DIVISION => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::unchecked_division::UncheckedDivision));
    },
    "contradictory_time", purity::contradictory_time::CONTRADICTORY_TIME => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::contradictory_time::ContradictoryTime));
    },
    "ambient_clock", purity::ambient_clock::AMBIENT_CLOCK => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::ambient_clock::AmbientClock));
    },
    "ambient_env", purity::effect_lints::AMBIENT_ENV => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::effect_lints::AmbientEnv));
    },
    "ambient_random", purity::effect_lints::AMBIENT_RANDOM => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::effect_lints::AmbientRandom));
    },
    "global_state_in_core", purity::effect_lints::GLOBAL_STATE_IN_CORE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(purity::effect_lints::GlobalStateInCore::new(core_source_scopes.clone())));
    },
    "impure_call_in_core", purity::effect_lints::IMPURE_CALL_IN_CORE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(purity::effect_lints::ImpureCallInCore::new(core_source_scopes.clone())));
    },
    "interior_mutability_in_core", purity::effect_lints::INTERIOR_MUTABILITY_IN_CORE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(purity::effect_lints::InteriorMutabilityInCore::new(core_source_scopes.clone())));
    },
    "logging_in_core", purity::effect_lints::LOGGING_IN_CORE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(purity::effect_lints::LoggingInCore::new(core_source_scopes.clone())));
    },
    "missing_const_fn", purity::effect_lints::MISSING_CONST_FN => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::effect_lints::MissingConstFn));
    },
    "mutating_input_in_pure", purity::effect_lints::MUTATING_INPUT_IN_PURE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(purity::effect_lints::MutatingInputInPure::new(core_source_scopes.clone())));
    },
    "nondeterministic_output", purity::effect_lints::NONDETERMINISTIC_OUTPUT => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::effect_lints::NondeterministicOutput));
    },
    "pure_attr_contract", purity::effect_lints::PURE_ATTR_CONTRACT => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::effect_lints::PureAttrContract));
    },
    "resource_return_from_core", purity::effect_lints::RESOURCE_RETURN_FROM_CORE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(purity::effect_lints::ResourceReturnFromCore::new(core_source_scopes.clone())));
    },
    "spawn_in_core", purity::effect_lints::SPAWN_IN_CORE => |lint_store, cfg| {
        let core_source_scopes = cfg.core_source_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(purity::effect_lints::SpawnInCore::new(core_source_scopes.clone())));
    },
    "zero_alloc_attr_contract", purity::effect_lints::ZERO_ALLOC_ATTR_CONTRACT => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(purity::effect_lints::ZeroAllocAttrContract));
    },
    "catch_all_on_enum", safety::catch_all_on_enum::CATCH_ALL_ON_ENUM => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::catch_all_on_enum::CatchAllOnEnum));
    },
    "foreign_enum_catch_all", safety::foreign_enum_catch_all::FOREIGN_ENUM_CATCH_ALL => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::foreign_enum_catch_all::ForeignEnumCatchAll));
    },
    "fragile_exhaustive_enum_match", safety::fragile_exhaustive_enum_match::FRAGILE_EXHAUSTIVE_ENUM_MATCH => |lint_store, cfg| {
        let sealed_enum_marker = cfg.sealed_enum_marker.clone();
        let state_machine_enum_marker = cfg.state_machine_enum_marker.clone();
        lint_store.register_late_pass(move |_| Box::new(safety::fragile_exhaustive_enum_match::FragileExhaustiveEnumMatch::new(
            sealed_enum_marker.clone(),
            state_machine_enum_marker.clone(),
        )));
    },
    "state_machine_exhaustiveness", safety::state_machine_exhaustiveness::STATE_MACHINE_EXHAUSTIVENESS => |lint_store, cfg| {
        let state_machine_core_marker = cfg.state_machine_core_marker.clone();
        let state_machine_enum_marker = cfg.state_machine_enum_marker.clone();
        let state_machine_core_scopes = cfg.state_machine_core_scopes.clone();
        let state_machine_generated_adapter_scopes = cfg.state_machine_generated_adapter_scopes.clone();
        lint_store.register_late_pass(move |_| Box::new(safety::state_machine_exhaustiveness::StateMachineExhaustiveness::new(
            state_machine_core_marker.clone(),
            state_machine_enum_marker.clone(),
            state_machine_core_scopes.clone(),
            state_machine_generated_adapter_scopes.clone(),
        )));
    },
    "platform_dependent_cast", structure::platform_dependent_cast::PLATFORM_DEPENDENT_CAST => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::platform_dependent_cast::PlatformDependentCast));
    },
    "float_for_currency", naming::float_for_currency::FLOAT_FOR_CURRENCY => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(naming::float_for_currency::FloatForCurrency));
    },
    "ambiguous_params", structure::ambiguous_params::AMBIGUOUS_PARAMS => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(structure::ambiguous_params::AmbiguousParams));
    },
    "raw_arithmetic_overflow", safety::raw_arithmetic_overflow::RAW_ARITHMETIC_OVERFLOW => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::raw_arithmetic_overflow::RawArithmeticOverflow));
    },
    "no_unwrap", safety::no_unwrap::NO_UNWRAP => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::no_unwrap::NoUnwrap));
    },
    "no_panic", safety::no_panic::NO_PANIC => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::no_panic::NoPanic));
    },
    "public_unsafe_api", safety::public_unsafe_api::PUBLIC_UNSAFE_API => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::public_unsafe_api::PublicUnsafeApi));
    },
    "defaulting_trait_method", safety::defaulting_trait_method::DEFAULTING_TRAIT_METHOD => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::defaulting_trait_method::DefaultingTraitMethod));
    },
    "no_todo", safety::no_todo::NO_TODO => |_lint_store, _cfg| {
    },
    "unjustified_no_todo_allow", safety::no_todo::UNJUSTIFIED_NO_TODO_ALLOW => |_lint_store, _cfg| {
    },
    "no_recursion", safety::no_recursion::NO_RECURSION => |lint_store, _cfg| {
        lint_store.register_late_pass(|_| Box::new(safety::no_recursion::NoRecursion));
    },
}

#[unsafe(no_mangle)]
pub extern "Rust" fn register_lints(sess: &rustc_session::Session, lint_store: &mut rustc_lint::LintStore) {
    dylint_linting::init_config(sess);
    let cfg = config::load(sess);
    apply_lint_level_overrides(sess, &cfg);
    register_inventory(lint_store, &cfg);
    register_no_todo_bundle(lint_store, &cfg);
    register_domain_groups(lint_store, &cfg);
    register_compatibility_aliases(lint_store, &cfg);
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    use serde::Deserialize;

    use super::LINT_INVENTORY;
    use super::default_level_name;
    use super::lint_id_for_name;
    use super::register_compatibility_aliases;
    use super::register_domain_groups;
    use super::register_inventory;
    use crate::config::Config;
    use crate::config::canonical_lint_names;
    use crate::diagnostics;
    use crate::diagnostics::ExampleMode;

    fn inventory_names() -> Vec<&'static str> {
        LINT_INVENTORY.iter().map(|lint| lint.name).collect()
    }

    fn inventory_levels() -> Vec<String> {
        LINT_INVENTORY
            .iter()
            .map(|lint| format!("{}={}", lint.name, default_level_name(lint.lint.default_level)))
            .collect()
    }

    fn diagnostic_metadata_names() -> Vec<&'static str> {
        diagnostics::all_metadata().iter().map(|meta| meta.lint_name).collect()
    }

    fn diagnostic_metadata_snapshot() -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(diagnostics::all_metadata()).expect("serialize diagnostic metadata snapshot")
        )
    }

    fn diagnostic_group_snapshot() -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&diagnostics::group_snapshots()).expect("serialize diagnostic group snapshot")
        )
    }

    fn compatibility_alias_snapshot() -> String {
        format!(
            "{}\n",
            serde_json::to_string_pretty(&diagnostics::compatibility_alias_snapshots())
                .expect("serialize compatibility alias snapshot")
        )
    }

    #[derive(Debug, Deserialize)]
    struct UiFixtureSet {
        all_fixtures: Vec<String>,
        expected_failures: Vec<String>,
        canonical_tool_group_fixtures: Vec<String>,
        compatibility_fixtures: Vec<String>,
        config_fixtures: Vec<String>,
    }

    #[derive(Debug, Deserialize)]
    struct LintFixtureCoverage {
        lint_name: String,
        fixture: String,
    }

    #[derive(Clone, Debug, Deserialize)]
    struct LintGuardrailFixture {
        lint_name: String,
        fixture: String,
        category: String,
    }

    fn read_fixture_json<T: for<'de> Deserialize<'de>>(name: &str) -> T {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name);
        let raw = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("read fixture artifact {}: {error}", path.display()));
        serde_json::from_str(&raw).unwrap_or_else(|error| panic!("parse fixture artifact {}: {error}", path.display()))
    }

    fn ui_fixture_set() -> UiFixtureSet {
        read_fixture_json("ui_fixture_set.json")
    }

    fn lint_fixture_coverage() -> Vec<LintFixtureCoverage> {
        read_fixture_json("lint_emitting_fixtures.json")
    }

    fn lint_guardrail_coverage() -> Vec<LintGuardrailFixture> {
        read_fixture_json("lint_guardrail_fixtures.json")
    }

    fn discovered_ui_fixtures() -> Vec<String> {
        let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
        let mut stack = vec![ui_dir.clone()];
        let mut fixtures = Vec::new();

        while let Some(dir) = stack.pop() {
            let mut entries = fs::read_dir(&dir)
                .unwrap_or_else(|error| panic!("read UI fixture dir {}: {error}", dir.display()))
                .collect::<Result<Vec<_>, _>>()
                .unwrap_or_else(|error| panic!("collect UI fixture dir {}: {error}", dir.display()));
            entries.sort_by_key(std::fs::DirEntry::path);

            for entry in entries {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }

                let relative = path.strip_prefix(&ui_dir).expect("fixture under ui_tests");
                if relative.components().any(|component| component.as_os_str() == "src") {
                    continue;
                }
                fixtures.push(relative.with_extension("").to_string_lossy().replace('\\', "/"));
            }
        }

        fixtures.sort();
        fixtures
    }

    fn fixture_stderr_path(fixture: &str) -> PathBuf {
        let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
        let fixture_path = ui_dir.join(format!("{fixture}.rs"));
        let stem = fixture_path.file_stem().expect("fixture file stem");
        ui_dir.join(format!("{}.stderr", stem.to_string_lossy()))
    }

    #[derive(Debug)]
    struct ExpectedLintDoc {
        name: String,
        domain: String,
        default_level: String,
        summary: String,
    }

    #[derive(Debug)]
    struct CatalogDocs<'a> {
        readme: &'a str,
        lint_reference: &'a str,
        template: &'a str,
    }

    #[derive(Debug)]
    struct DocumentedLintRow {
        domain: String,
        default_level: String,
        summary: String,
    }

    #[derive(Debug)]
    struct NamespaceContract {
        namespace: String,
        lint_attr: String,
        group_attr: String,
        example_attr: String,
        register_tool_attr: String,
    }

    #[derive(Debug)]
    struct GuidanceLintContract {
        concept: &'static str,
        lint_name: &'static str,
        default_level: &'static str,
        stale_phrases: &'static [&'static str],
        required_fragments: &'static [&'static str],
    }

    const README_TOTAL_MARKER: &str = "Dylint lints";
    const LINT_REFERENCE_TOTAL_MARKER: &str = "ships";
    const NAMESPACE_PROBE_LINT: &str = "no_unwrap";
    const NAMESPACE_EXAMPLE_LINT: &str = "unchecked_narrowing";
    const NAMESPACE_PROBE_DOMAIN: &str = "safety";
    const RUST_IDIOM_GUIDANCE_PATH: &str = "docs/rust-idiom-guidance.md";
    const BORROWED_ARGUMENT_CONCEPT: &str = "Borrowed argument type";
    const BORROWED_ARGUMENT_LINT: &str = "borrowed_argument_types";
    const WARN_DEFAULT_LEVEL: &str = "warn";
    const WARNING_LINT_PHRASE: &str = "warning lint";
    const FUTURE_LINT_CANDIDATE_PHRASE: &str = "future lint candidate";
    const STALE_LINT_TOTAL: &str = "999";
    const STALE_NAMESPACE: &str = "stale_namespace";
    const UNKNOWN_CATALOG_LINT: &str = "not_a_registered_lint";

    fn expected_lint_docs() -> Vec<ExpectedLintDoc> {
        LINT_INVENTORY
            .iter()
            .map(|lint| {
                let meta = diagnostics::diagnostic_meta(lint.name);
                ExpectedLintDoc {
                    name: lint.name.to_owned(),
                    domain: domain_label(meta.domain_group),
                    default_level: meta.default_level.to_owned(),
                    summary: meta.docs_summary.to_owned(),
                }
            })
            .collect()
    }

    fn domain_label(group_name: &str) -> String {
        group_name.rsplit("::").next().expect("domain group should include a label").to_owned()
    }

    fn expected_domain_counts(expected: &[ExpectedLintDoc]) -> BTreeMap<String, usize> {
        let mut counts = BTreeMap::new();
        for entry in expected {
            *counts.entry(entry.domain.clone()).or_default() += 1;
        }
        counts
    }

    fn metadata_coverage_diagnostics(
        registered_names: &[&str],
        catalog: &[diagnostics::LintDiagnosticMeta],
    ) -> Vec<String> {
        let mut diagnostics = Vec::new();
        let mut catalog_counts = BTreeMap::new();
        for entry in catalog {
            *catalog_counts.entry(entry.lint_name).or_insert(0usize) += 1;
            for (field, value) in [
                ("default_level", entry.default_level),
                ("domain_group", entry.domain_group),
                ("principle", entry.principle),
                ("guidance", entry.guidance),
                ("docs_summary", entry.docs_summary),
                ("effect_family", entry.effect_family),
            ] {
                if value.trim().is_empty() {
                    diagnostics.push(format!("catalog entry `{}` has blank `{field}`", entry.lint_name));
                }
            }
        }

        for registered_name in registered_names {
            match catalog_counts.get(registered_name).copied().unwrap_or_default() {
                1 => {}
                0 => diagnostics.push(format!("registered lint `{registered_name}` has no catalog metadata")),
                count => diagnostics
                    .push(format!("registered lint `{registered_name}` has {count} catalog metadata entries")),
            }
        }

        for catalog_name in catalog_counts.keys() {
            if !registered_names.iter().any(|registered_name| registered_name == catalog_name) {
                diagnostics.push(format!("catalog has stale entry for unknown lint `{catalog_name}`"));
            }
        }

        diagnostics
    }

    fn fake_catalog_entry(lint_name: &'static str) -> diagnostics::LintDiagnosticMeta {
        let mut entry = *diagnostics::all_metadata().first().expect("catalog has fixture entry");
        entry.lint_name = lint_name;
        entry
    }

    fn catalog_freshness_diagnostics(docs: CatalogDocs<'_>, expected: &[ExpectedLintDoc]) -> Vec<String> {
        let mut diagnostics = Vec::new();
        let expected_total = expected.len();
        check_documented_total("README", docs.readme, README_TOTAL_MARKER, expected_total, &mut diagnostics);
        check_documented_total(
            "lint-reference",
            docs.lint_reference,
            LINT_REFERENCE_TOTAL_MARKER,
            expected_total,
            &mut diagnostics,
        );

        let expected_counts = expected_domain_counts(expected);
        for (domain, expected_count) in &expected_counts {
            check_documented_domain_count("README", docs.readme, domain, *expected_count, &mut diagnostics);
            check_documented_domain_count(
                "lint-reference",
                docs.lint_reference,
                domain,
                *expected_count,
                &mut diagnostics,
            );
        }

        let documented_rows = documented_lint_rows(docs.lint_reference);
        for entry in expected {
            match documented_rows.get(&entry.name) {
                Some(rows) if rows.is_empty() => {
                    diagnostics.push(format!("lint-reference missing row for `{}`", entry.name))
                }
                Some(rows) => {
                    if rows.len() != 1 {
                        diagnostics.push(format!("lint-reference has duplicate rows for `{}`", entry.name));
                    }
                    let row = rows.first().expect("non-empty rows were checked");
                    if row.domain != entry.domain {
                        diagnostics.push(format!(
                            "lint-reference row for `{}` has domain `{}`, expected `{}`",
                            entry.name, row.domain, entry.domain
                        ));
                    }
                    if row.default_level != entry.default_level {
                        diagnostics.push(format!(
                            "lint-reference row for `{}` has default `{}`, expected `{}`",
                            entry.name, row.default_level, entry.default_level
                        ));
                    }
                    if row.summary != entry.summary {
                        diagnostics.push(format!(
                            "lint-reference row for `{}` has summary `{}`, expected `{}`",
                            entry.name, row.summary, entry.summary
                        ));
                    }
                }
                None => diagnostics.push(format!("lint-reference missing row for `{}`", entry.name)),
            }
        }

        for lint_name in documented_rows.keys() {
            if !expected.iter().any(|entry| entry.name == *lint_name) {
                diagnostics.push(format!("lint-reference has stale row for unknown lint `{lint_name}`"));
            }
        }

        check_namespace_prose(docs, &namespace_contract(), &mut diagnostics);

        diagnostics
    }

    fn namespace_contract() -> NamespaceContract {
        let lint_attr = diagnostics::canonical_lint_name(NAMESPACE_PROBE_LINT);
        let namespace = lint_attr.split("::").next().expect("canonical lint names include a namespace").to_owned();
        NamespaceContract {
            group_attr: format!("{namespace}::{NAMESPACE_PROBE_DOMAIN}"),
            example_attr: diagnostics::canonical_lint_name(NAMESPACE_EXAMPLE_LINT),
            register_tool_attr: format!("#![register_tool({namespace})]"),
            namespace,
            lint_attr,
        }
    }

    fn check_namespace_prose(docs: CatalogDocs<'_>, contract: &NamespaceContract, diagnostics: &mut Vec<String>) {
        let readme_lint_fragment = format!("`{}::<lint>`", contract.namespace);
        let readme_group_fragment = format!("`{}::<domain>`", contract.namespace);
        check_required_fragment("README", docs.readme, &readme_lint_fragment, diagnostics);
        check_required_fragment("README", docs.readme, &readme_group_fragment, diagnostics);
        check_required_fragment("lint-reference", docs.lint_reference, &contract.group_attr, diagnostics);
        check_required_fragment("lint-reference", docs.lint_reference, &contract.example_attr, diagnostics);
        check_required_fragment("lint-reference", docs.lint_reference, &contract.register_tool_attr, diagnostics);
        check_required_fragment("template", docs.template, &contract.lint_attr, diagnostics);
        check_required_fragment("template", docs.template, &contract.group_attr, diagnostics);
    }

    fn check_required_fragment(surface: &str, document: &str, expected_fragment: &str, diagnostics: &mut Vec<String>) {
        if !document.contains(expected_fragment) {
            diagnostics.push(format!("{surface} namespace prose missing `{expected_fragment}`"));
        }
    }

    fn borrowed_argument_guidance_contract() -> GuidanceLintContract {
        GuidanceLintContract {
            concept: BORROWED_ARGUMENT_CONCEPT,
            lint_name: BORROWED_ARGUMENT_LINT,
            default_level: WARN_DEFAULT_LEVEL,
            stale_phrases: &[FUTURE_LINT_CANDIDATE_PHRASE],
            required_fragments: &[BORROWED_ARGUMENT_LINT, WARNING_LINT_PHRASE],
        }
    }

    fn guidance_freshness_diagnostics(
        document_name: &str,
        document: &str,
        contracts: &[GuidanceLintContract],
        expected: &[ExpectedLintDoc],
    ) -> Vec<String> {
        let mut diagnostics = Vec::new();
        for contract in contracts {
            check_guidance_contract(document_name, document, contract, expected, &mut diagnostics);
        }
        diagnostics
    }

    fn check_guidance_contract(
        document_name: &str,
        document: &str,
        contract: &GuidanceLintContract,
        expected: &[ExpectedLintDoc],
        diagnostics: &mut Vec<String>,
    ) {
        let Some(lint_doc) = expected.iter().find(|entry| entry.name == contract.lint_name) else {
            diagnostics
                .push(format!("{document_name} guidance contract references unknown lint `{}`", contract.lint_name));
            return;
        };

        if lint_doc.default_level != contract.default_level {
            diagnostics.push(format!(
                "{document_name} guidance contract for `{}` expected default `{}`, found `{}`",
                contract.lint_name, contract.default_level, lint_doc.default_level
            ));
        }

        let Some(row) = guidance_row_for_concept(document, contract.concept) else {
            diagnostics.push(format!(
                "{document_name} guidance missing `{}` row for `{}`",
                contract.concept, contract.lint_name
            ));
            return;
        };

        let lower_row = row.to_ascii_lowercase();
        for stale_phrase in contract.stale_phrases {
            if lower_row.contains(&stale_phrase.to_ascii_lowercase()) {
                diagnostics.push(format!(
                    "{document_name} guidance `{}` describes `{}` with stale phrase `{stale_phrase}`",
                    contract.concept, contract.lint_name
                ));
            }
        }

        for required_fragment in contract.required_fragments {
            if !lower_row.contains(&required_fragment.to_ascii_lowercase()) {
                diagnostics.push(format!(
                    "{document_name} guidance `{}` for `{}` missing `{required_fragment}`",
                    contract.concept, contract.lint_name
                ));
            }
        }
    }

    fn guidance_row_for_concept(document: &str, concept: &str) -> Option<String> {
        for line in document.lines() {
            let cells = markdown_table_cells(line);
            match cells.as_slice() {
                [concept_cell, _outcome_cell, tier_cell] if concept_cell == concept => return Some(tier_cell.clone()),
                [concept_cell, _outcome_cell, tier_cell, ..] if concept_cell == concept => {
                    return Some(tier_cell.clone());
                }
                _ => {}
            }
        }
        None
    }

    fn check_documented_total(
        surface: &str,
        document: &str,
        marker: &str,
        expected_total: usize,
        diagnostics: &mut Vec<String>,
    ) {
        match documented_total(document, marker) {
            Some(total) if total == expected_total => {}
            Some(total) => diagnostics.push(format!("{surface} has stale total {total}, expected {expected_total}")),
            None => diagnostics.push(format!("{surface} is missing lint total marker `{marker}`")),
        }
    }

    fn check_documented_domain_count(
        surface: &str,
        document: &str,
        domain: &str,
        expected_count: usize,
        diagnostics: &mut Vec<String>,
    ) {
        match documented_domain_count(document, domain) {
            Some(count) if count == expected_count => {}
            Some(count) => diagnostics
                .push(format!("{surface} domain `{domain}` has stale count {count}, expected {expected_count}")),
            None => diagnostics.push(format!("{surface} is missing domain `{domain}`")),
        }
    }

    fn documented_total(document: &str, marker: &str) -> Option<usize> {
        document.lines().find(|line| line.contains(marker))?.split_whitespace().find_map(parse_usize_cell)
    }

    fn documented_domain_count(document: &str, domain: &str) -> Option<usize> {
        document.lines().find_map(|line| {
            let cells = markdown_table_cells(line);
            match cells.as_slice() {
                [documented_domain, ..] if documented_domain == domain => {
                    cells.iter().find_map(|cell| parse_usize_cell(cell))
                }
                _ => None,
            }
        })
    }

    fn documented_lint_rows(lint_reference: &str) -> BTreeMap<String, Vec<DocumentedLintRow>> {
        let mut current_domain = None;
        let mut rows: BTreeMap<String, Vec<DocumentedLintRow>> = BTreeMap::new();
        for line in lint_reference.lines() {
            if let Some(domain) = lint_section_domain(line) {
                current_domain = Some(domain);
                continue;
            }

            let Some(domain) = current_domain.clone() else {
                continue;
            };
            let cells = markdown_table_cells(line);
            if let [lint_cell, default_cell, summary_cell, ..] = cells.as_slice() {
                let Some(lint_name) = backticked_value(lint_cell) else {
                    continue;
                };
                let Some(default_level) = backticked_value(default_cell) else {
                    continue;
                };
                rows.entry(lint_name).or_default().push(DocumentedLintRow {
                    domain: domain.clone(),
                    default_level,
                    summary: summary_cell.clone(),
                });
            }
        }
        rows
    }

    fn lint_section_domain(line: &str) -> Option<String> {
        let title = line.strip_prefix("## ")?;
        let domain = title.split(" lints").next()?;
        (domain != title).then(|| domain.to_ascii_lowercase())
    }

    fn markdown_table_cells(line: &str) -> Vec<String> {
        let trimmed = line.trim();
        if !trimmed.starts_with('|') || !trimmed.ends_with('|') {
            return Vec::new();
        }
        trimmed.trim_matches('|').split('|').map(str::trim).map(str::to_owned).collect()
    }

    fn backticked_value(cell: &str) -> Option<String> {
        let value = cell.strip_prefix('`')?.strip_suffix('`')?;
        Some(value.to_owned())
    }

    fn parse_usize_cell(cell: &str) -> Option<usize> {
        cell.trim_matches('`').parse::<usize>().ok()
    }

    fn repo_text(relative_path: &str) -> String {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(relative_path);
        fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
    }

    #[test]
    fn lint_metadata_catalog_covers_registered_inventory() {
        let diagnostics = metadata_coverage_diagnostics(&inventory_names(), diagnostics::all_metadata());

        assert!(diagnostics.is_empty(), "metadata catalog drifted: {diagnostics:#?}");
    }

    #[test]
    fn lint_metadata_default_levels_match_registered_lints() {
        for lint in LINT_INVENTORY {
            let meta = diagnostics::diagnostic_meta(lint.name);
            assert_eq!(
                meta.default_level,
                default_level_name(lint.lint.default_level),
                "catalog default level drifted for `{}`",
                lint.name
            );
        }
    }

    #[test]
    fn lint_metadata_rejects_unknown_catalog_entries() {
        let mut catalog = diagnostics::all_metadata().to_vec();
        catalog.push(fake_catalog_entry(UNKNOWN_CATALOG_LINT));
        let diagnostics = metadata_coverage_diagnostics(&inventory_names(), &catalog);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("catalog has stale entry for unknown lint")));
    }

    #[test]
    fn lint_metadata_rejects_missing_catalog_entries() {
        let catalog = diagnostics::all_metadata()[1..].to_vec();
        let diagnostics = metadata_coverage_diagnostics(&inventory_names(), &catalog);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("registered lint `compound_assertion` has no catalog metadata"))
        );
    }

    #[test]
    fn catalog_docs_match_inventory_counts_defaults_and_rows() {
        let readme = repo_text("README.md");
        let lint_reference = repo_text("docs/lint-reference.md");
        let template = repo_text("templates/dylint.toml");
        let diagnostics = catalog_freshness_diagnostics(
            CatalogDocs {
                readme: &readme,
                lint_reference: &lint_reference,
                template: &template,
            },
            &expected_lint_docs(),
        );

        assert!(diagnostics.is_empty(), "catalog docs drifted: {diagnostics:#?}");
    }

    #[test]
    fn guidance_freshness_accepts_current_borrowed_argument_row() {
        let guidance = repo_text(RUST_IDIOM_GUIDANCE_PATH);
        let contract = borrowed_argument_guidance_contract();
        let diagnostics =
            guidance_freshness_diagnostics(RUST_IDIOM_GUIDANCE_PATH, &guidance, &[contract], &expected_lint_docs());

        assert!(diagnostics.is_empty(), "guidance docs drifted: {diagnostics:#?}");
    }

    #[test]
    fn guidance_freshness_rejects_stale_future_lint_candidate() {
        let stale_guidance = format!(
            "# Rust idiom guidance\n\n| Concept | Tiger Style outcome | First enforcement tier |\n| --- | --- | --- |\n| {BORROWED_ARGUMENT_CONCEPT} | APIs accept borrowed forms when ownership is unnecessary | Future lint candidate with positive and negative fixtures |\n"
        );
        let contract = borrowed_argument_guidance_contract();
        let diagnostics = guidance_freshness_diagnostics(
            RUST_IDIOM_GUIDANCE_PATH,
            &stale_guidance,
            &[contract],
            &expected_lint_docs(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.contains(BORROWED_ARGUMENT_CONCEPT)
                && diagnostic.contains(BORROWED_ARGUMENT_LINT)
                && diagnostic.contains(FUTURE_LINT_CANDIDATE_PHRASE)
        }));
    }

    #[test]
    fn catalog_freshness_rejects_stale_total() {
        let expected = expected_lint_docs();
        let readme = repo_text("README.md")
            .replace(&format!("{} Dylint lints", expected.len()), &format!("{STALE_LINT_TOTAL} Dylint lints"));
        let lint_reference = repo_text("docs/lint-reference.md");
        let template = repo_text("templates/dylint.toml");
        let diagnostics = catalog_freshness_diagnostics(
            CatalogDocs {
                readme: &readme,
                lint_reference: &lint_reference,
                template: &template,
            },
            &expected,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("README has stale total")));
    }

    #[test]
    fn catalog_freshness_rejects_missing_lint_row() {
        let readme = repo_text("README.md");
        let lint_reference = repo_text("docs/lint-reference.md")
            .lines()
            .filter(|line| !line.starts_with("| `compound_assertion` |"))
            .collect::<Vec<_>>()
            .join("\n");
        let template = repo_text("templates/dylint.toml");
        let diagnostics = catalog_freshness_diagnostics(
            CatalogDocs {
                readme: &readme,
                lint_reference: &lint_reference,
                template: &template,
            },
            &expected_lint_docs(),
        );

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("lint-reference missing row for `compound_assertion`"))
        );
    }

    #[test]
    fn catalog_freshness_rejects_mismatched_default_level() {
        let readme = repo_text("README.md");
        let lint_reference = repo_text("docs/lint-reference.md")
            .replace("| `compound_assertion` | `warn` |", "| `compound_assertion` | `allow` |");
        let template = repo_text("templates/dylint.toml");
        let diagnostics = catalog_freshness_diagnostics(
            CatalogDocs {
                readme: &readme,
                lint_reference: &lint_reference,
                template: &template,
            },
            &expected_lint_docs(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("lint-reference row for `compound_assertion` has default `allow`, expected `warn`")
        }));
    }

    #[test]
    fn catalog_freshness_rejects_mismatched_summary() {
        let readme = repo_text("README.md");
        let lint_reference = repo_text("docs/lint-reference.md").replace(
            "| `compound_assertion` | `warn` | split `assert!(a && b)` into separate assertions |",
            "| `compound_assertion` | `warn` | stale summary text |",
        );
        let template = repo_text("templates/dylint.toml");
        let diagnostics = catalog_freshness_diagnostics(
            CatalogDocs {
                readme: &readme,
                lint_reference: &lint_reference,
                template: &template,
            },
            &expected_lint_docs(),
        );

        assert!(diagnostics.iter().any(|diagnostic| {
            diagnostic.contains("lint-reference row for `compound_assertion` has summary `stale summary text`")
        }));
    }

    #[test]
    fn catalog_freshness_rejects_stale_namespace_prose() {
        let contract = namespace_contract();
        let readme = repo_text("README.md")
            .replace(&format!("`{}::<lint>`", contract.namespace), &format!("`{STALE_NAMESPACE}::<lint>`"));
        let lint_reference = repo_text("docs/lint-reference.md");
        let template = repo_text("templates/dylint.toml");
        let diagnostics = catalog_freshness_diagnostics(
            CatalogDocs {
                readme: &readme,
                lint_reference: &lint_reference,
                template: &template,
            },
            &expected_lint_docs(),
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("README namespace prose missing")));
    }

    #[test]
    fn lint_inventory_names_are_unique() {
        let mut seen = std::collections::BTreeSet::new();
        for lint in LINT_INVENTORY {
            assert!(seen.insert(lint.name), "duplicate lint inventory entry: {}", lint.name);
        }
    }

    #[test]
    fn config_validation_derives_names_from_lint_inventory() {
        assert_eq!(canonical_lint_names(), inventory_names());
    }

    #[test]
    fn diagnostic_metadata_names_match_lint_inventory() {
        assert_eq!(diagnostic_metadata_names(), inventory_names());
    }

    #[test]
    fn diagnostic_metadata_required_examples_are_concrete() {
        for meta in diagnostics::all_metadata() {
            match meta.example_mode {
                ExampleMode::Required => {
                    assert!(meta.example_text.is_some(), "missing required example text for {}", meta.lint_name);
                }
                ExampleMode::None => {
                    assert!(meta.example_text.is_none(), "unexpected example text for {}", meta.lint_name);
                }
            }
        }
    }

    #[test]
    fn lint_inventory_snapshot_matches() {
        let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_inventory_names.txt");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read lint inventory snapshot {}: {error}", snapshot_path.display()));
        let actual = format!("{}\n", inventory_names().join("\n"));
        assert_eq!(expected, actual, "lint inventory snapshot mismatch");
    }

    #[test]
    fn lint_inventory_default_levels_snapshot_matches() {
        let snapshot_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_inventory_default_levels.txt");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read lint default-level snapshot {}: {error}", snapshot_path.display()));
        let actual = format!("{}\n", inventory_levels().join("\n"));
        assert_eq!(expected, actual, "lint default-level snapshot mismatch");
    }

    #[test]
    fn diagnostic_metadata_snapshot_matches() {
        let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_diagnostic_metadata.json");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read diagnostic metadata snapshot {}: {error}", snapshot_path.display()));
        let actual = diagnostic_metadata_snapshot();
        assert_eq!(expected, actual, "diagnostic metadata snapshot mismatch");
    }

    fn lint_store_with_groups_and_aliases(cfg: &Config) -> rustc_lint::LintStore {
        let mut store = rustc_lint::LintStore::new();
        register_inventory(&mut store, cfg);
        register_domain_groups(&mut store, cfg);
        register_compatibility_aliases(&mut store, cfg);
        store
    }

    fn expected_group_ids(group_name: &str, cfg: &Config) -> Vec<rustc_lint::LintId> {
        diagnostics::lint_names_for_group(group_name)
            .into_iter()
            .filter(|lint_name| !cfg.lint_is_disabled(lint_name))
            .map(lint_id_for_name)
            .collect()
    }

    #[test]
    fn all_lints_register_canonical_and_bare_names() {
        let store = lint_store_with_groups_and_aliases(&Config::default());

        for lint in LINT_INVENTORY {
            let canonical_name = diagnostics::canonical_lint_name(lint.name);
            let canonical = store
                .find_lints(&canonical_name)
                .unwrap_or_else(|| panic!("missing canonical tool lint `{canonical_name}`"));
            let bare = store
                .find_lints(lint.name)
                .unwrap_or_else(|| panic!("missing bare compatibility alias `{}`", lint.name));
            assert_eq!(
                canonical, bare,
                "bare alias `{}` should resolve to canonical tool lint `{canonical_name}`",
                lint.name
            );
        }
    }

    #[test]
    fn domain_groups_register_expected_members() {
        let cfg = Config::default();
        let store = lint_store_with_groups_and_aliases(&cfg);

        for &group_name in diagnostics::GROUP_NAMES {
            let expected = expected_group_ids(group_name, &cfg);
            let actual = store
                .find_lints(group_name)
                .unwrap_or_else(|| panic!("missing canonical domain group `{group_name}`"));
            assert_eq!(actual, expected.as_slice(), "domain group `{group_name}` membership mismatch");
        }
    }

    #[test]
    fn disabled_lints_are_omitted_from_groups_and_aliases() {
        let mut cfg = Config::default();
        cfg.disabled_lints = vec!["no_unwrap".to_owned(), "no_panic".to_owned()];

        let store = lint_store_with_groups_and_aliases(&cfg);
        for lint_name in ["no_unwrap", "no_panic"] {
            let canonical_name = diagnostics::canonical_lint_name(lint_name);
            assert!(
                store.find_lints(&canonical_name).is_none(),
                "disabled canonical tool lint `{canonical_name}` should not register"
            );
            assert!(store.find_lints(lint_name).is_none(), "disabled bare alias `{lint_name}` should not register");
        }

        let safety = store
            .find_lints(diagnostics::GROUP_SAFETY)
            .expect("safety group should still register for remaining lints");
        let expected_safety = expected_group_ids(diagnostics::GROUP_SAFETY, &cfg);
        assert_eq!(safety, expected_safety.as_slice(), "disabled safety lints should be omitted from the safety group");
    }

    #[test]
    fn diagnostic_group_snapshot_matches() {
        let snapshot_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_group_memberships.json");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read diagnostic group snapshot {}: {error}", snapshot_path.display()));
        let actual = diagnostic_group_snapshot();
        assert_eq!(expected, actual, "diagnostic group snapshot mismatch");
    }

    #[test]
    fn compatibility_alias_snapshot_matches() {
        let snapshot_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/lint_compatibility_aliases.json");
        let expected = fs::read_to_string(&snapshot_path)
            .unwrap_or_else(|error| panic!("read compatibility alias snapshot {}: {error}", snapshot_path.display()));
        let actual = compatibility_alias_snapshot();
        assert_eq!(expected, actual, "compatibility alias snapshot mismatch");
    }

    #[test]
    fn ui_fixture_set_artifact_covers_the_full_corpus() {
        let fixture_set = ui_fixture_set();
        let discovered = discovered_ui_fixtures();
        let listed = fixture_set.all_fixtures.clone();
        let mut sorted_listed = listed.clone();
        sorted_listed.sort();
        sorted_listed.dedup();

        assert_eq!(listed.len(), sorted_listed.len(), "ui_fixture_set.json should not list duplicate fixtures");
        assert_eq!(sorted_listed, discovered, "ui_fixture_set.json must match the full checked-in UI fixture corpus");

        for fixture in fixture_set
            .expected_failures
            .iter()
            .chain(fixture_set.canonical_tool_group_fixtures.iter())
            .chain(fixture_set.compatibility_fixtures.iter())
            .chain(fixture_set.config_fixtures.iter())
        {
            assert!(
                listed.contains(fixture),
                "ui_fixture_set.json category entry `{fixture}` must appear in all_fixtures"
            );
        }
    }

    const GUARDRAIL_FIXTURE_CATEGORIES: &[&str] = &[
        "safe_shape",
        "configured_off",
        "within_limit",
        "test_context",
        "macro_guardrail",
        "known_exception",
    ];

    fn guardrail_fixture_diagnostics(
        registered_names: &[&str],
        fixture_set: &UiFixtureSet,
        guardrails: &[LintGuardrailFixture],
    ) -> Vec<String> {
        let mut diagnostics = Vec::new();
        for entry in guardrails {
            if !registered_names.contains(&entry.lint_name.as_str()) {
                diagnostics.push(format!("guardrail manifest references unknown lint `{}`", entry.lint_name));
            }
            if !fixture_set.all_fixtures.contains(&entry.fixture) {
                diagnostics.push(format!(
                    "guardrail fixture `{}` for `{}` is not listed in ui_fixture_set.json",
                    entry.fixture, entry.lint_name
                ));
            }
            if !GUARDRAIL_FIXTURE_CATEGORIES.contains(&entry.category.as_str()) {
                diagnostics.push(format!(
                    "guardrail fixture `{}` for `{}` uses unknown category `{}`",
                    entry.fixture, entry.lint_name, entry.category
                ));
            }
        }

        for &lint_name in registered_names {
            if !guardrails.iter().any(|entry| entry.lint_name == lint_name) {
                diagnostics.push(format!("registered lint `{lint_name}` has no guardrail fixture"));
            }
        }

        diagnostics
    }

    #[test]
    fn lint_guardrail_fixture_manifest_covers_inventory_and_names_valid_fixtures() {
        let guardrails = lint_guardrail_coverage();
        let fixture_set = ui_fixture_set();
        let diagnostics = guardrail_fixture_diagnostics(&inventory_names(), &fixture_set, &guardrails);

        assert!(diagnostics.is_empty(), "guardrail fixture manifest drifted: {diagnostics:#?}");

        for entry in guardrails {
            let stderr_path = fixture_stderr_path(&entry.fixture);
            let stderr = fs::read_to_string(&stderr_path)
                .unwrap_or_else(|error| panic!("read guardrail stderr {}: {error}", stderr_path.display()));
            let canonical_name = diagnostics::canonical_lint_name(&entry.lint_name);
            assert!(
                !stderr.contains(&canonical_name),
                "guardrail fixture `{}` must not emit `{canonical_name}` in {}",
                entry.fixture,
                stderr_path.display()
            );
        }
    }

    #[test]
    fn lint_guardrail_fixture_manifest_rejects_unknown_lint() {
        let fixture_set = ui_fixture_set();
        let guardrails = vec![LintGuardrailFixture {
            lint_name: UNKNOWN_CATALOG_LINT.to_owned(),
            fixture: "lint_guardrail_common".to_owned(),
            category: "safe_shape".to_owned(),
        }];
        let diagnostics = guardrail_fixture_diagnostics(&inventory_names(), &fixture_set, &guardrails);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains(UNKNOWN_CATALOG_LINT)));
    }

    #[test]
    fn lint_guardrail_fixture_manifest_rejects_missing_guardrail() {
        let fixture_set = ui_fixture_set();
        let mut guardrails = lint_guardrail_coverage();
        guardrails.retain(|entry| entry.lint_name != "compound_assertion");
        let diagnostics = guardrail_fixture_diagnostics(&inventory_names(), &fixture_set, &guardrails);

        assert!(
            diagnostics
                .iter()
                .any(|diagnostic| diagnostic.contains("registered lint `compound_assertion` has no guardrail fixture"))
        );
    }

    #[test]
    fn lint_guardrail_fixture_manifest_rejects_unknown_category() {
        let fixture_set = ui_fixture_set();
        let guardrails = vec![LintGuardrailFixture {
            lint_name: "compound_assertion".to_owned(),
            fixture: "lint_guardrail_common".to_owned(),
            category: "stale_category".to_owned(),
        }];
        let diagnostics = guardrail_fixture_diagnostics(&inventory_names(), &fixture_set, &guardrails);

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("stale_category")));
    }

    #[test]
    fn lint_emitting_fixture_manifest_matches_inventory_and_emits() {
        let coverage = lint_fixture_coverage();
        let coverage_names = coverage.iter().map(|entry| entry.lint_name.as_str()).collect::<Vec<_>>();
        let fixture_set = ui_fixture_set();

        assert_eq!(
            coverage_names,
            diagnostic_metadata_names(),
            "lint_emitting_fixtures.json must cover every lint exactly once in inventory order"
        );

        for entry in coverage {
            assert!(
                fixture_set.all_fixtures.contains(&entry.fixture),
                "lint fixture `{}` for `{}` must be listed in ui_fixture_set.json",
                entry.fixture,
                entry.lint_name
            );

            let stderr_path = fixture_stderr_path(&entry.fixture);
            let stderr = fs::read_to_string(&stderr_path)
                .unwrap_or_else(|error| panic!("read fixture stderr {}: {error}", stderr_path.display()));
            let canonical_name = diagnostics::canonical_lint_name(&entry.lint_name);
            assert!(
                stderr.contains(&canonical_name),
                "fixture `{}` must emit `{canonical_name}` in {}",
                entry.fixture,
                stderr_path.display()
            );
        }
    }

    const AUTOFIX_ROLLOUT_LINTS: &[&str] = &[
        "compound_assertion",
        "bool_naming",
        "acronym_style",
        "numeric_units",
        "unchecked_division",
        "raw_arithmetic_overflow",
        "unchecked_narrowing",
        "platform_dependent_cast",
        "unjustified_allow",
        // Phase 1: single-span name/type replacement
        "negated_predicate",
        "float_for_currency",
        "no_todo",
        "ambient_clock",
        // Phase 2: method-call / expression replacement
        "no_unwrap",
        "sentinel_fallback",
        "ignored_result",
        "contradictory_time",
        // Phase 3: constructor/call replacement
        "unbounded_channel",
        "explicit_defaults",
        "catch_all_on_enum",
        // Phase 4: structural / insertion
        "usize_in_public_api",
        "multi_lock_ordering",
        "compound_condition",
        // Phase 5: complex structural
        "unbounded_collection_growth",
    ];

    const MACHINE_APPLICABLE_AUTOFIX_LINTS: &[&str] = &["compound_assertion", "bool_naming", "negated_predicate"];

    const MACHINE_APPLICABLE_AUTOFIX_OUT_OF_SCOPE: &[AutofixOutOfScopeRationale] = &[];

    const AUTOFIX_OUT_OF_SCOPE_LINTS: &[&str] = &[
        "assertion_density",
        "drop_must_use",
        "fallible_int_fallback",
        "magic_number",
        "excessive_file_length",
        "module_file_count",
        "underscore_in_module_filename",
        "non_trait_imports",
        "renamed_imports",
        "path_segment_repetition",
        "function_length",
        "ambiguous_params",
        "no_recursion",
        "nested_conditionals",
        "no_panic",
        "too_many_parameters",
        "fat_trait",
        "concrete_construction_in_core",
        "type_accumulation",
        "unbounded_loop",
        "unjustified_no_todo_allow",
        "public_unsafe_api",
        "defaulting_trait_method",
        "foreign_enum_catch_all",
        "fragile_exhaustive_enum_match",
        "state_machine_exhaustiveness",
        "verified_purity",
    ];

    /// Lints that share a source file with a rollout lint and therefore
    /// cannot satisfy the "source must not contain `emit_tiger_style_lint_with_suggestion`"
    /// assertion for out-of-scope lints.
    const SHARED_SOURCE_EXEMPTIONS: &[&str] = &["unjustified_no_todo_allow"];

    #[derive(Debug)]
    struct AutofixOutOfScopeRationale {
        lint_name: &'static str,
        rationale: &'static str,
    }

    fn expected_autofix_tier(lint_name: &str) -> &'static str {
        match lint_name {
            "compound_assertion" | "bool_naming" => "MachineApplicable",
            "acronym_style"
            | "unchecked_division"
            | "raw_arithmetic_overflow"
            | "unchecked_narrowing"
            | "platform_dependent_cast"
            | "no_unwrap"
            | "sentinel_fallback"
            | "ignored_result"
            | "contradictory_time"
            | "usize_in_public_api"
            | "borrowed_argument_types"
            | "clone_then_borrow" => "MaybeIncorrect",
            "numeric_units"
            | "unjustified_allow"
            | "negated_predicate"
            | "float_for_currency"
            | "no_todo"
            | "ambient_clock"
            | "unbounded_channel"
            | "explicit_defaults"
            | "catch_all_on_enum"
            | "multi_lock_ordering"
            | "compound_condition"
            | "unbounded_collection_growth" => "HasPlaceholders",
            other => panic!("missing autofix tier for `{other}`"),
        }
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

    fn autofix_machine_proof_diagnostics(
        machine_lints: &[&str],
        fixed_fixtures: &[String],
        out_of_scope: &[AutofixOutOfScopeRationale],
    ) -> Vec<String> {
        let mut diagnostics = Vec::new();
        for &lint_name in machine_lints {
            if let Some(rationale) = out_of_scope.iter().find(|entry| entry.lint_name == lint_name) {
                if rationale.rationale.trim().is_empty() {
                    diagnostics.push(format!(
                        "machine-applicable lint `{lint_name}` has a blank autofix out-of-scope rationale"
                    ));
                }
                continue;
            }

            if !fixed_fixtures.iter().any(|fixture| fixture.contains(lint_name)) {
                diagnostics.push(format!(
                    "machine-applicable lint `{lint_name}` has no checked `.fixed` artifact or out-of-scope rationale"
                ));
            }
        }

        for rationale in out_of_scope {
            if !machine_lints.contains(&rationale.lint_name) {
                diagnostics.push(format!(
                    "autofix out-of-scope rationale references non-machine lint `{}`",
                    rationale.lint_name
                ));
            }
        }

        diagnostics
    }

    fn lint_source_path(lint_name: &str) -> &'static str {
        match lint_name {
            "compound_assertion" => "src/safety/compound_assertion.rs",
            "function_length" => "src/structure/function_length.rs",
            "acronym_style" => "src/naming/acronym_style.rs",
            "numeric_units" => "src/naming/numeric_units.rs",
            "excessive_file_length" => "src/structure/excessive_file_length.rs",
            "module_file_count" => "src/structure/module_file_count.rs",
            "underscore_in_module_filename" => "src/naming/underscore_in_module_filename.rs",
            "non_trait_imports" => "src/naming/non_trait_imports.rs",
            "renamed_imports" => "src/naming/renamed_imports.rs",
            "path_segment_repetition" => "src/naming/path_segment_repetition.rs",
            "unbounded_loop" => "src/safety/unbounded_loop.rs",
            "compound_condition" => "src/structure/compound_condition.rs",
            "verified_purity" => "src/purity/verified_purity.rs",
            "unjustified_allow" => "src/structure/unjustified_allow.rs",
            "nested_conditionals" => "src/structure/nested_conditionals.rs",
            "assertion_density" => "src/safety/assertion_density.rs",
            "multi_lock_ordering" => "src/concurrency/multi_lock_ordering.rs",
            "explicit_defaults" => "src/safety/explicit_defaults.rs",
            "unbounded_channel" => "src/safety/unbounded_channel.rs",
            "unbounded_collection_growth" => "src/safety/unbounded_collection_growth.rs",
            "sentinel_fallback" => "src/safety/sentinel_fallback.rs",
            "magic_number" => "src/structure/magic_number.rs",
            "drop_must_use" => "src/safety/drop_must_use.rs",
            "fallible_int_fallback" => "src/safety/fallible_int_fallback.rs",
            "usize_in_public_api" => "src/structure/usize_in_public_api.rs",
            "borrowed_argument_types" => "src/structure/borrowed_argument_types.rs",
            "clone_then_borrow" => "src/performance/clone_then_borrow.rs",
            "too_many_parameters" => "src/structure/too_many_parameters.rs",
            "fat_trait" => "src/structure/fat_trait.rs",
            "concrete_construction_in_core" => "src/structure/concrete_construction_in_core.rs",
            "type_accumulation" => "src/structure/type_accumulation.rs",
            "negated_predicate" => "src/naming/negated_predicate.rs",
            "bool_naming" => "src/naming/bool_naming.rs",
            "ignored_result" => "src/safety/ignored_result.rs",
            "unchecked_narrowing" => "src/safety/unchecked_narrowing.rs",
            "unchecked_division" => "src/safety/unchecked_division.rs",
            "contradictory_time" => "src/purity/contradictory_time.rs",
            "ambient_clock" => "src/purity/ambient_clock.rs",
            "catch_all_on_enum" => "src/safety/catch_all_on_enum.rs",
            "foreign_enum_catch_all" => "src/safety/foreign_enum_catch_all.rs",
            "fragile_exhaustive_enum_match" => "src/safety/fragile_exhaustive_enum_match.rs",
            "state_machine_exhaustiveness" => "src/safety/state_machine_exhaustiveness.rs",
            "platform_dependent_cast" => "src/structure/platform_dependent_cast.rs",
            "float_for_currency" => "src/naming/float_for_currency.rs",
            "ambiguous_params" => "src/structure/ambiguous_params.rs",
            "raw_arithmetic_overflow" => "src/safety/raw_arithmetic_overflow.rs",
            "no_unwrap" => "src/safety/no_unwrap.rs",
            "no_panic" => "src/safety/no_panic.rs",
            "public_unsafe_api" => "src/safety/public_unsafe_api.rs",
            "defaulting_trait_method" => "src/safety/defaulting_trait_method.rs",
            "no_todo" | "unjustified_no_todo_allow" => "src/safety/no_todo.rs",
            "no_recursion" => "src/safety/no_recursion.rs",
            other => panic!("missing lint source mapping for `{other}`"),
        }
    }

    #[test]
    fn autofix_machine_proof_covers_current_machine_lints() {
        let fixed_fixtures = fixed_fixture_names_for_library_tests();
        let diagnostics = autofix_machine_proof_diagnostics(
            MACHINE_APPLICABLE_AUTOFIX_LINTS,
            &fixed_fixtures,
            MACHINE_APPLICABLE_AUTOFIX_OUT_OF_SCOPE,
        );

        assert!(diagnostics.is_empty(), "autofix proof drifted: {diagnostics:#?}");
    }

    #[test]
    fn autofix_machine_proof_rejects_missing_fixed_artifact() {
        let fixed_fixtures = vec!["compound_assertion_machine_applicable".to_owned()];
        let diagnostics = autofix_machine_proof_diagnostics(
            &["compound_assertion", "bool_naming"],
            &fixed_fixtures,
            MACHINE_APPLICABLE_AUTOFIX_OUT_OF_SCOPE,
        );

        assert!(diagnostics.iter().any(|diagnostic| diagnostic.contains("`bool_naming`")));
    }

    #[test]
    fn indexable_policy_lints_have_registration_metadata_aliases_groups_and_sources() {
        let cfg = Config::default();
        let store = lint_store_with_groups_and_aliases(&cfg);

        for &lint_name in INDEXABLE_POLICY_LINTS {
            assert!(
                inventory_names().contains(&lint_name),
                "indexable lint `{lint_name}` must be registered in LINT_INVENTORY"
            );
            let meta = diagnostics::diagnostic_meta(lint_name);
            assert_eq!(meta.lint_name, lint_name);
            assert!(
                diagnostics::GROUP_NAMES.contains(&meta.domain_group),
                "indexable lint `{lint_name}` must belong to a registered domain group"
            );
            assert!(
                diagnostics::lint_names_for_group(meta.domain_group).contains(&lint_name),
                "indexable lint `{lint_name}` must appear in its domain group snapshot"
            );
            assert!(
                store.find_lints(&diagnostics::canonical_lint_name(lint_name)).is_some(),
                "indexable lint `{lint_name}` must resolve through its canonical tool lint name"
            );
            assert!(
                store.find_lints(lint_name).is_some(),
                "indexable lint `{lint_name}` must resolve through its bare compatibility alias"
            );
            let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(lint_source_path(lint_name));
            assert!(source_path.exists(), "indexable lint `{lint_name}` source path must exist");
        }
    }

    fn fixed_fixture_names_for_library_tests() -> Vec<String> {
        let ui_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("ui_tests");
        let mut fixed_fixtures = fs::read_dir(&ui_dir)
            .unwrap_or_else(|error| panic!("read autofix fixture dir {}: {error}", ui_dir.display()))
            .collect::<Result<Vec<_>, _>>()
            .unwrap_or_else(|error| panic!("collect autofix fixture dir {}: {error}", ui_dir.display()))
            .into_iter()
            .filter_map(|entry| {
                let path = entry.path();
                (path.extension().and_then(|ext| ext.to_str()) == Some("fixed"))
                    .then(|| path.file_stem().expect("fixed fixture file stem").to_string_lossy().into_owned())
            })
            .collect::<Vec<_>>();
        fixed_fixtures.sort();
        fixed_fixtures
    }

    #[test]
    fn autofix_rollout_scope_matches_change_contract() {
        let coverage = lint_fixture_coverage();
        let fixed_fixtures = fixed_fixture_names_for_library_tests();
        let proof_diagnostics = autofix_machine_proof_diagnostics(
            MACHINE_APPLICABLE_AUTOFIX_LINTS,
            &fixed_fixtures,
            MACHINE_APPLICABLE_AUTOFIX_OUT_OF_SCOPE,
        );
        assert!(proof_diagnostics.is_empty(), "autofix proof drifted: {proof_diagnostics:#?}");

        for &lint_name in AUTOFIX_ROLLOUT_LINTS {
            let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(lint_source_path(lint_name));
            let source = fs::read_to_string(&source_path)
                .unwrap_or_else(|error| panic!("read rollout lint source {}: {error}", source_path.display()));
            assert!(
                source.contains("emit_tiger_style_lint_with_suggestion"),
                "rollout lint `{lint_name}` must route through structured suggestion emission in {}",
                source_path.display()
            );
        }

        for &lint_name in AUTOFIX_OUT_OF_SCOPE_LINTS {
            let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(lint_source_path(lint_name));
            let source = fs::read_to_string(&source_path)
                .unwrap_or_else(|error| panic!("read out-of-scope lint source {}: {error}", source_path.display()));
            // Skip source-file check for lints that share a file with a rollout lint
            if !SHARED_SOURCE_EXEMPTIONS.contains(&lint_name) {
                assert!(
                    !source.contains("emit_tiger_style_lint_with_suggestion"),
                    "out-of-scope lint `{lint_name}` must stay diagnostic-only in {}",
                    source_path.display()
                );
            }
            assert!(
                fixed_fixtures.iter().all(|fixture| !fixture.contains(lint_name)),
                "out-of-scope lint `{lint_name}` must stay out of `.fixed` autofix artifacts"
            );
        }

        for entry in coverage {
            if !AUTOFIX_ROLLOUT_LINTS.contains(&entry.lint_name.as_str()) {
                continue;
            }
            let stderr_path = fixture_stderr_path(&entry.fixture);
            let stderr = fs::read_to_string(&stderr_path)
                .unwrap_or_else(|error| panic!("read fixture stderr {}: {error}", stderr_path.display()));
            let expected_tier = expected_autofix_tier(&entry.lint_name);
            assert!(
                stderr.contains(&format!("structured suggestion applicability: {expected_tier}")),
                "rollout lint `{}` must show tier `{expected_tier}` in {}",
                entry.lint_name,
                stderr_path.display()
            );
        }
    }

    #[test]
    fn lint_modules_route_help_and_note_through_diagnostics_module() {
        let lint_dirs = [
            "src/concurrency",
            "src/naming",
            "src/performance",
            "src/purity",
            "src/safety",
            "src/structure",
        ];
        let forbidden = [
            "Tiger Style (",
            "see Tiger Style guide:",
            ".help(",
            ".note(",
            ".span_suggestion(",
            ".span_suggestion_verbose(",
            ".multipart_suggestion(",
            "span_help(",
            "span_note(",
        ];

        for lint_dir in lint_dirs {
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join(lint_dir);
            for entry in fs::read_dir(&dir).unwrap_or_else(|error| panic!("read lint dir {}: {error}", dir.display())) {
                let path = entry.expect("lint dir entry").path();
                if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
                    continue;
                }

                let source = fs::read_to_string(&path)
                    .unwrap_or_else(|error| panic!("read lint source {}: {error}", path.display()));
                for needle in forbidden {
                    assert!(
                        !source.contains(needle),
                        "{} must route help/note text through src/diagnostics.rs; found `{needle}`",
                        path.display()
                    );
                }
            }
        }
    }
}

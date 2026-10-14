#!/bin/sh
set -eu

# Reusable pre-commit entry point for strict Octet/Tiger Style adoption.
# It runs from the consumer repository root, derives the pinned hook checkout,
# and asks the Dylint driver to treat every Octet lint as deny.

script_path=$0
case "$script_path" in
    */*)
        script_dir=$(CDPATH= cd "$(dirname "$script_path")" && pwd -P)
        ;;
    *)
        script_dir=$(pwd -P)
        ;;
esac

hook_repo=$(CDPATH= cd "$script_dir/.." && pwd -P)
flake_ref=${OCTET_PRECOMMIT_FLAKE:-path:$hook_repo}

deny_lint_flags='
-D unknown_lints
-D compound_assertion
-D function_length
-D acronym_style
-D numeric_units
-D magic_number
-D excessive_file_length
-D module_file_count
-D underscore_in_module_filename
-D non_trait_imports
-D renamed_imports
-D path_segment_repetition
-D unbounded_loop
-D compound_condition
-D verified_purity
-D unjustified_allow
-D nested_conditionals
-D assertion_density
-D multi_lock_ordering
-D ignored_join_handle
-D explicit_defaults
-D unbounded_channel
-D unbounded_collection_growth
-D sentinel_fallback
-D usize_in_public_api
-D borrowed_argument_types
-D deref_polymorphism
-D clone_then_borrow
-D allocation_in_loop
-D too_many_parameters
-D fat_trait
-D concrete_construction_in_core
-D type_accumulation
-D negated_predicate
-D bool_naming
-D ignored_result
-D drop_must_use
-D fallible_int_fallback
-D unchecked_narrowing
-D unchecked_division
-D contradictory_time
-D ambient_clock
-D ambient_env
-D ambient_random
-D global_state_in_core
-D impure_call_in_core
-D interior_mutability_in_core
-D logging_in_core
-D missing_const_fn
-D mutating_input_in_pure
-D nondeterministic_output
-D pure_attr_contract
-D zero_alloc_attr_contract
-D resource_return_from_core
-D spawn_in_core
-D catch_all_on_enum
-D foreign_enum_catch_all
-D fragile_exhaustive_enum_match
-D platform_dependent_cast
-D float_for_currency
-D ambiguous_params
-D raw_arithmetic_overflow
-D no_unwrap
-D no_panic
-D public_unsafe_api
-D defaulting_trait_method
-D no_todo
-D unjustified_no_todo_allow
-D no_recursion
'
deny_lint_flags=$(printf '%s\n' "$deny_lint_flags" | tr '\n' ' ')

if [ -n "${DYLINT_RUSTFLAGS:-}" ]; then
    DYLINT_RUSTFLAGS="$DYLINT_RUSTFLAGS $deny_lint_flags"
else
    DYLINT_RUSTFLAGS=$deny_lint_flags
fi
export DYLINT_RUSTFLAGS

if [ "${OCTET_PRECOMMIT_USE_INSTALLED:-false}" = "true" ]; then
    exec cargo octet check "$@"
fi

if command -v nix >/dev/null 2>&1; then
    exec nix run "${flake_ref}#cargo-octet" -- check "$@"
fi

exec cargo octet check "$@"

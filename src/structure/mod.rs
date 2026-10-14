pub mod ambiguous_params;
pub mod borrowed_argument_types;
pub mod compound_condition;
pub mod concrete_construction_in_core;
pub mod deref_polymorphism;
pub mod excessive_file_length;
pub mod fat_trait;
pub mod function_length;
pub mod magic_number;
pub mod module_file_count;
pub mod nested_conditionals;
pub mod newtype_invariant_bypass;
pub mod platform_dependent_cast;
pub mod primitive_domain_alias;
pub mod raw_domain_value;
pub mod too_many_parameters;
pub mod type_accumulation;
pub mod unjustified_allow;
pub mod usize_in_public_api;

#[cfg(test)]
mod raw_value_tests {
    use super::raw_domain_value::ContainerKind;
    use super::raw_domain_value::container_kind;
    use super::raw_domain_value::symbol_candidates;

    #[test]
    fn container_paths_have_explicit_classification() {
        assert_eq!(container_kind("alloc::vec::Vec"), ContainerKind::Sequence);
        assert_eq!(container_kind("core::option::Option"), ContainerKind::Optional);
        assert_eq!(container_kind("core::result::Result"), ContainerKind::Result);
        assert_eq!(container_kind("alloc::collections::btree::map::BTreeMap"), ContainerKind::Map);
        assert_eq!(container_kind("fixture::DomainId"), ContainerKind::Other);
    }

    #[test]
    fn symbol_candidates_prefer_qualified_domain_names() {
        let candidates = symbol_candidates("crate_name::module::DynamicPlaceholder::UnitOutput::unit");

        assert_eq!(candidates[0], "module::DynamicPlaceholder::UnitOutput::unit");
        assert_eq!(candidates[1], "DynamicPlaceholder::UnitOutput::unit");
        assert_eq!(candidates.last().map(String::as_str), Some("unit"));
    }
}

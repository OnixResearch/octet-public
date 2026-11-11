//! Exact, effect-free selection of a planned compiler unit.

use std::collections::BTreeSet;

use crate::CollectorCompilationUnit;

const ADJACENT_FEATURE_COUNT: usize = 2;

/// Compiler facts supplied by the adapter, with workspace-relative paths.
pub struct CompilerUnitObservation<'a> {
    pub crate_name: &'a str,
    pub manifest_dir: &'a str,
    pub source_path: &'a str,
    pub test_configuration: bool,
    pub target_triple: &'a str,
    pub features: &'a [String],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitSelectionError {
    InvalidContext,
    AmbiguousUnit,
    UnexpectedUnit,
}

impl std::fmt::Display for UnitSelectionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidContext => "invalid compiler architecture unit context",
            Self::AmbiguousUnit => "ambiguous compiler architecture unit observation key",
            Self::UnexpectedUnit => "unexpected compiler configuration for a selected architecture package",
        })
    }
}

impl std::error::Error for UnitSelectionError {}

/// Reject plans that cannot distinguish their compiler sessions.
///
/// # Errors
///
/// Returns an error when the plan is empty, malformed, or contains indistinguishable units.
pub fn validate_collector_units(units: &[CollectorCompilationUnit]) -> Result<(), UnitSelectionError> {
    if units.is_empty() {
        return Err(UnitSelectionError::InvalidContext);
    }
    let mut keys = BTreeSet::new();
    for selected in units {
        if selected.rustc_crate_name.is_empty()
            || selected.source_path != selected.unit.source_path
            || selected.unit.target_kind.ends_with("+test") != selected.test_configuration
            || crate::roles::root_role(&selected.source_path, &selected.source_roles) != Ok(selected.unit.role)
            || (selected.unit.role == crate::CompilerArchitectureRole::Test && selected.unit.target_kind != "test+test")
            || selected.unit.target_triple.is_empty()
            || selected.unit.features.windows(ADJACENT_FEATURE_COUNT).any(|pair| pair[0] >= pair[1])
        {
            return Err(UnitSelectionError::InvalidContext);
        }
        let key = (
            &selected.manifest_dir,
            &selected.rustc_crate_name,
            &selected.source_path,
            selected.test_configuration,
            &selected.unit.target_triple,
            &selected.unit.features,
        );
        if !keys.insert(key) {
            return Err(UnitSelectionError::AmbiguousUnit);
        }
    }
    Ok(())
}

/// External packages are unselected. Unknown sessions inside selected packages are errors.
///
/// # Errors
///
/// Returns an error when the planned units are invalid, several planned units match the
/// observation, or a session inside a selected package matches no planned unit.
pub fn select_compilation_unit<'a>(
    units: &'a [CollectorCompilationUnit],
    observation: &CompilerUnitObservation<'_>,
) -> Result<Option<&'a CollectorCompilationUnit>, UnitSelectionError> {
    validate_collector_units(units)?;
    let mut package_selected = false;
    let mut matched = None;
    for selected in units {
        if selected.manifest_dir != observation.manifest_dir {
            continue;
        }
        package_selected = true;
        if selected.rustc_crate_name != observation.crate_name
            || selected.source_path != observation.source_path
            || selected.test_configuration != observation.test_configuration
            || selected.unit.target_triple != observation.target_triple
            || selected.unit.features != observation.features
        {
            continue;
        }
        if matched.is_some() {
            return Err(UnitSelectionError::AmbiguousUnit);
        }
        matched = Some(selected);
    }
    if package_selected && matched.is_none() {
        return Err(UnitSelectionError::UnexpectedUnit);
    }
    Ok(matched)
}

#[cfg(test)]
mod tests;

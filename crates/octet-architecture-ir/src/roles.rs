//! Pure role selection from explicit policy and a Cargo-selected target root.

use crate::CompilerArchitectureRole;
use crate::SourceRoleScope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum UnitRoleError {
    InvalidSourceRoot,
    InvalidPackageRole,
    MissingSourceRole,
    ConflictingSourceRole,
}

impl std::fmt::Display for UnitRoleError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSourceRoot => "compiler source root or scope is not workspace-relative and canonical",
            Self::InvalidPackageRole => "selected compiler package must have a production role",
            Self::MissingSourceRole => "Cargo target root has no explicit compiler source role",
            Self::ConflictingSourceRole => "Cargo target root conflicts with the declared compiler role",
        })
    }
}

impl std::error::Error for UnitRoleError {}

/// Only an independent Cargo integration-test target can select a different test role.
/// Names, path spelling, and library/binary test configurations do not grant that role.
///
/// # Errors
///
/// Returns an error when the package role is reserved, the source root or a role scope is invalid,
/// no scope covers the source root, covering scopes disagree, or the source role conflicts with
/// the package role.
// r[impl octet.project_architecture_gate.test_ownership]
pub fn select_compilation_unit_role(
    package_role: CompilerArchitectureRole,
    target_kind: &str,
    source_path: &str,
    source_roles: &[SourceRoleScope],
) -> Result<CompilerArchitectureRole, UnitRoleError> {
    if matches!(package_role, CompilerArchitectureRole::Test | CompilerArchitectureRole::Generated) {
        return Err(UnitRoleError::InvalidPackageRole);
    }
    let source_role = root_role(source_path, source_roles)?;
    if source_role == package_role || (source_role == CompilerArchitectureRole::Test && target_kind == "test+test") {
        Ok(source_role)
    } else {
        Err(UnitRoleError::ConflictingSourceRole)
    }
}

pub(crate) fn root_role(
    source_path: &str,
    source_roles: &[SourceRoleScope],
) -> Result<CompilerArchitectureRole, UnitRoleError> {
    if !crate::core::valid_workspace_path(source_path)
        || source_roles.iter().any(|scope| !crate::core::valid_workspace_path(&scope.source_scope))
    {
        return Err(UnitRoleError::InvalidSourceRoot);
    }
    let mut role = None;
    for scope in source_roles.iter().filter(|scope| {
        source_path == scope.source_scope
            || source_path.strip_prefix(&scope.source_scope).is_some_and(|suffix| suffix.starts_with('/'))
    }) {
        if role.is_some_and(|previous| previous != scope.role) {
            return Err(UnitRoleError::ConflictingSourceRole);
        }
        role = Some(scope.role);
    }
    role.ok_or(UnitRoleError::MissingSourceRole)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scopes(role: CompilerArchitectureRole) -> Vec<SourceRoleScope> {
        vec![SourceRoleScope {
            source_scope: "tests".to_owned(),
            role,
        }]
    }

    // r[verify octet.project_architecture_gate.test_ownership]
    #[test]
    fn only_exact_integration_target_and_explicit_scope_select_test() {
        let core = CompilerArchitectureRole::DomainCore;
        let test = CompilerArchitectureRole::Test;
        assert_eq!(select_compilation_unit_role(core, "test+test", "tests/check.rs", &scopes(test)), Ok(test));
        // Cargo also permits an explicitly selected integration root outside a tests directory.
        let custom = vec![SourceRoleScope {
            source_scope: "checks/contract.rs".to_owned(),
            role: test,
        }];
        assert_eq!(select_compilation_unit_role(core, "test+test", "checks/contract.rs", &custom), Ok(test));
        assert_eq!(select_compilation_unit_role(core, "test+test", "tests/check.rs", &scopes(core)), Ok(core));
        for kind in [
            "lib",
            "lib+test",
            "bin",
            "bin+test",
            "test",
            "example+test",
            "bench+test",
            "contest+test",
        ] {
            assert_eq!(
                select_compilation_unit_role(core, kind, "tests/check.rs", &scopes(test)),
                Err(UnitRoleError::ConflictingSourceRole),
                "{kind}"
            );
            assert_eq!(select_compilation_unit_role(core, kind, "tests/check.rs", &scopes(core)), Ok(core), "{kind}");
        }
    }

    // r[verify octet.project_architecture_gate.test_ownership]
    #[test]
    fn missing_conflicting_and_noncanonical_roots_reject() {
        let core = CompilerArchitectureRole::DomainCore;
        let test = CompilerArchitectureRole::Test;
        for path in [
            "",
            "/tests/check.rs",
            "../tests/check.rs",
            "tests/../src/lib.rs",
            "tests//check.rs",
            "./tests/check.rs",
        ] {
            assert_eq!(
                select_compilation_unit_role(core, "test+test", path, &scopes(test)),
                Err(UnitRoleError::InvalidSourceRoot),
                "{path}"
            );
        }
        for path in ["src/lib.rs", "tests-other/check.rs"] {
            assert_eq!(
                select_compilation_unit_role(core, "test+test", path, &scopes(test)),
                Err(UnitRoleError::MissingSourceRole)
            );
        }
        let mut conflicting = scopes(test);
        conflicting.extend(scopes(core));
        for _ in 0..2 {
            assert_eq!(
                select_compilation_unit_role(core, "test+test", "tests/check.rs", &conflicting),
                Err(UnitRoleError::ConflictingSourceRole)
            );
            conflicting.reverse();
        }
        for role in [test, CompilerArchitectureRole::Generated] {
            assert_eq!(
                select_compilation_unit_role(role, "test+test", "tests/check.rs", &scopes(test)),
                Err(UnitRoleError::InvalidPackageRole)
            );
            assert_eq!(
                select_compilation_unit_role(core, "lib", "tests/check.rs", &scopes(role)),
                Err(UnitRoleError::ConflictingSourceRole)
            );
        }
    }
}

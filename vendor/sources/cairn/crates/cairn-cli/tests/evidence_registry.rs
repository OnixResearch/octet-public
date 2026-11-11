// r[impl cli_maintainability.evidence_registry_tests.focused_modules]
// r[impl cli_maintainability.evidence_registry_tests.focused_modules.visible_boundaries]
// r[impl cli_maintainability.evidence_registry_tests.focused_modules.narrow_bridges]
// r[impl cli_maintainability.evidence_registry_tests.behavior_preserved]
// r[verify cli_maintainability.evidence_registry_tests.behavior_preserved.focused_checks]
// r[verify cli_maintainability.evidence_registry_tests.behavior_preserved.repository_checks]
#[path = "evidence_registry/mod.rs"]
mod evidence_registry;

#[test]
fn evidence_export_materializes_complete_portable_chain() -> Result<(), Box<dyn std::error::Error>>
{
    let chain = evidence_registry::export_chain::materialize()?;
    let registry_chain = evidence_registry::registry_chain::verify(&chain)?;
    evidence_registry::fetch_execute::verify(&chain, &registry_chain)?;
    evidence_registry::manifest_binding::verify(&chain, &registry_chain)?;
    evidence_registry::failure_cases::verify(&chain, &registry_chain)?;
    Ok(())
}

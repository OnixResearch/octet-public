// r[impl cli_maintainability.lifecycle_tests.focused_modules]
// r[impl cli_maintainability.lifecycle_tests.focused_modules.visible_boundaries]
// r[verify cli_maintainability.lifecycle_tests.behavior_preserved]
// r[verify cli_maintainability.lifecycle_tests.behavior_preserved.focused_checks]
// r[verify cli_maintainability.lifecycle_tests.behavior_preserved.repository_checks]
// r[impl cli_maintainability.lifecycle_tests_followup.focused_modules]
// r[impl cli_maintainability.lifecycle_tests_followup.focused_modules.visible_boundaries]
// r[impl cli_maintainability.lifecycle_tests_followup.focused_modules.narrow_bridges]
// r[verify cli_maintainability.lifecycle_tests_followup.behavior_preserved]
// r[verify cli_maintainability.lifecycle_tests_followup.behavior_preserved.focused_checks]
// r[verify cli_maintainability.lifecycle_tests_followup.behavior_preserved.repository_checks]
#[path = "lifecycle/support.rs"]
mod support;

#[path = "lifecycle/change_authoring.rs"]
mod change_authoring;
#[path = "lifecycle/gate_cli.rs"]
mod gate_cli;
#[path = "lifecycle/migration_compat.rs"]
mod migration_compat;
#[path = "lifecycle/probe_execution.rs"]
mod probe_execution;
#[path = "lifecycle/read_validation.rs"]
mod read_validation;
#[path = "lifecycle/structured_lifecycle.rs"]
mod structured_lifecycle;
#[path = "lifecycle/workflow_profiles.rs"]
mod workflow_profiles;

// r[impl cli_maintainability.determinism_tests_followup.focused_modules]
// r[impl cli_maintainability.determinism_tests_followup.focused_modules.visible_boundaries]
// r[impl cli_maintainability.determinism_tests_followup.focused_modules.narrow_bridges]
// r[verify cli_maintainability.determinism_tests_followup.behavior_preserved]
// r[verify cli_maintainability.determinism_replay_submodules.behavior_preserved]
// r[verify cli_maintainability.determinism_replay_submodules.behavior_preserved.focused_checks]
// r[verify cli_maintainability.determinism_replay_submodules.behavior_preserved.repository_checks]
#[path = "determinism/coverage_audit.rs"]
mod coverage_audit;
#[path = "determinism/failures.rs"]
mod failures;
#[path = "determinism/replay.rs"]
mod replay;
#[path = "determinism/support.rs"]
mod support;
#[path = "determinism/verify_receipt.rs"]
mod verify_receipt;

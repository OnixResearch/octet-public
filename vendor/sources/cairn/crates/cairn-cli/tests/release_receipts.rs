// r[impl cli_maintainability.receipt_release_tests.focused_modules]
// r[impl cli_maintainability.receipt_release_tests.focused_modules.visible_boundaries]
// r[impl cli_maintainability.receipt_release_tests.focused_modules.narrow_bridges]
// r[verify cli_maintainability.receipt_release_tests.behavior_preserved]
// r[verify cli_maintainability.receipt_release_tests.behavior_preserved.focused_checks]
// r[verify cli_maintainability.receipt_release_tests.behavior_preserved.repository_checks]
// r[impl cli_maintainability.release_receipts_submodules.focused_modules]
// r[impl cli_maintainability.release_receipts_submodules.focused_modules.visible_boundaries]
// r[impl cli_maintainability.release_receipts_submodules.focused_modules.narrow_bridges]
// r[verify cli_maintainability.release_receipts_submodules.behavior_preserved]
// r[verify cli_maintainability.release_receipts_submodules.behavior_preserved.focused_checks]
// r[verify cli_maintainability.release_receipts_submodules.behavior_preserved.repository_checks]

mod release_receipts {
    mod agent_replay;
    mod bundle;
    mod manifest;
    mod portable;
    mod release_readiness;
    mod schema;
    mod support;
}

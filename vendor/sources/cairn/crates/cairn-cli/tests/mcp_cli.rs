#[path = "mcp_cli/agent_receipts.rs"]
mod agent_receipts;
#[path = "mcp_cli/catalog.rs"]
mod catalog;
#[path = "mcp_cli/common.rs"]
mod common;
#[path = "mcp_cli/drain_plan.rs"]
mod drain_plan;
#[path = "mcp_cli/json_rpc.rs"]
mod json_rpc;
#[path = "mcp_cli/validation.rs"]
mod validation;

// r[impl cli_maintainability.mcp_cli_tests_followup.focused_modules]
// r[impl cli_maintainability.mcp_cli_tests_followup.focused_modules.visible_boundaries]
// r[impl cli_maintainability.mcp_cli_tests_followup.focused_modules.narrow_bridges]
// r[verify cli_maintainability.mcp_cli_tests_followup.behavior_preserved]
// r[verify cli_maintainability.mcp_cli_tests_followup.behavior_preserved.focused_checks]
// r[verify cli_maintainability.mcp_cli_tests_followup.behavior_preserved.repository_checks]
// r[impl cli_maintainability.cli_integration_tests.focused_modules]
// r[impl cli_maintainability.cli_integration_tests.focused_modules.visible_boundaries]
// r[impl cli_maintainability.cli_integration_tests.focused_modules.narrow_bridges]
// r[verify cli_maintainability.cli_integration_tests.behavior_preserved]
// r[verify cli_maintainability.cli_integration_tests.behavior_preserved.focused_checks]
// r[verify cli_maintainability.cli_integration_tests.behavior_preserved.repository_checks]

use super::*;

pub(super) fn mcp_tools(is_mutation_allowed: bool) -> Vec<Value> {
    let mut tools = mcp_read_and_plan_tools();
    if is_mutation_allowed {
        tools.extend(mcp_mutation_tools());
    }
    tools
}

fn mcp_read_and_plan_tools() -> Vec<Value> {
    let mut tools = mcp_read_tools();
    tools.extend(mcp_plan_tools());
    tools
}

fn mcp_read_tools() -> Vec<Value> {
    vec![
        mcp_tool(
            "cairn_status",
            true,
            false,
            "List Cairn lifecycle state for a bounded root",
        ),
        mcp_tool(
            "cairn_list_content",
            true,
            false,
            "List accepted specs, active changes, and archived changes",
        ),
        mcp_tool(
            "cairn_read_spec",
            true,
            false,
            "Read a bounded accepted spec by id",
        ),
        mcp_tool(
            "cairn_change_show",
            true,
            false,
            "Read bounded active change metadata by id",
        ),
        mcp_tool(
            "cairn_context",
            true,
            false,
            "Return bounded project facts and advisory guidance with explicit authority labels",
        ),
        mcp_tool(
            "cairn_workflow_status",
            true,
            false,
            "Return policy-owned artifact states and next actions for one change",
        ),
        mcp_tool(
            "cairn_workflow_instructions",
            true,
            false,
            "Return one bounded policy-owned artifact instruction envelope",
        ),
        mcp_tool(
            "cairn_doctor",
            true,
            false,
            "Run Cairn diagnostics and return a structured receipt",
        ),
        mcp_tool(
            "cairn_validate",
            true,
            false,
            "Run Cairn validation and return a structured receipt",
        ),
        mcp_tool(
            "cairn_gate",
            true,
            false,
            "Evaluate a Cairn gate for proposal, design, or tasks",
        ),
        mcp_tool(
            "cairn_gate_context",
            true,
            false,
            "Return bounded gate context for a change",
        ),
        mcp_tool(
            "cairn_change_graph",
            true,
            false,
            "Return dependency graph readiness for active changes",
        ),
        mcp_tool(
            "cairn_agent_receipt_validate",
            true,
            false,
            "Inspect a legacy agent-run receipt without granting completion",
        ),
        mcp_tool(
            "cairn_agent_completion_validate",
            true,
            false,
            "Validate repository-owned completion evidence without executing checks",
        ),
        mcp_tool(
            "cairn_drain_plan",
            true,
            false,
            "Plan a repository-wide active-change drain without mutation",
        ),
    ]
}

fn mcp_plan_tools() -> Vec<Value> {
    vec![
        mcp_tool("cairn_sync_plan", true, false, "Plan sync without mutation"),
        mcp_tool(
            "cairn_archive_plan",
            true,
            false,
            "Plan archive without mutation",
        ),
        mcp_tool(
            "cairn_migrate_plan",
            true,
            false,
            "Plan legacy migration without mutation",
        ),
    ]
}

fn mcp_mutation_tools() -> Vec<Value> {
    vec![
        mcp_tool(
            "cairn_init",
            false,
            true,
            "Initialize Cairn lifecycle files",
        ),
        mcp_tool(
            "cairn_change_create",
            false,
            true,
            "Create a Cairn change skeleton",
        ),
        mcp_tool(
            "cairn_sync_execute",
            false,
            true,
            "Execute a gated sync plan",
        ),
        mcp_tool(
            "cairn_archive_execute",
            false,
            true,
            "Execute a gated archive plan",
        ),
        mcp_tool(
            "cairn_migrate_execute",
            false,
            true,
            "Execute a legacy migration plan",
        ),
    ]
}

fn mcp_tool(name: &str, read_only: bool, destructive: bool, description: &str) -> Value {
    // r[impl cairn.supersede.mcp_server.strict_schemas.catalog]
    json!({"name": name, "description": description, "inputSchema": mcp_input_schema(name), "annotations": {"readOnlyHint": read_only, "destructiveHint": destructive}})
}
pub(super) fn mutation_denied(name: &str) -> Value {
    json!({"tool": name, "allowed": false, "error": "mutation capability required", "mutated": false})
}

pub(super) fn is_mutating_mcp_tool(name: &str) -> bool {
    matches!(
        name,
        "cairn_init"
            | "cairn_change_create"
            | "cairn_sync_execute"
            | "cairn_archive_execute"
            | "cairn_migrate_execute"
    )
}

fn mcp_input_schema(name: &str) -> Value {
    match name {
        "cairn_status" | "cairn_list_content" => mcp_object_schema(&[], &["root"]),
        "cairn_doctor" | "cairn_init" => mcp_object_schema(&[], &["root", "policy"]),
        "cairn_validate" | "cairn_drain_plan" => json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "root": {"type": "string", "description": mcp_argument_description("root")},
                "policy": {"type": "string", "description": mcp_argument_description("policy")},
                "cross_repo_evidence": {
                    "type": "array",
                    "items": {"type": "string"},
                    "description": "Explicit receipt files or evidence bundle directories"
                }
            }
        }),
        "cairn_change_graph" => mcp_object_schema(&[], &["root"]),
        "cairn_read_spec" => mcp_object_schema(&["spec"], &["root"]),
        "cairn_change_show" | "cairn_gate_context" => mcp_object_schema(&["change"], &["root"]),
        "cairn_change_create" => mcp_object_schema(&["change"], &["root", "policy", "profile"]),
        "cairn_context" => {
            mcp_object_schema(&[], &["root", "policy", "profile", "artifact", "operation"])
        }
        "cairn_workflow_status" => mcp_object_schema(&["change"], &["root", "policy", "schema"]),
        "cairn_workflow_instructions" => {
            mcp_object_schema(&["change", "artifact"], &["root", "policy", "schema"])
        }
        "cairn_gate" => mcp_schema_with_enum(
            &["change", "stage"],
            &["root", "policy"],
            "stage",
            &["proposal", "design", "tasks"],
        ),
        "cairn_sync_plan"
        | "cairn_archive_plan"
        | "cairn_sync_execute"
        | "cairn_archive_execute" => mcp_schema_with_evidence(&["change"], &["root", "policy"]),
        "cairn_migrate_plan" => mcp_schema_with_enum(
            &["source"],
            &["root", "policy"],
            "source",
            &["cairn", "openspec", "ironspec"],
        ),
        "cairn_migrate_execute" => mcp_schema_with_enum(
            &["source", "plan_hash"],
            &["root", "policy"],
            "source",
            &["cairn", "openspec", "ironspec"],
        ),
        "cairn_agent_receipt_validate" => json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "root": {"type": "string", "description": "Repository root for path input"},
                "path": {"type": "string", "description": "Repository-relative bounded receipt path"},
                "receipt": {"type": "object", "description": "Inline bounded agent-run receipt"}
            },
            "anyOf": [{"required": ["path"]}, {"required": ["receipt"]}]
        }),
        "cairn_agent_completion_validate" => json!({
            "type": "object",
            "additionalProperties": false,
            "properties": {
                "root": {"type": "string"},
                "policy": {"type": "string"},
                "contract": {"type": "object"},
                "results": {"type": "array"},
                "subject_hash": {"type": "string"}
            },
            "required": ["contract", "results", "subject_hash"]
        }),
        _ => json!({"type": "object", "additionalProperties": false, "properties": {}}),
    }
}

fn mcp_object_schema(required: &[&str], optional: &[&str]) -> Value {
    let mut properties = serde_json::Map::new();
    for key in required.iter().chain(optional.iter()) {
        properties.insert(
            (*key).to_string(),
            json!({"type": "string", "description": mcp_argument_description(key)}),
        );
    }
    json!({"type": "object", "additionalProperties": false, "properties": properties, "required": required})
}

fn mcp_schema_with_evidence(required: &[&str], optional: &[&str]) -> Value {
    let mut extended_optional = optional.to_vec();
    extended_optional.push("cross_repo_evidence");
    let mut schema = mcp_object_schema(required, &extended_optional);
    schema["properties"]["cross_repo_evidence"] = json!({
        "type": "array",
        "items": {"type": "string"},
        "description": "Explicit receipt files or evidence bundle directories"
    });
    schema
}

fn mcp_schema_with_enum(
    required: &[&str],
    optional: &[&str],
    enum_key: &str,
    allowed: &[&str],
) -> Value {
    let mut schema = mcp_object_schema(required, optional);
    schema["properties"][enum_key]["enum"] = json!(allowed);
    schema
}

fn mcp_argument_description(key: &str) -> &'static str {
    match key {
        "root" => "Repository root; defaults to the current directory",
        "policy" => "Policy JSON path; defaults to Cairn's checked-in policy",
        "spec" => "Accepted spec lifecycle identifier",
        "change" => "Active change lifecycle identifier",
        "stage" => "Gate stage: proposal, design, or tasks",
        "source" => "Legacy migration source: cairn, openspec, or ironspec",
        "plan_hash" => "BLAKE3 plan hash from a reviewed migration preview",
        "schema" => "Policy-owned workflow schema identifier",
        "artifact" => "Policy-owned workflow artifact identifier",
        _ => "String argument",
    }
}

pub(super) fn validate_mcp_tool_input(name: &str, input: &Value) -> Result<(), Box<dyn Error>> {
    let Some(object) = input.as_object() else {
        return Err(format!("invalid arguments for {name}: expected object").into());
    };
    match name {
        "cairn_status" | "cairn_list_content" => validate_string_args(name, object, &[], &["root"]),
        "cairn_doctor" | "cairn_init" => {
            validate_string_args(name, object, &[], &["root", "policy"])
        }
        "cairn_validate" | "cairn_drain_plan" => validate_mcp_validate_args(name, object),
        "cairn_change_graph" => validate_string_args(name, object, &[], &["root"]),
        "cairn_read_spec" => validate_string_args(name, object, &["spec"], &["root"]),
        "cairn_change_show" | "cairn_gate_context" => {
            validate_string_args(name, object, &["change"], &["root"])
        }
        "cairn_change_create" => {
            validate_string_args(name, object, &["change"], &["root", "policy", "profile"])
        }
        "cairn_context" => validate_string_args(
            name,
            object,
            &[],
            &["root", "policy", "profile", "artifact", "operation"],
        ),
        "cairn_workflow_status" => {
            validate_string_args(name, object, &["change"], &["root", "policy", "schema"])
        }
        "cairn_workflow_instructions" => validate_string_args(
            name,
            object,
            &["change", "artifact"],
            &["root", "policy", "schema"],
        ),
        "cairn_gate" => {
            validate_string_args(name, object, &["change", "stage"], &["root", "policy"])?;
            validate_enum_arg(name, object, "stage", &["proposal", "design", "tasks"])
        }
        "cairn_sync_plan"
        | "cairn_archive_plan"
        | "cairn_sync_execute"
        | "cairn_archive_execute" => validate_lifecycle_mutation_args(name, object),
        "cairn_migrate_plan" => {
            validate_string_args(name, object, &["source"], &["root", "policy"])?;
            validate_enum_arg(name, object, "source", &["cairn", "openspec", "ironspec"])
        }
        "cairn_migrate_execute" => {
            validate_string_args(name, object, &["source", "plan_hash"], &["root", "policy"])?;
            validate_enum_arg(name, object, "source", &["cairn", "openspec", "ironspec"])
        }
        "cairn_agent_receipt_validate" => validate_agent_receipt_args(name, object),
        "cairn_agent_completion_validate" => validate_agent_completion_args(name, object),
        unknown => Err(format!("unknown MCP tool: {unknown}").into()),
    }
}

fn validate_lifecycle_mutation_args(
    tool: &str,
    object: &serde_json::Map<String, Value>,
) -> Result<(), Box<dyn Error>> {
    for key in object.keys() {
        if !["change", "root", "policy", "cross_repo_evidence"].contains(&key.as_str()) {
            return Err(format!("invalid arguments for {tool}: unknown argument {key}").into());
        }
    }
    let Some(change) = object.get("change") else {
        return Err(
            format!("invalid arguments for {tool}: missing required argument change").into(),
        );
    };
    if !change.is_string() {
        return Err(format!("invalid arguments for {tool}: change must be a string").into());
    }
    for key in ["root", "policy"] {
        if object.get(key).is_some_and(|value| !value.is_string()) {
            return Err(format!("invalid arguments for {tool}: {key} must be a string").into());
        }
    }
    validate_evidence_array(tool, object)
}

fn validate_evidence_array(
    tool: &str,
    object: &serde_json::Map<String, Value>,
) -> Result<(), Box<dyn Error>> {
    if let Some(value) = object.get("cross_repo_evidence") {
        let Some(items) = value.as_array() else {
            return Err(format!(
                "invalid arguments for {tool}: cross_repo_evidence must be an array"
            )
            .into());
        };
        if items.iter().any(|item| !item.is_string()) {
            return Err(format!(
                "invalid arguments for {tool}: cross_repo_evidence entries must be strings"
            )
            .into());
        }
    }
    Ok(())
}

fn validate_mcp_validate_args(
    tool: &str,
    object: &serde_json::Map<String, Value>,
) -> Result<(), Box<dyn Error>> {
    for key in object.keys() {
        if !["root", "policy", "cross_repo_evidence"].contains(&key.as_str()) {
            return Err(format!("invalid arguments for {tool}: unknown argument {key}").into());
        }
    }
    for key in ["root", "policy"] {
        if object.get(key).is_some_and(|value| !value.is_string()) {
            return Err(format!("invalid arguments for {tool}: {key} must be a string").into());
        }
    }
    validate_evidence_array(tool, object)
}

fn validate_string_args(
    tool: &str,
    object: &serde_json::Map<String, Value>,
    required: &[&str],
    optional: &[&str],
) -> Result<(), Box<dyn Error>> {
    for key in object.keys() {
        if !required.contains(&key.as_str()) && !optional.contains(&key.as_str()) {
            return Err(format!("invalid arguments for {tool}: unknown argument {key}").into());
        }
    }
    for key in required {
        if !object.contains_key(*key) {
            return Err(
                format!("invalid arguments for {tool}: missing required argument {key}").into(),
            );
        }
    }
    for key in required.iter().chain(optional.iter()) {
        if let Some(value) = object.get(*key) {
            if !value.is_string() {
                return Err(format!(
                    "invalid arguments for {tool}: argument {key} must be a string"
                )
                .into());
            }
        }
    }
    Ok(())
}

fn validate_enum_arg(
    tool: &str,
    object: &serde_json::Map<String, Value>,
    key: &str,
    allowed: &[&str],
) -> Result<(), Box<dyn Error>> {
    let value = object.get(key).and_then(Value::as_str).unwrap_or_default();
    if allowed.contains(&value) {
        return Ok(());
    }
    Err(format!(
        "invalid arguments for {tool}: argument {key} must be one of {}",
        allowed.join(", ")
    )
    .into())
}

fn validate_agent_receipt_args(
    tool: &str,
    object: &serde_json::Map<String, Value>,
) -> Result<(), Box<dyn Error>> {
    for key in object.keys() {
        if !matches!(key.as_str(), "root" | "path" | "receipt") {
            return Err(format!("invalid arguments for {tool}: unknown argument {key}").into());
        }
    }
    let has_path = object.contains_key("path");
    let has_receipt = object.contains_key("receipt");
    if has_path == has_receipt {
        return Err(format!(
            "invalid arguments for {tool}: provide exactly one of path or receipt"
        )
        .into());
    }
    if let Some(root) = object.get("root") {
        if !root.is_string() {
            return Err(
                format!("invalid arguments for {tool}: argument root must be a string").into(),
            );
        }
    }
    if let Some(path) = object.get("path") {
        if !path.is_string() {
            return Err(
                format!("invalid arguments for {tool}: argument path must be a string").into(),
            );
        }
    }
    if let Some(receipt) = object.get("receipt") {
        if !receipt.is_object() {
            return Err(format!(
                "invalid arguments for {tool}: argument receipt must be an object"
            )
            .into());
        }
    }
    Ok(())
}

fn validate_agent_completion_args(
    tool: &str,
    object: &serde_json::Map<String, Value>,
) -> Result<(), Box<dyn Error>> {
    for key in object.keys() {
        if !matches!(
            key.as_str(),
            "root" | "policy" | "contract" | "results" | "subject_hash"
        ) {
            return Err(format!("invalid arguments for {tool}: unknown argument {key}").into());
        }
    }
    for key in ["root", "policy", "subject_hash"] {
        if object.get(key).is_some_and(|value| !value.is_string()) {
            return Err(
                format!("invalid arguments for {tool}: argument {key} must be a string").into(),
            );
        }
    }
    if !object.get("contract").is_some_and(Value::is_object)
        || !object.get("results").is_some_and(Value::is_array)
        || !object.get("subject_hash").is_some_and(Value::is_string)
    {
        return Err(format!(
            "invalid arguments for {tool}: contract object, results array, and subject_hash string are required"
        )
        .into());
    }
    Ok(())
}

pub(super) fn mcp_summary(name: &str, receipt: &Value) -> String {
    format!(
        "{name}: {}",
        serde_json::to_string(receipt).unwrap_or_else(|_| "{}".to_string())
    )
}

use super::schema::mcp_tools;
use super::*;

pub(super) fn run_mcp_json_rpc(is_mutation_allowed: bool) -> Result<(), Box<dyn Error>> {
    let stdin = io::stdin();
    let mut stdout = io::stdout();
    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let request: Value = serde_json::from_str(&line)?;
        let id = request.get("id").cloned().unwrap_or(Value::Null);
        let method = request["method"].as_str().unwrap_or_default();
        let response = match method {
            "initialize" => json!({
                "jsonrpc": "2.0", "id": id,
                "result": {"protocolVersion": MCP_PROTOCOL_VERSION, "serverInfo": {"name": "cairn", "version": env!("CARGO_PKG_VERSION")}, "capabilities": {"tools": {}}}
            }),
            "tools/list" => {
                json!({"jsonrpc": "2.0", "id": id, "result": {"tools": mcp_tools(is_mutation_allowed)}})
            }
            "tools/call" => {
                let params = &request["params"];
                let name = params["name"].as_str().unwrap_or_default();
                let arguments = params.get("arguments").unwrap_or(&Value::Null);
                match mcp_call_tool(name, arguments, is_mutation_allowed) {
                    Ok(result) => json!({"jsonrpc": "2.0", "id": id, "result": result}),
                    Err(error) => {
                        json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32000, "message": error.to_string()}})
                    }
                }
            }
            _ => {
                json!({"jsonrpc": "2.0", "id": id, "error": {"code": -32601, "message": format!("unknown method: {method}")}})
            }
        };
        writeln!(stdout, "{}", serde_json::to_string(&response)?)?;
        stdout.flush()?;
    }
    Ok(())
}

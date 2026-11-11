use std::{error::Error, io::Write};

use serde_json::Value;

use super::common::cairn;

#[test]
fn mcp_json_rpc_lists_and_calls_tools() -> Result<(), Box<dyn Error>> {
    // r[verify cairn.supersede.mcp_server.verification.registration]
    let mut child = cairn()
        .arg("mcp")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .spawn()?;
    {
        let stdin = child.stdin.as_mut().ok_or("stdin")?;
        writeln!(stdin, r#"{{"jsonrpc":"2.0","id":1,"method":"tools/list"}}"#)?;
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{{"name":"cairn_validate","arguments":{{"root":"fixtures/basic"}}}}}}"#
        )?;
        writeln!(
            stdin,
            r#"{{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{{"name":"cairn_gate","arguments":{{"root":"fixtures/basic","change":"demo","stage":"archive"}}}}}}"#
        )?;
    }
    let output = child.wait_with_output()?;
    assert!(output.status.success());
    let lines = String::from_utf8(output.stdout)?;
    let responses = lines
        .lines()
        .map(serde_json::from_str::<Value>)
        .collect::<Result<Vec<_>, _>>()?;
    assert_eq!(responses.len(), 3);
    assert!(responses[0]["result"]["tools"]
        .as_array()
        .ok_or("tools")?
        .iter()
        .any(|tool| tool["name"] == "cairn_validate"));
    assert_eq!(responses[1]["result"]["structuredContent"]["valid"], true);
    assert!(responses[2]["error"]["message"]
        .as_str()
        .unwrap_or_default()
        .contains("invalid arguments for cairn_gate"));
    Ok(())
}

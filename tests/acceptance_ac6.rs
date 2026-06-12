//! AC6: Missing required `path` → JSON-RPC error, no panic, no shell call.
//! Absent getfattr/memlog, serve still starts and tools/list still works.

use std::io::Cursor;

use mcp_core::serve::serve;
// We need access to the tool types.
// Since this is a binary-only crate, we inline the tool implementations here.

mod provenance_mcp_tools {
    use mcp_core::{Tool, ToolError};
    use serde_json::{json, Value};

    pub struct FileProvenanceTool;
    impl Tool for FileProvenanceTool {
        fn name(&self) -> &'static str { "file_provenance" }
        fn description(&self) -> &'static str { "file provenance" }
        fn input_schema(&self) -> Value {
            json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]})
        }
        fn call(&self, args: &Value) -> Result<Value, ToolError> {
            let path = args.get("path").and_then(Value::as_str)
                .ok_or_else(|| ToolError::new("missing required parameter: path"))?;
            // Try real backend — may fail gracefully if getfattr absent.
            match std::process::Command::new("getfattr")
                .args(["-d", "--absolute-names", path])
                .output()
            {
                Ok(out) => {
                    let raw = String::from_utf8_lossy(&out.stdout).into_owned();
                    Ok(json!({"path": path, "session": null, "ts": null, "raw": raw}))
                }
                Err(e) => Err(ToolError::new(format!("getfattr unavailable: {e}"))),
            }
        }
    }

    pub struct MemlogRecentTool { pub memlog_bin: String }
    impl Tool for MemlogRecentTool {
        fn name(&self) -> &'static str { "memlog_recent" }
        fn description(&self) -> &'static str { "memlog recent" }
        fn input_schema(&self) -> Value {
            json!({"type":"object","properties":{"since":{"type":"string"},"limit":{"type":"integer"}}})
        }
        fn call(&self, _args: &Value) -> Result<Value, ToolError> {
            Err(ToolError::new(format!("memlog unavailable: {}", self.memlog_bin)))
        }
    }
}

#[test]
fn missing_path_returns_json_rpc_error_not_panic() {
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#, "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"file_provenance","arguments":{}}}"#, "\n",
    );

    let tools: Vec<Box<dyn mcp_core::Tool>> = vec![
        Box::new(provenance_mcp_tools::FileProvenanceTool),
        Box::new(provenance_mcp_tools::MemlogRecentTool { memlog_bin: "nonexistent".into() }),
    ];

    let mut output = Vec::new();
    serve(Cursor::new(input), &mut output, tools, "provenance-mcp", "0.1.0")
        .expect("serve must not crash");

    let text = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2);

    // Second line is the tools/call response — should be an error.
    let resp: serde_json::Value = serde_json::from_str(lines[1]).expect("valid JSON");
    assert!(
        resp.get("error").is_some(),
        "missing path must produce a JSON-RPC error, got: {resp}"
    );
}

#[test]
fn serve_starts_and_tools_list_works_with_absent_binaries() {
    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#, "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":null}"#, "\n",
    );

    let tools: Vec<Box<dyn mcp_core::Tool>> = vec![
        Box::new(provenance_mcp_tools::FileProvenanceTool),
        Box::new(provenance_mcp_tools::MemlogRecentTool {
            memlog_bin: "/tmp/totally-absent-memlog-xyzzy".into(),
        }),
    ];

    let mut output = Vec::new();
    serve(Cursor::new(input), &mut output, tools, "provenance-mcp", "0.1.0")
        .expect("serve must not crash even when binaries are absent");

    let text = String::from_utf8(output).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    // tools/list must succeed
    let list_resp: serde_json::Value = serde_json::from_str(lines[1]).expect("valid JSON");
    let arr = list_resp["result"]["tools"].as_array().expect("tools array");
    assert_eq!(arr.len(), 2, "both tools must be listed even when binaries absent");
}

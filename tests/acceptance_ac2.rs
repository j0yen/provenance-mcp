//! AC2: initialize handshake + tools/list returns exactly two tools with valid inputSchema.

use std::io::Cursor;

use mcp_core::{Tool, ToolError};
use serde_json::{json, Value};

// Re-instantiate the tools directly (we can't drive the binary process easily
// in a unit-style integration test, so we replicate the serve() call).

struct FakeFileProvenance;
impl Tool for FakeFileProvenance {
    fn name(&self) -> &str { "file_provenance" }
    fn description(&self) -> &str { "file provenance" }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"path":{"type":"string"}},"required":["path"]})
    }
    fn call(&self, _: &Value) -> Result<Value, ToolError> { Ok(json!(null)) }
}

struct FakeMemlogRecent;
impl Tool for FakeMemlogRecent {
    fn name(&self) -> &str { "memlog_recent" }
    fn description(&self) -> &str { "memlog recent" }
    fn input_schema(&self) -> Value {
        json!({"type":"object","properties":{"since":{"type":"string"},"limit":{"type":"integer"}}})
    }
    fn call(&self, _: &Value) -> Result<Value, ToolError> { Ok(json!([])) }
}

#[test]
fn tools_list_returns_two_tools_with_valid_input_schema() {
    use mcp_core::serve::serve;

    let input = concat!(
        r#"{"jsonrpc":"2.0","id":1,"method":"initialize","params":{}}"#,
        "\n",
        r#"{"jsonrpc":"2.0","id":2,"method":"tools/list","params":null}"#,
        "\n",
    );

    let tools: Vec<Box<dyn Tool>> = vec![
        Box::new(FakeFileProvenance),
        Box::new(FakeMemlogRecent),
    ];

    let mut output = Vec::new();
    serve(Cursor::new(input), &mut output, tools, "provenance-mcp", "0.1.0")
        .expect("serve must not fail");

    let text = String::from_utf8(output).expect("utf8 output");
    let lines: Vec<&str> = text.lines().collect();
    assert_eq!(lines.len(), 2, "expected initialize + tools/list responses");

    // tools/list response
    let resp: Value = serde_json::from_str(lines[1]).expect("valid JSON");
    let tools_arr = resp["result"]["tools"].as_array().expect("tools array");
    assert_eq!(tools_arr.len(), 2, "must expose exactly 2 tools");

    let names: Vec<&str> = tools_arr
        .iter()
        .map(|t| t["name"].as_str().expect("name"))
        .collect();
    assert!(names.contains(&"file_provenance"), "missing file_provenance");
    assert!(names.contains(&"memlog_recent"), "missing memlog_recent");

    // Each tool must have inputSchema with type == object
    for tool in tools_arr {
        let schema = &tool["inputSchema"];
        assert_eq!(
            schema["type"], "object",
            "tool {} inputSchema.type must be 'object'",
            tool["name"]
        );
    }
}

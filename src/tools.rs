//! MCP `Tool` implementations for provenance-mcp.
//!
//! Two read-only tools:
//! - `file_provenance { path: string }` — query provfs xattrs for a file.
//! - `memlog_recent { since?: string, limit?: integer }` — recent context snapshots.

use mcp_core::{Tool, ToolError};
use serde_json::{json, Value};

use crate::backend;

// ---------------------------------------------------------------------------
// file_provenance
// ---------------------------------------------------------------------------

/// MCP tool: query provfs xattrs for a file path.
pub struct FileProvenanceTool;

impl Tool for FileProvenanceTool {
    fn name(&self) -> &'static str {
        "file_provenance"
    }

    fn description(&self) -> &'static str {
        "Query kernel-stamped provenance xattrs (user.prov.session, user.prov.ts) for a file. \
         Returns {path, session, ts, raw}. session and ts are null when the file has no \
         provenance xattrs (not an error). Requires getfattr on PATH and provfs kernel module; \
         returns a ToolError if getfattr is absent."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "description": "Absolute or relative path to the file to query."
                }
            },
            "required": ["path"],
            "additionalProperties": false
        })
    }

    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let path = args
            .get("path")
            .and_then(Value::as_str)
            .ok_or_else(|| ToolError::new("missing required parameter: path"))?;

        match backend::file_provenance(path) {
            Ok(prov) => serde_json::to_value(&prov)
                .map_err(|e| ToolError::new(format!("serialization error: {e}"))),
            Err(e) => Err(ToolError::new(format!(
                "getfattr unavailable or failed: {e}"
            ))),
        }
    }
}

// ---------------------------------------------------------------------------
// memlog_recent
// ---------------------------------------------------------------------------

/// MCP tool: retrieve recent memlog context snapshots.
pub struct MemlogRecentTool {
    /// Path to the memlog binary (default: "memlog" resolved on PATH).
    pub memlog_bin: String,
}

impl Tool for MemlogRecentTool {
    fn name(&self) -> &'static str {
        "memlog_recent"
    }

    fn description(&self) -> &'static str {
        "Retrieve recent pre-compaction context snapshots from memlog. \
         Calls `memlog show --since <DUR> --limit <N> --format json`. \
         Requires the memlog binary and the memlog kernel module; \
         returns a ToolError if the binary is absent."
    }

    fn input_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "since": {
                    "type": "string",
                    "description": "Duration string, e.g. '1h', '30m', '7d'. Optional."
                },
                "limit": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 1000,
                    "description": "Maximum number of snapshots to return. Optional."
                }
            },
            "additionalProperties": false
        })
    }

    fn call(&self, args: &Value) -> Result<Value, ToolError> {
        let since = args.get("since").and_then(Value::as_str);
        let limit = args
            .get("limit")
            .and_then(Value::as_u64)
            .map(|n| u32::try_from(n).unwrap_or(u32::MAX));

        match backend::memlog_recent(&self.memlog_bin, since, limit) {
            Ok(snaps) => serde_json::to_value(&snaps)
                .map_err(|e| ToolError::new(format!("serialization error: {e}"))),
            Err(e) => Err(ToolError::new(format!(
                "memlog unavailable or failed: {e}"
            ))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_provenance_schema_is_object_with_required_path() {
        let schema = FileProvenanceTool.input_schema();
        assert_eq!(schema["type"], "object");
        let required = schema["required"].as_array().expect("required array");
        assert!(required.iter().any(|v| v == "path"));
    }

    #[test]
    fn memlog_recent_schema_is_object() {
        let tool = MemlogRecentTool {
            memlog_bin: "memlog".into(),
        };
        let schema = tool.input_schema();
        assert_eq!(schema["type"], "object");
    }

    #[test]
    fn file_provenance_missing_path_returns_error() {
        let result = FileProvenanceTool.call(&json!({}));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.message.contains("path"),
            "error should mention 'path': {}", err.message
        );
    }

    #[test]
    fn file_provenance_missing_getfattr_returns_tool_error_not_panic() {
        // On a system where getfattr isn't installed, the tool should return
        // a ToolError, not panic.
        let result = FileProvenanceTool.call(&json!({"path": "/tmp/nonexistent-provenance-test"}));
        // Either Ok (getfattr present, file has no xattrs) or Err (getfattr absent).
        // The key invariant is: no panic.
        let _ = result;
    }

    #[test]
    fn memlog_recent_nonexistent_bin_returns_tool_error() {
        let tool = MemlogRecentTool {
            memlog_bin: "/tmp/nonexistent-memlog-binary-xyz".into(),
        };
        let result = tool.call(&json!({}));
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(!err.message.is_empty());
    }
}

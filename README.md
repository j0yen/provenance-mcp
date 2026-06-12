# provenance-mcp

An MCP (Model Context Protocol) server that exposes two **read-only** forensic
tools over stdio:

- **`file_provenance`** — queries kernel-stamped provfs xattrs (`user.prov.session`,
  `user.prov.ts`) for a file path.
- **`memlog_recent`** — retrieves recent pre-compaction context snapshots from the
  memlog circular log.

Part of the [wintermute](https://github.com/j0yen/wintermute) ecosystem /
[conduit vision](https://github.com/j0yen/wintermute/blob/master/visions/conduit.md).

---

## Tools

### `file_provenance`

Query kernel-stamped provenance for a file.

**Input schema:**
```json
{
  "type": "object",
  "properties": {
    "path": { "type": "string", "description": "Absolute or relative path to the file." }
  },
  "required": ["path"]
}
```

**Output:** `{ "path": "...", "session": "sess-abc123" | null, "ts": "1717200000" | null, "raw": "..." }`

- `session` and `ts` are `null` when the file has no provenance xattrs — this is
  normal for files outside the provfs-stamped scope (e.g. `/tmp`, `.git`, `target`).
- Returns a `ToolError` if `getfattr` is not installed.

**Graceful degradation:** If `getfattr` is absent or the provfs kernel module is not
loaded, the tool returns a clean `ToolError` with an explanation — `serve` still starts
and `tools/list` still works.

---

### `memlog_recent`

Retrieve recent pre-compaction context snapshots from memlog.

**Input schema:**
```json
{
  "type": "object",
  "properties": {
    "since": { "type": "string", "description": "Duration string, e.g. '1h', '30m', '7d'." },
    "limit": { "type": "integer", "minimum": 1, "maximum": 1000 }
  }
}
```

**Output:** JSON array of snapshot objects from `memlog show --format json`.

**Graceful degradation:** If the `memlog` binary is absent (i.e. the wintermute kernel
memlog module is not installed), the tool returns a clean `ToolError` — `serve` still
starts and all other tools remain functional.

---

## Read-only guarantee

`provenance-mcp` is read-only by construction:

- `file_provenance` runs **`getfattr`** (xattr reader). `setfattr` never appears in the
  codebase.
- `memlog_recent` runs **`memlog show`** only. The verbs `memlog write` and `memlog clear`
  never appear in the codebase.

This is enforced by a test (`acceptance_ac4.rs`) that scans `backend.rs` source at test
time and fails if any forbidden verb appears.

---

## Kernel dependencies

| Feature | Kernel module | Graceful when absent? |
|---------|--------------|----------------------|
| File provenance xattrs | provfs LSM (`user.prov.session`, `user.prov.ts`) | Yes — returns `session:null, ts:null` |
| Context snapshots | memlog circular log | Yes — returns ToolError |

If you are running a standard kernel (not wintermute), neither module is present.
Both tools will return informative errors rather than crashing the server.

---

## Usage

```bash
# Run the MCP server (reads from stdin, writes to stdout)
provenance-mcp serve

# Custom memlog binary path
provenance-mcp serve --memlog-bin /path/to/memlog
# or via env
MEMLOG_BIN=/path/to/memlog provenance-mcp serve
```

---

## MCP client configuration

Add to your Claude Code or other MCP client settings:

```json
{
  "mcpServers": {
    "provenance-mcp": {
      "command": "provenance-mcp",
      "args": ["serve"]
    }
  }
}
```

Or with a custom memlog binary:

```json
{
  "mcpServers": {
    "provenance-mcp": {
      "command": "provenance-mcp",
      "args": ["serve", "--memlog-bin", "/home/jsy/.local/bin/memlog"]
    }
  }
}
```

---

## Build

```bash
cargo build --release
# Binary at: target/release/provenance-mcp
```

MSRV: **1.85**

---

## License

MIT OR Apache-2.0

© 2026 Joe Yen

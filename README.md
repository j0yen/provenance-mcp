# provenance-mcp

An MCP server that lets an agent ask two questions about its own past — where a file came from, and what it was thinking before the last compaction — and read the answers from the kernel, not from a story it tells itself.

An agent's memory of what it did is reconstructed and unreliable. The kernel's record isn't. provfs stamps every write with the session and timestamp that produced it; memlog keeps the context that scrolled off before compaction. Both already exist below the agent. What was missing was a way to read them from inside a conversation. This is that — two read-only tools over MCP stdio, each a thin, honest window onto a kernel facility.

## The two tools

### `file_provenance`

Reads the provfs provenance xattrs for a path.

- **Input:** `{ "path": "<absolute or relative path>" }`
- **Output:** `{ "path", "session", "ts", "raw" }`. `session` and `ts` are the `user.prov.session` and `user.prov.ts` xattrs, or `null` when the file carries no provenance — which is the normal case for anything outside provfs's stamped scope (`/tmp`, `.git`, `target`). A file with no provenance is an answer, not an error.
- **When `getfattr` is absent:** a clean `ToolError`, not a crash. `serve` still starts and `tools/list` still works.

### `memlog_recent`

Returns recent pre-compaction context snapshots by running `memlog show --since <DUR> --limit <N> --format json` and passing the array through.

- **Input:** `{ "since"?: "1h" | "30m" | "7d", "limit"?: 1..1000 }`, both optional.
- **Output:** the snapshot array from `memlog show`, fields untouched.
- **When the `memlog` binary is absent:** a clean `ToolError`. The other tool keeps working.

## Read-only by construction

The point of a provenance tool is that you can trust it not to alter what it reports, so this server can't. `file_provenance` only ever runs `getfattr`; `memlog_recent` only ever runs `memlog show`. The write verbs — `setfattr`, `memlog write`, `memlog clear` — appear nowhere in the backend, and that isn't a promise, it's a test: `tests/acceptance_ac4.rs` scans `src/backend.rs` at test time and fails the build if any of them shows up. The guarantee is enforced by the same CI that builds the binary.

## Kernel dependencies

| Capability | Needs | When absent |
|------------|-------|-------------|
| file provenance | `getfattr` + the provfs LSM (`user.prov.*` xattrs) | `session`/`ts` come back `null`, or a `ToolError` if `getfattr` itself is missing |
| context snapshots | the `memlog` binary + the memlog kernel module | a `ToolError` |

On a stock kernel neither module is loaded. Both tools degrade to informative errors rather than taking the server down — see [provfs](https://github.com/j0yen/provfs) for the filesystem side.

## Run it

```bash
provenance-mcp serve                              # reads MCP from stdin, writes to stdout
provenance-mcp serve --memlog-bin /path/to/memlog # point at a non-PATH memlog
MEMLOG_BIN=/path/to/memlog provenance-mcp serve   # same, via env
```

## MCP client configuration

Add to Claude Code (or any MCP client):

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

With a non-PATH memlog:

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

## Build

This crate depends on a sibling `mcp-core` crate by path (`../mcp-core`), so it builds from inside the wintermute workspace rather than on its own:

```bash
cargo build --release   # with ../mcp-core present alongside this checkout
# binary: target/release/provenance-mcp
```

MSRV 1.85.

## Where it fits

The MCP front door to wintermute's provenance plumbing: provfs writes the xattrs this reads, and memlog holds the snapshots this returns. `mcp-core` provides the stdio transport and `Tool` trait. Part of the [wintermute](https://github.com/j0yen/wintermute) / [conduit](https://github.com/j0yen/wintermute/blob/master/visions/conduit.md) line of work.

## License

MIT OR Apache-2.0 © 2026 Joe Yen

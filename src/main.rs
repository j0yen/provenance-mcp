//! provenance-mcp: kernel-stamped file provenance and context snapshots over MCP.
//!
//! # Usage
//!
//! ```text
//! provenance-mcp serve [--memlog-bin <path>]
//! MEMLOG_BIN=/path/to/memlog provenance-mcp serve
//! ```
//!
//! Exposes two read-only MCP tools over stdio:
//! - `file_provenance` — queries provfs xattrs for a file.
//! - `memlog_recent` — retrieves recent context snapshots from memlog.

mod backend;
mod tools;

use clap::{Parser, Subcommand};
use tools::{FileProvenanceTool, MemlogRecentTool};

#[derive(Debug, Parser)]
#[command(
    name = "provenance-mcp",
    about = "MCP server: kernel-stamped file provenance and context snapshots",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    /// Run the MCP stdio server.
    Serve {
        /// Path to the memlog binary.
        /// Defaults to "memlog" resolved on PATH.
        /// Override via `MEMLOG_BIN` environment variable.
        #[arg(long, env = "MEMLOG_BIN", default_value = "memlog")]
        memlog_bin: String,
    },
}

fn main() {
    sigpipe::reset();

    let cli = Cli::parse();
    let Command::Serve { memlog_bin } = cli.command;

    let tools: Vec<Box<dyn mcp_core::Tool>> = vec![
        Box::new(FileProvenanceTool),
        Box::new(MemlogRecentTool { memlog_bin }),
    ];

    if let Err(e) = mcp_core::serve_stdio(tools, "provenance-mcp", env!("CARGO_PKG_VERSION")) {
        eprintln!("[provenance-mcp] fatal: {e}");
        std::process::exit(1);
    }
}

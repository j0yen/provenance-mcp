//! Backend: subprocess shells for `getfattr` and `memlog show`.
//!
//! Both functions are read-only by construction:
//! - `file_provenance` runs `getfattr -d --absolute-names <path>` (read only).
//! - `memlog_recent` runs `memlog show --since <DUR> --limit <N> --format json` (read only).
//!
//! Write operations on xattrs or memlog are not exposed by this module.

use std::process::Command;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use serde_json::Value;

/// Result of querying file provenance from provfs xattrs.
#[derive(Debug, serde::Serialize)]
pub struct FileProvenance {
    /// The queried path.
    pub path: String,
    /// `user.prov.session` xattr value, or `null` if absent.
    pub session: Option<String>,
    /// `user.prov.ts` xattr value, or `null` if absent.
    pub ts: Option<String>,
    /// Raw `getfattr` output for diagnostics.
    pub raw: String,
}

/// Parse `getfattr -d --absolute-names` output into `FileProvenance`.
///
/// A path that has no prov xattrs returns `session: None, ts: None` — not an error.
pub fn parse_getfattr_output(path: &str, raw: &str) -> FileProvenance {
    let mut session = None;
    let mut ts = None;

    for line in raw.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("user.prov.session=") {
            session = Some(unquote(rest).to_owned());
        } else if let Some(rest) = line.strip_prefix("user.prov.ts=") {
            ts = Some(unquote(rest).to_owned());
        }
    }

    FileProvenance {
        path: path.to_owned(),
        session,
        ts,
        raw: raw.to_owned(),
    }
}

/// Strip surrounding `"` from a getfattr value if present.
fn unquote(s: &str) -> &str {
    if s.len() >= 2 && s.starts_with('"') && s.ends_with('"') {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Run `getfattr -d --absolute-names <path>` and parse provenance xattrs.
///
/// Returns `Ok(FileProvenance)` with `session: None, ts: None` when the path has
/// no provenance xattrs. Returns `Err` only on binary-not-found or I/O failure.
///
/// # Errors
///
/// Returns an error if `getfattr` is not found on `PATH` or the subprocess fails.
pub fn file_provenance(path: &str) -> Result<FileProvenance> {
    // ONLY permitted verb: getfattr (read-only xattr query).
    let output = Command::new("getfattr")
        .args(["-d", "--absolute-names", path])
        .output()
        .context("failed to run getfattr — is getfattr installed?")?;

    // getfattr exits non-zero when the file has no xattrs on some systems;
    // treat that as "no provenance" rather than an error.
    let raw = String::from_utf8_lossy(&output.stdout).into_owned();
    Ok(parse_getfattr_output(path, &raw))
}

/// Snapshot from `memlog show`.
#[derive(Debug, Deserialize, serde::Serialize)]
pub struct MemlogSnapshot {
    /// Arbitrary fields from the memlog JSON output — pass through as-is.
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, Value>,
}

/// Run `memlog show --since <since> --limit <limit> --format json` and return the
/// parsed snapshot array.
///
/// `memlog_bin` defaults to `"memlog"` (resolved on `PATH`).
/// Can be overridden via `MEMLOG_BIN` env var or `--memlog-bin` CLI flag.
///
/// # Errors
///
/// Returns an error if the binary is not found or the output is not valid JSON.
pub fn memlog_recent(
    memlog_bin: &str,
    since: Option<&str>,
    limit: Option<u32>,
) -> Result<Vec<MemlogSnapshot>> {
    // ONLY permitted subcommand: show (read-only).
    let mut cmd = Command::new(memlog_bin);
    cmd.arg("show");
    if let Some(s) = since {
        cmd.args(["--since", s]);
    }
    if let Some(n) = limit {
        cmd.args(["--limit", &n.to_string()]);
    }
    cmd.args(["--format", "json"]);

    let output = cmd
        .output()
        .with_context(|| format!("failed to run memlog binary '{memlog_bin}'"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!(
            "memlog show exited {}: {}",
            output.status,
            stderr.trim()
        ));
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    let snapshots: Vec<MemlogSnapshot> =
        serde_json::from_str(&stdout).context("memlog show output was not valid JSON array")?;
    Ok(snapshots)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_both_xattrs() {
        let raw = r#"# file: /home/jsy/foo.txt
user.prov.session="sess-abc123"
user.prov.ts="1717200000"
"#;
        let prov = parse_getfattr_output("/home/jsy/foo.txt", raw);
        assert_eq!(prov.session.as_deref(), Some("sess-abc123"));
        assert_eq!(prov.ts.as_deref(), Some("1717200000"));
    }

    #[test]
    fn parse_no_xattrs_yields_nulls() {
        let raw = "";
        let prov = parse_getfattr_output("/tmp/noattr.txt", raw);
        assert!(prov.session.is_none());
        assert!(prov.ts.is_none());
    }

    #[test]
    fn parse_only_session_xattr() {
        let raw = "user.prov.session=\"only-session\"\n";
        let prov = parse_getfattr_output("/some/path", raw);
        assert_eq!(prov.session.as_deref(), Some("only-session"));
        assert!(prov.ts.is_none());
    }

    #[test]
    fn unquote_strips_double_quotes() {
        assert_eq!(unquote("\"hello\""), "hello");
        assert_eq!(unquote("bare"), "bare");
        assert_eq!(unquote("\"\""), "");
    }
}

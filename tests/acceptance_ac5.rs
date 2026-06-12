//! AC5: memlog_recent invokes memlog show --since <DUR> --limit <N> --format json
//! and returns the parsed array — verified against a stubbed memlog binary (offline).

use std::fs;
use std::os::unix::fs::PermissionsExt;

use tempfile::TempDir;

#[path = "../src/backend.rs"]
#[allow(dead_code)]
mod backend;

/// Write a stub memlog shell script that echoes a fixed JSON array and captures
/// its argv to a sidecar file so we can assert the right flags were passed.
fn write_stub_memlog(dir: &TempDir) -> std::path::PathBuf {
    let bin = dir.path().join("memlog");
    let argv_log = dir.path().join("argv.log");
    let argv_log_str = argv_log.display();
    let script = format!(
        r#"#!/bin/sh
echo "$@" >> {argv_log_str}
echo '[{{"ts":1717200000,"session":"sess-abc","summary":"compact context"}}]'
"#
    );
    fs::write(&bin, script).expect("write stub");
    let mut perms = fs::metadata(&bin).unwrap().permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&bin, perms).expect("chmod stub");
    bin
}

#[test]
fn memlog_recent_invokes_correct_flags_and_parses_output() {
    let dir = TempDir::new().expect("tempdir");
    let bin = write_stub_memlog(&dir);
    let bin_str = bin.to_str().unwrap();

    let snaps = backend::memlog_recent(bin_str, Some("1h"), Some(5))
        .expect("memlog_recent must succeed against stub");

    assert_eq!(snaps.len(), 1, "stub emits one snapshot");

    // Check the argv log — the stub appended its argv to argv.log
    let argv_log = dir.path().join("argv.log");
    let argv = fs::read_to_string(&argv_log).expect("argv.log");
    assert!(argv.contains("show"), "must invoke 'show' subcommand");
    assert!(argv.contains("--since"), "must pass --since flag");
    assert!(argv.contains("1h"), "must pass since value '1h'");
    assert!(argv.contains("--limit"), "must pass --limit flag");
    assert!(argv.contains("5"), "must pass limit value 5");
    assert!(argv.contains("--format"), "must pass --format flag");
    assert!(argv.contains("json"), "must pass format value 'json'");
}

#[test]
fn memlog_recent_without_args_still_works() {
    let dir = TempDir::new().expect("tempdir");
    let bin = write_stub_memlog(&dir);
    let bin_str = bin.to_str().unwrap();

    let snaps = backend::memlog_recent(bin_str, None, None)
        .expect("memlog_recent with no args must succeed");

    assert_eq!(snaps.len(), 1);
}

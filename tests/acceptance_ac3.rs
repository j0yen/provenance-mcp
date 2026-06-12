//! AC3: file_provenance parses fixtured getfattr output correctly.
//! A file with no prov xattrs yields session:null, ts:null and Ok (not error).

// We test the backend parsing function directly — no real getfattr call.
// The parse_getfattr_output fn is pub(crate) so we need a helper.
// We replicate the parsing logic here via the public API:

#[path = "../src/backend.rs"]
#[allow(dead_code)]
mod backend;

#[test]
fn fixture_with_both_xattrs_parses_correctly() {
    let raw = r#"# file: /home/jsy/wintermute/some-tool/src/main.rs
user.prov.session="session-xyz-2026"
user.prov.ts="1717200000"
"#;
    let prov = backend::parse_getfattr_output("/home/jsy/wintermute/some-tool/src/main.rs", raw);
    assert_eq!(prov.session.as_deref(), Some("session-xyz-2026"));
    assert_eq!(prov.ts.as_deref(), Some("1717200000"));
    assert_eq!(prov.path, "/home/jsy/wintermute/some-tool/src/main.rs");
}

#[test]
fn fixture_with_no_xattrs_yields_nulls_not_error() {
    let raw = "";
    let prov = backend::parse_getfattr_output("/tmp/noprov.txt", raw);
    assert!(prov.session.is_none(), "session must be null when no xattr");
    assert!(prov.ts.is_none(), "ts must be null when no xattr");
    // The function returns Ok (no Result here — parse never errors).
}

#[test]
fn fixture_with_only_session_xattr() {
    let raw = "user.prov.session=\"only-sess\"\n";
    let prov = backend::parse_getfattr_output("/some/path", raw);
    assert_eq!(prov.session.as_deref(), Some("only-sess"));
    assert!(prov.ts.is_none());
}

#[test]
fn fixture_with_only_ts_xattr() {
    let raw = "user.prov.ts=\"1700000000\"\n";
    let prov = backend::parse_getfattr_output("/other/path", raw);
    assert!(prov.session.is_none());
    assert_eq!(prov.ts.as_deref(), Some("1700000000"));
}

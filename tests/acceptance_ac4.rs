//! AC4: Read-only by construction.
//!
//! Scans backend.rs source text and asserts:
//! - Permitted verbs: "getfattr" (read), "memlog" with "show" only.
//! - Forbidden verbs: "setfattr", "memlog write", "memlog clear" do NOT appear as literals.

use std::fs;

#[test]
fn forbidden_write_verbs_absent_from_backend() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/backend.rs"))
        .expect("src/backend.rs must be readable");

    // These must never appear as command literals.
    let forbidden = ["setfattr", "memlog write", "memlog clear", "\"write\"", "\"clear\""];
    for verb in &forbidden {
        assert!(
            !src.contains(verb),
            "forbidden verb '{}' found in backend.rs — read-only contract violated",
            verb
        );
    }
}

#[test]
fn only_permitted_external_commands_present() {
    let src = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/backend.rs"))
        .expect("src/backend.rs must be readable");

    // Verify the two permitted verbs appear.
    assert!(
        src.contains("\"getfattr\""),
        "expected getfattr command in backend.rs"
    );
    assert!(
        src.contains("\"show\""),
        "expected 'show' subcommand for memlog in backend.rs"
    );
}

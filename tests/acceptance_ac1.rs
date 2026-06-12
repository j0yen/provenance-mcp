//! AC1: sigpipe::reset() is first non-whitespace statement in main();
//! build success is implied by the test binary existing.

#[test]
fn sigpipe_reset_first_in_main() {
    let src = std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/src/main.rs"))
        .expect("src/main.rs must be readable");

    // Find the fn main() body and assert sigpipe::reset() appears before any other
    // meaningful statement.
    let after_main = src
        .split("fn main()")
        .nth(1)
        .expect("fn main() must exist in src/main.rs");

    // Strip the leading `{` brace and whitespace/comments to get the first real token.
    let body = after_main
        .trim_start()
        .strip_prefix('{')
        .expect("main body must start with {")
        .trim_start();

    assert!(
        body.starts_with("sigpipe::reset()"),
        "sigpipe::reset() must be the first statement in main(), found: {body:.60}"
    );
}

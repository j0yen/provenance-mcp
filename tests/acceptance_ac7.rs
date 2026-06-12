//! AC7: README.md documents required sections.

use std::fs;

#[test]
fn readme_contains_required_sections() {
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("README.md must exist");

    let required = [
        "file_provenance",
        "memlog_recent",
        "read-only",
        "provfs",
        "memlog",
        "MCP",
    ];

    for keyword in &required {
        assert!(
            readme.contains(keyword),
            "README.md must contain '{}' but does not",
            keyword
        );
    }
}

#[test]
fn readme_documents_mcp_config_snippet() {
    let readme = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/README.md"))
        .expect("README.md must exist");

    // MCP client config is typically JSON — look for a code block with the binary name.
    assert!(
        readme.contains("provenance-mcp"),
        "README.md must contain provenance-mcp in MCP config snippet"
    );
}

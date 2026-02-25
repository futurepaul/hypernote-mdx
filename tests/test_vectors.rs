/// Test that all test vector files from the Zig project parse without panicking
/// and produce valid JSON output.

const HELLO: &str = include_str!("../../zig-mdx/test_vectors/hello.hnmd");
const FEED: &str = include_str!("../../zig-mdx/test_vectors/feed.hnmd");
const PATHOLOGICAL: &str = include_str!("../../zig-mdx/test_vectors/pathological.hnmd");
const TEST_CASES: &str = include_str!("../../zig-mdx/test_vectors/test_cases.md");
const FAILED_ALLOC: &str = include_str!("../../zig-mdx/test_vectors/failed_to_allocate_memory.md");

fn parse_and_check(name: &str, source: &str) {
    let ast = hypernote_mdx::parse(source);
    let json = hypernote_mdx::serialize_tree(&ast);
    let rendered = hypernote_mdx::render(&ast);

    // Must produce non-empty output
    assert!(!json.is_empty(), "{}: JSON output is empty", name);
    assert!(!rendered.is_empty(), "{}: rendered output is empty", name);

    // JSON must be valid
    let parsed: serde_json::Value = serde_json::from_str(&json)
        .unwrap_or_else(|e| panic!("{}: invalid JSON: {}", name, e));

    let root = parsed.as_object().unwrap();
    assert!(root.contains_key("type"), "{}: missing 'type'", name);
    assert!(root.contains_key("children"), "{}: missing 'children'", name);
    assert!(root.contains_key("source"), "{}: missing 'source'", name);
    assert!(root.contains_key("errors"), "{}: missing 'errors'", name);

    eprintln!(
        "{}: nodes={}, errors={}, json_len={}, render_len={}",
        name,
        ast.nodes.len(),
        ast.errors.len(),
        json.len(),
        rendered.len()
    );
    for err in &ast.errors {
        eprintln!("  error: {} at token {}", err.tag.name(), err.token);
    }
}

#[test]
fn test_vector_hello() {
    parse_and_check("hello.hnmd", HELLO);
}

#[test]
fn test_vector_feed() {
    parse_and_check("feed.hnmd", FEED);
}

#[test]
fn test_vector_pathological() {
    parse_and_check("pathological.hnmd", PATHOLOGICAL);
}

#[test]
fn test_vector_test_cases() {
    parse_and_check("test_cases.md", TEST_CASES);
}

#[test]
fn test_vector_failed_alloc() {
    parse_and_check("failed_to_allocate_memory.md", FAILED_ALLOC);
}

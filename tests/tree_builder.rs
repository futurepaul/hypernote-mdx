use hypernote_mdx::tree_builder;

#[test]
fn serializes_simple_text() {
    let source = "Hello world";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    assert!(!json_str.is_empty());
    assert!(json_str.contains("\"type\":\"root\""));
    assert!(json_str.contains("Hello world"));
}

#[test]
fn serializes_heading_with_level() {
    let source = "# Hello";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    assert!(json_str.contains("\"type\":\"heading\""));
    assert!(json_str.contains("\"level\":1"));
}

#[test]
fn serializes_code_block_with_language() {
    let source = "```javascript\nconsole.log(\"hi\");\n```";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    assert!(json_str.contains("\"type\":\"code_block\""));
    assert!(json_str.contains("\"lang\":\"javascript\""));
}

#[test]
fn serializes_jsx_element_with_attributes() {
    let source = "<Button color=\"blue\" />";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    assert!(json_str.contains("\"type\":\"mdx_jsx_self_closing\""));
    assert!(json_str.contains("\"name\":\"Button\""));
    assert!(json_str.contains("\"attributes\""));
    assert!(json_str.contains("blue"));
}

#[test]
fn escapes_json_strings() {
    let source = "Text with \"quotes\" and \\backslash";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    assert!(json_str.contains("\\\"quotes\\\""));
    assert!(json_str.contains("\\\\backslash"));
}

#[test]
fn includes_errors_in_output() {
    let source = "<Unclosed";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    assert!(json_str.contains("\"errors\""));
}

#[test]
fn produces_valid_json() {
    let source = "# Title\n\nA paragraph with **bold** text.\n\n- Item 1\n- Item 2";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    // Parse as JSON to validate
    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let root = parsed.as_object().unwrap();

    assert!(root.contains_key("type"));
    assert!(root.contains_key("children"));
    assert!(root.contains_key("source"));
    assert!(root.contains_key("errors"));
}

#[test]
fn serializes_json_frontmatter() {
    let source = "```hnmd\n{\"title\": \"Test\"}\n```\n\n# Hello\n";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let children = parsed["children"].as_array().unwrap();
    let fm = &children[0];

    assert_eq!(fm["type"], "frontmatter");
    assert_eq!(fm["format"], "json");
    assert!(fm["value"]
        .as_str()
        .unwrap()
        .contains("\"title\": \"Test\""));
}

#[test]
fn serializes_yaml_frontmatter_with_format() {
    let source = "---\ntitle: Hello\n---\n\n# Content\n";
    let ast = hypernote_mdx::parse(source);
    let json_str = tree_builder::serialize_tree(&ast);

    let parsed: serde_json::Value = serde_json::from_str(&json_str).unwrap();
    let children = parsed["children"].as_array().unwrap();
    let fm = &children[0];

    assert_eq!(fm["type"], "frontmatter");
    assert_eq!(fm["format"], "yaml");
    assert!(fm["value"].as_str().unwrap().contains("title: Hello"));
}

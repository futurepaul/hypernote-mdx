use hypernote_mdx::{parse, render, serialize_tree};

#[test]
fn jsx_attribute_entities_roundtrip_and_decode_in_ast() {
    let source =
        "<Button label=\"Fish &amp; Chips &lt;3 &gt; 2 &quot;quote&quot;\" />\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);
    assert_eq!(source, render(&ast));

    let json = serialize_tree(&ast);
    let root: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        "Fish & Chips <3 > 2 \"quote\"",
        root["children"][0]["attributes"][0]["value"]
    );
}

#[test]
fn code_block_preserves_literal_htmlish_text() {
    let source = "```html\n<div class=\"x\">& raw</div>\n```\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);
    assert_eq!(source, render(&ast));

    let json = serialize_tree(&ast);
    let root: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(
        "<div class=\"x\">& raw</div>\n",
        root["children"][0]["value"]
    );
}

#[test]
fn inline_code_preserves_literal_htmlish_text() {
    let source = "`<tag>&value`\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);
    assert_eq!(source, render(&ast));

    let json = serialize_tree(&ast);
    let root: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!("<tag>&value", root["children"][0]["children"][0]["value"]);
}

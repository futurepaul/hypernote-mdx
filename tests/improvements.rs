use hypernote_mdx::ast::{ErrorTag, NodeTag};
use hypernote_mdx::{parse, parse_with_options, render, serialize_tree, ParseOptions};

#[test]
fn shortcode_normalization_is_opt_in() {
    let source = ":thumbsup:\n";
    let ast = parse(source);
    let rendered = render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn shortcode_normalization_option_converts_known_codes() {
    let source = ":thumbsup:\n";
    let options = ParseOptions {
        normalize_emoji_shortcodes: true,
    };
    let ast = parse_with_options(source, &options);
    let rendered = render(&ast);
    assert_eq!("👍\n", rendered);
}

#[test]
fn keycap_emoji_is_not_misparsed_as_markdown_syntax() {
    let source = "#️⃣ not a heading\n*️⃣ not emphasis\n";
    let ast = parse(source);
    assert!(
        ast.errors.is_empty(),
        "Expected no parse errors, got: {:?}",
        ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>()
    );

    let has_heading = ast.nodes.iter().any(|n| n.tag == NodeTag::Heading);
    let has_emphasis = ast.nodes.iter().any(|n| n.tag == NodeTag::Emphasis);
    assert!(!has_heading, "Keycap #️⃣ should not become a heading");
    assert!(!has_emphasis, "Keycap *️⃣ should not become emphasis");

    let rendered = render(&ast);
    assert!(rendered.contains("#️⃣ not a heading"));
    assert!(rendered.contains("*️⃣ not emphasis"));
}

#[test]
fn jsx_attributes_have_explicit_value_types() {
    let source =
        r#"<Widget count=4 enabled label="ok" active=true ratio=-1.5 expr={state.count} />"#;
    let ast = parse(source);
    assert!(
        ast.errors.is_empty(),
        "Expected no parse errors, got: {:?}",
        ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>()
    );

    let json = serialize_tree(&ast);
    let root: serde_json::Value = serde_json::from_str(&json).unwrap();
    let attrs = root["children"][0]["attributes"].as_array().unwrap();

    let mut by_name = std::collections::BTreeMap::new();
    for attr in attrs {
        let name = attr["name"].as_str().unwrap().to_string();
        by_name.insert(name, attr.clone());
    }

    assert_eq!("number", by_name["count"]["value_type"]);
    assert_eq!(Some(4.0), by_name["count"]["value"].as_f64());

    assert_eq!("boolean", by_name["enabled"]["value_type"]);
    assert_eq!(Some(true), by_name["enabled"]["value"].as_bool());

    assert_eq!("string", by_name["label"]["value_type"]);
    assert_eq!(Some("ok"), by_name["label"]["value"].as_str());

    assert_eq!("boolean", by_name["active"]["value_type"]);
    assert_eq!(Some(true), by_name["active"]["value"].as_bool());

    assert_eq!("number", by_name["ratio"]["value_type"]);
    assert_eq!(Some(-1.5), by_name["ratio"]["value"].as_f64());

    assert_eq!("expression", by_name["expr"]["value_type"]);
    assert_eq!(Some("state.count"), by_name["expr"]["value"].as_str());
}

#[test]
fn malformed_jsx_reports_actionable_byte_offsets() {
    let source = "<Card><Body>hi</Card>\n";
    let ast = parse(source);

    let mismatch = ast
        .errors
        .iter()
        .find(|e| e.tag == ErrorTag::MismatchedTags);
    assert!(
        mismatch.is_some(),
        "Expected mismatched tag error, got: {:?}",
        ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>()
    );
    let mismatch = mismatch.unwrap();

    let expected_offset = source.find("</Card>").unwrap() as u32;
    assert_eq!(expected_offset, mismatch.byte_offset);
}

#[test]
fn serializer_emits_schema_version() {
    let source = "# hi\n";
    let ast = parse(source);
    let json = serialize_tree(&ast);
    let root: serde_json::Value = serde_json::from_str(&json).unwrap();

    assert_eq!("hypernote-mdx-ast", root["schema"]["name"]);
    assert_eq!(1, root["schema"]["version"]);
}

#[test]
fn mixed_markdown_jsx_emoji_multibyte_roundtrip_is_stable() {
    let source = r#"# 週報 🚀

こんにちは 👋

<Card title="進捗 &amp; リスク">
チームは **順調** です 👨‍👩‍👧‍👦
</Card>
"#;
    let ast1 = parse(source);
    assert!(
        ast1.errors.is_empty(),
        "Expected no parse errors, got: {:?}",
        ast1.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>()
    );

    let rendered1 = render(&ast1);
    let ast2 = parse(&rendered1);
    let rendered2 = render(&ast2);

    assert_eq!(rendered1, rendered2);
    assert!(rendered2.contains("👨‍👩‍👧‍👦"));
    assert!(rendered2.contains("週報"));
}

fn lcg_next(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *state
}

#[test]
fn generated_corpus_does_not_panic_and_is_json_valid() {
    let fragments = [
        "plain text",
        "emoji 👨‍👩‍👧‍👦 and 🇯🇵 and ✨",
        "**bold** text",
        "*emphasis* text",
        "`inline code`",
        "[link](https://example.com)",
        "- [ ] task",
        "1. ordered",
        "<Box count=4 enabled />",
        "<Card>nested <B>text</B> ✅</Card>",
        "{state.count}",
        "3️⃣ keycap",
        "*️⃣ star keycap",
        "<Broken a=>",
        "<Mismatch><A>x</B></Mismatch>",
    ];

    let mut seed = 0xA5A5_F00Du64;
    for _case in 0..128 {
        let mut source = String::new();
        let line_count = (lcg_next(&mut seed) % 6 + 1) as usize;
        for _line in 0..line_count {
            let idx = (lcg_next(&mut seed) % fragments.len() as u64) as usize;
            source.push_str(fragments[idx]);
            source.push('\n');
        }

        let ast = parse(&source);
        let json = serialize_tree(&ast);
        let parsed_json: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!("root", parsed_json["type"]);

        let rendered = render(&ast);
        let ast2 = parse(&rendered);
        let json2 = serialize_tree(&ast2);
        serde_json::from_str::<serde_json::Value>(&json2).unwrap();
    }
}

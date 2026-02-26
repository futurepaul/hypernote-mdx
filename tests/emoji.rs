use hypernote_mdx::ast::NodeTag;

// ── Plain text emoji ──────────────────────────────────────────────

#[test]
fn emoji_in_paragraph() {
    let source = "Hello 🌍 world\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);
    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_only_paragraph() {
    let source = "🔥🎉✨\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);
    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_at_start_and_end_of_line() {
    let source = "🎯 target acquired 🎯\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);
    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn complex_emoji_sequences() {
    // Skin tone modifier, ZWJ sequence (family), flag emoji
    let source = "👋🏽 hello from 👨‍👩‍👧‍👦 in 🇯🇵\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);
    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

// ── Emoji in headings ──────────────────────────────────────────────

#[test]
fn emoji_in_heading() {
    let source = "# 🚀 Launch Plan\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_heading = ast.nodes.iter().any(|n| n.tag == NodeTag::Heading);
    assert!(has_heading, "Should parse as heading");

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_only_heading() {
    let source = "## 🎉🎊🥳\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

// ── Emoji in inline formatting ──────────────────────────────────────

#[test]
fn emoji_in_bold() {
    let source = "**🔥 hot take**\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_strong = ast.nodes.iter().any(|n| n.tag == NodeTag::Strong);
    assert!(has_strong, "Should have Strong node");

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_in_emphasis() {
    let source = "*✨ sparkly ✨*\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_emphasis = ast.nodes.iter().any(|n| n.tag == NodeTag::Emphasis);
    assert!(has_emphasis, "Should have Emphasis node");

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_in_inline_code() {
    let source = "`🐛 bug`\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_code = ast.nodes.iter().any(|n| n.tag == NodeTag::CodeInline);
    assert!(has_code, "Should have CodeInline node");

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_in_link_text() {
    let source = "[🔗 click here](https://example.com)\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_link = ast.nodes.iter().any(|n| n.tag == NodeTag::Link);
    assert!(has_link, "Should have Link node");

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

// ── Emoji in checklists ──────────────────────────────────────────────

#[test]
fn emoji_in_unchecked_checkbox() {
    let source = "- [ ] 📝 Write docs\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();
    let info = ast.list_item_info(item_idx);
    assert_eq!(Some(false), info.checked);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_in_checked_checkbox() {
    let source = "- [x] ✅ All done\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();
    let info = ast.list_item_info(item_idx);
    assert_eq!(Some(true), info.checked);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn mixed_emoji_checklist() {
    let source = "- [x] 🍕 Order pizza\n- [ ] 🧹 Clean house\n- [x] 💤 Take nap\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let list_items: Vec<_> = ast
        .nodes
        .iter()
        .enumerate()
        .filter(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .collect();
    assert_eq!(3, list_items.len());

    assert_eq!(Some(true), ast.list_item_info(list_items[0]).checked);
    assert_eq!(Some(false), ast.list_item_info(list_items[1]).checked);
    assert_eq!(Some(true), ast.list_item_info(list_items[2]).checked);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_only_checklist_item() {
    let source = "- [ ] 🎯\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let item_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::ListItem)
        .map(|(i, _)| i as u32)
        .unwrap();
    let info = ast.list_item_info(item_idx);
    assert_eq!(Some(false), info.checked);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

// ── Emoji in regular lists ──────────────────────────────────────────

#[test]
fn emoji_in_unordered_list() {
    let source = "- 🍎 Apple\n- 🍌 Banana\n- 🍊 Orange\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let item_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::ListItem)
        .count();
    assert_eq!(3, item_count);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_in_ordered_list() {
    let source = "1. 🥇 Gold\n2. 🥈 Silver\n3. 🥉 Bronze\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

// ── Emoji in MDX components ──────────────────────────────────────────

#[test]
fn emoji_as_jsx_child_text() {
    let source = "<Card>🎉 Congratulations!</Card>\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_jsx = ast.nodes.iter().any(|n| n.tag == NodeTag::MdxJsxElement);
    assert!(has_jsx, "Should have MdxJsxElement");

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_in_jsx_attribute_value() {
    let source = r#"<Button label="🚀 Launch" />"#;
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let json = hypernote_mdx::serialize_tree(&ast);
    assert!(
        json.contains("🚀 Launch"),
        "JSON should preserve emoji in attribute: {}",
        json
    );

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(format!("{}\n", source), rendered);
}

#[test]
fn emoji_in_nested_jsx() {
    let source = r#"<Card>
  <Heading>🏆 Winner</Heading>
  <Body>You earned 🌟🌟🌟</Body>
</Card>"#;
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let json = hypernote_mdx::serialize_tree(&ast);
    assert!(
        json.contains("🏆 Winner"),
        "JSON should contain heading emoji"
    );
    assert!(json.contains("🌟🌟🌟"), "JSON should contain star emoji");
}

#[test]
fn emoji_in_jsx_expression() {
    let source = "{'🎵 music note'}\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let json = hypernote_mdx::serialize_tree(&ast);
    assert!(
        json.contains("🎵"),
        "JSON should contain emoji in expression: {}",
        json
    );
}

#[test]
fn emoji_in_jsx_with_markdown_children() {
    let source = "<Alert>\n\n# ⚠️ Warning\n\nThis is **🔴 critical**\n\n</Alert>\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let rendered = hypernote_mdx::render(&ast);
    assert!(
        rendered.contains("⚠️ Warning"),
        "Should preserve warning emoji"
    );
    assert!(
        rendered.contains("🔴 critical"),
        "Should preserve red circle emoji"
    );
}

// ── Emoji in code blocks ──────────────────────────────────────────────

#[test]
fn emoji_in_code_block() {
    let source = "```\n🦀 Rust is great 🦀\n```\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let json = hypernote_mdx::serialize_tree(&ast);
    assert!(
        json.contains("🦀"),
        "Code block should preserve emoji: {}",
        json
    );

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

// ── Emoji in blockquotes ──────────────────────────────────────────────

#[test]
fn emoji_in_blockquote() {
    let source = "> 💡 Pro tip: use emoji\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_blockquote = ast.nodes.iter().any(|n| n.tag == NodeTag::Blockquote);
    assert!(has_blockquote, "Should have Blockquote node");

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

// ── JSON serialization ──────────────────────────────────────────────

#[test]
fn emoji_json_serialization_valid() {
    let source = "# 🎉 Party\n\n- [ ] 🍕 Food\n- [x] 🎵 Music\n\n<Card>🎊 Confetti</Card>\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let json = hypernote_mdx::serialize_tree(&ast);

    // Should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&json).expect("JSON with emoji should be valid");

    // Emoji should be preserved as-is (not escaped to \uXXXX)
    let json_pretty = serde_json::to_string_pretty(&parsed).unwrap();
    assert!(
        json_pretty.contains("🎉"),
        "Party emoji missing: {}",
        json_pretty
    );
    assert!(
        json_pretty.contains("🍕"),
        "Pizza emoji missing: {}",
        json_pretty
    );
    assert!(
        json_pretty.contains("🎵"),
        "Music emoji missing: {}",
        json_pretty
    );
    assert!(
        json_pretty.contains("🎊"),
        "Confetti emoji missing: {}",
        json_pretty
    );
}

// ── Roundtrip stability ──────────────────────────────────────────────

#[test]
fn emoji_double_roundtrip() {
    let source = "- [ ] 🎯 Goal\n- [x] ✅ Done\n- 🔄 Regular\n";
    let ast1 = hypernote_mdx::parse(source);
    let rendered1 = hypernote_mdx::render(&ast1);
    let ast2 = hypernote_mdx::parse(&rendered1);
    let rendered2 = hypernote_mdx::render(&ast2);
    assert_eq!(rendered1, rendered2, "Double round-trip should be stable");
}

#[test]
fn emoji_jsx_roundtrip() {
    let source = "<Card title=\"🏠 Home\">\n  <Body>Welcome 👋</Body>\n</Card>\n";
    let ast = hypernote_mdx::parse(source);
    let rendered = hypernote_mdx::render(&ast);

    let ast2 = hypernote_mdx::parse(&rendered);
    let rendered2 = hypernote_mdx::render(&ast2);
    assert_eq!(
        rendered, rendered2,
        "JSX emoji double round-trip should be stable"
    );
}

// ── Edge cases ──────────────────────────────────────────────────────

#[test]
fn emoji_adjacent_to_markdown_syntax() {
    // Emoji right next to syntax markers with no space
    let source = "**🔥**hot and *🧊*cold\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let has_strong = ast.nodes.iter().any(|n| n.tag == NodeTag::Strong);
    let has_emphasis = ast.nodes.iter().any(|n| n.tag == NodeTag::Emphasis);
    assert!(has_strong, "Should parse bold emoji");
    assert!(has_emphasis, "Should parse italic emoji");
}

#[test]
fn emoji_between_two_jsx_elements() {
    let source = "<A>hello</A>\n\n🎉\n\n<B>world</B>\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let rendered = hypernote_mdx::render(&ast);
    assert!(
        rendered.contains("🎉"),
        "Emoji between components should survive"
    );
}

#[test]
fn many_emoji_in_a_row() {
    let source = "🏁🏁🏁🏁🏁🏁🏁🏁🏁🏁\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

#[test]
fn emoji_with_text_number_prefix() {
    // Make sure emoji after numbers doesn't confuse ordered list detection
    let source = "3️⃣ three\n";
    let ast = hypernote_mdx::parse(source);
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors);

    let rendered = hypernote_mdx::render(&ast);
    assert_eq!(source, rendered);
}

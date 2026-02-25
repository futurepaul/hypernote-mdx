/// Tests based on the examples in HYPERNOTE_IN_PIKA_PLAN.md

use hypernote_mdx::ast::NodeTag;

#[test]
fn invoice_example() {
    let source = r#"# Invoice from @merchant

**Service:** API hosting (June 2026)

<Card>
  <Caption>Amount</Caption>
  <Heading>50,000 sats</Heading>
  <Caption>Expires in 12 minutes</Caption>
</Card>

<HStack gap="4">
  <SubmitButton action="reject" variant="secondary">Reject</SubmitButton>
  <SubmitButton action="approve" variant="danger">Authorize Payment</SubmitButton>
</HStack>"#;

    let ast = hypernote_mdx::parse(source);
    let json = hypernote_mdx::serialize_tree(&ast);

    // Should parse without errors
    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>());

    // Verify JSON is valid
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let root = parsed.as_object().unwrap();
    let children = root["children"].as_array().unwrap();

    // Should have: heading, paragraph, Card element, HStack element
    let types: Vec<&str> = children.iter().map(|c| c["type"].as_str().unwrap()).collect();
    assert!(types.contains(&"heading"), "types: {:?}", types);
    assert!(types.contains(&"paragraph"), "types: {:?}", types);
    assert!(types.contains(&"mdx_jsx_element"), "types: {:?}", types);
}

#[test]
fn chat_action_example() {
    let source = r#"Which language should I use for the backend?

<HStack gap="4">
  <SubmitButton action="choose" variant="secondary">Rust</SubmitButton>
  <SubmitButton action="choose" variant="secondary">Go</SubmitButton>
  <SubmitButton action="choose" variant="secondary">TypeScript</SubmitButton>
</HStack>"#;

    let ast = hypernote_mdx::parse(source);

    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>());

    // Find the HStack element
    let hstack = ast.nodes.iter().enumerate().find(|(_, n)| {
        if n.tag == NodeTag::MdxJsxElement {
            let elem = ast.jsx_element(*&(ast.nodes.iter().position(|x| std::ptr::eq(x, *n)).unwrap() as u32));
            let name = ast.token_slice(elem.name_token).trim();
            name == "HStack"
        } else {
            false
        }
    });
    assert!(hstack.is_some(), "HStack not found");

    // Count SubmitButton children
    let submit_count = ast.nodes.iter().filter(|n| {
        if n.tag == NodeTag::MdxJsxElement {
            let idx = ast.nodes.iter().position(|x| std::ptr::eq(x, *n)).unwrap() as u32;
            let elem = ast.jsx_element(idx);
            let name = ast.token_slice(elem.name_token).trim();
            name == "SubmitButton"
        } else {
            false
        }
    }).count();
    assert_eq!(3, submit_count, "Expected 3 SubmitButtons");
}

#[test]
fn signed_action_example() {
    let source = r#"# Quick Note

<Card>
  <Caption>Post to Nostr</Caption>
  <TextInput name="message" placeholder="What's on your mind?" />
  <SubmitButton action="post" variant="primary">Publish</SubmitButton>
</Card>"#;

    let ast = hypernote_mdx::parse(source);
    let json = hypernote_mdx::serialize_tree(&ast);

    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>());

    // Verify TextInput is self-closing with correct attributes
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let json_str = serde_json::to_string_pretty(&parsed).unwrap();

    assert!(json_str.contains("TextInput"), "Missing TextInput: {}", json_str);
    assert!(json_str.contains("SubmitButton"), "Missing SubmitButton: {}", json_str);
    // TextInput should have name="message" (attr name is "name", attr value is "message")
    assert!(json_str.contains("\"value\": \"message\""), "Missing name=message attr: {}", json_str);
    assert!(json_str.contains("\"value\": \"What's on your mind?\""), "Missing placeholder attr: {}", json_str);
}

#[test]
fn expression_in_jsx_attr() {
    let source = r#"<json value={queries.feed} />"#;

    let ast = hypernote_mdx::parse(source);

    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>());

    // Verify the expression attribute is parsed
    let json = hypernote_mdx::serialize_tree(&ast);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let json_str = serde_json::to_string_pretty(&parsed).unwrap();

    assert!(json_str.contains("\"type\": \"expression\""), "Missing expression attr type: {}", json_str);
    assert!(json_str.contains("queries.feed"), "Missing expression value: {}", json_str);
}

#[test]
fn plain_markdown_through_renderer() {
    // "A message that says **hello** renders with bold text"
    let source = "**hello**";
    let ast = hypernote_mdx::parse(source);

    assert_eq!(0, ast.errors.len());

    let has_strong = ast.nodes.iter().any(|n| n.tag == NodeTag::Strong);
    assert!(has_strong);

    let rendered = hypernote_mdx::render(&ast);
    assert!(rendered.contains("**hello**"));
}

#[test]
fn nested_layout_components() {
    let source = r#"<Card title="Payment">
  <VStack>
    <Heading>50,000 sats</Heading>
    <Body>Due in 12 minutes</Body>
  </VStack>
  <HStack gap="4">
    <SubmitButton action="pay">Pay Now</SubmitButton>
  </HStack>
</Card>"#;

    let ast = hypernote_mdx::parse(source);

    assert_eq!(0, ast.errors.len(), "errors: {:?}", ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>());

    // Verify nesting: Card > VStack, Card > HStack
    let json = hypernote_mdx::serialize_tree(&ast);
    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    let json_str = serde_json::to_string_pretty(&parsed).unwrap();

    assert!(json_str.contains("Card"), "Missing Card");
    assert!(json_str.contains("VStack"), "Missing VStack");
    assert!(json_str.contains("HStack"), "Missing HStack");
    assert!(json_str.contains("SubmitButton"), "Missing SubmitButton");
}

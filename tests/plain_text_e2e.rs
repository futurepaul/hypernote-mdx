use hypernote_mdx::ast::{NodeIndex, NodeTag};
use hypernote_mdx::semantic::{ExpressionTextPolicy, PlainTextOptions};
use hypernote_mdx::parse;

fn document_index(ast: &hypernote_mdx::ast::Ast) -> NodeIndex {
    ast.nodes
        .iter()
        .enumerate()
        .find_map(|(idx, node)| (node.tag == NodeTag::Document).then_some(idx as NodeIndex))
        .expect("expected document node")
}

#[test]
fn plain_text_invoice_document_is_useful_end_to_end() {
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

    let ast = parse(source);
    let document = document_index(&ast);
    let plain = ast.plain_text(document).expect("expected document plain text");

    assert!(plain.contains("Invoice from @merchant"), "{plain}");
    assert!(plain.contains("Service: API hosting (June 2026)"), "{plain}");
    assert!(plain.contains("Amount\n50,000 sats\nExpires in 12 minutes"), "{plain}");
    assert!(plain.contains("Reject\nAuthorize Payment"), "{plain}");
}

#[test]
fn plain_text_chat_workflow_document_supports_expression_policies_end_to_end() {
    let source = r#"Which language should I use for the backend?

<Card>
  <Body>Current choice: {state.choice}</Body>
</Card>

<HStack gap="4">
  <SubmitButton action="choose" variant="secondary">Rust</SubmitButton>
  <SubmitButton action="choose" variant="secondary">Go</SubmitButton>
  <SubmitButton action="choose" variant="secondary">TypeScript</SubmitButton>
</HStack>"#;

    let ast = parse(source);
    let document = document_index(&ast);

    let default_text = ast.plain_text(document).expect("expected default plain text");
    let omitted = ast
        .plain_text_with_options(
            document,
            &PlainTextOptions {
                expression_policy: ExpressionTextPolicy::Omit,
            },
        )
        .expect("expected omitted plain text");
    let placeholder = ast
        .plain_text_with_options(
            document,
            &PlainTextOptions {
                expression_policy: ExpressionTextPolicy::Placeholder("{expr}"),
            },
        )
        .expect("expected placeholder plain text");

    assert!(default_text.contains("Current choice: state.choice"), "{default_text}");
    assert!(omitted.contains("Current choice: "), "{omitted}");
    assert!(!omitted.contains("state.choice"), "{omitted}");
    assert!(placeholder.contains("Current choice: {expr}"), "{placeholder}");
    assert!(default_text.contains("Rust\nGo\nTypeScript"), "{default_text}");
}

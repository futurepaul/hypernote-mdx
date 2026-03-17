use hypernote_mdx::ast::{NodeIndex, NodeTag};
use hypernote_mdx::semantic::{ExpressionKind, ExpressionTextPolicy, PlainTextOptions, PlainTextPart};
use hypernote_mdx::parse;

fn first_node_by_tag(ast: &hypernote_mdx::ast::Ast, tag: NodeTag) -> NodeIndex {
    ast.nodes
        .iter()
        .enumerate()
        .find_map(|(idx, node)| (node.tag == tag).then_some(idx as NodeIndex))
        .expect("expected node tag")
}

#[test]
fn plain_text_parts_preserve_expression_structure() {
    let ast = parse("Value: {state.count}\n");
    let paragraph = first_node_by_tag(&ast, NodeTag::Paragraph);
    let parts = ast
        .plain_text_parts(paragraph)
        .expect("expected plain text parts");

    assert_eq!(
        vec![
            PlainTextPart::Text("Value: "),
            PlainTextPart::Expression {
                kind: ExpressionKind::Text,
                source: "state.count",
            },
        ],
        parts
    );
}

#[test]
fn plain_text_with_options_controls_expression_output() {
    let ast = parse("Value: {state.count}\n");
    let paragraph = first_node_by_tag(&ast, NodeTag::Paragraph);

    assert_eq!("Value: state.count", ast.plain_text(paragraph).unwrap());
    assert_eq!(
        "Value: ",
        ast.plain_text_with_options(
            paragraph,
            &PlainTextOptions {
                expression_policy: ExpressionTextPolicy::Omit,
            }
        )
        .unwrap()
    );
    assert_eq!(
        "Value: {expr}",
        ast.plain_text_with_options(
            paragraph,
            &PlainTextOptions {
                expression_policy: ExpressionTextPolicy::Placeholder("{expr}"),
            }
        )
        .unwrap()
    );
}

#[test]
fn plain_text_handles_links_images_code_and_hard_breaks() {
    let ast = parse("[**bold** label](https://example.com)  \n![*alt* text](image.png) `code`\n");
    let paragraph = first_node_by_tag(&ast, NodeTag::Paragraph);

    assert_eq!(
        "bold label\nalt text code",
        ast.plain_text(paragraph).expect("expected plain text")
    );
}

#[test]
fn plain_text_link_falls_back_to_url_when_label_is_empty() {
    let ast = parse("[](https://example.com)\n");
    let link = first_node_by_tag(&ast, NodeTag::Link);

    assert_eq!(
        "https://example.com",
        ast.plain_text(link).expect("expected link plain text")
    );
}

#[test]
fn plain_text_recurse_inside_jsx_children() {
    let ast = parse("<Card>**bold** {state.count}</Card>\n");
    let jsx = first_node_by_tag(&ast, NodeTag::MdxJsxElement);
    let parts = ast.plain_text_parts(jsx).expect("expected plain text parts");

    assert_eq!(
        vec![
            PlainTextPart::Text("bold"),
            PlainTextPart::Text(" "),
            PlainTextPart::Expression {
                kind: ExpressionKind::Text,
                source: "state.count",
            },
        ],
        parts
    );
    assert_eq!("bold state.count", ast.plain_text(jsx).unwrap());
}

#[test]
fn plain_text_children_is_inline_focused() {
    let ast = parse("[**bold** label](https://example.com)\n");
    let link = first_node_by_tag(&ast, NodeTag::Link);
    let children = ast.link_view(link).expect("expected link").label_children;

    assert_eq!("bold label", ast.plain_text_children(children));
}

use hypernote_mdx::ast::{ErrorTag, NodeIndex, NodeTag, SourcePosition};
use hypernote_mdx::parse;

fn first_node_by_tag(ast: &hypernote_mdx::ast::Ast, tag: NodeTag) -> NodeIndex {
    ast.nodes
        .iter()
        .enumerate()
        .find_map(|(idx, node)| (node.tag == tag).then_some(idx as NodeIndex))
        .expect("expected node tag")
}

#[test]
fn invalid_jsx_attribute_reports_precise_byte_offset() {
    let source = "<Button label=>\n";
    let ast = parse(source);

    let err = ast
        .errors
        .iter()
        .find(|err| err.tag == ErrorTag::InvalidJsxAttribute)
        .expect("expected invalid_jsx_attribute error");

    assert_eq!(source.find('>').unwrap() as u32, err.byte_offset);
}

#[test]
fn unclosed_jsx_attribute_expression_reports_eof_offset() {
    let source = "<Button expr={foo\n";
    let ast = parse(source);

    let err = ast
        .errors
        .iter()
        .find(|err| err.tag == ErrorTag::UnclosedExpression)
        .expect("expected unclosed_expression error");

    assert_eq!(source.len() as u32, err.byte_offset);
}

#[test]
fn line_col_maps_byte_offsets_to_one_based_positions() {
    let source = "alpha\n<Button label=>\n";
    let ast = parse(source);
    let err = ast
        .errors
        .iter()
        .find(|err| err.tag == ErrorTag::InvalidJsxAttribute)
        .expect("expected invalid_jsx_attribute error");

    assert_eq!(
        SourcePosition {
            line: 2,
            column: 15
        },
        ast.line_col(err.byte_offset)
    );
    assert_eq!(SourcePosition { line: 1, column: 1 }, ast.line_col(0));
    assert_eq!(
        SourcePosition { line: 3, column: 1 },
        ast.line_col(source.len() as u32)
    );
}

#[test]
fn node_position_uses_node_start_offset() {
    let source = "before\n\n<Card>\nhello\n</Card>\n";
    let ast = parse(source);
    let card = first_node_by_tag(&ast, NodeTag::MdxJsxElement);

    assert_eq!(
        SourcePosition { line: 3, column: 1 },
        ast.node_position(card)
    );
}

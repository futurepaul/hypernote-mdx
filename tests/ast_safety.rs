use hypernote_mdx::ast::{Ast, FrontmatterFormat, Node, NodeData, NodeTag, Range, TableAlignment};
use hypernote_mdx::token::Tag as TokenTag;
use hypernote_mdx::{render, serialize_tree};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn malformed_ast() -> Ast {
    Ast {
        source: "ok".to_string(),
        token_tags: vec![TokenTag::Text, TokenTag::Eof],
        token_starts: vec![0, 2],
        nodes: vec![
            Node {
                tag: NodeTag::Document,
                main_token: 0,
                data: NodeData::Children(Range { start: 0, end: 6 }),
            },
            Node {
                tag: NodeTag::Heading,
                main_token: 99,
                data: NodeData::None,
            },
            Node {
                tag: NodeTag::ListItem,
                main_token: 99,
                data: NodeData::None,
            },
            Node {
                tag: NodeTag::MdxJsxElement,
                main_token: 99,
                data: NodeData::None,
            },
            Node {
                tag: NodeTag::Link,
                main_token: 99,
                data: NodeData::None,
            },
            Node {
                tag: NodeTag::Frontmatter,
                main_token: 99,
                data: NodeData::None,
            },
            Node {
                tag: NodeTag::Table,
                main_token: 99,
                data: NodeData::None,
            },
        ],
        extra_data: vec![1, 2, 3, 4, 5, 6],
        errors: vec![],
    }
}

#[test]
fn ast_accessors_return_safe_defaults_for_invalid_indices() {
    let ast = malformed_ast();

    assert!(ast.children(999).is_empty());
    assert_eq!("", ast.token_slice(999));
    assert_eq!("", ast.node_source(999));
    let span = ast.node_span(999);
    assert_eq!(0, span.start);
    assert_eq!(0, span.end);
    assert_eq!(None, ast.node_at_offset(1_000));

    let heading = ast.heading_info(999);
    assert_eq!(0, heading.level);
    assert!(ast.code_block_info(999).is_none());

    let list_item = ast.list_item_info(999);
    assert_eq!(None, list_item.checked);

    let jsx = ast.jsx_element(999);
    assert_eq!(0, jsx.name_token);
    assert!(ast.jsx_attribute_views(999).is_none());

    let link = ast.link_info(999);
    assert_eq!(0, link.url_token);
    assert!(ast.link_children(999).is_empty());
    assert!(ast.link_view(999).is_none());
    assert!(ast.image_view(999).is_none());

    let frontmatter = ast.frontmatter_info(999);
    assert_eq!(FrontmatterFormat::Yaml, frontmatter.format);
    assert!(ast.frontmatter_view(999).is_none());
    assert!(ast.expression_info(999).is_none());
    assert!(ast.plain_text_parts(999).is_none());
    assert!(ast.plain_text(999).is_none());
    assert_eq!("", ast.plain_text_children(&[]));

    let table = ast.table_info(999);
    assert_eq!(0, table.num_columns);
    assert!(ast.table_alignments(999).is_empty());

    let range = ast.extra_range(999);
    assert_eq!(0, range.start);
    assert_eq!(0, range.end);
}

#[test]
fn render_and_serialize_do_not_panic_on_malformed_ast_shapes() {
    let ast = malformed_ast();

    let result = catch_unwind(AssertUnwindSafe(|| {
        let json = serialize_tree(&ast);
        let rendered = render(&ast);

        serde_json::from_str::<serde_json::Value>(&json)
            .expect("serialized tree should remain valid JSON");

        (json, rendered)
    }));

    assert!(result.is_ok(), "render/serialize should not panic");
}

#[test]
fn malformed_ast_defaults_are_consistent() {
    let ast = malformed_ast();

    assert_eq!(Vec::<TableAlignment>::new(), ast.table_alignments(6));
    let attrs = ast.jsx_attribute_views(3).expect("JSX node should produce typed attrs");
    assert!(attrs.is_empty());
    assert!(ast.link_children(4).is_empty());
    assert!(ast.link_view(4).is_some());
    assert!(ast.frontmatter_view(5).is_some());
}

use hypernote_mdx::ast::NodeTag;
use hypernote_mdx::{parse, render, serialize_tree};
use serde_json::Value;

fn parsed(source: &str) -> (hypernote_mdx::ast::Ast, Value) {
    let ast = parse(source);
    assert!(
        ast.errors.is_empty(),
        "errors: {:?}",
        ast.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>()
    );

    let json: Value =
        serde_json::from_str(&serialize_tree(&ast)).expect("serialized AST should be valid JSON");
    (ast, json)
}

fn children(value: &Value) -> &[Value] {
    value["children"]
        .as_array()
        .expect("node should have children")
        .as_slice()
}

fn find_jsx_element<'a>(children: &'a [Value], name: &str) -> &'a Value {
    children
        .iter()
        .find(|child| child["type"] == "mdx_jsx_element" && child["name"] == name)
        .expect("missing JSX element")
}

#[test]
fn strikethrough_is_parsed_rendered_and_serialized() {
    let source = "~~gone~~\n";
    let (ast, json) = parsed(source);

    assert!(
        ast.nodes.iter().any(|n| n.tag == NodeTag::Strikethrough),
        "expected a strikethrough node"
    );

    let paragraph = &children(&json)[0];
    let strike = &children(paragraph)[0];
    assert_eq!("strikethrough", strike["type"]);
    assert_eq!("gone", children(strike)[0]["value"]);

    assert_eq!(source, render(&ast));
}

#[test]
fn underscore_emphasis_matches_existing_emphasis_nodes() {
    let source = "_italics_ and __bold__\n";
    let (ast, json) = parsed(source);

    let paragraph = &children(&json)[0];
    let inline = children(paragraph);
    assert_eq!("emphasis", inline[0]["type"]);
    assert_eq!("italics", children(&inline[0])[0]["value"]);
    assert_eq!(" and ", inline[1]["value"]);
    assert_eq!("strong", inline[2]["type"]);
    assert_eq!("bold", children(&inline[2])[0]["value"]);

    assert_eq!("*italics* and **bold**\n", render(&ast));
}

#[test]
fn link_labels_accept_rich_inline_content() {
    let source = "[**bold** label](https://example.com)\n";
    let (ast, json) = parsed(source);

    assert!(
        ast.nodes.iter().any(|n| n.tag == NodeTag::Link),
        "expected a link node"
    );

    let paragraph = &children(&json)[0];
    let link = &children(paragraph)[0];
    let label = children(link);
    assert_eq!("link", link["type"]);
    assert_eq!("https://example.com", link["url"]);
    assert_eq!("strong", label[0]["type"]);
    assert_eq!("bold", children(&label[0])[0]["value"]);
    assert_eq!(" label", label[1]["value"]);

    assert_eq!(source, render(&ast));
}

#[test]
fn image_alt_text_accepts_rich_inline_content() {
    let source = "![*alt* text](image.png)\n";
    let (ast, json) = parsed(source);

    assert!(
        ast.nodes.iter().any(|n| n.tag == NodeTag::Image),
        "expected an image node"
    );

    let paragraph = &children(&json)[0];
    let image = &children(paragraph)[0];
    let alt = children(image);
    assert_eq!("image", image["type"]);
    assert_eq!("image.png", image["url"]);
    assert_eq!("emphasis", alt[0]["type"]);
    assert_eq!("alt", children(&alt[0])[0]["value"]);
    assert_eq!(" text", alt[1]["value"]);

    assert_eq!(source, render(&ast));
}

#[test]
fn blockquotes_preserve_multiple_paragraphs() {
    let source = "> first paragraph\n>\n> second paragraph with ~~gone~~ and [**bold** label](https://example.com)\n";
    let (ast, json) = parsed(source);

    assert!(
        ast.nodes.iter().any(|n| n.tag == NodeTag::Blockquote),
        "expected a blockquote node"
    );

    let quote = &children(&json)[0];
    let blocks = children(quote);
    assert_eq!("blockquote", quote["type"]);
    assert_eq!(2, blocks.len());
    assert_eq!("paragraph", blocks[0]["type"]);
    assert_eq!("first paragraph", children(&blocks[0])[0]["value"]);
    assert_eq!("paragraph", blocks[1]["type"]);
    assert_eq!("strikethrough", children(&blocks[1])[1]["type"]);
    assert_eq!("link", children(&blocks[1])[3]["type"]);

    assert_eq!(source, render(&ast));
}

#[test]
fn list_items_preserve_multiple_paragraphs() {
    let source = "- first paragraph\n\n  second paragraph with _italics_ and [**bold** label](https://example.com)\n";
    let (ast, json) = parsed(source);

    let list = &children(&json)[0];
    let items = children(list);
    assert_eq!("list_unordered", list["type"]);
    assert_eq!(1, items.len());

    let item_blocks = children(&items[0]);
    assert_eq!(2, item_blocks.len());
    assert_eq!("paragraph", item_blocks[0]["type"]);
    assert_eq!("first paragraph", children(&item_blocks[0])[0]["value"]);
    assert_eq!("paragraph", item_blocks[1]["type"]);
    assert_eq!("emphasis", children(&item_blocks[1])[1]["type"]);
    assert_eq!("link", children(&item_blocks[1])[3]["type"]);

    let rendered = render(&ast);
    assert!(rendered.contains("- first paragraph"));
    assert!(rendered.contains("second paragraph with *italics*"));
    assert!(rendered.contains("[**bold** label](https://example.com)"));
}

#[test]
fn mixed_document_keeps_new_markdown_features_inside_mdx_blocks() {
    let source = r#"---
title: Mixed
---

# Ship 🚀

Before {status.label} and `code` with ~~gone~~, _italics_, __bold__, [**bold** label](https://example.com), and ![*alt* text](image.png).

> quoted ~~outside~~
>
> second __quote__ line

- [x] shipped
- first paragraph

  second paragraph with _italics_ and [**bold** label](https://example.com)

| Name | Status |
| :--- | ---: |
| Widget | ~~gone~~ |

<Card title="Status">
~~gone~~ and _italics_ and [**bold** label](https://example.com) and ![*alt* text](image.png)

> quoted __inside__
>
> second ~~line~~

- [x] done
- first paragraph

  second paragraph with _italics_
</Card>
"#;

    let (ast, json) = parsed(source);
    assert!(ast.nodes.iter().any(|n| n.tag == NodeTag::Frontmatter));
    assert!(ast.nodes.iter().any(|n| n.tag == NodeTag::Heading));
    assert!(ast.nodes.iter().any(|n| n.tag == NodeTag::Table));
    assert!(ast.nodes.iter().any(|n| n.tag == NodeTag::MdxJsxElement));

    let root_children = children(&json);
    let card = find_jsx_element(root_children, "Card");
    let card_children = children(card);
    assert!(
        card_children
            .iter()
            .any(|child| child["type"] == "strikethrough"),
        "missing strikethrough inside JSX block"
    );
    assert!(
        card_children
            .iter()
            .any(|child| child["type"] == "emphasis"),
        "missing underscore emphasis inside JSX block"
    );
    assert!(
        card_children.iter().any(|child| child["type"] == "link"),
        "missing rich link inside JSX block"
    );
    assert!(
        card_children.iter().any(|child| child["type"] == "image"),
        "missing rich image alt text inside JSX block"
    );

    let quote = card_children
        .iter()
        .find(|child| child["type"] == "blockquote")
        .expect("missing blockquote inside JSX block");
    assert_eq!(2, children(quote).len());

    let list = card_children
        .iter()
        .find(|child| child["type"] == "list_unordered")
        .expect("missing list inside JSX block");
    let card_items = children(list);
    assert_eq!(2, card_items.len());
    assert_eq!(2, children(&card_items[1]).len());

    let rendered1 = render(&ast);
    let ast2 = parse(&rendered1);
    assert!(
        ast2.errors.is_empty(),
        "round-trip errors: {:?}",
        ast2.errors.iter().map(|e| e.tag.name()).collect::<Vec<_>>()
    );
    let rendered2 = render(&ast2);
    assert!(rendered1.contains("<Card title=\"Status\">"));
    assert!(rendered1.contains("~~gone~~"));
    assert!(rendered1.contains("[**bold** label](https://example.com)"));
    assert!(rendered2.contains("<Card title=\"Status\">"));
    assert!(rendered2.contains("~~gone~~"));
    assert!(rendered2.contains("[**bold** label](https://example.com)"));
}

use hypernote_mdx::ast::{NodeIndex, NodeTag};
use hypernote_mdx::semantic::{
    ExpressionKind, JsxAttributeValue, JsxElementKind, decode_html_entities, decode_jsx_string,
};
use hypernote_mdx::{parse, serialize_tree};
use serde_json::Value;

fn first_node_by_tag(ast: &hypernote_mdx::ast::Ast, tag: NodeTag) -> NodeIndex {
    ast.nodes
        .iter()
        .enumerate()
        .find_map(|(idx, node)| (node.tag == tag).then_some(idx as NodeIndex))
        .expect("expected node tag")
}

fn first_jsx_node_by_name(ast: &hypernote_mdx::ast::Ast, tag: NodeTag, name: &str) -> NodeIndex {
    ast.nodes
        .iter()
        .enumerate()
        .find_map(|(idx, node)| {
            (node.tag == tag
                && ast
                    .jsx_element_view(idx as NodeIndex)
                    .map(|value| value.name)
                    == Some(name))
            .then_some(idx as NodeIndex)
        })
        .expect("expected JSX node")
}

fn json_node_by_type<'a>(value: &'a Value, node_type: &str) -> &'a Value {
    if value.get("type").and_then(Value::as_str) == Some(node_type) {
        return value;
    }

    value["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| json_node_by_type_opt(child, node_type))
        .unwrap_or_else(|| panic!("expected JSON node type {node_type}"))
}

fn json_node_by_type_opt<'a>(value: &'a Value, node_type: &str) -> Option<&'a Value> {
    if value.get("type").and_then(Value::as_str) == Some(node_type) {
        return Some(value);
    }

    value["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| json_node_by_type_opt(child, node_type))
}

fn json_jsx_node_by_name<'a>(value: &'a Value, node_type: &str, name: &str) -> &'a Value {
    if value.get("type").and_then(Value::as_str) == Some(node_type)
        && value.get("name").and_then(Value::as_str) == Some(name)
    {
        return value;
    }

    value["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| json_jsx_node_by_name_opt(child, node_type, name))
        .unwrap_or_else(|| panic!("expected JSON node type {node_type} with name {name}"))
}

fn json_jsx_node_by_name_opt<'a>(
    value: &'a Value,
    node_type: &str,
    name: &str,
) -> Option<&'a Value> {
    if value.get("type").and_then(Value::as_str) == Some(node_type)
        && value.get("name").and_then(Value::as_str) == Some(name)
    {
        return Some(value);
    }

    value["children"]
        .as_array()
        .into_iter()
        .flatten()
        .find_map(|child| json_jsx_node_by_name_opt(child, node_type, name))
}

fn ast_child_type_names(
    ast: &hypernote_mdx::ast::Ast,
    children: &[NodeIndex],
) -> Vec<&'static str> {
    children
        .iter()
        .map(|&idx| ast.nodes[idx as usize].tag.name())
        .collect()
}

fn json_child_type_names(node: &Value) -> Vec<&str> {
    node["children"]
        .as_array()
        .expect("expected children array")
        .iter()
        .map(|child| child["type"].as_str().expect("expected child type"))
        .collect()
}

#[test]
fn code_block_info_matches_serialized_tree() {
    let cases = [
        (
            "```rust\nfn main() {}\n```\n",
            Some("rust"),
            "fn main() {}\n",
        ),
        ("```\nplain text\n```\n", None, "plain text\n"),
    ];

    for (source, expected_lang, expected_code) in cases {
        let ast = parse(source);
        let node = first_node_by_tag(&ast, NodeTag::CodeBlock);
        let info = ast.code_block_info(node).expect("expected code block info");
        let json: Value =
            serde_json::from_str(&serialize_tree(&ast)).expect("serialized tree should parse");
        let json_node = json_node_by_type(&json, "code_block");

        assert_eq!(expected_lang, info.lang);
        assert_eq!(expected_code, info.code);
        assert_eq!(json_node["lang"].as_str(), info.lang);
        assert_eq!(json_node["value"].as_str().unwrap_or(""), info.code);
    }
}

#[test]
fn link_and_image_views_match_serialized_tree() {
    let source = "[**bold** label](https://example.com)\n\n![*alt* text](image.png)\n";
    let ast = parse(source);
    let json: Value =
        serde_json::from_str(&serialize_tree(&ast)).expect("serialized tree should parse");

    let link_node = first_node_by_tag(&ast, NodeTag::Link);
    let link = ast.link_view(link_node).expect("expected link view");
    let link_json = json_node_by_type(&json, "link");
    assert_eq!("https://example.com", link.url);
    assert_eq!(link_json["url"].as_str().unwrap_or(""), link.url);
    assert_eq!(
        ast_child_type_names(&ast, link.label_children),
        json_child_type_names(link_json)
    );

    let image_node = first_node_by_tag(&ast, NodeTag::Image);
    let image = ast.image_view(image_node).expect("expected image view");
    let image_json = json_node_by_type(&json, "image");
    assert_eq!("image.png", image.url);
    assert_eq!(image_json["url"].as_str().unwrap_or(""), image.url);
    assert_eq!(
        ast_child_type_names(&ast, image.alt_children),
        json_child_type_names(image_json)
    );
}

#[test]
fn frontmatter_and_expression_views_match_serialized_tree() {
    let yaml_ast = parse("---\ntitle: Hello\n---\n");
    let yaml_node = first_node_by_tag(&yaml_ast, NodeTag::Frontmatter);
    let yaml = yaml_ast
        .frontmatter_view(yaml_node)
        .expect("expected frontmatter view");
    let yaml_json: Value =
        serde_json::from_str(&serialize_tree(&yaml_ast)).expect("serialized tree should parse");
    let yaml_json_node = json_node_by_type(&yaml_json, "frontmatter");

    assert_eq!(hypernote_mdx::ast::FrontmatterFormat::Yaml, yaml.format);
    assert_eq!("title: Hello", yaml.value);
    assert_eq!("yaml", yaml_json_node["format"].as_str().unwrap_or(""));
    assert_eq!(yaml.value, yaml_json_node["value"].as_str().unwrap_or(""));

    let expr_ast = parse("Before {state.count} after\n");
    let expr_node = first_node_by_tag(&expr_ast, NodeTag::MdxTextExpression);
    let expr = expr_ast
        .expression_info(expr_node)
        .expect("expected expression info");
    let expr_json: Value =
        serde_json::from_str(&serialize_tree(&expr_ast)).expect("serialized tree should parse");
    let expr_json_node = json_node_by_type(&expr_json, "mdx_text_expression");

    assert_eq!(ExpressionKind::Text, expr.kind);
    assert_eq!("state.count", expr.value);
    assert_eq!(expr.value, expr_json_node["value"].as_str().unwrap_or(""));
}

#[test]
fn jsx_attribute_views_decode_values_and_match_serialized_tree() {
    let source =
        "<Widget label=\"Fish &amp; Chips\" count=4 enabled visible=false expr={state.count} />";
    let ast = parse(source);
    let node = first_node_by_tag(&ast, NodeTag::MdxJsxSelfClosing);
    let attrs = ast
        .jsx_attribute_views(node)
        .expect("expected JSX attribute views");
    let json: Value =
        serde_json::from_str(&serialize_tree(&ast)).expect("serialized tree should parse");
    let json_node = json_node_by_type(&json, "mdx_jsx_self_closing");
    let json_attrs = json_node["attributes"]
        .as_array()
        .expect("expected attributes array");

    assert_eq!(attrs.len(), json_attrs.len());

    for (attr, json_attr) in attrs.iter().zip(json_attrs.iter()) {
        assert_eq!(attr.name, json_attr["name"].as_str().unwrap_or(""));

        match &attr.value {
            JsxAttributeValue::String(value) => {
                assert_eq!("string", json_attr["value_type"].as_str().unwrap_or(""));
                assert_eq!(value, json_attr["value"].as_str().unwrap_or(""));
            }
            JsxAttributeValue::Number(value) => {
                assert_eq!("number", json_attr["value_type"].as_str().unwrap_or(""));
                assert_eq!(Some(*value), json_attr["value"].as_f64());
            }
            JsxAttributeValue::InvalidNumber(value) => {
                assert_eq!("number", json_attr["value_type"].as_str().unwrap_or(""));
                assert_eq!(*value, json_attr["value"].as_str().unwrap_or(""));
            }
            JsxAttributeValue::Boolean(value) => {
                assert_eq!("boolean", json_attr["value_type"].as_str().unwrap_or(""));
                assert_eq!(Some(*value), json_attr["value"].as_bool());
            }
            JsxAttributeValue::Expression(value) => {
                assert_eq!("expression", json_attr["value_type"].as_str().unwrap_or(""));
                assert_eq!(*value, json_attr["value"].as_str().unwrap_or(""));
            }
        }
    }
}

#[test]
fn jsx_element_view_matches_serialized_tree() {
    let source =
        "<Card title=\"Inbox\"><Body>hello {state.count}</Body></Card>\n<Widget enabled />\n";
    let ast = parse(source);
    let json: Value =
        serde_json::from_str(&serialize_tree(&ast)).expect("serialized tree should parse");

    let card_node = first_jsx_node_by_name(&ast, NodeTag::MdxJsxElement, "Card");
    let card = ast
        .jsx_element_view(card_node)
        .expect("expected JSX element view");
    let card_json = json_jsx_node_by_name(&json, "mdx_jsx_element", "Card");

    assert_eq!("Card", card.name);
    assert_eq!(JsxElementKind::Normal, card.kind);
    assert_eq!(1, card.attrs.len());
    assert_eq!(1, card.children.len());
    assert_eq!(
        ast_child_type_names(&ast, card.children),
        json_child_type_names(card_json)
    );
    assert_eq!(card.name, card_json["name"].as_str().unwrap_or(""));

    let widget_node = first_jsx_node_by_name(&ast, NodeTag::MdxJsxSelfClosing, "Widget");
    let widget = ast
        .jsx_element_view(widget_node)
        .expect("expected self-closing JSX element view");
    let widget_json = json_jsx_node_by_name(&json, "mdx_jsx_self_closing", "Widget");

    assert_eq!("Widget", widget.name);
    assert_eq!(JsxElementKind::SelfClosing, widget.kind);
    assert_eq!(1, widget.attrs.len());
    assert!(widget.children.is_empty());
    assert_eq!(widget.name, widget_json["name"].as_str().unwrap_or(""));
}

#[test]
fn semantic_accessors_return_none_on_wrong_node() {
    let ast = parse("# Heading\n");
    let heading = first_node_by_tag(&ast, NodeTag::Heading);

    assert!(ast.code_block_info(heading).is_none());
    assert!(ast.link_view(heading).is_none());
    assert!(ast.image_view(heading).is_none());
    assert!(ast.expression_info(heading).is_none());
    assert!(ast.frontmatter_view(heading).is_none());
    assert!(ast.jsx_attribute_views(heading).is_none());
    assert!(ast.jsx_element_view(heading).is_none());
}

#[test]
fn semantic_decode_helpers_are_public() {
    assert_eq!("Fish & Chips", decode_html_entities("Fish &amp; Chips"));
    assert_eq!(
        "Line 1\nLine 2 & more",
        decode_jsx_string("\"Line 1\\nLine 2 &amp; more\"")
    );
}

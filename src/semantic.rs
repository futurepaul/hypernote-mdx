use crate::ast::{
    Ast, FrontmatterFormat, JsxAttributeType, NodeData, NodeIndex, NodeTag, TokenIndex,
};
use crate::token::Tag as TokenTag;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CodeBlockInfo<'a> {
    pub lang: Option<&'a str>,
    pub code: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct LinkInfo<'a> {
    pub label_children: &'a [NodeIndex],
    pub url: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImageInfo<'a> {
    pub alt_children: &'a [NodeIndex],
    pub url: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionKind {
    Text,
    Flow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlainTextPart<'a> {
    Text(&'a str),
    Code(&'a str),
    HardBreak,
    Expression {
        kind: ExpressionKind,
        source: &'a str,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpressionTextPolicy<'a> {
    Omit,
    Source,
    Placeholder(&'a str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlainTextOptions<'a> {
    pub expression_policy: ExpressionTextPolicy<'a>,
}

impl<'a> Default for PlainTextOptions<'a> {
    fn default() -> Self {
        Self {
            expression_policy: ExpressionTextPolicy::Source,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ExpressionInfo<'a> {
    pub kind: ExpressionKind,
    pub value: &'a str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum JsxAttributeValue<'a> {
    String(String),
    Number(f64),
    InvalidNumber(&'a str),
    Boolean(bool),
    Expression(&'a str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct JsxAttributeView<'a> {
    pub name: &'a str,
    pub value: JsxAttributeValue<'a>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FrontmatterInfoView<'a> {
    pub format: FrontmatterFormat,
    pub value: &'a str,
}

pub fn decode_html_entities(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

pub fn decode_jsx_string(raw: &str) -> String {
    let trimmed = raw.trim();
    let inner = if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        &trimmed[1..trimmed.len() - 1]
    } else {
        trimmed
    };

    let mut output = String::with_capacity(inner.len());
    let mut escaped = false;
    for ch in inner.chars() {
        if escaped {
            match ch {
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                '\\' => output.push('\\'),
                '"' => output.push('"'),
                '\'' => output.push('\''),
                other => {
                    output.push('\\');
                    output.push(other);
                }
            }
            escaped = false;
            continue;
        }

        if ch == '\\' {
            escaped = true;
        } else {
            output.push(ch);
        }
    }

    if escaped {
        output.push('\\');
    }

    decode_html_entities(&output)
}

pub(crate) fn code_block_info(ast: &Ast, node_idx: NodeIndex) -> Option<CodeBlockInfo<'_>> {
    let node = ast.nodes.get(node_idx as usize)?;
    if node.tag != NodeTag::CodeBlock {
        return None;
    }

    let fence_token = node.main_token;
    let lang = match ast.token_tags.get(fence_token.saturating_add(1) as usize) {
        Some(TokenTag::Text) => {
            let trimmed = ast.token_slice(fence_token.saturating_add(1)).trim();
            (!trimmed.is_empty()).then_some(trimmed)
        }
        _ => None,
    };

    Some(CodeBlockInfo {
        lang,
        code: code_block_content_from_fence(ast, fence_token),
    })
}

pub(crate) fn link_view(ast: &Ast, node_idx: NodeIndex) -> Option<LinkInfo<'_>> {
    let node = ast.nodes.get(node_idx as usize)?;
    if node.tag != NodeTag::Link {
        return None;
    }

    let info = ast.link_info(node_idx);
    Some(LinkInfo {
        label_children: ast.link_children(node_idx),
        url: ast.token_slice(info.url_token),
    })
}

pub(crate) fn image_view(ast: &Ast, node_idx: NodeIndex) -> Option<ImageInfo<'_>> {
    let node = ast.nodes.get(node_idx as usize)?;
    if node.tag != NodeTag::Image {
        return None;
    }

    let info = ast.link_info(node_idx);
    Some(ImageInfo {
        alt_children: ast.link_children(node_idx),
        url: ast.token_slice(info.url_token),
    })
}

pub(crate) fn frontmatter_view(ast: &Ast, node_idx: NodeIndex) -> Option<FrontmatterInfoView<'_>> {
    let node = ast.nodes.get(node_idx as usize)?;
    if node.tag != NodeTag::Frontmatter {
        return None;
    }

    let info = ast.frontmatter_info(node_idx);
    Some(FrontmatterInfoView {
        format: info.format,
        value: trimmed_token_range_source(ast, info.content_start, info.content_end),
    })
}

pub(crate) fn expression_info(ast: &Ast, node_idx: NodeIndex) -> Option<ExpressionInfo<'_>> {
    let node = ast.nodes.get(node_idx as usize)?;
    let kind = match node.tag {
        NodeTag::MdxTextExpression => ExpressionKind::Text,
        NodeTag::MdxFlowExpression => ExpressionKind::Flow,
        _ => return None,
    };

    let value = match node.data {
        NodeData::Extra(idx) => {
            let range = ast.extra_range(idx);
            trimmed_token_range_source(ast, range.start, range.end)
        }
        _ => "",
    };

    Some(ExpressionInfo { kind, value })
}

pub(crate) fn jsx_attribute_views(ast: &Ast, node_idx: NodeIndex) -> Option<Vec<JsxAttributeView<'_>>> {
    let node = ast.nodes.get(node_idx as usize)?;
    if node.tag != NodeTag::MdxJsxElement && node.tag != NodeTag::MdxJsxSelfClosing {
        return None;
    }

    let attrs = ast
        .jsx_attributes(node_idx)
        .into_iter()
        .map(|attr| {
            let name = ast.token_slice(attr.name_token).trim();
            let value = match attr.value_type {
                JsxAttributeType::String => JsxAttributeValue::String(
                    attr.value_token
                        .map(|token| decode_jsx_string(ast.token_slice(token)))
                        .unwrap_or_default(),
                ),
                JsxAttributeType::Number => {
                    let raw = attr
                        .value_token
                        .map(|token| ast.token_slice(token).trim())
                        .unwrap_or("");
                    match raw.parse::<f64>() {
                        Ok(parsed) => JsxAttributeValue::Number(parsed),
                        Err(_) => JsxAttributeValue::InvalidNumber(raw),
                    }
                }
                JsxAttributeType::Boolean => {
                    let value = attr
                        .value_token
                        .map(|token| ast.token_slice(token).trim() == "true")
                        .unwrap_or(true);
                    JsxAttributeValue::Boolean(value)
                }
                JsxAttributeType::Expression => JsxAttributeValue::Expression(
                    attr.value_token
                        .map(|token| ast.token_slice(token).trim())
                        .unwrap_or(""),
                ),
            };

            JsxAttributeView {
                name,
                value,
            }
        })
        .collect();

    Some(attrs)
}

pub(crate) fn jsx_attribute_type_name(value: &JsxAttributeValue<'_>) -> &'static str {
    match value {
        JsxAttributeValue::String(_) => "string",
        JsxAttributeValue::Number(_) | JsxAttributeValue::InvalidNumber(_) => "number",
        JsxAttributeValue::Boolean(_) => "boolean",
        JsxAttributeValue::Expression(_) => "expression",
    }
}

pub(crate) fn plain_text_parts(ast: &Ast, node_idx: NodeIndex) -> Option<Vec<PlainTextPart<'_>>> {
    ast.nodes.get(node_idx as usize)?;

    let mut out = Vec::new();
    collect_plain_text_parts(ast, node_idx, &mut out);
    Some(out)
}

pub(crate) fn plain_text_parts_children<'a>(
    ast: &'a Ast,
    children: &[NodeIndex],
) -> Vec<PlainTextPart<'a>> {
    let mut out = Vec::new();
    collect_plain_text_children(ast, children, &mut out, ChildSeparator::None);
    out
}

pub(crate) fn plain_text_with_options(
    ast: &Ast,
    node_idx: NodeIndex,
    options: &PlainTextOptions<'_>,
) -> Option<String> {
    let parts = plain_text_parts(ast, node_idx)?;
    Some(render_plain_text_parts(&parts, options))
}

pub(crate) fn plain_text_children_with_options(
    ast: &Ast,
    children: &[NodeIndex],
    options: &PlainTextOptions<'_>,
) -> String {
    let parts = plain_text_parts_children(ast, children);
    render_plain_text_parts(&parts, options)
}

fn code_block_content_from_fence<'a>(ast: &'a Ast, fence_token: TokenIndex) -> &'a str {
    let mut code_start = u32::MAX;
    let mut code_end = 0;
    let mut in_code = false;
    let mut token = fence_token;

    while let Some(tag) = ast.token_tags.get(token as usize).copied() {
        if tag == TokenTag::CodeFenceEnd {
            break;
        }
        if tag == TokenTag::Newline && !in_code {
            in_code = true;
            token = token.saturating_add(1);
            continue;
        }
        if in_code {
            let start = ast
                .token_starts
                .get(token as usize)
                .copied()
                .unwrap_or(ast.source.len() as u32);
            let end = token_end(ast, token);
            code_start = code_start.min(start);
            code_end = code_end.max(end);
        }
        token = token.saturating_add(1);
    }

    source_slice(ast, code_start, code_end)
}

fn trimmed_token_range_source<'a>(ast: &'a Ast, start_token: u32, end_token: u32) -> &'a str {
    token_range_source(ast, start_token, end_token).trim()
}

fn token_range_source<'a>(ast: &'a Ast, start_token: u32, end_token: u32) -> &'a str {
    if start_token >= end_token {
        return "";
    }

    let mut start = u32::MAX;
    let mut end = 0;
    for token in start_token..end_token {
        let Some(&token_start) = ast.token_starts.get(token as usize) else {
            continue;
        };
        start = start.min(token_start);
        end = end.max(token_end(ast, token));
    }

    source_slice(ast, start, end)
}

fn token_end(ast: &Ast, token: TokenIndex) -> u32 {
    if (token as usize + 1) < ast.token_starts.len() {
        ast.token_starts[token as usize + 1]
    } else {
        ast.source.len() as u32
    }
}

fn source_slice(ast: &Ast, start: u32, end: u32) -> &str {
    if start >= end {
        return "";
    }

    ast.source.get(start as usize..end as usize).unwrap_or("")
}

#[derive(Clone, Copy)]
enum ChildSeparator<'a> {
    None,
    HardBreak,
    Text(&'a str),
}

fn collect_plain_text_parts<'a>(
    ast: &'a Ast,
    node_idx: NodeIndex,
    out: &mut Vec<PlainTextPart<'a>>,
) -> bool {
    let Some(node) = ast.nodes.get(node_idx as usize) else {
        return false;
    };

    match node.tag {
        NodeTag::Text => {
            let text = ast.token_slice(node.main_token);
            if text.is_empty() {
                false
            } else {
                out.push(PlainTextPart::Text(text));
                true
            }
        }
        NodeTag::CodeInline => {
            let text = match node.data {
                NodeData::Token(token) => ast.token_slice(token),
                _ => "",
            };
            if text.is_empty() {
                false
            } else {
                out.push(PlainTextPart::Code(text));
                true
            }
        }
        NodeTag::CodeBlock => {
            let text = code_block_info(ast, node_idx).map(|info| info.code).unwrap_or("");
            if text.is_empty() {
                false
            } else {
                out.push(PlainTextPart::Code(text));
                true
            }
        }
        NodeTag::HardBreak => {
            out.push(PlainTextPart::HardBreak);
            true
        }
        NodeTag::MdxTextExpression | NodeTag::MdxFlowExpression => {
            if let Some(info) = expression_info(ast, node_idx) {
                out.push(PlainTextPart::Expression {
                    kind: info.kind,
                    source: info.value,
                });
                true
            } else {
                false
            }
        }
        NodeTag::Link => {
            if let Some(info) = link_view(ast, node_idx) {
                if !info.label_children.is_empty() {
                    let had_children =
                        collect_plain_text_children(ast, info.label_children, out, ChildSeparator::None);
                    if had_children {
                        return true;
                    }
                }
                if !info.url.is_empty() {
                    out.push(PlainTextPart::Text(info.url));
                    return true;
                }
            }
            false
        }
        NodeTag::Image => image_view(ast, node_idx)
            .map(|info| collect_plain_text_children(ast, info.alt_children, out, ChildSeparator::None))
            .unwrap_or(false),
        NodeTag::Document
        | NodeTag::Blockquote
        | NodeTag::ListUnordered
        | NodeTag::ListOrdered
        | NodeTag::ListItem
        | NodeTag::Table => {
            collect_plain_text_children(ast, ast.children(node_idx), out, ChildSeparator::HardBreak)
        }
        NodeTag::MdxJsxElement | NodeTag::MdxJsxFragment => {
            collect_plain_text_children_smart_jsx(ast, ast.children(node_idx), out)
        }
        NodeTag::TableRow => {
            collect_plain_text_children(ast, ast.children(node_idx), out, ChildSeparator::Text("\t"))
        }
        NodeTag::Heading
        | NodeTag::Paragraph
        | NodeTag::Strong
        | NodeTag::Emphasis
        | NodeTag::Strikethrough
        | NodeTag::TableCell => {
            collect_plain_text_children(ast, ast.children(node_idx), out, ChildSeparator::None)
        }
        NodeTag::Hr
        | NodeTag::Frontmatter
        | NodeTag::MdxJsxSelfClosing
        | NodeTag::MdxJsxAttribute
        | NodeTag::MdxEsmImport
        | NodeTag::MdxEsmExport => false,
    }
}

fn collect_plain_text_children<'a>(
    ast: &'a Ast,
    children: &[NodeIndex],
    out: &mut Vec<PlainTextPart<'a>>,
    separator: ChildSeparator<'a>,
) -> bool {
    let mut wrote_any = false;

    for &child in children {
        let mut child_parts = Vec::new();
        if !collect_plain_text_parts(ast, child, &mut child_parts) {
            continue;
        }

        if wrote_any {
            push_separator(out, separator);
        }

        out.extend(child_parts);
        wrote_any = true;
    }

    wrote_any
}

fn collect_plain_text_children_smart_jsx<'a>(
    ast: &'a Ast,
    children: &[NodeIndex],
    out: &mut Vec<PlainTextPart<'a>>,
) -> bool {
    let mut wrote_any = false;
    let mut previous_child: Option<NodeIndex> = None;

    for &child in children {
        let mut child_parts = Vec::new();
        if !collect_plain_text_parts(ast, child, &mut child_parts) {
            continue;
        }

        if let Some(previous_child) = previous_child {
            push_separator(out, jsx_child_separator(ast, previous_child, child));
        }

        out.extend(child_parts);
        wrote_any = true;
        previous_child = Some(child);
    }

    wrote_any
}

fn push_separator<'a>(out: &mut Vec<PlainTextPart<'a>>, separator: ChildSeparator<'a>) {
    match separator {
        ChildSeparator::None => {}
        ChildSeparator::HardBreak => out.push(PlainTextPart::HardBreak),
        ChildSeparator::Text(text) => out.push(PlainTextPart::Text(text)),
    }
}

fn jsx_child_separator(ast: &Ast, previous_child: NodeIndex, next_child: NodeIndex) -> ChildSeparator<'static> {
    let previous_tag = ast
        .nodes
        .get(previous_child as usize)
        .map(|node| node.tag)
        .unwrap_or(NodeTag::Text);
    let next_tag = ast
        .nodes
        .get(next_child as usize)
        .map(|node| node.tag)
        .unwrap_or(NodeTag::Text);

    if is_inline_plain_text_tag(previous_tag) && is_inline_plain_text_tag(next_tag) {
        ChildSeparator::None
    } else {
        ChildSeparator::HardBreak
    }
}

fn is_inline_plain_text_tag(tag: NodeTag) -> bool {
    matches!(
        tag,
        NodeTag::Text
            | NodeTag::Strong
            | NodeTag::Emphasis
            | NodeTag::Strikethrough
            | NodeTag::CodeInline
            | NodeTag::Link
            | NodeTag::Image
            | NodeTag::HardBreak
            | NodeTag::MdxTextExpression
            | NodeTag::MdxFlowExpression
    )
}

fn render_plain_text_parts(parts: &[PlainTextPart<'_>], options: &PlainTextOptions<'_>) -> String {
    let mut output = String::new();

    for part in parts {
        match part {
            PlainTextPart::Text(value) | PlainTextPart::Code(value) => output.push_str(value),
            PlainTextPart::HardBreak => output.push('\n'),
            PlainTextPart::Expression { source, .. } => match options.expression_policy {
                ExpressionTextPolicy::Omit => {}
                ExpressionTextPolicy::Source => output.push_str(source),
                ExpressionTextPolicy::Placeholder(value) => output.push_str(value),
            },
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::NodeTag;
    use crate::parse;
    use crate::tree_builder::serialize_tree;
    use serde_json::Value;

    fn first_node_by_tag(ast: &Ast, tag: NodeTag) -> NodeIndex {
        ast.nodes
            .iter()
            .enumerate()
            .find_map(|(idx, node)| (node.tag == tag).then_some(idx as NodeIndex))
            .expect("expected node tag")
    }

    fn json_node_by_type<'a>(value: &'a Value, node_type: &str) -> &'a Value {
        if value.get("type").and_then(Value::as_str) == Some(node_type) {
            return value;
        }

        if let Some(children) = value.get("children").and_then(Value::as_array) {
            for child in children {
                if let Some(found) = json_node_by_type_opt(child, node_type) {
                    return found;
                }
            }
        }

        panic!("expected JSON node type {node_type}");
    }

    fn json_node_by_type_opt<'a>(value: &'a Value, node_type: &str) -> Option<&'a Value> {
        if value.get("type").and_then(Value::as_str) == Some(node_type) {
            return Some(value);
        }

        value.get("children")
            .and_then(Value::as_array)
            .and_then(|children| {
                children
                    .iter()
                    .find_map(|child| json_node_by_type_opt(child, node_type))
            })
    }

    fn ast_child_type_names(ast: &Ast, children: &[NodeIndex]) -> Vec<&'static str> {
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
            ("```rust\nfn main() {}\n```\n", Some("rust"), "fn main() {}\n"),
            ("```\nplain text\n```\n", None, "plain text\n"),
        ];

        for (source, expected_lang, expected_code) in cases {
            let ast = parse(source);
            let node = first_node_by_tag(&ast, NodeTag::CodeBlock);
            let info = code_block_info(&ast, node).expect("expected code block info");
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
        let link = link_view(&ast, link_node).expect("expected link view");
        let link_json = json_node_by_type(&json, "link");
        assert_eq!("https://example.com", link.url);
        assert_eq!(link_json["url"].as_str().unwrap_or(""), link.url);
        assert_eq!(
            ast_child_type_names(&ast, link.label_children),
            json_child_type_names(link_json)
        );

        let image_node = first_node_by_tag(&ast, NodeTag::Image);
        let image = image_view(&ast, image_node).expect("expected image view");
        let image_json = json_node_by_type(&json, "image");
        assert_eq!("image.png", image.url);
        assert_eq!(image_json["url"].as_str().unwrap_or(""), image.url);
        assert_eq!(
            ast_child_type_names(&ast, image.alt_children),
            json_child_type_names(image_json)
        );
    }

    #[test]
    fn frontmatter_view_matches_serialized_tree() {
        let cases = [
            ("---\ntitle: Hello\n---\n", FrontmatterFormat::Yaml, "title: Hello"),
            (
                "```hnmd\n{\"title\":\"Hello\"}\n```\n",
                FrontmatterFormat::Json,
                "{\"title\":\"Hello\"}",
            ),
        ];

        for (source, expected_format, expected_value) in cases {
            let ast = parse(source);
            let node = first_node_by_tag(&ast, NodeTag::Frontmatter);
            let info = frontmatter_view(&ast, node).expect("expected frontmatter view");
            let json: Value =
                serde_json::from_str(&serialize_tree(&ast)).expect("serialized tree should parse");
            let json_node = json_node_by_type(&json, "frontmatter");

            assert_eq!(expected_format, info.format);
            assert_eq!(expected_value, info.value);
            let format = match info.format {
                FrontmatterFormat::Yaml => "yaml",
                FrontmatterFormat::Json => "json",
            };
            assert_eq!(format, json_node["format"].as_str().unwrap_or(""));
            assert_eq!(info.value, json_node["value"].as_str().unwrap_or(""));
        }
    }

    #[test]
    fn expression_info_matches_serialized_tree() {
        let source = "Before {state.count} after\n";
        let ast = parse(source);
        let node = first_node_by_tag(&ast, NodeTag::MdxTextExpression);
        let info = expression_info(&ast, node).expect("expected expression info");
        let json: Value =
            serde_json::from_str(&serialize_tree(&ast)).expect("serialized tree should parse");
        let json_node = json_node_by_type(&json, "mdx_text_expression");

        assert_eq!(ExpressionKind::Text, info.kind);
        assert_eq!("state.count", info.value);
        assert_eq!(info.value, json_node["value"].as_str().unwrap_or(""));
    }

    #[test]
    fn jsx_attribute_views_match_serialized_tree() {
        let source =
            "<Widget label=\"Fish &amp; Chips\" count=4 enabled visible=false expr={state.count} />";
        let ast = parse(source);
        let node = first_node_by_tag(&ast, NodeTag::MdxJsxSelfClosing);
        let attrs = jsx_attribute_views(&ast, node).expect("expected JSX attribute views");
        let json: Value =
            serde_json::from_str(&serialize_tree(&ast)).expect("serialized tree should parse");
        let json_node = json_node_by_type(&json, "mdx_jsx_self_closing");
        let json_attrs = json_node["attributes"]
            .as_array()
            .expect("expected attributes array");

        assert_eq!(attrs.len(), json_attrs.len());

        for (attr, json_attr) in attrs.iter().zip(json_attrs.iter()) {
            assert_eq!(attr.name, json_attr["name"].as_str().unwrap_or(""));
            assert_eq!(
                jsx_attribute_type_name(&attr.value),
                json_attr["value_type"].as_str().unwrap_or("")
            );

            match &attr.value {
                JsxAttributeValue::String(value) => {
                    assert_eq!(value, json_attr["value"].as_str().unwrap_or(""));
                }
                JsxAttributeValue::Number(value) => {
                    assert_eq!(Some(*value), json_attr["value"].as_f64());
                }
                JsxAttributeValue::InvalidNumber(value) => {
                    assert_eq!(*value, json_attr["value"].as_str().unwrap_or(""));
                }
                JsxAttributeValue::Boolean(value) => {
                    assert_eq!(Some(*value), json_attr["value"].as_bool());
                }
                JsxAttributeValue::Expression(value) => {
                    assert_eq!(*value, json_attr["value"].as_str().unwrap_or(""));
                }
            }
        }
    }
}

use crate::ast::*;
use crate::semantic::{
    JsxAttributeValue, code_block_info, expression_info, frontmatter_view, image_view,
    jsx_attribute_type_name, jsx_element_view, link_view,
};
use std::fmt::Write;

/// Write a JSON-escaped string
fn write_json_string(output: &mut String, s: &str) {
    const HEX: &[u8; 16] = b"0123456789abcdef";

    output.push('"');
    let bytes = s.as_bytes();
    let mut chunk_start = 0usize;

    for (i, &byte) in bytes.iter().enumerate() {
        let escaped = match byte {
            b'"' => Some("\\\""),
            b'\\' => Some("\\\\"),
            b'\n' => Some("\\n"),
            b'\r' => Some("\\r"),
            b'\t' => Some("\\t"),
            _ => None,
        };

        if let Some(escape) = escaped {
            if chunk_start < i {
                output.push_str(&s[chunk_start..i]);
            }
            output.push_str(escape);
            chunk_start = i + 1;
            continue;
        }

        if matches!(byte, 0x00..=0x08 | 0x0b | 0x0c | 0x0e..=0x1f) {
            if chunk_start < i {
                output.push_str(&s[chunk_start..i]);
            }
            output.push_str("\\u00");
            output.push(HEX[(byte >> 4) as usize] as char);
            output.push(HEX[(byte & 0x0f) as usize] as char);
            chunk_start = i + 1;
        }
    }

    if chunk_start < s.len() {
        output.push_str(&s[chunk_start..]);
    }
    output.push('"');
}

fn estimated_serialized_capacity(ast: &Ast) -> usize {
    ast.source.len() + ast.nodes.len() * 48 + ast.errors.len() * 64 + 128
}
pub struct SerializeOptions {
    pub include_positions: bool,
}

impl Default for SerializeOptions {
    fn default() -> Self {
        SerializeOptions {
            include_positions: false,
        }
    }
}

/// Serialize the AST as a nested tree structure to JSON
pub fn serialize_tree(ast: &Ast) -> String {
    serialize_tree_with_options(ast, &SerializeOptions::default())
}

/// Serialize the AST with options
pub fn serialize_tree_with_options(ast: &Ast, options: &SerializeOptions) -> String {
    let mut output = String::with_capacity(estimated_serialized_capacity(ast));

    output.push_str("{\"schema\":{\"name\":");
    write_json_string(&mut output, AST_SCHEMA_NAME);
    output.push_str(",\"version\":");
    write!(output, "{}", AST_SCHEMA_VERSION)
        .expect("writing schema version into a String cannot fail");
    output.push_str("},\"type\":\"root\",\"children\":[");

    // Find the document node
    let doc_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::Document)
        .map(|(i, _)| i as NodeIndex);

    if let Some(idx) = doc_idx {
        let children = ast.children(idx);
        for (i, &child_idx) in children.iter().enumerate() {
            if i > 0 {
                output.push(',');
            }
            serialize_node(ast, child_idx, &mut output, options);
        }
    }

    output.push_str("],\"source\":");
    write_json_string(&mut output, &ast.source);

    // Include errors
    output.push_str(",\"errors\":[");
    for (i, err) in ast.errors.iter().enumerate() {
        if i > 0 {
            output.push(',');
        }
        output.push('{');
        output.push_str("\"tag\":\"");
        output.push_str(err.tag.name());
        output.push('"');
        write!(output, ",\"token\":{}", err.token)
            .expect("writing error token into a String cannot fail");
        write!(output, ",\"byte_offset\":{}", err.byte_offset)
            .expect("writing error byte offset into a String cannot fail");
        output.push_str(",\"message\":");
        write_json_string(&mut output, err.tag.message());
        output.push('}');
    }
    output.push_str("]}");

    output
}

fn serialize_node(ast: &Ast, node_idx: NodeIndex, output: &mut String, options: &SerializeOptions) {
    let node = &ast.nodes[node_idx as usize];

    output.push('{');
    output.push_str("\"type\":\"");
    output.push_str(node.tag.name());
    output.push('"');

    if options.include_positions {
        let span = ast.node_span(node_idx);
        write!(
            output,
            ",\"position\":{{\"start\":{},\"end\":{}}}",
            span.start, span.end
        )
        .expect("writing node position into a String cannot fail");
    }

    match node.tag {
        NodeTag::Heading => {
            let info = ast.heading_info(node_idx);
            write!(output, ",\"level\":{}", info.level)
                .expect("writing heading level into a String cannot fail");
            output.push_str(",\"children\":[");
            let children =
                &ast.extra_data[info.children_start as usize..info.children_end as usize];
            for (i, &child_raw) in children.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_node(ast, child_raw, output, options);
            }
            output.push(']');
        }

        NodeTag::Text => {
            let text = ast.token_slice(node.main_token);
            output.push_str(",\"value\":");
            write_json_string(output, text);
        }

        NodeTag::CodeBlock => {
            let info = code_block_info(ast, node_idx);
            output.push_str(",\"lang\":");
            if let Some(l) = info.and_then(|value| value.lang) {
                write_json_string(output, l);
            } else {
                output.push_str("null");
            }

            output.push_str(",\"value\":");
            write_json_string(output, info.map(|value| value.code).unwrap_or(""));
        }

        NodeTag::CodeInline => {
            if let NodeData::Token(content_token) = node.data {
                let text = ast.token_slice(content_token);
                output.push_str(",\"value\":");
                write_json_string(output, text);
            }
        }

        NodeTag::Link | NodeTag::Image => {
            output.push_str(",\"url\":");
            let url = if node.tag == NodeTag::Link {
                link_view(ast, node_idx)
                    .map(|value| value.url)
                    .unwrap_or("")
            } else {
                image_view(ast, node_idx)
                    .map(|value| value.url)
                    .unwrap_or("")
            };
            write_json_string(output, url);

            output.push_str(",\"children\":[");
            let children = if node.tag == NodeTag::Link {
                link_view(ast, node_idx)
                    .map(|value| value.label_children)
                    .unwrap_or(&[])
            } else {
                image_view(ast, node_idx)
                    .map(|value| value.alt_children)
                    .unwrap_or(&[])
            };
            for (i, &child_idx) in children.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_node(ast, child_idx, output, options);
            }
            output.push(']');
        }

        NodeTag::MdxJsxElement | NodeTag::MdxJsxSelfClosing => {
            let element = jsx_element_view(ast, node_idx);

            output.push_str(",\"name\":");
            write_json_string(
                output,
                element.as_ref().map(|value| value.name).unwrap_or(""),
            );

            // Serialize attributes
            output.push_str(",\"attributes\":[");
            let attrs = element
                .as_ref()
                .map(|value| value.attrs.as_slice())
                .unwrap_or(&[]);
            for (i, attr) in attrs.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                output.push('{');

                output.push_str("\"name\":");
                write_json_string(output, attr.name);

                let value_type = jsx_attribute_type_name(&attr.value);
                output.push_str(",\"value_type\":\"");
                output.push_str(value_type);
                output.push('"');

                // Kept for backward compatibility with existing payload consumers.
                output.push_str(",\"type\":\"");
                output.push_str(value_type);
                output.push('"');

                match &attr.value {
                    JsxAttributeValue::String(value) => {
                        output.push_str(",\"value\":");
                        write_json_string(output, value);
                    }
                    JsxAttributeValue::Number(value) => {
                        output.push_str(",\"value\":");
                        output.push_str(&value.to_string());
                    }
                    JsxAttributeValue::InvalidNumber(value) => {
                        output.push_str(",\"value\":");
                        write_json_string(output, value);
                    }
                    JsxAttributeValue::Boolean(value) => {
                        output.push_str(",\"value\":");
                        output.push_str(if *value { "true" } else { "false" });
                    }
                    JsxAttributeValue::Expression(value) => {
                        output.push_str(",\"value\":");
                        write_json_string(output, value);
                    }
                }

                output.push('}');
            }
            output.push(']');

            output.push_str(",\"children\":[");
            if let Some(children) = element.as_ref().map(|value| value.children) {
                for (i, &child_idx) in children.iter().enumerate() {
                    if i > 0 {
                        output.push(',');
                    }
                    serialize_node(ast, child_idx, output, options);
                }
            }
            output.push(']');
        }

        NodeTag::Frontmatter => {
            let info = frontmatter_view(ast, node_idx);

            let format_str = match info
                .map(|value| value.format)
                .unwrap_or(FrontmatterFormat::Yaml)
            {
                FrontmatterFormat::Yaml => "yaml",
                FrontmatterFormat::Json => "json",
            };
            output.push_str(",\"format\":\"");
            output.push_str(format_str);
            output.push('"');

            output.push_str(",\"value\":");
            write_json_string(output, info.map(|value| value.value).unwrap_or(""));
        }

        NodeTag::MdxTextExpression | NodeTag::MdxFlowExpression => {
            if let Some(info) = expression_info(ast, node_idx) {
                output.push_str(",\"value\":");
                write_json_string(output, info.value);
            }
        }

        NodeTag::ListItem => {
            let info = ast.list_item_info(node_idx);
            output.push_str(",\"checked\":");
            match info.checked {
                Some(true) => output.push_str("true"),
                Some(false) => output.push_str("false"),
                None => output.push_str("null"),
            }
            output.push_str(",\"children\":[");
            let children = ast.children(node_idx);
            for (i, &child_idx) in children.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_node(ast, child_idx, output, options);
            }
            output.push(']');
        }

        NodeTag::Table => {
            let alignments = ast.table_alignments(node_idx);
            output.push_str(",\"alignments\":[");
            for (i, align) in alignments.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                let align_str = match align {
                    TableAlignment::None => "\"none\"",
                    TableAlignment::Left => "\"left\"",
                    TableAlignment::Center => "\"center\"",
                    TableAlignment::Right => "\"right\"",
                };
                output.push_str(align_str);
            }
            output.push(']');

            output.push_str(",\"children\":[");
            let children = ast.children(node_idx);
            for (i, &child_idx) in children.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_node(ast, child_idx, output, options);
            }
            output.push(']');
        }

        NodeTag::TableRow | NodeTag::TableCell => {
            output.push_str(",\"children\":[");
            let children = ast.children(node_idx);
            for (i, &child_idx) in children.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_node(ast, child_idx, output, options);
            }
            output.push(']');
        }

        NodeTag::ListUnordered | NodeTag::ListOrdered => {
            output.push_str(",\"ordered\":");
            output.push_str(if node.tag == NodeTag::ListOrdered {
                "true"
            } else {
                "false"
            });
            output.push_str(",\"children\":[");
            let children = ast.children(node_idx);
            for (i, &child_idx) in children.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_node(ast, child_idx, output, options);
            }
            output.push(']');
        }

        // Nodes with children arrays
        NodeTag::Document
        | NodeTag::Paragraph
        | NodeTag::Blockquote
        | NodeTag::Strong
        | NodeTag::Emphasis
        | NodeTag::Strikethrough
        | NodeTag::MdxJsxFragment => {
            output.push_str(",\"children\":[");
            let children = ast.children(node_idx);
            for (i, &child_idx) in children.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                serialize_node(ast, child_idx, output, options);
            }
            output.push(']');
        }

        NodeTag::Hr | NodeTag::HardBreak => {
            // No additional data
        }

        _ => {
            // Unknown node type - just output type
        }
    }

    output.push('}');
}

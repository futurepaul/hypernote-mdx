use crate::ast::*;
use crate::token::Tag as TokenTag;

/// Write a JSON-escaped string
fn write_json_string(output: &mut String, s: &str) {
    output.push('"');
    for c in s.chars() {
        match c {
            '"' => output.push_str("\\\""),
            '\\' => output.push_str("\\\\"),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            '\x00'..='\x08' | '\x0b' | '\x0c' | '\x0e'..='\x1f' => {
                output.push_str(&format!("\\u{:04x}", c as u32));
            }
            _ => output.push(c),
        }
    }
    output.push('"');
}

fn decode_html_entities(value: &str) -> String {
    value
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn decode_jsx_quoted_value(raw: &str) -> String {
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
    let mut output = String::new();

    output.push_str("{\"schema\":{\"name\":");
    write_json_string(&mut output, AST_SCHEMA_NAME);
    output.push_str(",\"version\":");
    output.push_str(&AST_SCHEMA_VERSION.to_string());
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
        output.push_str(&format!("\"tag\":\"{}\"", err.tag.name()));
        output.push_str(&format!(",\"token\":{}", err.token));
        output.push_str(&format!(",\"byte_offset\":{}", err.byte_offset));
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
    output.push_str(&format!("\"type\":\"{}\"", node.tag.name()));

    if options.include_positions {
        let span = ast.node_span(node_idx);
        output.push_str(&format!(
            ",\"position\":{{\"start\":{},\"end\":{}}}",
            span.start, span.end
        ));
    }

    match node.tag {
        NodeTag::Heading => {
            let info = ast.heading_info(node_idx);
            output.push_str(&format!(",\"level\":{}", info.level));
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
            let fence_token = node.main_token;

            // Check if there's a language token after the fence
            let mut lang: Option<&str> = None;
            if fence_token + 1 < ast.token_tags.len() as u32 {
                let next_token = fence_token + 1;
                if ast.token_tags[next_token as usize] == TokenTag::Text {
                    let lang_text = ast.token_slice(next_token);
                    let trimmed = lang_text.trim();
                    if !trimmed.is_empty() {
                        lang = Some(trimmed);
                    }
                }
            }

            output.push_str(",\"lang\":");
            if let Some(l) = lang {
                write_json_string(output, l);
            } else {
                output.push_str("null");
            }

            // Get the code content
            let mut code_start: u32 = u32::MAX;
            let mut code_end: u32 = 0;
            let mut in_code = false;

            let mut i = fence_token;
            while (i as usize) < ast.token_tags.len() {
                if ast.token_tags[i as usize] == TokenTag::CodeFenceEnd {
                    break;
                }
                if ast.token_tags[i as usize] == TokenTag::Newline && !in_code {
                    in_code = true;
                    i += 1;
                    continue;
                }
                if in_code {
                    let start = ast.token_starts[i as usize];
                    let end = if (i as usize + 1) < ast.token_starts.len() {
                        ast.token_starts[i as usize + 1]
                    } else {
                        ast.source.len() as u32
                    };
                    code_start = code_start.min(start);
                    code_end = code_end.max(end);
                }
                i += 1;
            }

            let code = if code_start < code_end {
                &ast.source[code_start as usize..code_end as usize]
            } else {
                ""
            };

            output.push_str(",\"value\":");
            write_json_string(output, code);
        }

        NodeTag::CodeInline => {
            if let NodeData::Token(content_token) = node.data {
                let text = ast.token_slice(content_token);
                output.push_str(",\"value\":");
                write_json_string(output, text);
            }
        }

        NodeTag::Link | NodeTag::Image => {
            if let NodeData::Extra(idx) = node.data {
                let text_node_raw = ast.extra_data[idx as usize];
                let url_token = ast.extra_data[idx as usize + 1];
                let url = ast.token_slice(url_token);

                output.push_str(",\"url\":");
                write_json_string(output, url);

                if text_node_raw != u32::MAX {
                    output.push_str(",\"children\":[");
                    serialize_node(ast, text_node_raw, output, options);
                    output.push(']');
                } else {
                    output.push_str(",\"children\":[]");
                }
            }
        }

        NodeTag::MdxJsxElement | NodeTag::MdxJsxSelfClosing => {
            let elem = ast.jsx_element(node_idx);
            let name_raw = ast.token_slice(elem.name_token);
            let name = name_raw.trim();

            output.push_str(",\"name\":");
            write_json_string(output, name);

            // Serialize attributes
            output.push_str(",\"attributes\":[");
            let attrs = ast.jsx_attributes(node_idx);
            for (i, attr) in attrs.iter().enumerate() {
                if i > 0 {
                    output.push(',');
                }
                output.push('{');

                let attr_name_raw = ast.token_slice(attr.name_token);
                let attr_name = attr_name_raw.trim();
                output.push_str("\"name\":");
                write_json_string(output, attr_name);

                let value_type = match attr.value_type {
                    JsxAttributeType::String => "string",
                    JsxAttributeType::Number => "number",
                    JsxAttributeType::Boolean => "boolean",
                    JsxAttributeType::Expression => "expression",
                };
                output.push_str(",\"value_type\":\"");
                output.push_str(value_type);
                output.push('"');

                // Kept for backward compatibility with existing payload consumers.
                output.push_str(",\"type\":\"");
                output.push_str(value_type);
                output.push('"');

                match attr.value_type {
                    JsxAttributeType::String => {
                        let value = if let Some(val_tok) = attr.value_token {
                            decode_jsx_quoted_value(ast.token_slice(val_tok))
                        } else {
                            String::new()
                        };
                        output.push_str(",\"value\":");
                        write_json_string(output, &value);
                    }
                    JsxAttributeType::Number => {
                        output.push_str(",\"value\":");
                        if let Some(val_tok) = attr.value_token {
                            let raw = ast.token_slice(val_tok).trim();
                            if let Ok(parsed) = raw.parse::<f64>() {
                                output.push_str(&parsed.to_string());
                            } else {
                                write_json_string(output, raw);
                            }
                        } else {
                            output.push('0');
                        }
                    }
                    JsxAttributeType::Boolean => {
                        let bool_value = if let Some(val_tok) = attr.value_token {
                            ast.token_slice(val_tok).trim() == "true"
                        } else {
                            true
                        };
                        output.push_str(",\"value\":");
                        output.push_str(if bool_value { "true" } else { "false" });
                    }
                    JsxAttributeType::Expression => {
                        let expr = if let Some(val_tok) = attr.value_token {
                            ast.token_slice(val_tok).trim()
                        } else {
                            ""
                        };
                        output.push_str(",\"value\":");
                        write_json_string(output, expr);
                    }
                }

                output.push('}');
            }
            output.push(']');

            output.push_str(",\"children\":[");
            if node.tag == NodeTag::MdxJsxElement {
                let children =
                    &ast.extra_data[elem.children_start as usize..elem.children_end as usize];
                for (i, &child_raw) in children.iter().enumerate() {
                    if i > 0 {
                        output.push(',');
                    }
                    serialize_node(ast, child_raw, output, options);
                }
            }
            output.push(']');
        }

        NodeTag::Frontmatter => {
            let info = ast.frontmatter_info(node_idx);
            let range = Range {
                start: info.content_start,
                end: info.content_end,
            };

            let format_str = match info.format {
                FrontmatterFormat::Yaml => "yaml",
                FrontmatterFormat::Json => "json",
            };
            output.push_str(",\"format\":\"");
            output.push_str(format_str);
            output.push('"');

            let mut fm_start: u32 = u32::MAX;
            let mut fm_end: u32 = 0;

            for i in range.start..range.end {
                let start = ast.token_starts[i as usize];
                let end = if (i as usize + 1) < ast.token_starts.len() {
                    ast.token_starts[i as usize + 1]
                } else {
                    ast.source.len() as u32
                };
                fm_start = fm_start.min(start);
                fm_end = fm_end.max(end);
            }

            let content = if fm_start < fm_end {
                ast.source[fm_start as usize..fm_end as usize].trim()
            } else {
                ""
            };

            output.push_str(",\"value\":");
            write_json_string(output, content);
        }

        NodeTag::MdxTextExpression | NodeTag::MdxFlowExpression => {
            if let NodeData::Extra(idx) = node.data {
                let range = ast.extra_range(idx);

                let mut expr_start: u32 = u32::MAX;
                let mut expr_end: u32 = 0;

                for i in range.start..range.end {
                    let start = ast.token_starts[i as usize];
                    let end = if (i as usize + 1) < ast.token_starts.len() {
                        ast.token_starts[i as usize + 1]
                    } else {
                        ast.source.len() as u32
                    };
                    expr_start = expr_start.min(start);
                    expr_end = expr_end.max(end);
                }

                let content = if expr_start < expr_end {
                    ast.source[expr_start as usize..expr_end as usize].trim()
                } else {
                    ""
                };

                output.push_str(",\"value\":");
                write_json_string(output, content);
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

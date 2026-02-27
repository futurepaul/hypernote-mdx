use crate::ast::*;
use crate::token::Tag as TokenTag;

/// Render an AST back to canonical MDX source.
pub fn render(ast: &Ast) -> String {
    let mut output = String::new();

    // Find the document node
    let doc_idx = ast
        .nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == NodeTag::Document)
        .map(|(i, _)| i as NodeIndex);

    if let Some(idx) = doc_idx {
        let children = ast.children(idx);
        let mut last_was_content = false;
        for &child_idx in children {
            let child_node = &ast.nodes[child_idx as usize];

            // Skip empty paragraphs
            if child_node.tag == NodeTag::Paragraph {
                let para_children = ast.children(child_idx);
                if para_children.is_empty() {
                    continue;
                }
                if para_children.len() == 1 {
                    let para_child = &ast.nodes[para_children[0] as usize];
                    if para_child.tag == NodeTag::Text {
                        let text = ast.token_slice(para_child.main_token);
                        if text.trim().is_empty() {
                            continue;
                        }
                    }
                }
            }

            // Add blank line between content blocks
            if last_was_content {
                output.push('\n');
            }

            render_node(ast, child_idx, &mut output, &RenderContext::default());
            last_was_content = child_node.tag != NodeTag::Frontmatter;
        }
    }

    output
}

#[derive(Default)]
#[allow(dead_code)]
struct RenderContext {
    in_list: bool,
    list_index: u32,
    indent_level: u32,
    in_jsx: bool,
}

fn write_indent(output: &mut String, level: u32) {
    for _ in 0..level {
        output.push_str("  ");
    }
}

/// Check if a JSX element can be rendered on a single line
fn can_render_jsx_inline(ast: &Ast, children: &[NodeIndex]) -> bool {
    if children.len() != 1 {
        return false;
    }
    let child = &ast.nodes[children[0] as usize];
    child.tag == NodeTag::Text || child.tag == NodeTag::MdxTextExpression
}

fn can_render_all_jsx_children_inline(ast: &Ast, children: &[NodeIndex]) -> bool {
    if children.is_empty() {
        return true;
    }

    children.iter().all(|&child_idx| {
        let child = &ast.nodes[child_idx as usize];
        matches!(
            child.tag,
            NodeTag::Text
                | NodeTag::Strong
                | NodeTag::Emphasis
                | NodeTag::CodeInline
                | NodeTag::Link
                | NodeTag::Image
                | NodeTag::MdxTextExpression
                | NodeTag::HardBreak
        )
    })
}

/// Check if a node is a "content block" that should have blank lines between siblings
fn is_content_block(tag: NodeTag) -> bool {
    matches!(
        tag,
        NodeTag::MdxTextExpression
            | NodeTag::MdxFlowExpression
            | NodeTag::Paragraph
            | NodeTag::Heading
            | NodeTag::CodeBlock
            | NodeTag::Blockquote
            | NodeTag::ListUnordered
            | NodeTag::ListOrdered
            | NodeTag::Table
    )
}

fn render_node(ast: &Ast, node_idx: NodeIndex, output: &mut String, ctx: &RenderContext) {
    let node = &ast.nodes[node_idx as usize];

    match node.tag {
        NodeTag::Document => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_node(ast, child_idx, output, ctx);
            }
        }

        NodeTag::Frontmatter => {
            let info = ast.frontmatter_info(node_idx);
            let range = Range {
                start: info.content_start,
                end: info.content_end,
            };
            let content = extract_token_range_content(ast, &range);

            match info.format {
                FrontmatterFormat::Yaml => {
                    output.push_str("---\n");
                    output.push_str(content);
                    if !content.is_empty() && !content.ends_with('\n') {
                        output.push('\n');
                    }
                    output.push_str("---\n\n");
                }
                FrontmatterFormat::Json => {
                    output.push_str("```hnmd\n");
                    output.push_str(content);
                    if !content.is_empty() && !content.ends_with('\n') {
                        output.push('\n');
                    }
                    output.push_str("```\n\n");
                }
            }
        }

        NodeTag::Heading => {
            let info = ast.heading_info(node_idx);
            for _ in 0..info.level {
                output.push('#');
            }
            output.push(' ');
            let children =
                &ast.extra_data[info.children_start as usize..info.children_end as usize];
            for &child_raw in children {
                render_node(ast, child_raw, output, ctx);
            }
            output.push('\n');
        }

        NodeTag::Paragraph => {
            let children = ast.children(node_idx);
            if children.is_empty() {
                return;
            }
            if children.len() == 1 {
                let child = &ast.nodes[children[0] as usize];
                if child.tag == NodeTag::Text {
                    let text = ast.token_slice(child.main_token);
                    if text.trim().is_empty() {
                        return;
                    }
                }
            }
            for &child_idx in children {
                render_node(ast, child_idx, output, ctx);
            }
            if !ctx.in_jsx {
                output.push('\n');
            }
        }

        NodeTag::Text => {
            let text = ast.token_slice(node.main_token);
            output.push_str(text);
        }

        NodeTag::Strong => {
            output.push_str("**");
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_node(ast, child_idx, output, ctx);
            }
            output.push_str("**");
        }

        NodeTag::Emphasis => {
            output.push('*');
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_node(ast, child_idx, output, ctx);
            }
            output.push('*');
        }

        NodeTag::CodeInline => {
            output.push('`');
            if let NodeData::Token(content_token) = node.data {
                let text = ast.token_slice(content_token);
                output.push_str(text);
            }
            output.push('`');
        }

        NodeTag::CodeBlock => {
            output.push_str("```");
            let fence_token = node.main_token;

            if fence_token + 1 < ast.token_tags.len() as u32 {
                let next_token = fence_token + 1;
                if ast.token_tags[next_token as usize] == TokenTag::Text {
                    let lang_text = ast.token_slice(next_token);
                    let trimmed = lang_text.trim();
                    if !trimmed.is_empty() {
                        output.push_str(trimmed);
                    }
                }
            }
            output.push('\n');

            let code = extract_code_block_content(ast, fence_token);
            output.push_str(code);
            if !code.is_empty() && !code.ends_with('\n') {
                output.push('\n');
            }
            output.push_str("```\n");
        }

        NodeTag::Blockquote => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                output.push_str("> ");
                render_node(ast, child_idx, output, ctx);
            }
            output.push('\n');
        }

        NodeTag::ListUnordered => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                let child_ctx = RenderContext {
                    in_list: true,
                    list_index: 0,
                    indent_level: ctx.indent_level,
                    in_jsx: ctx.in_jsx,
                };
                render_node(ast, child_idx, output, &child_ctx);
            }
        }

        NodeTag::ListOrdered => {
            let children = ast.children(node_idx);
            for (i, &child_idx) in children.iter().enumerate() {
                let child_ctx = RenderContext {
                    in_list: true,
                    list_index: (i + 1) as u32,
                    indent_level: ctx.indent_level,
                    in_jsx: ctx.in_jsx,
                };
                render_node(ast, child_idx, output, &child_ctx);
            }
        }

        NodeTag::ListItem => {
            write_indent(output, ctx.indent_level);
            if ctx.list_index == 0 {
                output.push_str("- ");
            } else {
                output.push_str(&format!("{}. ", ctx.list_index));
            }
            let info = ast.list_item_info(node_idx);
            if let Some(checked) = info.checked {
                output.push_str(if checked { "[x] " } else { "[ ] " });
            }
            let children = ast.children(node_idx);
            for &child_idx in children {
                let child = &ast.nodes[child_idx as usize];
                if child.tag == NodeTag::Paragraph {
                    let para_children = ast.children(child_idx);
                    for &para_child_idx in para_children {
                        render_node(ast, para_child_idx, output, ctx);
                    }
                } else {
                    render_node(ast, child_idx, output, ctx);
                }
            }
            output.push('\n');
        }

        NodeTag::Hr => {
            output.push_str("---\n");
        }

        NodeTag::HardBreak => {
            output.push_str("  \n");
        }

        NodeTag::Link => {
            if let NodeData::Extra(idx) = node.data {
                let text_node_raw = ast.extra_data[idx as usize];
                let url_token = ast.extra_data[idx as usize + 1];

                output.push('[');
                if text_node_raw != u32::MAX {
                    render_node(ast, text_node_raw, output, ctx);
                }
                output.push_str("](");
                let url = ast.token_slice(url_token);
                output.push_str(url);
                output.push(')');
            }
        }

        NodeTag::Image => {
            if let NodeData::Extra(idx) = node.data {
                let text_node_raw = ast.extra_data[idx as usize];
                let url_token = ast.extra_data[idx as usize + 1];

                output.push_str("![");
                if text_node_raw != u32::MAX {
                    render_node(ast, text_node_raw, output, ctx);
                }
                output.push_str("](");
                let url = ast.token_slice(url_token);
                output.push_str(url);
                output.push(')');
            }
        }

        NodeTag::MdxTextExpression => {
            output.push('{');
            if let NodeData::Extra(idx) = node.data {
                let range = ast.extra_range(idx);
                let content = extract_token_range_content(ast, &range);
                output.push_str(content.trim());
            }
            output.push('}');
        }

        NodeTag::MdxFlowExpression => {
            output.push('{');
            if let NodeData::Extra(idx) = node.data {
                let range = ast.extra_range(idx);
                let content = extract_token_range_content(ast, &range);
                output.push_str(content.trim());
            }
            output.push_str("}\n");
        }

        NodeTag::MdxJsxElement => {
            let elem = ast.jsx_element(node_idx);
            let name_raw = ast.token_slice(elem.name_token);
            let name = name_raw.trim();

            let children =
                &ast.extra_data[elem.children_start as usize..elem.children_end as usize];
            let children_as_nodes: Vec<NodeIndex> = children.to_vec();

            let render_inline = can_render_jsx_inline(ast, &children_as_nodes)
                || can_render_all_jsx_children_inline(ast, &children_as_nodes);

            write_indent(output, ctx.indent_level);
            output.push('<');
            output.push_str(name);
            render_jsx_attributes(ast, node_idx, output);
            output.push('>');

            if render_inline {
                let child_ctx = RenderContext {
                    indent_level: ctx.indent_level + 1,
                    in_jsx: true,
                    ..*ctx
                };
                for &child_idx in &children_as_nodes {
                    render_node(ast, child_idx, output, &child_ctx);
                }
            } else {
                output.push('\n');
                let mut prev_was_content_block = false;
                for (i, &child_idx) in children_as_nodes.iter().enumerate() {
                    let child = &ast.nodes[child_idx as usize];
                    let is_content = is_content_block(child.tag);

                    if prev_was_content_block && is_content {
                        output.push('\n');
                    }

                    let child_ctx = RenderContext {
                        indent_level: ctx.indent_level + 1,
                        in_jsx: true,
                        ..*ctx
                    };
                    render_node(ast, child_idx, output, &child_ctx);

                    let next_is_hard_break = if i + 1 < children_as_nodes.len() {
                        ast.nodes[children_as_nodes[i + 1] as usize].tag == NodeTag::HardBreak
                    } else {
                        false
                    };

                    if child.tag != NodeTag::HardBreak && !next_is_hard_break {
                        output.push('\n');
                    }
                    prev_was_content_block = is_content;
                }
                write_indent(output, ctx.indent_level);
            }

            output.push_str("</");
            output.push_str(name);
            output.push('>');

            if !ctx.in_jsx {
                output.push('\n');
            }
        }

        NodeTag::MdxJsxSelfClosing => {
            let elem = ast.jsx_element(node_idx);
            let name_raw = ast.token_slice(elem.name_token);
            let name = name_raw.trim();

            write_indent(output, ctx.indent_level);
            output.push('<');
            output.push_str(name);
            render_jsx_attributes(ast, node_idx, output);
            output.push_str(" />");

            if !ctx.in_jsx {
                output.push('\n');
            }
        }

        NodeTag::Table => {
            let info = ast.table_info(node_idx);
            let alignments = ast.table_alignments(node_idx);
            let rows = ast.children(node_idx);

            if !rows.is_empty() {
                // Render header row
                render_table_row(ast, rows[0], output, ctx);

                // Render separator row
                output.push('|');
                for align in &alignments {
                    output.push(' ');
                    match align {
                        TableAlignment::Left => output.push_str(":---"),
                        TableAlignment::Center => output.push_str(":---:"),
                        TableAlignment::Right => output.push_str("---:"),
                        TableAlignment::None => output.push_str("---"),
                    }
                    output.push_str(" |");
                }
                output.push('\n');

                // Render body rows
                for &row_idx in &rows[1..] {
                    render_table_row(ast, row_idx, output, ctx);
                }
            }
            let _ = info; // suppress unused warning
        }

        NodeTag::TableRow | NodeTag::TableCell => {
            // These are handled by render_table_row; if called directly, fall through
        }

        NodeTag::MdxJsxFragment => {
            write_indent(output, ctx.indent_level);
            output.push_str("<>\n");
            let children = ast.children(node_idx);
            for &child_idx in children {
                let child_ctx = RenderContext {
                    indent_level: ctx.indent_level + 1,
                    in_jsx: true,
                    ..*ctx
                };
                render_node(ast, child_idx, output, &child_ctx);
                output.push('\n');
            }
            write_indent(output, ctx.indent_level);
            output.push_str("</>");
            if !ctx.in_jsx {
                output.push('\n');
            }
        }

        _ => {
            let source = ast.node_source(node_idx);
            output.push_str(source);
        }
    }
}

fn render_table_row(ast: &Ast, row_idx: NodeIndex, output: &mut String, ctx: &RenderContext) {
    let cells = ast.children(row_idx);
    output.push('|');
    for &cell_idx in cells {
        output.push(' ');
        let cell_children = ast.children(cell_idx);
        for &child_idx in cell_children {
            render_node(ast, child_idx, output, ctx);
        }
        output.push_str(" |");
    }
    output.push('\n');
}

fn render_jsx_attributes(ast: &Ast, node_idx: NodeIndex, output: &mut String) {
    let attrs = ast.jsx_attributes(node_idx);
    for attr in &attrs {
        output.push(' ');
        let attr_name_raw = ast.token_slice(attr.name_token);
        let attr_name = attr_name_raw.trim();
        output.push_str(attr_name);

        match attr.value_type {
            JsxAttributeType::Boolean => {
                if let Some(val_tok) = attr.value_token {
                    let raw = ast.token_slice(val_tok).trim();
                    let bool_value = raw == "true";
                    output.push('=');
                    output.push('{');
                    output.push_str(if bool_value { "true" } else { "false" });
                    output.push('}');
                }
            }
            JsxAttributeType::Expression => {
                output.push('=');
                output.push('{');
                if let Some(val_tok) = attr.value_token {
                    let val_text = ast.token_slice(val_tok).trim();
                    output.push_str(val_text);
                }
                output.push('}');
            }
            JsxAttributeType::Number => {
                output.push('=');
                if let Some(val_tok) = attr.value_token {
                    let raw = ast.token_slice(val_tok).trim();
                    if let Ok(number) = raw.parse::<f64>() {
                        output.push_str(&number.to_string());
                    } else {
                        let decoded = decode_jsx_quoted_value(raw);
                        output.push('"');
                        output.push_str(&escape_jsx_attribute_string(&decoded));
                        output.push('"');
                    }
                } else {
                    output.push('0');
                }
            }
            JsxAttributeType::String => {
                output.push('=');
                let decoded = if let Some(val_tok) = attr.value_token {
                    decode_jsx_quoted_value(ast.token_slice(val_tok))
                } else {
                    String::new()
                };
                output.push('"');
                output.push_str(&escape_jsx_attribute_string(&decoded));
                output.push('"');
            }
        }
    }
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

    output
        .replace("&quot;", "\"")
        .replace("&gt;", ">")
        .replace("&lt;", "<")
        .replace("&amp;", "&")
}

fn escape_jsx_attribute_string(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn extract_token_range_content<'a>(ast: &'a Ast, range: &Range) -> &'a str {
    if range.start >= range.end {
        return "";
    }

    let start = ast.token_starts[range.start as usize] as usize;
    let end = if (range.end as usize) < ast.token_starts.len() {
        ast.token_starts[range.end as usize] as usize
    } else {
        ast.source.len()
    };

    &ast.source[start..end]
}

fn extract_code_block_content<'a>(ast: &'a Ast, fence_token: TokenIndex) -> &'a str {
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

    if code_start < code_end {
        &ast.source[code_start as usize..code_end as usize]
    } else {
        ""
    }
}

#[cfg(test)]
mod tests {
    use crate::parser;

    use super::*;

    #[test]
    fn render_simple_heading() {
        let source = "# Hello";
        let ast = parser::parse(source);
        let rendered = render(&ast);
        assert_eq!("# Hello\n", rendered);
    }

    #[test]
    fn render_paragraph_with_bold() {
        let source = "Hello **world**";
        let ast = parser::parse(source);
        let rendered = render(&ast);
        assert_eq!("Hello **world**\n", rendered);
    }

    #[test]
    fn roundtrip_preserves_structure() {
        let source = "# Heading\n\nParagraph with **bold** and *italic*.\n";
        let ast1 = parser::parse(source);
        let rendered = render(&ast1);
        let ast2 = parser::parse(&rendered);
        assert_eq!(ast1.nodes.len(), ast2.nodes.len());
    }

    #[test]
    fn node_at_offset_finds_correct_node() {
        let source = "# Hello";
        let ast = parser::parse(source);

        let at_hash = ast.node_at_offset(0);
        assert!(at_hash.is_some());
        if let Some(idx) = at_hash {
            assert_eq!(NodeTag::Heading, ast.nodes[idx as usize].tag);
        }

        let at_text = ast.node_at_offset(2);
        assert!(at_text.is_some());
        if let Some(idx) = at_text {
            assert_eq!(NodeTag::Text, ast.nodes[idx as usize].tag);
        }
    }

    #[test]
    fn node_span_returns_correct_bounds() {
        let source = "# Hello";
        let ast = parser::parse(source);

        let heading_idx = ast
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.tag == NodeTag::Heading)
            .map(|(i, _)| i as NodeIndex);

        assert!(heading_idx.is_some());
        if let Some(idx) = heading_idx {
            let span = ast.node_span(idx);
            assert_eq!(0, span.start);
            assert!(span.end >= 7);
        }
    }

    #[test]
    fn render_image_with_alt_text() {
        let source = "![Alt text](image.jpg)";
        let ast = parser::parse(source);
        let rendered = render(&ast);
        assert_eq!("![Alt text](image.jpg)\n", rendered);
    }

    #[test]
    fn render_link() {
        let source = "[Click here](https://example.com)";
        let ast = parser::parse(source);
        let rendered = render(&ast);
        assert_eq!("[Click here](https://example.com)\n", rendered);
    }

    #[test]
    fn render_jsx_self_closing() {
        let source = "<Button label=\"Click\" />";
        let ast = parser::parse(source);
        let rendered = render(&ast);
        assert_eq!("<Button label=\"Click\" />\n", rendered);
    }

    #[test]
    fn roundtrip_json_frontmatter() {
        let source = "```hnmd\n{\"title\": \"Hello\"}\n```\n\n# Content\n";
        let ast1 = parser::parse(source);
        assert!(
            ast1.errors.is_empty(),
            "First parse had errors: {:?}",
            ast1.errors
        );

        let rendered = render(&ast1);
        assert!(
            rendered.starts_with("```hnmd\n"),
            "Rendered should start with ```hnmd, got: {}",
            rendered
        );

        let ast2 = parser::parse(&rendered);
        assert!(
            ast2.errors.is_empty(),
            "Second parse had errors: {:?}",
            ast2.errors
        );

        // Both ASTs should have the same number of nodes
        assert_eq!(ast1.nodes.len(), ast2.nodes.len());

        // Both should have a Frontmatter node with JSON format
        let fm1 = ast1
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.tag == NodeTag::Frontmatter)
            .map(|(i, _)| i as NodeIndex);
        let fm2 = ast2
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.tag == NodeTag::Frontmatter)
            .map(|(i, _)| i as NodeIndex);

        assert!(fm1.is_some());
        assert!(fm2.is_some());

        let info1 = ast1.frontmatter_info(fm1.unwrap());
        let info2 = ast2.frontmatter_info(fm2.unwrap());
        assert_eq!(info1.format, info2.format);
        assert_eq!(info1.format, FrontmatterFormat::Json);
    }
}

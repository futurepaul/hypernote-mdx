use std::env;
use std::fs;

use hypernote_mdx::ast::*;
use hypernote_mdx::parse;

// ANSI escape codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const ITALIC: &str = "\x1b[3m";
const UNDERLINE: &str = "\x1b[4m";
const REVERSE: &str = "\x1b[7m";
const BRIGHT_WHITE: &str = "\x1b[97m";
const BRIGHT_CYAN: &str = "\x1b[96m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const GRAY: &str = "\x1b[90m";
const GREEN: &str = "\x1b[32m";

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.md|file.hnmd>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", filename, e);
        std::process::exit(1);
    });

    let ast = parse(&source);
    let mut output = String::new();
    render_pretty(&ast, &mut output);
    print!("{}", output);
}

fn render_pretty(ast: &Ast, output: &mut String) {
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
                output.push('\n');
            }
            render_node(ast, child_idx, output);
        }
    }
}

fn render_node(ast: &Ast, node_idx: NodeIndex, output: &mut String) {
    let node = &ast.nodes[node_idx as usize];

    match node.tag {
        NodeTag::Document => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_node(ast, child_idx, output);
            }
        }

        NodeTag::Frontmatter => {
            let info = ast.frontmatter_info(node_idx);
            let range = Range {
                start: info.content_start,
                end: info.content_end,
            };
            let content = extract_token_range(ast, &range);
            let fmt_label = match info.format {
                FrontmatterFormat::Yaml => "YAML",
                FrontmatterFormat::Json => "JSON",
            };
            output.push_str(&format!(
                "{DIM}--- {fmt_label} frontmatter ---{RESET}\n"
            ));
            output.push_str(&format!("{DIM}{}{RESET}\n", content.trim()));
            output.push_str(&format!("{DIM}---{RESET}\n"));
        }

        NodeTag::Heading => {
            let info = ast.heading_info(node_idx);
            let style = match info.level {
                1 => format!("{BOLD}{BRIGHT_WHITE}{UNDERLINE}"),
                2 => format!("{BOLD}{BRIGHT_CYAN}"),
                3 => format!("{BOLD}{YELLOW}"),
                _ => format!("{BOLD}{DIM}"),
            };
            let prefix = "#".repeat(info.level as usize);
            output.push_str(&format!("{style}{prefix} "));
            let children =
                &ast.extra_data[info.children_start as usize..info.children_end as usize];
            for &child_raw in children {
                render_inline(ast, child_raw, output);
            }
            output.push_str(&format!("{RESET}\n"));
        }

        NodeTag::Paragraph => {
            let children = ast.children(node_idx);
            if children.is_empty() {
                return;
            }
            for &child_idx in children {
                render_inline(ast, child_idx, output);
            }
            output.push('\n');
        }

        NodeTag::CodeBlock => {
            render_code_block(ast, node, output);
        }

        NodeTag::Blockquote => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                output.push_str(&format!("{GRAY}  | {RESET}"));
                output.push_str(DIM);
                render_inline(ast, child_idx, output);
                output.push_str(RESET);
            }
            output.push('\n');
        }

        NodeTag::Hr => {
            output.push_str(&format!(
                "{DIM}────────────────────────────────{RESET}\n"
            ));
        }

        NodeTag::ListUnordered => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_list_item(ast, child_idx, output, None);
            }
        }

        NodeTag::ListOrdered => {
            let children = ast.children(node_idx);
            for (i, &child_idx) in children.iter().enumerate() {
                render_list_item(ast, child_idx, output, Some(i + 1));
            }
        }

        NodeTag::Table => {
            render_table(ast, node_idx, output);
        }

        NodeTag::MdxJsxElement | NodeTag::MdxJsxSelfClosing => {
            let elem = ast.jsx_element(node_idx);
            let name = ast.token_slice(elem.name_token).trim();
            output.push_str(&format!("{DIM}<{name}"));
            let attrs = ast.jsx_attributes(node_idx);
            for attr in &attrs {
                let attr_name = ast.token_slice(attr.name_token).trim();
                output.push_str(&format!(" {attr_name}"));
                if let Some(val_tok) = attr.value_token {
                    let val = ast.token_slice(val_tok).trim();
                    output.push_str(&format!("={val}"));
                }
            }
            if node.tag == NodeTag::MdxJsxSelfClosing {
                output.push_str(&format!(" />{RESET}\n"));
            } else {
                output.push_str(&format!(">{RESET}\n"));
                let children =
                    &ast.extra_data[elem.children_start as usize..elem.children_end as usize];
                for &child_raw in children {
                    render_node(ast, child_raw, output);
                }
                output.push_str(&format!("{DIM}</{name}>{RESET}\n"));
            }
        }

        NodeTag::MdxJsxFragment => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_node(ast, child_idx, output);
            }
        }

        NodeTag::MdxTextExpression | NodeTag::MdxFlowExpression => {
            if let NodeData::Extra(idx) = node.data {
                let range = ast.extra_range(idx);
                let content = extract_token_range(ast, &range);
                output.push_str(&format!("{DIM}{{{}}}{RESET}", content.trim()));
            }
        }

        _ => {
            render_inline(ast, node_idx, output);
        }
    }
}

fn render_inline(ast: &Ast, node_idx: NodeIndex, output: &mut String) {
    let node = &ast.nodes[node_idx as usize];

    match node.tag {
        NodeTag::Text => {
            let text = ast.token_slice(node.main_token);
            output.push_str(text);
        }

        NodeTag::Strong => {
            output.push_str(BOLD);
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_inline(ast, child_idx, output);
            }
            output.push_str(RESET);
        }

        NodeTag::Emphasis => {
            output.push_str(ITALIC);
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_inline(ast, child_idx, output);
            }
            output.push_str(RESET);
        }

        NodeTag::CodeInline => {
            output.push_str(REVERSE);
            if let NodeData::Token(content_token) = node.data {
                let text = ast.token_slice(content_token);
                output.push_str(text);
            }
            output.push_str(RESET);
        }

        NodeTag::Link => {
            if let NodeData::Extra(idx) = node.data {
                let text_node_raw = ast.extra_data[idx as usize];
                let url_token = ast.extra_data[idx as usize + 1];
                let url = ast.token_slice(url_token);

                output.push_str(&format!("{BLUE}{UNDERLINE}"));
                if text_node_raw != u32::MAX {
                    render_inline(ast, text_node_raw, output);
                }
                output.push_str(&format!("{RESET} {DIM}({url}){RESET}"));
            }
        }

        NodeTag::Image => {
            if let NodeData::Extra(idx) = node.data {
                let text_node_raw = ast.extra_data[idx as usize];
                let url_token = ast.extra_data[idx as usize + 1];
                let url = ast.token_slice(url_token);

                output.push_str(&format!("{MAGENTA}[img: "));
                if text_node_raw != u32::MAX {
                    render_inline(ast, text_node_raw, output);
                }
                output.push_str(&format!("]{RESET} {DIM}({url}){RESET}"));
            }
        }

        NodeTag::HardBreak => {
            output.push('\n');
        }

        NodeTag::MdxTextExpression => {
            if let NodeData::Extra(idx) = node.data {
                let range = ast.extra_range(idx);
                let content = extract_token_range(ast, &range);
                output.push_str(&format!("{DIM}{{{}}}{RESET}", content.trim()));
            }
        }

        NodeTag::Paragraph => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_inline(ast, child_idx, output);
            }
        }

        _ => {
            let text = ast.token_slice(node.main_token);
            output.push_str(text);
        }
    }
}

fn render_list_item(ast: &Ast, node_idx: NodeIndex, output: &mut String, number: Option<usize>) {
    let info = ast.list_item_info(node_idx);

    let bullet = match number {
        Some(n) => format!("  {n}. "),
        None => "  * ".to_string(),
    };
    output.push_str(&bullet);

    if let Some(checked) = info.checked {
        if checked {
            output.push_str(&format!("{GREEN}[x]{RESET} "));
        } else {
            output.push_str(&format!("{DIM}[ ]{RESET} "));
        }
    }

    let children = ast.children(node_idx);
    for &child_idx in children {
        let child = &ast.nodes[child_idx as usize];
        if child.tag == NodeTag::Paragraph {
            let para_children = ast.children(child_idx);
            for &para_child_idx in para_children {
                render_inline(ast, para_child_idx, output);
            }
        } else {
            render_inline(ast, child_idx, output);
        }
    }
    output.push('\n');
}

fn render_code_block(ast: &Ast, node: &Node, output: &mut String) {
    use hypernote_mdx::token::Tag as TokenTag;

    let fence_token = node.main_token;

    // Language label
    let mut lang: Option<&str> = None;
    if fence_token + 1 < ast.token_tags.len() as u32 {
        let next_token = fence_token + 1;
        if ast.token_tags[next_token as usize] == TokenTag::Text {
            let lang_text = ast.token_slice(next_token).trim();
            if !lang_text.is_empty() {
                lang = Some(lang_text);
            }
        }
    }

    if let Some(l) = lang {
        output.push_str(&format!("{YELLOW}{l}{RESET}\n"));
    }

    // Extract code content
    let code = extract_code_block_content(ast, fence_token);
    output.push_str(&format!("{DIM}"));
    output.push_str(code);
    if !code.is_empty() && !code.ends_with('\n') {
        output.push('\n');
    }
    output.push_str(&format!("{RESET}"));
}

fn render_table(ast: &Ast, node_idx: NodeIndex, output: &mut String) {
    let alignments = ast.table_alignments(node_idx);
    let rows = ast.children(node_idx);

    if rows.is_empty() {
        return;
    }

    let num_cols = alignments.len();

    // First pass: compute max column widths
    let mut col_widths: Vec<usize> = vec![0; num_cols];
    let mut cell_strings: Vec<Vec<String>> = Vec::new();

    for &row_idx in rows {
        let cells = ast.children(row_idx);
        let mut row_strings: Vec<String> = Vec::new();
        for (col, &cell_idx) in cells.iter().enumerate() {
            let mut cell_out = String::new();
            let cell_children = ast.children(cell_idx);
            for &child_idx in cell_children {
                render_inline_plain(ast, child_idx, &mut cell_out);
            }
            let trimmed = cell_out.trim().to_string();
            if col < num_cols {
                col_widths[col] = col_widths[col].max(trimmed.len());
            }
            row_strings.push(trimmed);
        }
        // Pad missing columns
        while row_strings.len() < num_cols {
            row_strings.push(String::new());
        }
        cell_strings.push(row_strings);
    }

    // Ensure minimum width of 3 for each column
    for w in &mut col_widths {
        if *w < 3 {
            *w = 3;
        }
    }

    // Draw top border
    output.push_str(&format!("{DIM}"));
    output.push('\u{250c}'); // ┌
    for (i, &w) in col_widths.iter().enumerate() {
        for _ in 0..w + 2 {
            output.push('\u{2500}'); // ─
        }
        if i < num_cols - 1 {
            output.push('\u{252c}'); // ┬
        }
    }
    output.push('\u{2510}'); // ┐
    output.push_str(&format!("{RESET}\n"));

    // Render rows
    for (row_i, row_cells) in cell_strings.iter().enumerate() {
        output.push_str(&format!("{DIM}\u{2502}{RESET}")); // │
        for (col, cell) in row_cells.iter().enumerate() {
            if col >= num_cols {
                break;
            }
            let w = col_widths[col];
            let padded = pad_cell(cell, w, &alignments[col]);
            if row_i == 0 {
                // Header row: bold
                output.push_str(&format!(" {BOLD}{padded}{RESET} "));
            } else {
                output.push_str(&format!(" {padded} "));
            }
            output.push_str(&format!("{DIM}\u{2502}{RESET}")); // │
        }
        output.push('\n');

        // After header row, draw separator
        if row_i == 0 {
            output.push_str(&format!("{DIM}"));
            output.push('\u{251c}'); // ├
            for (i, &w) in col_widths.iter().enumerate() {
                for _ in 0..w + 2 {
                    output.push('\u{2500}'); // ─
                }
                if i < num_cols - 1 {
                    output.push('\u{253c}'); // ┼
                }
            }
            output.push('\u{2524}'); // ┤
            output.push_str(&format!("{RESET}\n"));
        }
    }

    // Draw bottom border
    output.push_str(&format!("{DIM}"));
    output.push('\u{2514}'); // └
    for (i, &w) in col_widths.iter().enumerate() {
        for _ in 0..w + 2 {
            output.push('\u{2500}'); // ─
        }
        if i < num_cols - 1 {
            output.push('\u{2534}'); // ┴
        }
    }
    output.push('\u{2518}'); // ┘
    output.push_str(&format!("{RESET}\n"));
}

/// Render inline content to plain text (no ANSI) for width calculation
fn render_inline_plain(ast: &Ast, node_idx: NodeIndex, output: &mut String) {
    let node = &ast.nodes[node_idx as usize];
    match node.tag {
        NodeTag::Text => {
            let text = ast.token_slice(node.main_token);
            output.push_str(text);
        }
        NodeTag::Strong | NodeTag::Emphasis => {
            let children = ast.children(node_idx);
            for &child_idx in children {
                render_inline_plain(ast, child_idx, output);
            }
        }
        NodeTag::CodeInline => {
            if let NodeData::Token(content_token) = node.data {
                let text = ast.token_slice(content_token);
                output.push_str(text);
            }
        }
        NodeTag::Link => {
            if let NodeData::Extra(idx) = node.data {
                let text_node_raw = ast.extra_data[idx as usize];
                if text_node_raw != u32::MAX {
                    render_inline_plain(ast, text_node_raw, output);
                }
            }
        }
        _ => {
            let text = ast.token_slice(node.main_token);
            output.push_str(text);
        }
    }
}

fn pad_cell(content: &str, width: usize, alignment: &TableAlignment) -> String {
    let len = content.len();
    if len >= width {
        return content.to_string();
    }
    let padding = width - len;
    match alignment {
        TableAlignment::Right => format!("{}{}", " ".repeat(padding), content),
        TableAlignment::Center => {
            let left = padding / 2;
            let right = padding - left;
            format!("{}{}{}", " ".repeat(left), content, " ".repeat(right))
        }
        _ => format!("{}{}", content, " ".repeat(padding)),
    }
}

fn extract_token_range<'a>(ast: &'a Ast, range: &Range) -> &'a str {
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

fn extract_code_block_content<'a>(ast: &'a Ast, fence_token: u32) -> &'a str {
    use hypernote_mdx::token::Tag as TokenTag;

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

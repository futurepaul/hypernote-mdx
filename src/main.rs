use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <file.hnmd>", args[0]);
        std::process::exit(1);
    }

    let filename = &args[1];
    let source = fs::read_to_string(filename).unwrap_or_else(|e| {
        eprintln!("Error reading {}: {}", filename, e);
        std::process::exit(1);
    });

    println!("Parsing: {}\n", filename);

    let ast = hypernote_mdx::parse(&source);

    print_ast(&ast);

    if !ast.errors.is_empty() {
        println!("\n=== ERRORS ({}) ===", ast.errors.len());
        for err in &ast.errors {
            println!(
                "  - {} at token {} (byte {}): {}",
                err.tag.name(),
                err.token,
                err.byte_offset,
                err.tag.message()
            );
        }
    }
}

fn print_ast(ast: &hypernote_mdx::ast::Ast) {
    use hypernote_mdx::ast::*;

    println!("=== AST ===");
    println!("Nodes: {}", ast.nodes.len());
    println!("Tokens: {}", ast.token_tags.len());
    println!("Extra data: {}", ast.extra_data.len());
    println!("\n=== NODES ===");

    for (i, node) in ast.nodes.iter().enumerate() {
        let node_idx = i as NodeIndex;
        print!("[{}] {}", node_idx, node.tag.name());

        match node.tag {
            NodeTag::Heading => {
                let info = ast.heading_info(node_idx);
                let children = ast.children(node_idx);
                print!(" (level={}, children={})", info.level, children.len());
            }
            NodeTag::Text => {
                let token_text = ast.token_slice(node.main_token);
                let trimmed = if token_text.len() > 50 {
                    &token_text[..50]
                } else {
                    token_text
                };
                print!(" \"{}\"", trimmed);
            }
            NodeTag::Document
            | NodeTag::Paragraph
            | NodeTag::Blockquote
            | NodeTag::ListUnordered
            | NodeTag::ListOrdered
            | NodeTag::ListItem
            | NodeTag::MdxJsxElement
            | NodeTag::MdxJsxFragment => {
                let children = ast.children(node_idx);
                if !children.is_empty() {
                    print!(" (children={})", children.len());
                }
            }
            _ => {}
        }

        println!();
    }

    println!("\n=== TREE ===");
    if !ast.nodes.is_empty() {
        let doc_idx = ast
            .nodes
            .iter()
            .enumerate()
            .find(|(_, n)| n.tag == NodeTag::Document)
            .map(|(i, _)| i as NodeIndex);

        if let Some(root_idx) = doc_idx {
            print_node(ast, root_idx, 0);
        }
    }
}

fn print_node(
    ast: &hypernote_mdx::ast::Ast,
    node_idx: hypernote_mdx::ast::NodeIndex,
    indent: usize,
) {
    use hypernote_mdx::ast::*;

    let node = &ast.nodes[node_idx as usize];

    for _ in 0..indent {
        print!("  ");
    }

    print!("[{}] {}", node_idx, node.tag.name());

    match node.tag {
        NodeTag::Heading => {
            let info = ast.heading_info(node_idx);
            print!(" (level={})", info.level);
        }
        NodeTag::Text => {
            let token_text = ast.token_slice(node.main_token);
            print!(" \"{}\"", token_text);
        }
        NodeTag::MdxJsxElement | NodeTag::MdxJsxSelfClosing => {
            let elem = ast.jsx_element(node_idx);
            let name = ast.token_slice(elem.name_token);
            print!(" <{}>", name);
        }
        NodeTag::Link | NodeTag::Image => {
            if let NodeData::Extra(idx) = node.data {
                let url_token = ast.extra_data[idx as usize + 1];
                let url = ast.token_slice(url_token);
                print!(" (url={})", url);
            }
        }
        NodeTag::Frontmatter => {
            let info = ast.frontmatter_info(node_idx);
            let fmt = match info.format {
                FrontmatterFormat::Yaml => "YAML",
                FrontmatterFormat::Json => "JSON",
            };
            print!(" ({} frontmatter)", fmt);
        }
        _ => {}
    }

    println!();

    let children = ast.children(node_idx);
    for &child_idx in children {
        print_node(ast, child_idx, indent + 1);
    }
}

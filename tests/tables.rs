use hypernote_mdx::ast::*;
use hypernote_mdx::{parse, render, serialize_tree};

fn find_node(ast: &Ast, tag: NodeTag) -> Option<NodeIndex> {
    ast.nodes
        .iter()
        .enumerate()
        .find(|(_, n)| n.tag == tag)
        .map(|(i, _)| i as NodeIndex)
}

fn count_nodes(ast: &Ast, tag: NodeTag) -> usize {
    ast.nodes.iter().filter(|n| n.tag == tag).count()
}

#[test]
fn basic_2x2_table() {
    let source = "| A | B |\n| --- | --- |\n| 1 | 2 |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let info = ast.table_info(table_idx);
    assert_eq!(info.num_columns, 2);
    assert_eq!(info.num_rows, 2); // header + 1 body row

    let rows = ast.children(table_idx);
    assert_eq!(rows.len(), 2);

    // Header row
    let header_cells = ast.children(rows[0]);
    assert_eq!(header_cells.len(), 2);

    // Body row
    let body_cells = ast.children(rows[1]);
    assert_eq!(body_cells.len(), 2);
}

#[test]
fn table_with_alignments() {
    let source = "| Left | Center | Right | None |\n| :--- | :---: | ---: | --- |\n| a | b | c | d |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let alignments = ast.table_alignments(table_idx);

    assert_eq!(alignments.len(), 4);
    assert_eq!(alignments[0], TableAlignment::Left);
    assert_eq!(alignments[1], TableAlignment::Center);
    assert_eq!(alignments[2], TableAlignment::Right);
    assert_eq!(alignments[3], TableAlignment::None);
}

#[test]
fn table_with_inline_formatting() {
    let source = "| Header |\n| --- |\n| **bold** |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let rows = ast.children(table_idx);

    // Body row should contain a cell with Strong node
    let body_cells = ast.children(rows[1]);
    let cell_children = ast.children(body_cells[0]);
    let has_strong = cell_children
        .iter()
        .any(|&idx| ast.nodes[idx as usize].tag == NodeTag::Strong);
    assert!(has_strong, "cell should contain a Strong node");
}

#[test]
fn table_with_italic_and_code() {
    let source = "| H1 | H2 |\n| --- | --- |\n| *italic* | `code` |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let rows = ast.children(table_idx);
    let body_cells = ast.children(rows[1]);

    // First cell: emphasis
    let cell0_children = ast.children(body_cells[0]);
    let has_emphasis = cell0_children
        .iter()
        .any(|&idx| ast.nodes[idx as usize].tag == NodeTag::Emphasis);
    assert!(has_emphasis);

    // Second cell: inline code
    let cell1_children = ast.children(body_cells[1]);
    let has_code = cell1_children
        .iter()
        .any(|&idx| ast.nodes[idx as usize].tag == NodeTag::CodeInline);
    assert!(has_code);
}

#[test]
fn table_with_links() {
    let source = "| Link |\n| --- |\n| [text](url) |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let has_link = ast.nodes.iter().any(|n| n.tag == NodeTag::Link);
    assert!(has_link, "table cell should contain a Link node");
}

#[test]
fn table_with_empty_cells() {
    let source = "| A | B |\n| --- | --- |\n|  |  |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let rows = ast.children(table_idx);
    assert_eq!(rows.len(), 2);
}

#[test]
fn single_column_table() {
    let source = "| One |\n| --- |\n| val |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let info = ast.table_info(table_idx);
    assert_eq!(info.num_columns, 1);
}

#[test]
fn table_multiple_body_rows() {
    let source = "| H |\n| --- |\n| r1 |\n| r2 |\n| r3 |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let info = ast.table_info(table_idx);
    assert_eq!(info.num_rows, 4); // 1 header + 3 body rows
}

#[test]
fn roundtrip_basic_table() {
    let source = "| A | B |\n| --- | --- |\n| 1 | 2 |\n";
    let ast1 = parse(source);
    let rendered = render(&ast1);
    let ast2 = parse(&rendered);

    assert!(ast2.errors.is_empty(), "roundtrip errors: {:?}", ast2.errors);

    // Both should have a Table node
    assert!(find_node(&ast1, NodeTag::Table).is_some());
    assert!(find_node(&ast2, NodeTag::Table).is_some());

    // Same number of table-related nodes
    assert_eq!(
        count_nodes(&ast1, NodeTag::Table),
        count_nodes(&ast2, NodeTag::Table)
    );
    assert_eq!(
        count_nodes(&ast1, NodeTag::TableRow),
        count_nodes(&ast2, NodeTag::TableRow)
    );
}

#[test]
fn roundtrip_table_with_alignments() {
    let source = "| L | C | R |\n| :--- | :---: | ---: |\n| a | b | c |\n";
    let ast1 = parse(source);
    let rendered = render(&ast1);
    let ast2 = parse(&rendered);

    assert!(ast2.errors.is_empty(), "roundtrip errors: {:?}", ast2.errors);

    let table1 = find_node(&ast1, NodeTag::Table).unwrap();
    let table2 = find_node(&ast2, NodeTag::Table).unwrap();
    let align1 = ast1.table_alignments(table1);
    let align2 = ast2.table_alignments(table2);
    assert_eq!(align1, align2);
}

#[test]
fn json_serialization_table() {
    let source = "| A | B |\n| :--- | ---: |\n| 1 | 2 |\n";
    let ast = parse(source);
    let json = serialize_tree(&ast);

    // Parse the JSON
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid JSON");

    // Find the table node
    let children = parsed["children"].as_array().unwrap();
    let table = children.iter().find(|c| c["type"] == "table").expect("table in JSON");

    // Check alignments
    let alignments = table["alignments"].as_array().unwrap();
    assert_eq!(alignments.len(), 2);
    assert_eq!(alignments[0], "left");
    assert_eq!(alignments[1], "right");

    // Check children (rows)
    let rows = table["children"].as_array().unwrap();
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0]["type"], "table_row");
    assert_eq!(rows[1]["type"], "table_row");

    // Check cells
    let header_cells = rows[0]["children"].as_array().unwrap();
    assert_eq!(header_cells.len(), 2);
    assert_eq!(header_cells[0]["type"], "table_cell");
}

#[test]
fn pipe_in_non_table_context_is_paragraph() {
    let source = "This is a | pipe in text\n";
    let ast = parse(source);

    // Should not create a table
    assert!(find_node(&ast, NodeTag::Table).is_none());
    // Should be a paragraph
    assert!(find_node(&ast, NodeTag::Paragraph).is_some());
}

#[test]
fn table_followed_by_other_blocks() {
    let source = "| A |\n| --- |\n| 1 |\n\n# Heading\n\nParagraph\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    assert!(find_node(&ast, NodeTag::Table).is_some());
    assert!(find_node(&ast, NodeTag::Heading).is_some());
    assert!(find_node(&ast, NodeTag::Paragraph).is_some());
}

#[test]
fn table_preceded_by_other_blocks() {
    let source = "# Title\n\n| A |\n| --- |\n| 1 |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    assert!(find_node(&ast, NodeTag::Heading).is_some());
    assert!(find_node(&ast, NodeTag::Table).is_some());
}

#[test]
fn render_preserves_table_structure() {
    let source = "| A | B |\n| --- | --- |\n| 1 | 2 |\n";
    let ast = parse(source);
    let rendered = render(&ast);

    // Should contain pipes
    assert!(rendered.contains('|'));
    // Should contain separator
    assert!(rendered.contains("---"));
}

#[test]
fn table_cell_text_content() {
    let source = "| Hello | World |\n| --- | --- |\n| foo | bar |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let rows = ast.children(table_idx);

    // Check header cell text
    let header_cells = ast.children(rows[0]);
    let cell0_children = ast.children(header_cells[0]);
    assert!(!cell0_children.is_empty(), "header cell should have children");

    // Find text node and check content
    let text_node = &ast.nodes[cell0_children[0] as usize];
    assert_eq!(text_node.tag, NodeTag::Text);
    let text = ast.token_slice(text_node.main_token).trim();
    assert_eq!(text, "Hello");
}

#[test]
fn three_column_table() {
    let source = "| A | B | C |\n| --- | --- | --- |\n| 1 | 2 | 3 |\n| 4 | 5 | 6 |\n";
    let ast = parse(source);

    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);

    let table_idx = find_node(&ast, NodeTag::Table).expect("should have a Table node");
    let info = ast.table_info(table_idx);
    assert_eq!(info.num_columns, 3);
    assert_eq!(info.num_rows, 3); // header + 2 body
}

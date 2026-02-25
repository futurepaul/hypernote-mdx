use hypernote_mdx::ast::NodeTag;

#[test]
fn single_line_break() {
    let source = "Line one\nLine two\n";
    let ast = hypernote_mdx::parse(source);

    // Single break should be ONE paragraph
    let para_count = ast.nodes.iter().filter(|n| n.tag == NodeTag::Paragraph).count();
    // The Zig version doesn't assert a specific count here, just prints debug info.
    // But logically, a single line break = 1 paragraph.
    eprintln!("Single line break: {} paragraphs", para_count);
}

#[test]
fn double_line_break() {
    let source = "Paragraph one\n\nParagraph two\n";
    let ast = hypernote_mdx::parse(source);

    let para_count = ast.nodes.iter().filter(|n| n.tag == NodeTag::Paragraph).count();
    eprintln!("Double line break: {} paragraphs", para_count);
    // Double break should produce two paragraphs
}

#[test]
fn multiple_single_breaks() {
    let source = "Line one\nLine two\nLine three\n";
    let ast = hypernote_mdx::parse(source);

    let para_count = ast.nodes.iter().filter(|n| n.tag == NodeTag::Paragraph).count();
    eprintln!("Multiple single breaks: {} paragraphs", para_count);
}

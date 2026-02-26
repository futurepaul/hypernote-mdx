use hypernote_mdx::ast::NodeTag;

#[test]
fn hard_break_trailing_two_spaces() {
    let source = "Line one  \nLine two\n";
    let ast = hypernote_mdx::parse(source);

    let br_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::HardBreak)
        .count();
    let para_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::Paragraph)
        .count();

    assert_eq!(1, br_count);
    assert_eq!(1, para_count);
}

#[test]
fn hard_break_backslash() {
    let source = "Line one\\\nLine two\n";
    let ast = hypernote_mdx::parse(source);

    let br_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::HardBreak)
        .count();
    let para_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::Paragraph)
        .count();

    assert_eq!(1, br_count);
    assert_eq!(1, para_count);
}

#[test]
fn hard_break_multiple_in_one_paragraph() {
    let source = "Line one  \nLine two\\\nLine three\n";
    let ast = hypernote_mdx::parse(source);

    let br_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::HardBreak)
        .count();
    let para_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::Paragraph)
        .count();

    assert_eq!(2, br_count);
    assert_eq!(1, para_count);
}

#[test]
fn soft_break_vs_hard_break() {
    let source = "Soft break\nHard break  \nAnother line\n";
    let ast = hypernote_mdx::parse(source);

    let br_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::HardBreak)
        .count();
    let para_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::Paragraph)
        .count();

    assert_eq!(1, br_count);
    assert_eq!(1, para_count);
}

#[test]
fn paragraph_break_with_trailing_spaces() {
    let source = "Para one  \n\nPara two\n";
    let ast = hypernote_mdx::parse(source);

    let br_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::HardBreak)
        .count();
    let para_count = ast
        .nodes
        .iter()
        .filter(|n| n.tag == NodeTag::Paragraph)
        .count();

    assert_eq!(1, br_count);
    assert_eq!(2, para_count);
}

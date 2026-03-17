use hypernote_mdx::ast::ErrorTag;
use hypernote_mdx::parse;

#[test]
fn invalid_jsx_attribute_reports_precise_byte_offset() {
    let source = "<Button label=>\n";
    let ast = parse(source);

    let err = ast
        .errors
        .iter()
        .find(|err| err.tag == ErrorTag::InvalidJsxAttribute)
        .expect("expected invalid_jsx_attribute error");

    assert_eq!(source.find('>').unwrap() as u32, err.byte_offset);
}

#[test]
fn unclosed_jsx_attribute_expression_reports_eof_offset() {
    let source = "<Button expr={foo\n";
    let ast = parse(source);

    let err = ast
        .errors
        .iter()
        .find(|err| err.tag == ErrorTag::UnclosedExpression)
        .expect("expected unclosed_expression error");

    assert_eq!(source.len() as u32, err.byte_offset);
}

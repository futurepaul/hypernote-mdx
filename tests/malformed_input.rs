use hypernote_mdx::ast::ErrorTag;
use hypernote_mdx::{parse, render, serialize_tree};
use std::panic::{AssertUnwindSafe, catch_unwind};

fn parse_render_serialize_without_panicking(source: &str) -> hypernote_mdx::ast::Ast {
    let result = catch_unwind(AssertUnwindSafe(|| {
        let ast = parse(source);
        let json = serialize_tree(&ast);
        let rendered = render(&ast);

        serde_json::from_str::<serde_json::Value>(&json)
            .expect("serialized tree should stay valid JSON");
        let _ = rendered;

        ast
    }));

    match result {
        Ok(ast) => ast,
        Err(_) => panic!("library panicked for malformed input:\n{source}"),
    }
}

#[test]
fn malformed_inputs_return_errors_without_panicking() {
    let cases = [
        ("<Button label=>\n", ErrorTag::InvalidJsxAttribute),
        ("<Foo>\ntext\n</Bar>\n", ErrorTag::MismatchedTags),
        (
            "text {unclosed expression here\n",
            ErrorTag::UnclosedExpression,
        ),
        ("---\nkey: value\n", ErrorTag::UnclosedFrontmatter),
        ("[**bold** label](\n", ErrorTag::ExpectedToken),
        ("![*alt* text](\n", ErrorTag::ExpectedToken),
        ("> **quote\n", ErrorTag::ExpectedToken),
        ("- [x] **task\n", ErrorTag::ExpectedToken),
        ("~~broken\n", ErrorTag::ExpectedToken),
        ("_broken\n", ErrorTag::ExpectedToken),
    ];

    for (source, expected_error) in cases {
        let ast = parse_render_serialize_without_panicking(source);
        assert!(
            ast.errors.iter().any(|err| err.tag == expected_error),
            "expected {expected_error:?} for input:\n{source}\nactual: {:?}",
            ast.errors.iter().map(|err| err.tag).collect::<Vec<_>>()
        );
    }
}

#[test]
fn pathological_malformed_inputs_stay_bounded_and_non_panicking() {
    let cases = [
        "[[".repeat(256),
        "<Broken a=>".repeat(64),
        "**bold *italic `code [link\n".repeat(32),
        "<Foo>\n".repeat(64),
        "~~ ~~ ~~ ".repeat(256),
        "_ _ _ ".repeat(256),
    ];

    for source in cases {
        let ast = parse_render_serialize_without_panicking(&source);
        assert!(
            ast.errors.len() <= 4096,
            "error list should stay bounded for malformed input"
        );
        assert!(
            !ast.nodes.is_empty(),
            "parser should still return an AST for malformed input"
        );
    }
}

#[test]
fn malformed_input_error_offsets_survive_render_and_serialize_paths() {
    let source = "<Button expr={foo\n";
    let ast = parse_render_serialize_without_panicking(source);

    let err = ast
        .errors
        .iter()
        .find(|err| err.tag == ErrorTag::UnclosedExpression)
        .expect("expected unclosed_expression error");

    assert_eq!(source.len() as u32, err.byte_offset);
}

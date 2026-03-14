use hypernote_mdx::{parse, serialize_tree};
use serde_json::{Value, json};

fn serialized(source: &str) -> Value {
    let ast = parse(source);
    assert!(ast.errors.is_empty(), "errors: {:?}", ast.errors);
    serde_json::from_str(&serialize_tree(&ast)).expect("serialized AST should be valid JSON")
}

#[test]
fn markdown_ast_snapshot_is_stable() {
    let source = "# Launch\n\nParagraph with **bold** and [link](https://example.com)\n";

    let actual = serialized(source);
    let expected = json!({
        "schema": {
            "name": "hypernote-mdx-ast",
            "version": 1
        },
        "type": "root",
        "children": [
            {
                "type": "heading",
                "level": 1,
                "children": [
                    {
                        "type": "text",
                        "value": "Launch"
                    }
                ]
            },
            {
                "type": "paragraph",
                "children": [
                    {
                        "type": "text",
                        "value": "Paragraph with "
                    },
                    {
                        "type": "strong",
                        "children": [
                            {
                                "type": "text",
                                "value": "bold"
                            }
                        ]
                    },
                    {
                        "type": "text",
                        "value": " and "
                    },
                    {
                        "type": "link",
                        "url": "https://example.com",
                        "children": [
                            {
                                "type": "text",
                                "value": "link"
                            }
                        ]
                    }
                ]
            }
        ],
        "source": source,
        "errors": []
    });

    assert_eq!(expected, actual);
}

#[test]
fn jsx_ast_snapshot_is_stable() {
    let source = "<Widget count=4 enabled label=\"Fish &amp; Chips\" expr={state.count}>Hi 👋</Widget>\n";

    let actual = serialized(source);
    let expected = json!({
        "schema": {
            "name": "hypernote-mdx-ast",
            "version": 1
        },
        "type": "root",
        "children": [
            {
                "type": "mdx_jsx_element",
                "name": "Widget",
                "attributes": [
                    {
                        "name": "count",
                        "value_type": "number",
                        "type": "number",
                        "value": 4
                    },
                    {
                        "name": "enabled",
                        "value_type": "boolean",
                        "type": "boolean",
                        "value": true
                    },
                    {
                        "name": "label",
                        "value_type": "string",
                        "type": "string",
                        "value": "Fish & Chips"
                    },
                    {
                        "name": "expr",
                        "value_type": "expression",
                        "type": "expression",
                        "value": "state.count"
                    }
                ],
                "children": [
                    {
                        "type": "text",
                        "value": "Hi 👋"
                    }
                ]
            }
        ],
        "source": source,
        "errors": []
    });

    assert_eq!(expected, actual);
}

#[test]
fn table_ast_snapshot_is_stable() {
    let source = "| Name | Score |\n| :--- | ---: |\n| Paul | 42 |\n";

    let actual = serialized(source);
    let expected = json!({
        "schema": {
            "name": "hypernote-mdx-ast",
            "version": 1
        },
        "type": "root",
        "children": [
            {
                "type": "table",
                "alignments": ["left", "right"],
                "children": [
                    {
                        "type": "table_row",
                        "children": [
                            {
                                "type": "table_cell",
                                "children": [
                                    {
                                        "type": "text",
                                        "value": " Name "
                                    }
                                ]
                            },
                            {
                                "type": "table_cell",
                                "children": [
                                    {
                                        "type": "text",
                                        "value": " Score "
                                    }
                                ]
                            }
                        ]
                    },
                    {
                        "type": "table_row",
                        "children": [
                            {
                                "type": "table_cell",
                                "children": [
                                    {
                                        "type": "text",
                                        "value": " Paul "
                                    }
                                ]
                            },
                            {
                                "type": "table_cell",
                                "children": [
                                    {
                                        "type": "text",
                                        "value": " 42 "
                                    }
                                ]
                            }
                        ]
                    }
                ]
            }
        ],
        "source": source,
        "errors": []
    });

    assert_eq!(expected, actual);
}

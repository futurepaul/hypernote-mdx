# hypernote-mdx

`hypernote-mdx` is a Rust parser for Markdown + JSX with a compact AST, parser-owned semantic helpers, stable source-position utilities, JSON serialization when you need it, and canonical rendering back to MDX.

It is designed for downstream Rust code that wants parser-adjacent semantics without re-decoding token order, `extra_data`, JSX strings, HTML entities, or source offsets by hand.

## Quick Start

```rust
use hypernote_mdx::ast::NodeTag;
use hypernote_mdx::semantic::{ExpressionTextPolicy, JsxElementKind, PlainTextOptions};

let source = r#"
<Card title="Inbox">
  Count: {state.count}
</Card>
"#;

let ast = hypernote_mdx::parse(source);

// Canonical round-trip rendering
let rendered = hypernote_mdx::render(&ast);

// JSON tree when you need a serialized semantic payload
let json = hypernote_mdx::serialize_tree(&ast);

// Typed semantic access for Rust consumers
let card = ast
  .nodes
  .iter()
  .enumerate()
  .find_map(|(idx, node)| (node.tag == NodeTag::MdxJsxElement).then_some(idx as u32))
  .and_then(|idx| ast.jsx_element_view(idx))
  .expect("expected JSX element");

assert_eq!("Card", card.name);
assert_eq!(JsxElementKind::Normal, card.kind);

let plain = ast
  .plain_text_with_options(
    card.children[0],
    &PlainTextOptions {
      expression_policy: ExpressionTextPolicy::Placeholder("{expr}"),
    },
  )
  .unwrap();
assert!(plain.contains("{expr}"));

let pos = ast.node_position(card.children[0]);
assert!(pos.line >= 1);

assert!(!json.is_empty());
assert!(!rendered.is_empty());
```

## Choose The Right API

- Use `parse` / `parse_with_options` when you want the AST and parser errors.
- Use semantic accessors on `Ast` when you are writing Rust code and want typed information directly.
- Use `serialize_tree()` when you need a JSON semantic tree across a process or language boundary.
- Use `render()` when you want canonical MDX output from the parsed tree.
- Use `plain_text*` when you need a lossy text projection for search, indexing, or diagnostics.
- Use `line_col()` and `node_position()` when downstream validation needs one-based `line:column` locations.

## Semantic API

The parser-owned semantic layer lives in [`src/semantic.rs`](src/semantic.rs). Current `Ast` helpers include:

- `code_block_info`
- `link_view`
- `image_view`
- `expression_info`
- `frontmatter_view`
- `jsx_attribute_views`
- `jsx_element_view`
- `plain_text_parts`
- `plain_text_parts_children`
- `plain_text`
- `plain_text_children`
- `plain_text_with_options`
- `plain_text_children_with_options`

Key semantic types include:

- `CodeBlockInfo`
- `LinkInfo`
- `ImageInfo`
- `ExpressionInfo`
- `FrontmatterInfoView`
- `JsxAttributeView`
- `JsxAttributeValue`
- `JsxElementView`
- `JsxElementKind`
- `PlainTextPart`
- `PlainTextOptions`
- `ExpressionTextPolicy`
- `SourcePosition`

Important behavior:

- JSX string attributes are already unquoted and HTML-entity decoded.
- Numeric and boolean JSX attributes come back typed.
- `JsxAttributeValue::InvalidNumber(&str)` preserves malformed manual AST data without panicking.
- Plain-text extraction keeps expression handling explicit through `ExpressionTextPolicy`.

## Syntax Support

Markdown support includes:

- headings, paragraphs, horizontal rules, hard breaks
- emphasis/strong with both `*` / `**` and `_` / `__`
- strikethrough `~~`
- inline code and fenced code blocks
- links and images, including rich inline label / alt content
- blockquotes
- ordered, unordered, and task lists
- multi-paragraph blockquotes and list items
- tables

MDX support includes:

- JSX elements and self-closing JSX
- JSX fragments
- JSX attributes with literal and expression values
- inline and flow expressions

Frontmatter support includes:

- YAML frontmatter with `---`
- JSON frontmatter in fenced `hnmd` code blocks

## Error Model And Safety

`parse` records structured parser errors in `ast.errors` instead of panicking on malformed input. The library is tested and fuzzed around malformed Markdown/MDX source, malformed JSX attributes, unclosed expressions/frontmatter, render/serialize no-panic behavior, and malformed-AST accessor safety.

The crate’s boundary is source parsing plus parser-adjacent semantics. It does not own app-specific component registries, action semantics, validation policy, or renderer-facing document models.

## Related Docs

- [`TESTING.md`](TESTING.md): test commands, fuzzing, pathological fixtures, and debug binaries
- [`DESIGN.md`](DESIGN.md): AST layout, semantic layering, crate boundaries, and safety model
- [`AUTORESEARCH.md`](AUTORESEARCH.md): timing, benchmarking, and the autoresearch workflow

## License

MIT

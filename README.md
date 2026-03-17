# hypernote-mdx

A Rust parser for MDX (Markdown + JSX), built for the [hypernote](https://github.com/futurepaul/hypernote-pages) format. A Rust port of [zig-mdx](https://github.com/futurepaul/zig-mdx).

It parses source into a compact AST, exposes typed semantic helpers for downstream Rust code, can serialize to a semantic JSON tree when needed, and renders back to canonical source.

## Usage

```rust
use hypernote_mdx::ast::NodeTag;

// Parse MDX source into AST
let ast = hypernote_mdx::parse(source);

// Optional parser behavior (for example, emoji shortcode normalization)
let ast = hypernote_mdx::parse_with_options(
  source,
  &hypernote_mdx::ParseOptions {
    normalize_emoji_shortcodes: true,
  },
);

// Serialize AST to JSON when you need a semantic tree payload
let json = hypernote_mdx::serialize_tree(&ast);

// Render AST back to canonical MDX source
let roundtripped = hypernote_mdx::render(&ast);

// Downstream Rust code can also use typed semantic accessors directly.
let maybe_code_block = ast
  .nodes
  .iter()
  .enumerate()
  .find_map(|(idx, node)| (node.tag == NodeTag::CodeBlock).then_some(idx as u32))
  .and_then(|idx| ast.code_block_info(idx));
```

### Semantic API

The [`semantic`](/Users/futurepaul/dev/sec/hypernote-mdx/src/semantic.rs) module is the parser-owned, typed surface for downstream Rust crates that do not want to reverse-engineer `extra_data`, token ordering, or JSX decoding rules.

Typed semantic accessors live on `Ast`:

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

Typed semantic values include:

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

Example:

```rust
use hypernote_mdx::ast::NodeTag;
use hypernote_mdx::semantic::JsxAttributeValue;

let ast = hypernote_mdx::parse(r#"<Widget label="Fish &amp; Chips" count=4 expr={state.count} />"#);

let jsx = ast
  .nodes
  .iter()
  .enumerate()
  .find_map(|(idx, node)| (node.tag == NodeTag::MdxJsxSelfClosing).then_some(idx as u32))
  .expect("expected JSX node");

let attrs = ast.jsx_attribute_views(jsx).expect("expected attrs");

assert!(matches!(attrs[0].value, JsxAttributeValue::String(_)));
assert!(matches!(attrs[1].value, JsxAttributeValue::Number(4.0)));
assert!(matches!(attrs[2].value, JsxAttributeValue::Expression("state.count")));
```

JSX string values are already unquoted and entity-decoded. Numeric and boolean values come back typed. If a caller constructs a malformed AST manually, `JsxAttributeValue::InvalidNumber(&str)` preserves the raw value instead of panicking or guessing.

For element-level JSX inspection, `jsx_element_view` packages the element name, decoded attrs, child node slice, and whether the node is normal or self-closing:

```rust
use hypernote_mdx::ast::NodeTag;
use hypernote_mdx::semantic::JsxElementKind;

let ast = hypernote_mdx::parse(r#"<Card title="Inbox"><Body>hello</Body></Card>"#);
let card = ast
  .nodes
  .iter()
  .enumerate()
  .find_map(|(idx, node)| (node.tag == NodeTag::MdxJsxElement).then_some(idx as u32))
  .and_then(|idx| ast.jsx_element_view(idx))
  .expect("expected JSX element");

assert_eq!("Card", card.name);
assert_eq!(JsxElementKind::Normal, card.kind);
assert_eq!(1, card.attrs.len());
assert_eq!(1, card.children.len());
```

### Plain Text And Expressions

`plain_text_parts*` exposes a structured, lossy text projection without throwing away expression boundaries:

```rust
use hypernote_mdx::ast::NodeTag;
use hypernote_mdx::semantic::{ExpressionTextPolicy, PlainTextOptions};

let ast = hypernote_mdx::parse("Value: {state.count}\n");
let paragraph = ast
  .nodes
  .iter()
  .enumerate()
  .find_map(|(idx, node)| (node.tag == NodeTag::Paragraph).then_some(idx as u32))
  .expect("expected paragraph");

let default_text = ast.plain_text(paragraph).unwrap();
let omitted = ast.plain_text_with_options(
  paragraph,
  &PlainTextOptions {
    expression_policy: ExpressionTextPolicy::Omit,
  },
).unwrap();
let placeholder = ast.plain_text_with_options(
  paragraph,
  &PlainTextOptions {
    expression_policy: ExpressionTextPolicy::Placeholder("{expr}"),
  },
).unwrap();

assert_eq!("Value: state.count", default_text);
assert_eq!("Value: ", omitted);
assert_eq!("Value: {expr}", placeholder);
```

The default policy is `ExpressionTextPolicy::Source`. That keeps expression parsing in the parser crate while leaving application-specific interpretation to downstream code.

### Source Positions

For downstream diagnostics, the AST can map byte offsets and node starts to one-based `line:column` positions:

```rust
use hypernote_mdx::ast::NodeTag;

let source = "before\n\n<Card />\n";
let ast = hypernote_mdx::parse(source);

assert_eq!(1, ast.line_col(0).line);
assert_eq!(1, ast.line_col(0).column);
assert_eq!(3, ast.line_col(8).line);

let jsx = ast
  .nodes
  .iter()
  .enumerate()
  .find_map(|(idx, node)| (node.tag == NodeTag::MdxJsxSelfClosing).then_some(idx as u32))
  .expect("expected JSX node");

let pos = ast.node_position(jsx);
assert_eq!(3, pos.line);
assert_eq!(1, pos.column);
```

### CLI

```sh
cargo run -- example.hnmd
```

Prints the tokenized AST and tree structure for a given `.hnmd` file.

## Library Behavior

`parse` records structured parse errors in `ast.errors` for malformed input instead of panicking. The main library paths are covered by tests and fuzzing for:

- malformed markdown / MDX source
- malformed JSX attrs and unclosed expressions / frontmatter
- render and `serialize_tree` no-panic behavior
- malformed-AST accessor safety for invalid indices and bad node/data shapes

The crate still treats source parsing and parser-adjacent semantics as its boundary. It does not own app-specific component behavior, action semantics, or rendering policy.

## Development

This repo includes a `justfile` for common test and timing workflows.

```sh
just test
just test-one markdown_ast_snapshot_is_stable
just test-suite ast_snapshots
just time
just time-one markdown_ast_snapshot_is_stable
just time-suite ast_snapshots
just test-build-timings
just bench
just bench-one markdown_ast_snapshot_is_stable
just bench-suite ast_snapshots
just fuzz
just fuzz-corpus
just fuzz-build
```

`just bench*` recipes use [`hyperfine`](https://github.com/sharkdp/hyperfine). Install it with:

```sh
cargo install hyperfine
```

## Autoresearch

This worktree also supports an `autoresearch`-style optimization loop for `src/` only.

```sh
just research-baseline
just research-run "reduce repeated token slicing"
just research-plot
just research-serve
```

Open `http://localhost:8765` after starting `just research-serve` to watch the progress chart and recent experiment table in your browser.

### Fuzzing

A `cargo-fuzz` harness lives under [`fuzz/`](/Users/futurepaul/dev/sec/hypernote-mdx/fuzz/Cargo.toml). The first target stresses the main library path on arbitrary input:

- `parse`
- `serialize_tree`
- `render`
- parse/render round-trips
- the `normalize_emoji_shortcodes` parser option

Install the tool once:

```sh
cargo install cargo-fuzz
```

Then run either:

```sh
just fuzz
```

or, if you want corpus growth checked into the repo:

```sh
just fuzz-corpus
```

or directly:

```sh
cargo fuzz run parse_render_serialize -- -max_total_time=10
```

Starter corpus seeds live in `fuzz/corpus/parse_render_serialize/`.
The fuzz dictionary lives in `fuzz/dictionaries/mdx.dict`, and `just fuzz` uses it by default so libFuzzer can mutate common Markdown/MDX syntax more intelligently.
`just fuzz` runs against a temporary corpus copy so it does not churn the repo; use `just fuzz-corpus` when you intentionally want libFuzzer to grow the checked-in corpus.

## What it parses

**Markdown:** headings, paragraphs, strong/emphasis with both `*` / `**` and `_` / `__`, strikethrough `~~`, inline code, fenced code blocks, links, images, rich inline content inside link labels and image alt text, blockquotes, ordered / unordered / task lists, multi-paragraph blockquotes, multi-paragraph list items, tables, horizontal rules, hard breaks.

**MDX extensions:** JSX elements (`<Card>...</Card>`), self-closing JSX (`<Button />`), JSX fragments (`<>...</>`), JSX attributes (literal and expression), and inline expressions (`{name}`).

**Frontmatter:** YAML (`---`) and JSON (`` ```hnmd ```) frontmatter blocks.

## AST design

The AST uses a flat, index-based representation inspired by Zig's data-oriented design. Nodes reference tokens and children by index into parallel arrays rather than using heap-allocated tree pointers. This keeps the structure compact and cache-friendly while still allowing typed semantic accessors to be layered on top.

```rust
let ast = hypernote_mdx::parse("# Hello **world**");

// Walk children by index
let children = ast.children(node_idx);

// Get source text for any token
let text = ast.token_slice(token_idx);

// Read typed semantic data without decoding extra_data manually
let maybe_link = ast.link_view(node_idx);

// Read typed JSX element data in one call
let maybe_jsx = ast.jsx_element_view(node_idx);

// Find the deepest node at a byte offset
let node = ast.node_at_offset(42);

// Convert byte offsets or node starts into one-based positions
let pos = ast.line_col(42);
let node_pos = ast.node_position(node_idx);
```

## License

MIT
